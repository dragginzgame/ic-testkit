//! Repository-local operator tooling. Owns builds, worker runs and evidence files;
//! the library and workload retain server lifecycle and measurement semantics.

mod arguments;
mod report;

use std::{
    collections::BTreeMap,
    error::Error as StdError,
    ffi::OsString,
    fs::{self, File, Permissions},
    io::{self, Seek as _, Write as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::{fs::PermissionsExt as _, process::CommandExt as _};

use ic_build::{
    artifacts::{WasmBuildError, WasmBuildOutcome, WasmBuildSpec, build_wasm_canisters_cached},
    cargo::{
        native::{NativeExecutableSpec, NativeTargetKind, build_native_executable},
        observation::workspace_root_for,
        selection::CargoSelection,
    },
};
use ic_host_artifacts::artifact::ArtifactError;
use ic_host_fs::{
    durable::{
        NamedWriteError, PublicationMode, WriteOptions, write_existing_parent_validated_with,
    },
    read::{hash_file, read_opened_file},
};
use ic_host_process::{
    child::OwnedChild,
    provenance::{
        GitError, GitObservations, IgnoreSubmodules, StatusOptions, UntrackedFiles, capture_git,
    },
    sampling,
    tool::{
        AdmittedTool, CompletedExecution, ExecutionContext, OutputLimit, OutputLimits,
        ResolutionError, SuccessfulExit, ToolError, VersionSpec, communicate_child,
        resolve_executable,
    },
};
use ic_host_text::table::{self, Alignment};
use serde_json::{Value, json};

use crate::fixture_reuse_protocol::{Mode, WorkerArguments};
use arguments::{HELP, Options, parse_arguments};
use report::{DriverReport, MeasuredRun, Summary, summarize};

const WORKSPACE_DISCOVERY_LIMITS: OutputLimits = OutputLimits {
    stdout: OutputLimit::Terminate(64 * 1024),
    stderr: OutputLimit::Terminate(64 * 1024),
    timeout: None,
};
const SAMPLE_INTERVAL: Duration = Duration::from_millis(10);
const REPORT_LIMIT: u64 = 16 * 1024 * 1024;
const WASM_LIMIT: u64 = 128 * 1024 * 1024;
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum Error {
    #[display("invalid benchmark argument: {_0}")]
    InvalidArgument(String),
    #[display("invalid benchmark data: {_0}")]
    InvalidData(&'static str),
    #[from]
    Io(io::Error),
    #[from]
    Json(serde_json::Error),
    Statistics(crate::statistics::SampleStatisticsError),
    Artifact(ArtifactError),
    Resolution(ResolutionError),
    Build(Box<WasmBuildError>),
    CargoObservation(Box<dyn StdError>),
    #[from]
    Sampling(sampling::SamplingError),
    Tool(Box<ToolError>),
    Git(Box<GitError>),
    #[display("{} exited with {status}", program.display())]
    CommandFailed {
        program: PathBuf,
        status: ExitStatus,
    },
    #[display("benchmark interrupted")]
    Interrupted,
}

// Project through boxed causes so callers can downcast the original error directly.
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Statistics(error) => Some(error),
            Self::Artifact(error) => Some(error),
            Self::Resolution(error) => Some(error),
            Self::Build(error) => Some(error.as_ref()),
            Self::CargoObservation(error) => Some(error.as_ref()),
            Self::Sampling(error) => Some(error),
            Self::Tool(error) => Some(error.as_ref()),
            Self::Git(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

#[cfg(unix)]
struct InterruptHandler(libc::sigaction);

#[cfg(unix)]
extern "C" fn mark_interrupted(_signal: libc::c_int) {
    // The handler performs only a lock-free flag update, never I/O or teardown.
    INTERRUPTED.store(true, Ordering::Relaxed);
}

#[cfg(unix)]
impl InterruptHandler {
    fn install() -> Result<Self, Error> {
        INTERRUPTED.store(false, Ordering::Relaxed);
        // SAFETY: sigaction is a C value with valid zero-initialized fields.
        let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
        // SAFETY: the set is writable; sigemptyset initializes it for sigaction.
        if unsafe { libc::sigemptyset(&raw mut action.sa_mask) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
        action.sa_sigaction = mark_interrupted as *const () as libc::sighandler_t;
        // SAFETY: previous is writable; the handler has the required C ABI and
        // remains valid for this process. Only this standalone driver installs it.
        let mut previous: libc::sigaction = unsafe { std::mem::zeroed() };
        if unsafe { libc::sigaction(libc::SIGINT, &raw const action, &raw mut previous) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
        Ok(Self(previous))
    }
}

#[cfg(unix)]
impl Drop for InterruptHandler {
    fn drop(&mut self) {
        // SAFETY: this is the exact previous action returned by sigaction.
        let _ = unsafe { libc::sigaction(libc::SIGINT, &raw const self.0, std::ptr::null_mut()) };
    }
}

fn check_interrupted() -> Result<(), Error> {
    if INTERRUPTED.load(Ordering::Relaxed) {
        Err(Error::Interrupted)
    } else {
        Ok(())
    }
}

fn publish_report(
    output: &Path,
    write: impl FnOnce(&mut File) -> Result<(), Error>,
) -> Result<(), Error> {
    check_interrupted()?;
    match write_existing_parent_validated_with(
        output,
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        write,
        |()| check_interrupted(),
    ) {
        Ok(()) => Ok(()),
        Err(NamedWriteError::Producer {
            source,
            cleanup_error,
        }) => {
            if let Some(error) = cleanup_error {
                eprintln!("could not clean report staging: {error}");
            }
            Err(source)
        }
        Err(NamedWriteError::BeforePublication {
            source,
            cleanup_error,
        }) => {
            if let Some(error) = cleanup_error {
                eprintln!("could not clean report staging: {error}");
            }
            Err(source.into())
        }
        Err(NamedWriteError::AfterPublication { source }) => {
            // The report is complete and visible. This driver promises atomic
            // visibility, not power-loss durability; do not report a failed
            // replacement or invite a retry after publication has occurred.
            eprintln!("report published; final durability or cleanup is unconfirmed: {source}");
            Ok(())
        }
    }
}

struct CapturedRun {
    report: crate::fixture_reuse_report::WorkerReport,
    rss: (u64, usize, usize),
}

fn measure(
    command: &mut Command,
    mut snapshot: impl FnMut() -> Result<sampling::Snapshot, sampling::SamplingError>,
) -> Result<CapturedRun, Error> {
    check_interrupted()?;
    let mut output = tempfile::Builder::new()
        .permissions(Permissions::from_mode(0o600))
        .tempfile()?
        .into_file();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(output.try_clone()?));
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn()?;
    let observed = (|| -> Result<_, Error> {
        let mut peak_rss = 0;
        let mut peak_processes = 0;
        let mut count = 0;
        while child.try_wait()?.is_none() && !INTERRUPTED.load(Ordering::Relaxed) {
            let (resident, processes) = sampling::tree_rss(child.id(), &snapshot()?)?;
            peak_rss = peak_rss.max(resident);
            peak_processes = peak_processes.max(processes);
            count += 1;
            thread::sleep(SAMPLE_INTERVAL);
        }
        Ok((peak_rss, peak_processes, count))
    })();
    // Even sampling failure or Ctrl-C waits for the worker's RAII server
    // cleanup. Killing the workload here would strand its owned process group.
    let status = child.wait()?;
    check_interrupted()?;
    let rss = observed?;
    if !status.success() {
        return Err(Error::CommandFailed {
            program: command.get_program().into(),
            status,
        });
    }
    output.rewind()?;
    let bytes = read_opened_file(
        output.try_clone()?,
        usize::try_from(REPORT_LIMIT).expect("report limit fits supported hosts"),
    )
    .map_err(Error::Artifact)?;
    let report = serde_json::from_slice(&bytes)?;
    Ok(CapturedRun { report, rss })
}

fn execute_command(command: &mut Command) -> Result<CompletedExecution, Error> {
    check_interrupted()?;
    let mut child = OwnedChild::spawn(
        command.stdin(Stdio::null()).stderr(Stdio::inherit()),
        ic_host_process::child::CleanupPolicy::KillAndWait,
    )?;
    communicate_child(
        &mut child,
        None,
        OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(0),
            timeout: None,
        },
        SuccessfulExit::Cleanup,
        || INTERRUPTED.load(Ordering::Relaxed),
    )
    .and_then(CompletedExecution::require_success)
    .map_err(|error| Error::Tool(Box::new(error)))
}

fn text_command(command: &mut Command) -> Result<String, Error> {
    let output = execute_command(command.stdout(Stdio::piped()))?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|_| Error::InvalidData("tool output is not UTF-8"))
}

fn print_summary(summaries: &BTreeMap<String, Summary>, labels: &[String]) -> Result<(), Error> {
    let mut rows = Vec::with_capacity(labels.len());
    for label in labels {
        let summary = &summaries[label];
        let (Some(work), Some(fixture), Some(rate), Some(peak), Some(wait)) = (
            summary.work_wall_ms.as_ref(),
            summary.fixture_wall_ms.as_ref(),
            summary.iterations_per_second.as_ref(),
            summary.sampled_peak_tree_rss_bytes.as_ref(),
            summary.phases_ms["wait_ms"].as_ref(),
        ) else {
            return Err(Error::InvalidData("summary lacks required measurements"));
        };
        let restore = summary.phases_ms["restore_ms"]
            .as_ref()
            .map_or_else(|| "-".into(), |restore| format!("{:.2}", restore.mean));
        rows.push([
            label.clone(),
            format!("{:.2}", work.p50),
            format!("{:.2}", fixture.p50),
            format!("{:.2}", rate.p50),
            restore,
            format!("{:.2}", wait.mean),
            format!("{:.2}", peak.p50 / 1024.0 / 1024.0),
        ]);
    }
    println!(
        "{}",
        table::render(
            &[
                "Mode",
                "work ms",
                "incl. setup/drop ms",
                "tasks/s",
                "restore mean ms",
                "wait mean ms",
                "peak RSS MiB"
            ],
            &rows,
            &[
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right
            ],
            table::Options {
                column_gap: 2,
                header_separator: false
            },
        )
    );
    Ok(())
}

struct Inputs {
    root: PathBuf,
    server: PathBuf,
    worker: PathBuf,
    wasm: WasmBuildOutcome,
    provenance: Value,
}

fn tool_program(program: OsString) -> Result<PathBuf, Error> {
    if program.is_empty() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "tool name must not be empty").into(),
        );
    }
    let path = PathBuf::from(program);
    // Preserve the invocation name used by multicall proxies such as rustup.
    // Anchor explicit relative paths before commands switch to the workspace;
    // bare names retain Command's native PATH lookup.
    if path.is_relative() && path.components().count() > 1 {
        Ok(std::env::current_dir()?.join(path))
    } else {
        Ok(path)
    }
}

fn cargo_command(cargo: &Path, root: &Path, rustc: Option<&Path>) -> Command {
    let mut command = Command::new(cargo);
    command.current_dir(root);
    if let Some(rustc) = rustc {
        command.env("RUSTC", rustc);
    }
    command
}

fn prepare(options: &Options) -> Result<Inputs, Error> {
    let cargo = tool_program(std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")))?;
    let rustc = std::env::var_os("RUSTC").map(tool_program).transpose()?;
    let root = workspace_root_for(
        env!("CARGO_MANIFEST_DIR"),
        WORKSPACE_DISCOVERY_LIMITS,
        cargo_command(&cargo, &std::env::current_dir()?, rustc.as_deref()),
    )?;
    if !root
        .join("crates/ic_testkit_perf_probe/Cargo.toml")
        .is_file()
    {
        return Err(Error::InvalidArgument(
            "benchmark requires the repository's perf_probe checkout".into(),
        ));
    }
    // The operator supplies an exact path. Canonicalization prevents a bare name
    // from silently turning into discovery through PATH.
    let server = resolve_executable(&fs::canonicalize(&options.server)?, &root, &[])
        .map_err(Error::Resolution)?;
    let server_version = text_command(Command::new(&server).arg("--version"))?;
    if !ic_testkit::pic::supports_pocket_ic_server(server_version.as_bytes()) {
        return Err(Error::InvalidData("expected pocket-ic-server 16.x.y"));
    }
    check_interrupted()?;
    let native = build_native_executable(
        &NativeExecutableSpec {
            package: "ic-testkit-dev",
            target_name: "fixture_reuse_benchmark",
            target_kind: NativeTargetKind::Binary,
            selection: CargoSelection {
                profile: Some(options.profile.clone()),
                locked: true,
                offline: true,
                ..CargoSelection::default()
            },
            max_message_bytes: 16 * 1024 * 1024,
        },
        || cargo_command(&cargo, &root, rustc.as_deref()),
        io::stderr().lock(),
        || {
            if INTERRUPTED.load(Ordering::Relaxed) {
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                Ok(())
            }
        },
        |command| OwnedChild::spawn(command, ic_host_process::child::CleanupPolicy::KillAndWait),
        || INTERRUPTED.load(Ordering::Relaxed),
    )
    .map_err(|error| Error::CargoObservation(error))?;
    let target = &native.target_directory;
    let mut spec = WasmBuildSpec::new(
        &root,
        &target.join("ic-testkit-fixture-benchmark-wasm"),
        &["ic_testkit_perf_probe"],
    )
    .with_cargo_program(cargo)
    .with_cargo_selection(CargoSelection {
        locked: true,
        offline: true,
        ..CargoSelection::default()
    })
    .with_shared_incremental_target(target.join("ic-testkit-incremental"));
    if let Some(rustc) = &rustc {
        spec = spec.with_extra_env([("RUSTC", rustc.as_os_str())]);
    }
    check_interrupted()?;
    let wasm = build_wasm_canisters_cached(&spec, |command| {
        OwnedChild::spawn(command, ic_host_process::child::CleanupPolicy::KillAndWait)
    });
    check_interrupted()?;
    let wasm = wasm.map_err(|error| Error::Build(Box::new(error)))?;
    let worker = native.executable;
    let provenance = provenance(
        &root,
        options,
        &server,
        &server_version,
        &wasm.record().artifacts()[0],
        rustc.as_deref(),
    )?;
    Ok(Inputs {
        root,
        server,
        worker,
        wasm,
        provenance,
    })
}

// Git is selected from the operator's installed PATH, then admitted and
// rechecked by Host. The observed identity is not a trusted installation pin.
fn git_provenance(root: &Path, limits: OutputLimits) -> Result<GitObservations, Error> {
    check_interrupted()?;
    let environment = std::env::vars_os().collect::<Vec<_>>();
    let search = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default();
    let executable =
        resolve_executable(Path::new("git"), root, &search).map_err(Error::Resolution)?;
    let context = ExecutionContext {
        current_dir: root,
        environment: &environment,
    };
    let git = AdmittedTool::observe_version(
        &VersionSpec {
            executable: &executable,
            executable_bytes: 128 * 1024 * 1024,
            version_arguments: &["--version".into()],
        },
        &context,
        limits,
    )
    .map_err(|error| Error::Tool(Box::new(error)))?;
    let observed = capture_git(
        &git,
        &context,
        StatusOptions {
            untracked: UntrackedFiles::Normal,
            ignore_submodules: IgnoreSubmodules::None,
        },
        limits,
    )
    .map_err(|error| Error::Git(Box::new(error)))?;
    check_interrupted()?;
    Ok(observed)
}

fn provenance(
    root: &Path,
    options: &Options,
    server: &Path,
    server_version: &str,
    wasm: &Path,
    rustc: Option<&Path>,
) -> Result<Value, Error> {
    // Capture the selected build and source facts before running cases.
    let git = git_provenance(
        root,
        OutputLimits {
            stdout: OutputLimit::Terminate(16 * 1024 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        },
    )?;
    // SAFETY: sysconf reads the host's online CPU count with no pointer arguments.
    #[cfg(unix)]
    let cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    #[cfg(not(unix))]
    let cpus = 0;
    let cpus = u64::try_from(cpus)
        .ok()
        .filter(|count| *count > 0)
        .ok_or(Error::InvalidData("logical CPU count unavailable"))?;
    Ok(json!({
        "server": server, "server_version": server_version,
        "platform": text_command(Command::new("uname").arg("-srm"))?, "logical_cpus": cpus,
        "rustc": text_command(Command::new(rustc.unwrap_or_else(|| Path::new("rustc"))).arg("--version").current_dir(root))?,
        "revision": git.revision(),
        "working_tree_dirty": git.is_dirty(),
        "host_profile": options.profile, "wasm_profile": "dev", "wasm_sha256": hash_file(wasm, WASM_LIMIT).map_err(Error::Artifact)?.sha256.to_string(),
        "rss_sample_interval_ms": SAMPLE_INTERVAL.as_millis(), "rss_sampler": sampling::source(),
        "rss_command_stdout_bytes": options.rss_limits.stdout.retained_bytes(),
        "rss_command_stderr_bytes": options.rss_limits.stderr.retained_bytes(),
        "iterations": options.iterations.get(), "workers": options.workers.get(), "repeats": options.repeats.get(),
        "state_bytes_per_canister": options.state_bytes, "modes": options.modes.iter().map(|mode| mode.as_str()).collect::<Vec<_>>(),
        "pool_capacities": if options.modes.contains(&Mode::Pooled) { options.capacities.iter().map(|capacity| capacity.get()).collect::<Vec<_>>() } else { vec![] },
    }))
}

pub fn run() -> Result<(), Error> {
    let Some(options) = parse_arguments(std::env::args_os().skip(1))? else {
        println!("{HELP}");
        return Ok(());
    };
    if !cfg!(any(target_os = "linux", target_os = "macos")) {
        return Err(Error::InvalidArgument(
            "native sampling requires Linux or macOS".into(),
        ));
    }
    #[cfg(unix)]
    let _interrupt_handler = InterruptHandler::install()?;
    let inputs = prepare(&options)?;
    let cases = options.benchmark_cases();
    let mut runs: BTreeMap<String, Vec<MeasuredRun>> = cases
        .iter()
        .map(|case| (case.label.clone(), Vec::new()))
        .collect();
    for repeat in 0..options.repeats.get() {
        for case in cases
            .iter()
            .cycle()
            .skip(repeat % cases.len())
            .take(cases.len())
        {
            check_interrupted()?;
            eprintln!("Run {}/{}: {}", repeat + 1, options.repeats, case.label);
            let invocation = WorkerArguments {
                mode: case.mode,
                capacity: case.capacity,
                iterations: options.iterations,
                workers: options.workers,
                state_bytes: options.state_bytes,
                wasm: inputs.wasm.record().artifacts()[0].clone(),
                server: inputs.server.clone(),
            };
            let captured = measure(
                Command::new(&inputs.worker)
                    .args(<[OsString; 7]>::from(invocation))
                    .current_dir(&inputs.root),
                || sampling::snapshot(64 * 1024, options.rss_limits),
            )?;
            let measured = MeasuredRun::from_worker(captured.report, &options, case, captured.rss)?;
            runs.get_mut(&case.label)
                .expect("case owns run list")
                .push(measured);
        }
    }
    let summaries = runs
        .iter()
        .map(|(label, runs)| Ok((label.clone(), summarize(runs)?)))
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
    check_interrupted()?;
    if let Some(output) = &options.output {
        let document = DriverReport {
            format: "ic-testkit-fixture-benchmark-v1",
            provenance: &inputs.provenance,
            runs: &runs,
            summary: &summaries,
        };
        publish_report(output, |file| {
            serde_json::to_writer_pretty(&mut *file, &document)?;
            file.write_all(b"\n")?;
            Ok(())
        })?;
    }
    print_summary(
        &summaries,
        &cases
            .iter()
            .map(|case| case.label.clone())
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::arguments::parse_arguments;
    use super::{
        Error, Permissions, REPORT_LIMIT, execute_command, measure, prepare, publish_report,
        text_command,
    };
    use ic_host_process::tool::{ExecutionFailure, OutputStream, ToolError};

    #[cfg(unix)]
    use super::InterruptHandler;
    use serde_json::{Value, json};
    use std::{
        ffi::OsString,
        fs,
        io::{self, Read as _, Write as _},
        path::{Path, PathBuf},
        process::Command,
    };

    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt as _, symlink};

    fn snapshot()
    -> Result<ic_host_process::sampling::Snapshot, ic_host_process::sampling::SamplingError> {
        ic_host_process::sampling::snapshot(64 * 1024, super::arguments::RSS_COMMAND_LIMITS)
    }

    fn fixture_directory() -> tempfile::TempDir {
        tempfile::Builder::new()
            .permissions(Permissions::from_mode(0o700))
            .tempdir()
            .unwrap()
    }

    #[test]
    fn error_projection_preserves_direct_typed_sources() {
        use std::error::Error as _;

        for error in [
            Error::from(io::Error::from(io::ErrorKind::PermissionDenied)),
            Error::CargoObservation(Box::new(io::Error::from(io::ErrorKind::PermissionDenied))),
        ] {
            assert_eq!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .unwrap()
                    .kind(),
                io::ErrorKind::PermissionDenied
            );
        }
        let error = Error::from(serde_json::from_str::<Value>("invalid json").unwrap_err());
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .is_some()
        );
        assert!(Error::Interrupted.source().is_none());
        assert!(Error::InvalidArgument("argument".into()).source().is_none());
    }

    #[test]
    #[cfg(unix)]
    fn git_observations_preserve_dirty_path_bytes_and_typed_failures() {
        use ic_host_process::{
            provenance::GitQuery,
            tool::{OutputLimit, OutputLimits},
        };
        use std::{error::Error as _, os::unix::ffi::OsStringExt as _};

        let scratch = fixture_directory();
        let root = scratch.path().canonicalize().unwrap();
        let limits = OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        };
        for args in [
            vec!["init", "-q"],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "fixture",
            ],
        ] {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(&root)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let clean = super::git_provenance(&root, limits).unwrap();
        assert!(!clean.is_dirty());
        // macOS filesystems reject arbitrary invalid UTF-8 names before Git runs.
        // Every host still checks raw control/quote bytes; Linux also exercises
        // an actual invalid UTF-8 directory entry.
        for name in std::iter::once(b"untracked-\n\t\"".as_slice())
            .chain(cfg!(target_os = "linux").then_some(b"untracked-\xff".as_slice()))
        {
            let path = root.join(OsString::from_vec(name.to_vec()));
            fs::write(&path, b"fixture").unwrap();
            let dirty = super::git_provenance(&root, limits).unwrap();
            assert_eq!(dirty.revision(), clean.revision());
            assert!(dirty.is_dirty());
            assert_eq!(
                dirty.status_evidence().stdout,
                [b"?? ".as_slice(), name, b"\0"].concat()
            );
            assert_eq!(
                dirty.options.untracked,
                ic_host_process::provenance::UntrackedFiles::Normal
            );
            assert_eq!(
                dirty.options.ignore_submodules,
                ic_host_process::provenance::IgnoreSubmodules::None
            );
            fs::remove_file(path).unwrap();
        }

        let missing = fixture_directory();
        let error =
            super::git_provenance(&missing.path().canonicalize().unwrap(), limits).unwrap_err();
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<ic_host_process::provenance::GitError>()
                .is_some()
        );
        assert!(matches!(error, Error::Git(error) if error.query == GitQuery::Revision));
        let error = super::git_provenance(
            &root,
            OutputLimits {
                stdout: OutputLimit::Terminate(1),
                ..limits
            },
        )
        .unwrap_err();
        assert!(matches!(error, Error::Tool(_)));
    }

    #[cfg(unix)]
    fn assert_prepared_tool_context(root: &Path, missing: bool) {
        let options = parse_arguments([
            OsString::from("--server"),
            root.join("server").into_os_string(),
        ])
        .unwrap()
        .unwrap();
        if missing {
            assert!(
                matches!(prepare(&options), Err(Error::Io(error)) if error.kind() == io::ErrorKind::NotFound)
            );
        } else if std::env::var_os("IC_TESTKIT_BENCHMARK_TOOL_FAIL_BUILD").is_some() {
            let Err(Error::Build(error)) = prepare(&options) else {
                panic!("failed producer must preserve its typed cause");
            };
            assert!(matches!(
                *error,
                ic_build::artifacts::WasmBuildError::CommandFailed {
                    phase: ic_build::artifacts::WasmBuildPhase::CargoBuild,
                    status,
                    ref stdout,
                    ref stderr,
                } if status.code() == Some(23)
                    && stdout.contains("producer output")
                    && stderr.contains("producer failure")
            ));
        } else {
            let inputs = prepare(&options).unwrap();
            assert_eq!(inputs.provenance["rustc"], "fixture rustc");
            assert_eq!(
                inputs.worker,
                root.join("target/cargo-reported directory/worker fixture")
            );
            let trace = fs::read_to_string(root.join("trace")).unwrap();
            let calls = trace.lines().collect::<Vec<_>>();
            assert_ne!(calls.len(), 0);
            assert_eq!(calls.len() % 3, 0);
            for call in calls.as_chunks::<3>().0 {
                if call[2].contains("locate-project") {
                    assert_eq!(call[1], root.to_str().unwrap());
                    assert!(call[2].contains("--offline"));
                    assert!(call[2].contains("--workspace"));
                    continue;
                }
                assert_eq!(call[1], inputs.root.to_str().unwrap());
                if call[0].ends_with("/cargo") || call[0] == "cargo" {
                    if !call[2].starts_with("--version") {
                        assert!(call[2].contains("--locked"));
                        assert!(call[2].contains("--offline"));
                    }
                } else {
                    assert!(call[0].ends_with("/rustc") || call[0] == "rustc");
                    assert!(matches!(call[2], "--version" | "-vV"));
                }
            }

            let target = root.join("target");
            let mutable_paths = [
                target.join("wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm"),
                target.join("ic-testkit-incremental/wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm"),
                target.join("ic-testkit-fixture-benchmark-wasm/wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm"),
            ];
            for path in &mutable_paths {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                if path.exists() {
                    fs::remove_file(path).unwrap();
                }
                fs::write(path, b"replaced public Cargo output").unwrap();
            }
            let retained = &inputs.wasm.record().artifacts()[0];
            assert_eq!(
                inputs.provenance["wasm_sha256"],
                super::hash_file(retained, super::WASM_LIMIT)
                    .unwrap()
                    .sha256
                    .to_string()
            );
            for _ in 0..2 {
                let worker = Command::new("cat").arg(retained).output().unwrap();
                assert!(worker.status.success());
                assert_eq!(worker.stdout, b"\0asm\x01\0\0\0");
                for path in &mutable_paths {
                    let _ = fs::remove_file(path);
                }
            }
            publish_report(&root.join("report.json"), |file| {
                Ok(serde_json::to_writer(file, &inputs.provenance)?)
            })
            .unwrap();
        }
    }

    #[cfg(unix)]
    fn prepare_cargo_fixture(root: &Path, target: &Path) {
        let metadata = Command::new(env!("CARGO"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["metadata", "--locked", "--offline", "--format-version", "1"])
            .output()
            .unwrap();
        assert!(
            metadata.status.success(),
            "{}",
            String::from_utf8_lossy(&metadata.stderr)
        );
        let mut catalog: Value = serde_json::from_slice(&metadata.stdout).unwrap();
        let probe = catalog["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|package| package["name"] == "ic_testkit_perf_probe")
            .unwrap();
        let artifact = json!({
            "reason": "compiler-artifact",
            "package_id": probe["id"],
            "manifest_path": probe["manifest_path"],
            "target": probe["targets"][0],
            "profile": {
                "opt_level": "0", "debuginfo": 2, "debug_assertions": true,
                "overflow_checks": true, "test": false
            },
            "features": [],
            "filenames": [target.join("ic-testkit-incremental/wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm")],
            "executable": null,
            "fresh": false
        });
        let package = catalog["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|package| package["name"] == "ic-testkit-dev")
            .unwrap();
        let native_target = package["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["name"] == "fixture_reuse_benchmark")
            .unwrap();
        let mut native_artifact = artifact.clone();
        native_artifact["package_id"] = package["id"].clone();
        native_artifact["manifest_path"] = package["manifest_path"].clone();
        native_artifact["target"] = native_target.clone();
        let native_executable = target.join("cargo-reported directory/worker fixture");
        native_artifact["filenames"] = json!([native_executable]);
        native_artifact["executable"] = json!(native_executable);
        fs::write(
            root.join("native-artifact-message.json"),
            format!("{native_artifact}\n"),
        )
        .unwrap();
        fs::write(root.join("artifact-message.json"), format!("{artifact}\n")).unwrap();
        fs::write(root.join("full-metadata.json"), &metadata.stdout).unwrap();
        catalog["target_directory"] = json!(target);
        catalog["resolve"] = Value::Null;
        fs::write(
            root.join("metadata.json"),
            serde_json::to_vec(&catalog).unwrap(),
        )
        .unwrap();
        fs::write(root.join("probe.wasm"), b"\0asm\x01\0\0\0").unwrap();
        fs::write(
            root.join("workspace.json"),
            serde_json::to_vec(&json!({"root":
                ic_build::cargo::observation::workspace_root_for(
                    env!("CARGO_MANIFEST_DIR"),
                    super::WORKSPACE_DISCOVERY_LIMITS,
                    Command::new(env!("CARGO")),
                )
                    .unwrap().join("Cargo.toml")
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn standalone_tools_and_retained_wasm_preserve_context_and_bytes() {
        const ROOT_ENV: &str = "IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT";
        const MISSING_ENV: &str = "IC_TESTKIT_BENCHMARK_TOOL_EXPECT_MISSING";
        if let Some(root) = std::env::var_os(ROOT_ENV) {
            assert_prepared_tool_context(
                &PathBuf::from(root),
                std::env::var_os(MISSING_ENV).is_some(),
            );
            return;
        }

        let scratch = fixture_directory();
        let root = scratch.path().canonicalize().unwrap();
        let bin = root.join("bin");
        fs::create_dir(&bin).unwrap();
        let target = root.join("target");
        let wasm = target.join("wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm");
        fs::create_dir_all(wasm.parent().unwrap()).unwrap();
        fs::write(wasm, b"\0asm\x01\0\0\0").unwrap();
        prepare_cargo_fixture(&root, &target);
        let proxy = root.join("multicall-proxy");
        fs::write(
            &proxy,
            r#"#!/bin/sh
set -eu
printf '%s\n%s\n%s\n' "$0" "$PWD" "$*" >> "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/trace"
case "${0##*/}" in
  cargo)
    if [ "$1" = --offline ]; then shift; fi
    if [ "$1" = locate-project ]; then
      cat "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/workspace.json"
      exit 0
    fi
    if [ -n "${RUSTC-}" ]; then
      [ "$RUSTC" = "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/bin/rustc" ] || exit 98
      "$RUSTC" --version >/dev/null
    fi
    case "$1" in
      --version) printf 'cargo fixture %s\n' "${IC_TESTKIT_BENCHMARK_TOOL_FAIL_BUILD-}" ;;
      metadata)
        case " $* " in
          *" --no-deps "*) cat "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/metadata.json" ;;
          *) cat "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/full-metadata.json" ;;
        esac ;;
      build)
        if [ -n "${CARGO_TARGET_DIR-}" ]; then
          if [ -n "${IC_TESTKIT_BENCHMARK_TOOL_FAIL_BUILD-}" ]; then
            printf 'producer output\n'
            printf 'producer failure\n' >&2
            exit 23
          fi
          output="$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm"
          mkdir -p "${output%/*}"
          cp "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/probe.wasm" "$output"
          cat "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/artifact-message.json"
        else
          cat "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/native-artifact-message.json"
        fi ;;
      *) exit 96 ;;
    esac ;;
  rustc) printf 'fixture rustc\n' ;;
  *) exit 97 ;;
esac
"#,
        )
        .unwrap();
        fs::set_permissions(&proxy, fs::Permissions::from_mode(0o700)).unwrap();
        symlink(&proxy, bin.join("cargo")).unwrap();
        symlink(&proxy, bin.join("rustc")).unwrap();
        let server = root.join("server");
        fs::write(
            &server,
            format!(
                "#!/bin/sh\nprintf 'pocket-ic-server {}\\n'\n",
                ic_testkit::pocket_ic::LATEST_SERVER_VERSION
            ),
        )
        .unwrap();
        fs::set_permissions(server, fs::Permissions::from_mode(0o700)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(bin.clone())
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        for (cargo, rustc, missing, failed_build) in [
            (None, None, false, false),
            (
                Some(bin.join("cargo").into_os_string()),
                Some(bin.join("rustc").into_os_string()),
                false,
                false,
            ),
            (
                Some(OsString::from("bin/cargo")),
                Some(OsString::from("bin/rustc")),
                false,
                false,
            ),
            (
                Some(root.join("missing-cargo").into_os_string()),
                None,
                true,
                false,
            ),
            (None, None, false, true),
        ] {
            let mut child = Command::new(std::env::current_exe().unwrap());
            child.args(["--exact", "fixture_reuse_driver::tests::standalone_tools_and_retained_wasm_preserve_context_and_bytes", "--test-threads=1"])
                .current_dir(&root)
                .env(ROOT_ENV, &root)
                .env("PATH", &path)
                .env_remove("CARGO")
                .env_remove("RUSTC")
                .env_remove(MISSING_ENV)
                .env_remove("IC_TESTKIT_BENCHMARK_TOOL_FAIL_BUILD");
            if let Some(cargo) = cargo {
                child.env("CARGO", cargo);
            }
            if let Some(rustc) = rustc {
                child.env("RUSTC", rustc);
            }
            if missing {
                child.env(MISSING_ENV, "1");
            }
            if failed_build {
                child.env("IC_TESTKIT_BENCHMARK_TOOL_FAIL_BUILD", "1");
            }
            let output = child.output().unwrap();
            assert!(
                output.status.success(),
                "tool invocation failed: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if !missing {
                fs::remove_file(root.join("trace")).unwrap();
            }
        }
    }

    #[test]
    fn report_write_failure_preserves_existing_report_and_cleans_scratch() {
        let scratch = fixture_directory();
        let report = scratch.path().join("report.json");
        fs::write(&report, b"previous complete report").unwrap();
        let result = publish_report(&report, |file| {
            file.write_all(b"partial replacement")?;
            assert_eq!(fs::read(&report).unwrap(), b"previous complete report");
            Err(io::Error::from(io::ErrorKind::StorageFull).into())
        });
        assert!(
            matches!(result, Err(Error::Io(error)) if error.kind() == io::ErrorKind::StorageFull)
        );
        assert_eq!(fs::read(&report).unwrap(), b"previous complete report");
        assert_eq!(fs::read_dir(scratch.path()).unwrap().count(), 1);
    }

    #[test]
    fn report_rename_failure_preserves_destination_and_cleans_scratch() {
        let scratch = fixture_directory();
        let report = scratch.path().join("report.json");
        fs::create_dir(&report).unwrap();
        let retained = report.join("retained report.json");
        fs::write(&retained, b"caller-owned evidence").unwrap();
        assert!(matches!(
            publish_report(&report, |file| Ok(file.write_all(b"replacement")?)),
            Err(Error::Io(_))
        ));
        assert_eq!(fs::read(retained).unwrap(), b"caller-owned evidence");
        assert_eq!(fs::read_dir(scratch.path()).unwrap().count(), 1);
        assert!(matches!(
            publish_report(&scratch.path().join("missing/report.json"), |_| {
                panic!("a missing parent must fail before writing")
            }),
            Err(Error::Io(error)) if error.kind() == io::ErrorKind::NotFound
        ));
        let ordinary = scratch.path().join("ordinary.json");
        fs::write(&ordinary, b"previous").unwrap();
        for suffix in ["ordinary.json/", "ordinary.json/."] {
            assert!(matches!(
                publish_report(&scratch.path().join(suffix), |_| {
                    panic!("a directory-required target must not be normalized")
                }),
                Err(Error::Io(error)) if error.kind() == io::ErrorKind::InvalidInput
            ));
            assert_eq!(fs::read(&ordinary).unwrap(), b"previous");
        }
    }

    #[test]
    fn report_replacement_keeps_old_readers_and_publishes_complete_json() {
        let scratch = fixture_directory();
        let report = scratch.path().join("report with spaces.json");
        fs::write(&report, b"previous complete report").unwrap();
        let mut old_reader = fs::File::open(&report).unwrap();
        let retained = scratch.path().join("retained.json");
        fs::hard_link(&report, &retained).unwrap();
        let document = json!({"format": "ic-testkit-fixture-benchmark-v1", "complete": true});
        publish_report(&report, |file| {
            serde_json::to_writer_pretty(&mut *file, &document)?;
            Ok(file.write_all(b"\n")?)
        })
        .unwrap();
        let mut previous = String::new();
        old_reader.read_to_string(&mut previous).unwrap();
        assert_eq!(previous, "previous complete report");
        assert_eq!(fs::read(retained).unwrap(), previous.as_bytes());
        let published = fs::read(&report).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&published).unwrap(),
            document
        );
        assert!(published.ends_with(b"\n"));
        assert_eq!(fs::read_dir(scratch.path()).unwrap().count(), 2);
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&report).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    #[cfg(unix)]
    fn report_replacement_preserves_symlink_target() {
        let scratch = fixture_directory();
        let target = scratch.path().join("original.json");
        let report = scratch.path().join("linked.json");
        fs::write(&target, b"caller-owned target").unwrap();
        symlink(&target, &report).unwrap();
        publish_report(&report, |file| Ok(file.write_all(b"complete replacement")?)).unwrap();
        assert!(
            !fs::symlink_metadata(&report)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(report).unwrap(), b"complete replacement");
        assert_eq!(fs::read(target).unwrap(), b"caller-owned target");

        let parent_alias = scratch.path().join("parent-alias");
        symlink(scratch.path(), &parent_alias).unwrap();
        publish_report(&parent_alias.join("linked.json"), |file| {
            Ok(file.write_all(b"replacement through parent alias")?)
        })
        .unwrap();
        assert_eq!(
            fs::read(scratch.path().join("linked.json")).unwrap(),
            b"replacement through parent alias"
        );
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn sampling_failure_waits_for_worker_completion() {
        let scratch = fixture_directory();
        let finished = scratch.path().join("finished");
        let result = measure(
            Command::new("/bin/sh")
                .args([
                    "-c",
                    "sleep 0.1; printf done > \"$1\"",
                    "sampling-failure-worker",
                ])
                .arg(&finished),
            || {
                ic_host_process::sampling::snapshot(
                    0,
                    ic_host_process::tool::OutputLimits {
                        stdout: ic_host_process::tool::OutputLimit::Terminate(0),
                        ..super::arguments::RSS_COMMAND_LIMITS
                    },
                )
            },
        );
        assert_eq!(fs::read(finished).unwrap(), b"done");
        let Err(Error::Sampling(error)) = result else {
            panic!("sampling must fail with its original cause");
        };
        assert!(std::error::Error::source(&error).is_some());
        #[cfg(target_os = "linux")]
        assert!(matches!(
            error,
            ic_host_process::sampling::SamplingError::Io(_)
        ));
        #[cfg(target_os = "macos")]
        assert!(matches!(
            error,
            ic_host_process::sampling::SamplingError::Command(_)
        ));
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn native_measurement_keeps_large_reports_and_rejects_failed_workers() {
        let scratch = fixture_directory();
        let input = scratch.path().join("input.json");
        let samples = (0..1024).map(|iteration| json!({
            "iteration": iteration,
            "phases": { "wait_ms": 0.0, "build_ms": 1.0, "restore_ms": null,
                "reset_ms": null, "readiness_ms": null, "validation_ms": 1.0, "acquisition_ms": 2.0 },
            "body_ms": 1.0, "release_ms": 1.0, "total_ms": 4.0,
        })).collect::<Vec<_>>();
        fs::write(
            &input,
            serde_json::to_vec(&json!({
                "mode": "fresh", "capacity": 1, "iterations": 1024, "workers": 1,
                "state_bytes_per_canister": 0, "wall_ms": 4.0, "fixture_wall_ms": 5.0,
                "server_startup_ms": 1.0, "cold_preparation": [], "preparation_wall_ms": 0.0,
                "pool_teardown_ms": 0.0, "server_teardown_ms": 0.0, "samples": samples,
            }))
            .unwrap(),
        )
        .unwrap();
        let captured = measure(
            Command::new("/bin/sh")
                .args(["-c", "sleep 0.1; cat \"$1\"", "test-worker"])
                .arg(&input),
            snapshot,
        )
        .unwrap();
        assert_eq!(captured.report.samples.len(), 1024);
        assert!(captured.rss.2 > 0);
        assert!(captured.rss.0 > 0);
        assert!(
            matches!(measure(Command::new("/bin/sh").args(["-c", "exit 23"]), snapshot), Err(Error::CommandFailed { status, .. }) if status.code() == Some(23))
        );
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn worker_report_limits_and_json_errors_preserve_caller_files() {
        let scratch = fixture_directory();
        let input = scratch.path().join("oversized.json");
        fs::File::create(&input)
            .unwrap()
            .set_len(REPORT_LIMIT + 1)
            .unwrap();
        let result = measure(
            Command::new("/bin/sh")
                .args(["-c", "cat \"$1\"", "large-worker"])
                .arg(&input),
            snapshot,
        );
        assert!(matches!(
            result,
            Err(Error::Artifact(
                ic_host_artifacts::artifact::ArtifactError::LimitExceeded { .. }
            ))
        ));
        assert_eq!(fs::metadata(input).unwrap().len(), REPORT_LIMIT + 1);
        assert!(matches!(
            measure(
                Command::new("/bin/sh").args(["-c", "printf 'not json'"]),
                snapshot
            ),
            Err(Error::Json(_))
        ));
    }

    #[test]
    #[cfg(unix)]
    fn preparation_commands_preserve_text_status_and_capture_failures() {
        assert_eq!(
            text_command(Command::new("/bin/sh").args(["-c", "printf '  ready\\n' "])).unwrap(),
            "ready"
        );
        assert!(matches!(
            text_command(Command::new("/bin/sh").args(["-c", "printf '\\377'"])),
            Err(Error::InvalidData("tool output is not UTF-8"))
        ));
        // Keep the producer alive until capture kills it: an immediately exiting
        // producer races group signalling on macOS and can retain a cleanup error.
        let Err(Error::Tool(error)) = text_command(
            Command::new("/bin/sh").args(["-c", "printf '%065537d' 0; exec /bin/sleep 30"]),
        ) else {
            panic!("expected bounded-output failure")
        };
        let ToolError::Execution(execution) = error.as_ref() else {
            panic!("expected execution failure")
        };
        assert!(matches!(
            execution.failure,
            ExecutionFailure::OutputLimit {
                stream: OutputStream::Stdout
            }
        ));
        assert_eq!(execution.evidence.stdout.len(), 64 * 1024);
        assert!(execution.evidence.stdout_truncated);
        assert!(
            execution.cleanup.is_none(),
            "cleanup evidence: {execution:?}"
        );
        assert_eq!(
            execution
                .evidence
                .status
                .as_ref()
                .and_then(std::os::unix::process::ExitStatusExt::signal),
            Some(9)
        );
        execute_command(Command::new("/bin/sh").args(["-c", "sleep 0.05"])).unwrap();
        let Err(Error::Tool(error)) =
            execute_command(Command::new("/bin/sh").args(["-c", "exit 23"]))
        else {
            panic!("expected exit failure")
        };
        assert!(
            matches!(error.as_ref(), ToolError::Execution(execution) if matches!(execution.failure, ExecutionFailure::ExitStatus))
        );
        assert_eq!(error.evidence().unwrap().status.unwrap().code(), Some(23));
        assert!(matches!(
            text_command(&mut Command::new("/missing/testkit-command")),
            Err(Error::Io(error)) if error.kind() == io::ErrorKind::NotFound
        ));
    }

    #[test]
    #[cfg(unix)]
    fn interruption_waits_for_worker_completion() {
        const CHILD_ENV: &str = "IC_TESTKIT_FIXTURE_BENCHMARK_INTERRUPT_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            // Keep signal-handler installation out of parallel test threads.
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "fixture_reuse_driver::tests::interruption_waits_for_worker_completion",
                    "--test-threads=1",
                ])
                .env(CHILD_ENV, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "interrupt worker failed: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let _handler = InterruptHandler::install().unwrap();
        let scratch = fixture_directory();
        let finished = scratch.path().join("finished");
        let result = measure(
            Command::new("/bin/sh")
                .args([
                    "-c",
                    "kill -INT \"$1\"; sleep 0.1; printf done > \"$2\"",
                    "interrupted-worker",
                ])
                .arg(std::process::id().to_string())
                .arg(&finished),
            snapshot,
        );
        assert!(matches!(result, Err(Error::Interrupted)));
        assert_eq!(fs::read(finished).unwrap(), b"done");

        for capture_text in [true, false] {
            let _command_handler = InterruptHandler::install().unwrap();
            let pid_path = scratch.path().join("preparation.pid");
            let mut command = Command::new("/bin/sh");
            command
                .args([
                    "-c",
                    "echo $$ > \"$2\"; kill -INT \"$1\"; sleep 30",
                    "interrupted-preparation",
                ])
                .arg(std::process::id().to_string())
                .arg(&pid_path);
            let result = if capture_text {
                text_command(&mut command).map(|_| ())
            } else {
                execute_command(&mut command).map(|_| ())
            };
            let Err(Error::Tool(error)) = result else {
                panic!("expected command cancellation")
            };
            let ToolError::Execution(execution) = error.as_ref() else {
                panic!("expected execution failure")
            };
            assert!(matches!(execution.failure, ExecutionFailure::Cancelled));
            assert!(execution.cleanup.is_none());
            let pid: libc::pid_t = fs::read_to_string(&pid_path)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            // SAFETY: signal zero only probes whether this reaped child exists.
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
        }

        let _publication_handler = InterruptHandler::install().unwrap();
        let report = scratch.path().join("report.json");
        fs::write(&report, b"previous complete report").unwrap();
        let before = fs::read_dir(scratch.path()).unwrap().count();
        let result = publish_report(&report, |file| {
            file.write_all(b"partial replacement")?;
            // SAFETY: the scoped handler is installed on this isolated test
            // process; raise delivers SIGINT to the calling thread.
            assert_eq!(unsafe { libc::raise(libc::SIGINT) }, 0);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Interrupted)));
        assert_eq!(fs::read(report).unwrap(), b"previous complete report");
        assert_eq!(fs::read_dir(scratch.path()).unwrap().count(), before);
    }
}
