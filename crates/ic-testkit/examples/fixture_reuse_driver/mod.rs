//! Repository-local operator tooling. Owns builds, worker runs and evidence files;
//! the library and workload retain server lifecycle and measurement semantics.

mod arguments;
mod report;
mod sampling;

use std::{
    collections::BTreeMap,
    error::Error as StdError,
    ffi::{OsStr, OsString},
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Seek as _, Write as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::{
    fs::{DirBuilderExt as _, OpenOptionsExt as _},
    process::CommandExt as _,
};

use ic_testkit::{
    artifacts::workspace_root_for,
    ic_host_artifacts::artifact::ArtifactError,
    ic_host_fs::read::{hash_file, read_opened_file},
    ic_host_process::tool::{ResolutionError, resolve_executable},
};
use serde_json::{Map, Value, json};

use arguments::{HELP, Options, parse_arguments};
use report::{MeasuredRun, Summary, summarize};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(10);
const REPORT_LIMIT: u64 = 16 * 1024 * 1024;
const WASM_LIMIT: u64 = 128 * 1024 * 1024;
static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub enum Error {
    InvalidArgument(String),
    InvalidData(&'static str),
    Io(io::Error),
    Json(serde_json::Error),
    Artifact(ArtifactError),
    Resolution(ResolutionError),
    CommandFailed {
        program: PathBuf,
        status: ExitStatus,
    },
    Interrupted,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid benchmark argument: {message}"),
            Self::InvalidData(message) => write!(f, "invalid benchmark data: {message}"),
            Self::Io(error) => error.fmt(f),
            Self::Json(error) => error.fmt(f),
            Self::Artifact(error) => error.fmt(f),
            Self::Resolution(error) => error.fmt(f),
            Self::CommandFailed { program, status } => {
                write!(f, "{} exited with {status}", program.display())
            }
            Self::Interrupted => f.write_str("benchmark interrupted"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Artifact(error) => Some(error),
            Self::Resolution(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
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

struct ScratchOutput {
    directory: PathBuf,
    file: File,
}

impl ScratchOutput {
    fn create() -> Result<Self, Error> {
        Self::create_in(&std::env::temp_dir(), None)
    }

    fn create_in(parent: &Path, reserved_name: Option<&OsStr>) -> Result<Self, Error> {
        loop {
            let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let name = format!(
                "ic-testkit-fixture-benchmark-{}-{sequence}",
                std::process::id()
            );
            // Do not allocate the destination itself, even on case-insensitive
            // filesystems or when an operator uses our temporary-name namespace.
            if reserved_name.is_some_and(|reserved| {
                reserved
                    .as_encoded_bytes()
                    .eq_ignore_ascii_case(name.as_bytes())
            }) {
                continue;
            }
            let directory = parent.join(name);
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            builder.mode(0o700);
            match builder.create(&directory) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            match options.open(directory.join("worker.json")) {
                Ok(file) => return Ok(Self { directory, file }),
                Err(error) => {
                    let _ = fs::remove_dir(&directory);
                    return Err(error.into());
                }
            }
        }
    }
}

impl Drop for ScratchOutput {
    fn drop(&mut self) {
        // Only this driver's private scratch is removed, never Cargo artifacts
        // or the operator's requested report. Open handles close normally.
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn publish_report(
    output: &Path,
    write: impl FnOnce(&mut File) -> Result<(), Error>,
) -> Result<(), Error> {
    check_interrupted()?;
    let name = output.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "report path requires a file name",
        )
    })?;
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    // The private directory is beside the report, so rename never crosses
    // filesystems. The destination is untouched until this final commit point.
    let mut scratch = ScratchOutput::create_in(parent, Some(name))?;
    write(&mut scratch.file)?;
    scratch.file.sync_all()?;
    check_interrupted()?;
    fs::rename(scratch.directory.join("worker.json"), output)?;
    Ok(())
}

struct CapturedRun {
    raw: Map<String, Value>,
    rss: (u64, usize, usize),
}

fn measure(command: &mut Command) -> Result<CapturedRun, Error> {
    check_interrupted()?;
    let mut scratch = ScratchOutput::create()?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(scratch.file.try_clone()?));
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn()?;
    let observed = (|| -> Result<_, Error> {
        let mut peak_rss = 0;
        let mut peak_processes = 0;
        let mut count = 0;
        while child.try_wait()?.is_none() && !INTERRUPTED.load(Ordering::Relaxed) {
            let (resident, processes) = sampling::tree_rss(child.id(), &sampling::snapshot()?)?;
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
    scratch.file.rewind()?;
    let bytes = read_opened_file(
        scratch.file.try_clone()?,
        usize::try_from(REPORT_LIMIT).expect("report limit fits supported hosts"),
    )
    .map_err(Error::Artifact)?;
    let raw = serde_json::from_slice(&bytes)?;
    Ok(CapturedRun { raw, rss })
}

fn text_command(command: &mut Command) -> Result<String, Error> {
    check_interrupted()?;
    let output = command
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;
    check_interrupted()?;
    if !output.status.success() {
        return Err(Error::CommandFailed {
            program: command.get_program().into(),
            status: output.status,
        });
    }
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|_| Error::InvalidData("tool output is not UTF-8"))
}

fn build_command(command: &mut Command) -> Result<(), Error> {
    check_interrupted()?;
    let status = command.stdin(Stdio::null()).status()?;
    check_interrupted()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed {
            program: command.get_program().into(),
            status,
        })
    }
}

fn pocket_ic_16(version: &str) -> bool {
    let Some(numbers) = version.strip_prefix("pocket-ic-server 16.") else {
        return false;
    };
    let fields = numbers.split('.').collect::<Vec<_>>();
    fields.len() == 2
        && fields
            .iter()
            .all(|field| !field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit()))
}

fn print_summary(summaries: &BTreeMap<String, Summary>, labels: &[String]) -> Result<(), Error> {
    println!(
        "Mode        work ms   incl. setup/drop ms   tasks/s   restore mean ms   wait mean ms   peak RSS MiB"
    );
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
        println!(
            "{label:10} {:9.2} {:21.2} {:9.2} {restore:>17} {:14.2} {:14.2}",
            work.p50,
            fixture.p50,
            rate.p50,
            wait.mean,
            peak.p50 / 1024.0 / 1024.0
        );
    }
    Ok(())
}

struct Inputs {
    root: PathBuf,
    server: PathBuf,
    worker: PathBuf,
    wasm: PathBuf,
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
    let root = workspace_root_for(env!("CARGO_MANIFEST_DIR"))?;
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
    if !pocket_ic_16(&server_version) {
        return Err(Error::InvalidData("expected pocket-ic-server 16.x.y"));
    }
    let cargo = tool_program(std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")))?;
    let rustc = std::env::var_os("RUSTC").map(tool_program).transpose()?;
    let metadata: Value = serde_json::from_str(&text_command(
        cargo_command(&cargo, &root, rustc.as_deref()).args([
            "metadata",
            "--locked",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ]),
    )?)?;
    let target = metadata["target_directory"]
        .as_str()
        .filter(|path| Path::new(path).is_absolute())
        .ok_or(Error::InvalidData(
            "Cargo metadata lacks absolute target directory",
        ))?;
    build_command(cargo_command(&cargo, &root, rustc.as_deref()).args([
        "build",
        "--locked",
        "--offline",
        "-p",
        "ic-testkit",
        "--profile",
        &options.profile,
        "--example",
        "fixture_reuse_benchmark",
    ]))?;
    build_command(cargo_command(&cargo, &root, rustc.as_deref()).args([
        "build",
        "--locked",
        "--offline",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        "ic_testkit_perf_probe",
    ]))?;
    let worker = Path::new(target)
        .join(if options.profile == "dev" {
            "debug"
        } else {
            "release"
        })
        .join("examples/fixture_reuse_benchmark");
    let wasm = Path::new(target).join("wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm");
    let provenance = provenance(
        &root,
        options,
        &server,
        &server_version,
        &wasm,
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

fn provenance(
    root: &Path,
    options: &Options,
    server: &Path,
    server_version: &str,
    wasm: &Path,
    rustc: Option<&Path>,
) -> Result<Value, Error> {
    // Capture the selected build and source facts before running cases.
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
        "revision": text_command(Command::new("git").args(["rev-parse", "HEAD"]).current_dir(root))?,
        "working_tree_dirty": !text_command(Command::new("git").args(["status", "--porcelain"]).current_dir(root))?.is_empty(),
        "host_profile": options.profile, "wasm_profile": "dev", "wasm_sha256": hash_file(wasm, WASM_LIMIT).map_err(Error::Artifact)?.sha256.to_string(),
        "rss_sample_interval_ms": SAMPLE_INTERVAL.as_millis(), "rss_sampler": sampling::source(),
        "iterations": options.iterations.get(), "workers": options.workers.get(), "repeats": options.repeats.get(),
        "state_bytes_per_canister": options.state_bytes, "modes": options.modes.iter().map(|mode| mode.as_str()).collect::<Vec<_>>(),
        "pool_capacities": if options.modes.contains(&arguments::Mode::Pooled) { options.capacities.iter().map(|capacity| capacity.get()).collect::<Vec<_>>() } else { vec![] },
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
            let captured = measure(
                Command::new(&inputs.worker)
                    .args([
                        case.mode.as_str().to_owned(),
                        case.capacity.to_string(),
                        options.iterations.to_string(),
                        options.workers.to_string(),
                        options.state_bytes.to_string(),
                    ])
                    .arg(&inputs.wasm)
                    .arg(&inputs.server)
                    .current_dir(&inputs.root),
            )?;
            let measured = MeasuredRun::from_worker(captured.raw, &options, case, captured.rss)?;
            runs.get_mut(&case.label)
                .expect("case owns run list")
                .push(measured);
        }
    }
    let summaries = runs
        .iter()
        .map(|(label, runs)| (label.clone(), summarize(runs)))
        .collect::<BTreeMap<_, _>>();
    check_interrupted()?;
    if let Some(output) = &options.output {
        let document = json!({"format": "ic-testkit-fixture-benchmark-v1", "provenance": inputs.provenance, "runs": runs, "summary": summaries});
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
        Error, REPORT_LIMIT, ScratchOutput, measure, pocket_ic_16, prepare, publish_report,
    };

    #[cfg(unix)]
    use super::InterruptHandler;
    use serde_json::json;
    use std::{
        ffi::OsString,
        fs,
        io::{self, Read as _, Write as _},
        path::{Path, PathBuf},
        process::Command,
    };

    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt as _, symlink};

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
        } else {
            let inputs = prepare(&options).unwrap();
            assert_eq!(inputs.provenance["rustc"], "fixture rustc");
            let trace = fs::read_to_string(root.join("trace")).unwrap();
            let calls = trace.lines().collect::<Vec<_>>();
            assert!(!calls.is_empty());
            assert_eq!(calls.len() % 3, 0);
            for call in calls.chunks_exact(3) {
                if call[2].contains("locate-project") {
                    assert_eq!(call[1], root.to_str().unwrap());
                    assert!(call[2].contains("--offline"));
                    assert!(call[2].contains("--workspace"));
                    continue;
                }
                assert_eq!(call[1], inputs.root.to_str().unwrap());
                if call[0].ends_with("/cargo") || call[0] == "cargo" {
                    assert!(call[2].contains("--locked"));
                    assert!(call[2].contains("--offline"));
                } else {
                    assert!(call[0].ends_with("/rustc") || call[0] == "rustc");
                    assert_eq!(call[2], "--version");
                }
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn standalone_tools_preserve_proxy_names_and_workspace_context() {
        const ROOT_ENV: &str = "IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT";
        const MISSING_ENV: &str = "IC_TESTKIT_BENCHMARK_TOOL_EXPECT_MISSING";
        if let Some(root) = std::env::var_os(ROOT_ENV) {
            assert_prepared_tool_context(
                &PathBuf::from(root),
                std::env::var_os(MISSING_ENV).is_some(),
            );
            return;
        }

        let scratch = ScratchOutput::create().unwrap();
        let root = scratch.directory.canonicalize().unwrap();
        let bin = root.join("bin");
        fs::create_dir(&bin).unwrap();
        let target = root.join("target");
        let wasm = target.join("wasm32-unknown-unknown/debug/ic_testkit_perf_probe.wasm");
        fs::create_dir_all(wasm.parent().unwrap()).unwrap();
        fs::write(wasm, b"\0asm\x01\0\0\0").unwrap();
        fs::write(
            root.join("metadata.json"),
            serde_json::to_vec(&json!({"target_directory": target})).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("workspace.json"),
            serde_json::to_vec(&json!({"root":
                ic_testkit::artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"))
                    .unwrap().join("Cargo.toml")
            }))
            .unwrap(),
        )
        .unwrap();
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
      metadata) cat "$IC_TESTKIT_BENCHMARK_TOOL_TEST_ROOT/metadata.json" ;;
      build) ;;
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
        for (cargo, rustc, missing) in [
            (None, None, false),
            (
                Some(bin.join("cargo").into_os_string()),
                Some(bin.join("rustc").into_os_string()),
                false,
            ),
            (
                Some(OsString::from("bin/cargo")),
                Some(OsString::from("bin/rustc")),
                false,
            ),
            (
                Some(root.join("missing-cargo").into_os_string()),
                None,
                true,
            ),
        ] {
            let mut child = Command::new(std::env::current_exe().unwrap());
            child.args(["--exact", "fixture_reuse_driver::tests::standalone_tools_preserve_proxy_names_and_workspace_context", "--test-threads=1"])
                .current_dir(&root)
                .env(ROOT_ENV, &root)
                .env("PATH", &path)
                .env_remove("CARGO")
                .env_remove("RUSTC")
                .env_remove(MISSING_ENV);
            if let Some(cargo) = cargo {
                child.env("CARGO", cargo);
            }
            if let Some(rustc) = rustc {
                child.env("RUSTC", rustc);
            }
            if missing {
                child.env(MISSING_ENV, "1");
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
        let scratch = ScratchOutput::create().unwrap();
        let report = scratch.directory.join("report.json");
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
        assert_eq!(fs::read_dir(&scratch.directory).unwrap().count(), 2);
    }

    #[test]
    fn report_rename_failure_preserves_destination_and_cleans_scratch() {
        let scratch = ScratchOutput::create().unwrap();
        let report = scratch.directory.join("report.json");
        fs::create_dir(&report).unwrap();
        let retained = report.join("retained report.json");
        fs::write(&retained, b"caller-owned evidence").unwrap();
        assert!(matches!(
            publish_report(&report, |file| Ok(file.write_all(b"replacement")?)),
            Err(Error::Io(_))
        ));
        assert_eq!(fs::read(retained).unwrap(), b"caller-owned evidence");
        assert_eq!(fs::read_dir(&scratch.directory).unwrap().count(), 2);
        assert!(matches!(
            publish_report(&scratch.directory.join("missing/report.json"), |_| {
                panic!("a missing parent must fail before writing")
            }),
            Err(Error::Io(error)) if error.kind() == io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn report_replacement_keeps_old_readers_and_publishes_complete_json() {
        let scratch = ScratchOutput::create().unwrap();
        let report = scratch.directory.join("report with spaces.json");
        fs::write(&report, b"previous complete report").unwrap();
        let mut old_reader = fs::File::open(&report).unwrap();
        let retained = scratch.directory.join("retained.json");
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
        assert_eq!(fs::read_dir(&scratch.directory).unwrap().count(), 3);
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&report).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    #[cfg(unix)]
    fn report_replacement_preserves_symlink_target() {
        let scratch = ScratchOutput::create().unwrap();
        let target = scratch.directory.join("original.json");
        let report = scratch.directory.join("linked.json");
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
    }

    #[test]
    fn exact_server_major_is_required() {
        for version in ["pocket-ic-server 16.0.0", "pocket-ic-server 16.12.34"] {
            assert!(pocket_ic_16(version));
        }
        for version in [
            "pocket-ic-server 15.0.0",
            "pocket-ic-server 16.0.0-beta",
            "pocket-ic-server 16.x.0",
            "pocket-ic-server 16.0.0 extra",
        ] {
            assert!(!pocket_ic_16(version));
        }
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn native_measurement_keeps_large_reports_and_rejects_failed_workers() {
        let scratch = ScratchOutput::create().unwrap();
        let input = scratch.directory.join("input.json");
        let padding = "a".repeat(128 * 1024);
        fs::write(
            &input,
            serde_json::to_vec(&json!({"padding": padding})).unwrap(),
        )
        .unwrap();
        let captured = measure(
            Command::new("/bin/sh")
                .args(["-c", "sleep 0.1; cat \"$1\"", "test-worker"])
                .arg(&input),
        )
        .unwrap();
        assert_eq!(captured.raw["padding"], padding);
        assert!(captured.rss.2 > 0);
        assert!(captured.rss.0 > 0);
        assert!(
            matches!(measure(Command::new("/bin/sh").args(["-c", "exit 23"])), Err(Error::CommandFailed { status, .. }) if status.code() == Some(23))
        );
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn worker_report_limits_and_json_errors_preserve_caller_files() {
        let scratch = ScratchOutput::create().unwrap();
        let input = scratch.directory.join("oversized.json");
        fs::File::create(&input)
            .unwrap()
            .set_len(REPORT_LIMIT + 1)
            .unwrap();
        let result = measure(
            Command::new("/bin/sh")
                .args(["-c", "cat \"$1\"", "large-worker"])
                .arg(&input),
        );
        assert!(matches!(
            result,
            Err(Error::Artifact(
                ic_testkit::ic_host_artifacts::artifact::ArtifactError::LimitExceeded { .. }
            ))
        ));
        assert_eq!(fs::metadata(input).unwrap().len(), REPORT_LIMIT + 1);
        assert!(matches!(
            measure(Command::new("/bin/sh").args(["-c", "printf 'not json'"])),
            Err(Error::Json(_))
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
        let scratch = ScratchOutput::create().unwrap();
        let finished = scratch.directory.join("finished");
        let result = measure(
            Command::new("/bin/sh")
                .args([
                    "-c",
                    "kill -INT \"$1\"; sleep 0.1; printf done > \"$2\"",
                    "interrupted-worker",
                ])
                .arg(std::process::id().to_string())
                .arg(&finished),
        );
        assert!(matches!(result, Err(Error::Interrupted)));
        assert_eq!(fs::read(finished).unwrap(), b"done");

        let _publication_handler = InterruptHandler::install().unwrap();
        let report = scratch.directory.join("report.json");
        fs::write(&report, b"previous complete report").unwrap();
        let before = fs::read_dir(&scratch.directory).unwrap().count();
        let result = publish_report(&report, |file| {
            file.write_all(b"partial replacement")?;
            // SAFETY: the scoped handler is installed on this isolated test
            // process; raise delivers SIGINT to the calling thread.
            assert_eq!(unsafe { libc::raise(libc::SIGINT) }, 0);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Interrupted)));
        assert_eq!(fs::read(report).unwrap(), b"previous complete report");
        assert_eq!(fs::read_dir(&scratch.directory).unwrap().count(), before);
    }
}
