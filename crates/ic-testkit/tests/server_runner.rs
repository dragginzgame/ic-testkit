#![cfg(unix)]

mod support;

use ic_host_process::testing::write_executable_script;

use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct Fixture(tempfile::TempDir);
impl Fixture {
    fn new() -> Self {
        let fixture = Self(support::unique_temp_directory("ic-testkit-server-runner-"));
        write_executable_script(
            &fixture.0.path().join("server"),
            format!(
                "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'pocket-ic-server {}\\n'; exit 0; fi\nprintf '%s\\n' \"$$\" > \"$SERVER_PID_FILE\"\nif [ \"$1\" = --hard-ttl ]; then printf '%s' \"$2\" > \"$TTL_FILE\"; shift 2; fi\nif [ \"$1\" = --ttl ]; then printf '%s' \"$2\" > \"$IDLE_TTL_FILE\"; shift 2; fi\n[ \"$1\" = --port-file ] || exit 99\ndd if=/dev/zero bs=1024 count=20 2>/dev/null\nprintf stdout-end\ndd if=/dev/zero bs=1024 count=20 >&2 2>/dev/null\nprintf stderr-end >&2\nprintf '34567\\n' > \"$2\"\nexec sleep 30\n",
                ic_testkit::pocket_ic::LATEST_SERVER_VERSION
            ),
        ).expect("write executable test script");
        fixture
    }
    fn runner(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"));
        command
            .env_remove("IC_TESTKIT_POCKET_IC_URL")
            .env("POCKET_IC_BIN", self.0.path().join("server"))
            .env("SERVER_PID_FILE", self.0.path().join("server.pid"))
            .env("TTL_FILE", self.0.path().join("ttl"))
            .env("IDLE_TTL_FILE", self.0.path().join("idle-ttl"));
        command
    }
    fn pid(&self, file: &str) -> u32 {
        fs::read_to_string(self.0.path().join(file))
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }
}
fn stopped(pid: u32) -> bool {
    let output = Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "stat="])
        .output()
        .unwrap();
    assert!(output.status.success() || output.status.code() == Some(1));
    let state = String::from_utf8(output.stdout).unwrap();
    state.trim().is_empty() || state.trim().starts_with('Z')
}

fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "runner condition timed out");
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn owned_server_exports_url_passes_ttl_and_preserves_command_exit() {
    let fixture = Fixture::new();
    let output = fixture
        .runner()
        .args([
            "run",
            "--ttl",
            "17",
            "--idle-ttl",
            "42",
            "--",
            "/bin/sh",
            "-c",
            "printf '%s' \"$IC_TESTKIT_POCKET_IC_URL\"; exit 37",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(37),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"http://127.0.0.1:34567/");
    assert_eq!(
        fs::read_to_string(fixture.0.path().join("ttl")).unwrap(),
        "17"
    );
    assert_eq!(
        fs::read_to_string(fixture.0.path().join("idle-ttl")).unwrap(),
        "42"
    );
    assert!(stopped(fixture.pid("server.pid")));
}

#[test]
fn runner_retains_complete_server_streams_after_command_failure() {
    let fixture = Fixture::new();
    let stdout = fixture.0.path().join("stdout");
    let stderr = fixture.0.path().join("stderr");
    let output = fixture
        .runner()
        .current_dir(fixture.0.path())
        .args(["run", "--server-stdout"])
        .arg("stdout")
        .arg("--server-stderr")
        .arg("stderr")
        .args(["--", "/bin/sh", "-c", "exit 37"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(37),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"");
    assert!(fs::read(stdout).unwrap().ends_with(b"stdout-end"));
    assert!(fs::read(stderr).unwrap().ends_with(b"stderr-end"));
    assert!(stopped(fixture.pid("server.pid")));
}

#[test]
fn external_server_precedence_borrows_ownership_and_rejects_owned_ttl() {
    let fixture = Fixture::new();
    let output = fixture
        .runner()
        .env("IC_TESTKIT_POCKET_IC_URL", "http://127.0.0.1:45678/")
        .env("POCKET_IC_BIN", "/missing/server")
        .args([
            "run",
            "--",
            "/bin/sh",
            "-c",
            "printf '%s' \"$IC_TESTKIT_POCKET_IC_URL\"",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"http://127.0.0.1:45678/");
    assert!(!fixture.0.path().join("server.pid").exists());
    let output = fixture
        .runner()
        .env("IC_TESTKIT_POCKET_IC_URL", "http://127.0.0.1:45678/")
        .args(["run", "--ttl", "1", "--", "/bin/sh", "-c", "exit 0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--ttl requires an owned server"));
    let output = fixture
        .runner()
        .env("IC_TESTKIT_POCKET_IC_URL", "http://127.0.0.1:45678/")
        .args(["run", "--idle-ttl", "120", "--", "/bin/sh", "-c", "exit 99"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.0.path().join("server.pid").exists());
}

#[test]
fn invalid_configuration_and_version_failure_never_run_the_command() {
    let fixture = Fixture::new();
    let marker = fixture.0.path().join("command-ran");
    for args in [
        vec!["run", "--ttl", "0", "--", "/bin/sh"],
        vec!["run", "--idle-ttl", "0", "--", "/bin/sh"],
        vec!["run", "--idle-ttl", "1", "--idle-ttl", "2", "--", "/bin/sh"],
        vec!["run", "--"],
        vec!["run", "--server-stdout", "out", "--", "/bin/sh"],
        vec![
            "run",
            "--server-stdout",
            "out",
            "--server-stdout",
            "other",
            "--",
            "/bin/sh",
        ],
        vec![
            "run",
            "--startup-timeout",
            "1",
            "--startup-timeout",
            "2",
            "--",
            "/bin/sh",
        ],
    ] {
        assert_eq!(
            fixture.runner().args(args).status().unwrap().code(),
            Some(2)
        );
    }
    for selected in [None, Some("")] {
        let mut command = fixture.runner();
        command.env_remove("POCKET_IC_BIN");
        if let Some(selected) = selected {
            command.env("IC_TESTKIT_POCKET_IC_URL", selected);
        }
        let output = command
            .args(["run", "--", "/bin/sh", "-c", "touch \"$1\"", "test"])
            .arg(&marker)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    write_executable_script(
        &fixture.0.path().join("failed-version"),
        format!(
            "#!/bin/sh\nprintf 'pocket-ic-server {}\\n'\nexit 23\n",
            ic_testkit::pocket_ic::LATEST_SERVER_VERSION
        ),
    )
    .expect("write executable test script");
    let output = fixture
        .runner()
        .env("POCKET_IC_BIN", fixture.0.path().join("failed-version"))
        .args(["run", "--", "/bin/sh", "-c", "touch \"$1\"", "test"])
        .arg(&marker)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!marker.exists());
    assert!(!fixture.0.path().join("server.pid").exists());
}

struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn command_spawn_failure_releases_the_started_server() {
    let fixture = Fixture::new();
    let output = fixture
        .runner()
        .args(["run", "--", "/missing/command"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(stopped(fixture.pid("server.pid")));
}

#[test]
fn command_completion_stops_remaining_owned_descendants() {
    let fixture = Fixture::new();
    let output = fixture
        .runner()
        .env(
            "DESCENDANT_PID_FILE",
            fixture.0.path().join("descendant.pid"),
        )
        .args([
            "run",
            "--",
            "/bin/sh",
            "-c",
            "sleep 30 & printf '%s' \"$!\" > \"$DESCENDANT_PID_FILE\"; exit 19",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(19));
    wait_until(|| stopped(fixture.pid("descendant.pid")));
    assert!(stopped(fixture.pid("server.pid")));
}

#[test]
fn server_exit_returns_diagnostics_and_stops_the_pending_command_group() {
    use ic_testkit::pic::{PocketIcStartupConfig, PocketIcStartupFailure};

    let fixture = Fixture::new();
    let binary = fixture.0.path().join("server");
    write_executable_script(
        &binary,
        "#!/bin/sh\nprintf '%s' \"$$\" > \"$0.pid\"\nprintf '34567\\n' > \"$2\"\nwhile [ ! -s \"$0.descendant.pid\" ]; do sleep 0.02; done\nprintf 'server stopped\\n'\nprintf 'failure detail\\n' >&2\nexit 42\n",
    ).expect("write executable test script");
    let started = Instant::now();
    let error = PocketIcStartupConfig::spawn(&binary, Duration::from_secs(2))
        .run_command(
            Command::new("/bin/sh")
                .env("SERVER_SCRIPT", &binary)
                .args([
                    "-c",
                    "printf '%s' \"$$\" > \"$SERVER_SCRIPT.command.pid\"; sleep 30 & printf '%s' \"$!\" > \"$SERVER_SCRIPT.descendant.pid\"; wait",
                ]),
            || started.elapsed() > Duration::from_secs(5),
        )
        .unwrap_err();
    let PocketIcStartupFailure::ServerExited {
        server_binary,
        status,
        ..
    } = error.failure()
    else {
        panic!("expected the owned server exit, got {error:?}");
    };
    assert_eq!(server_binary, &binary);
    assert_eq!(status.code(), Some(42));
    assert_eq!(error.output().stdout(), "server stopped\n");
    assert_eq!(error.output().stderr(), "failure detail\n");
    for file in ["server.pid", "server.command.pid", "server.descendant.pid"] {
        wait_until(|| stopped(fixture.pid(file)));
    }
}

#[test]
#[ignore = "requires a prepared POCKET_IC_BIN; launches a real server and instance"]
fn real_server_runs_a_separate_process_using_environment_startup() {
    let fixture = Fixture::new();
    let binary = std::env::var_os("POCKET_IC_BIN").expect("prepare POCKET_IC_BIN");
    let output = fixture
        .runner()
        .env("POCKET_IC_BIN", binary)
        .args(["run", "--ttl", "60", "--"])
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "runner_environment_worker"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "invoked by the real server runner test in a separate process"]
fn runner_environment_worker() {
    use ic_testkit::pic::{PocketIcBuilder, PocketIcStartupConfig};
    if std::env::var_os("IC_TESTKIT_TEST_PRECLIENT_DELAY").is_some() {
        thread::sleep(Duration::from_secs(65));
    }
    assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_some());
    let pic = PocketIcStartupConfig::from_env(Duration::from_secs(30))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    assert_eq!(
        pic.instance.get_server_url().as_str(),
        std::env::var("IC_TESTKIT_POCKET_IC_URL").unwrap()
    );
    drop(pic);
}

#[test]
#[ignore = "requires explicit ic-testkit-server setup in .tools/ic-testkit-server"]
fn provisioned_cli_runs_a_real_instance_without_consumer_server_selection() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"))
        .current_dir(workspace)
        .env_remove("POCKET_IC_BIN")
        .env_remove("IC_TESTKIT_POCKET_IC_URL")
        .args(["run", "--ttl", "60", "--"])
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "runner_environment_worker"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires explicit PocketIC setup; exercises 65 seconds of pre-client work"]
fn managed_idle_ttl_allows_preclient_work_beyond_server_default() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let evidence = tempfile::Builder::new()
        .prefix("idle-lifetime-")
        .tempdir_in(workspace.join("target"))
        .unwrap()
        .keep();
    eprintln!("Idle lifetime evidence retained at {}", evidence.display());
    let output = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"))
        .current_dir(workspace)
        .env_remove("POCKET_IC_BIN")
        .env_remove("IC_TESTKIT_POCKET_IC_URL")
        .env("IC_TESTKIT_TEST_PRECLIENT_DELAY", "1")
        .args([
            "run",
            "--ttl",
            "180",
            "--idle-ttl",
            "120",
            "--server-stdout",
        ])
        .arg(evidence.join("server.stdout"))
        .arg("--server-stderr")
        .arg(evidence.join("server.stderr"))
        .arg("--")
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "runner_environment_worker"])
        .output()
        .unwrap();
    fs::write(evidence.join("command.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("command.stderr"), &output.stderr).unwrap();
    fs::write(evidence.join("status"), output.status.to_string()).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn interrupted_setup_terminates_downloader_descendants_and_retains_failure() {
    use ic_host_process::tool::{ExecutionError, ExecutionEvidence, ExecutionFailure, ToolError};

    struct DownloadGroup(u32);
    impl Drop for DownloadGroup {
        fn drop(&mut self) {
            if !stopped(self.0) {
                // SAFETY: the fixture downloader owns this process group.
                unsafe { libc::kill(-i32::try_from(self.0).unwrap(), libc::SIGKILL) };
            }
        }
    }

    let expected = ToolError::Execution(Box::new(ExecutionError {
        failure: ExecutionFailure::Cancelled,
        evidence: ExecutionEvidence::default(),
        cleanup: None,
    }))
    .to_string();
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        for block_stderr in [false, true] {
            let fixture = Fixture::new();
            let store = fixture.0.path().canonicalize().unwrap().join("store");
            let obstruction = if block_stderr {
                "/bin/mkdir \"${2%/*}/download.stderr\""
            } else {
                ""
            };
            write_executable_script(
                &fixture.0.path().join("curl"),
                format!(
                    "#!/bin/sh\nwhile [ \"$1\" != --output ]; do shift; done\nprintf partial > \"$2\"\n{obstruction}\nprintf 'download diagnostic' >&2\nprintf '%s\\n' \"$$\" > \"$CURL_PID_FILE\"\n/bin/sleep 30 &\nprintf '%s\\n' \"$!\" > \"$DESCENDANT_PID_FILE\"\nwait\n"
                ),
            ).expect("write executable test script");
            let diagnostic = fixture.0.path().join("diagnostic");
            let mut runner = Running(
                Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"))
                    .env("PATH", fixture.0.path())
                    .env("CURL_PID_FILE", fixture.0.path().join("curl.pid"))
                    .env(
                        "DESCENDANT_PID_FILE",
                        fixture.0.path().join("descendant.pid"),
                    )
                    .args(["setup", "--directory"])
                    .arg(&store)
                    .stdout(Stdio::null())
                    .stderr(fs::File::create(&diagnostic).unwrap())
                    .spawn()
                    .unwrap(),
            );
            wait_until(|| {
                fs::read_to_string(fixture.0.path().join("curl.pid"))
                    .is_ok_and(|pid| !pid.is_empty())
            });
            let downloader = DownloadGroup(fixture.pid("curl.pid"));
            wait_until(|| {
                fs::read_to_string(fixture.0.path().join("descendant.pid"))
                    .is_ok_and(|pid| !pid.is_empty())
            });
            let descendant = fixture.pid("descendant.pid");
            // SAFETY: this live CLI child is owned by the test.
            assert_eq!(
                unsafe { libc::kill(i32::try_from(runner.0.id()).unwrap(), signal) },
                0
            );
            let mut status = None;
            wait_until(|| {
                status = runner.0.try_wait().unwrap();
                status.is_some()
            });
            assert_eq!(status.unwrap().code(), Some(128 + signal));
            wait_until(|| stopped(downloader.0) && stopped(descendant));
            let attempts = fs::read_dir(&store)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>();
            assert_eq!(attempts.len(), 1);
            let attempt = &attempts[0];
            assert_eq!(fs::read(attempt.join("archive.gz")).unwrap(), b"partial");
            assert_eq!(
                fs::read_to_string(attempt.join("failure.txt")).unwrap(),
                expected
            );
            assert!(String::from_utf8_lossy(&fs::read(diagnostic).unwrap()).contains(&expected));
            assert_eq!(attempt.join("download.stderr").is_dir(), block_stderr);
            assert!(attempt.join("download.stderr").exists());
            assert!(!attempt.join("pocket-ic").exists());
        }
    }
}

#[test]
fn failed_download_retains_partial_archive_and_original_process_failure() {
    use ic_host_process::tool::{OutputLimit, OutputLimits, capture_group_command};

    for (ending, block_stderr) in [
        ("exit 23", false),
        ("kill -TERM \"$$\"", false),
        ("exit 23", true),
        ("kill -TERM \"$$\"", true),
    ] {
        let fixture = Fixture::new();
        let store = fixture.0.path().canonicalize().unwrap().join("store");
        let obstruction = if block_stderr {
            "/bin/mkdir \"${2%/*}/download.stderr\""
        } else {
            ""
        };
        write_executable_script(
            &fixture.0.path().join("curl"),
            format!(
                "#!/bin/sh\nwhile [ \"$1\" != --output ]; do shift; done\nprintf partial > \"$2\"\n{obstruction}\nprintf 'download diagnostic' >&2\n{ending}\n"
            ),
        ).expect("write executable test script");
        let expected = capture_group_command(
            Command::new("/bin/sh").args(["-c", ending]),
            OutputLimits {
                stdout: OutputLimit::Terminate(128),
                stderr: OutputLimit::Terminate(128),
                timeout: Some(Duration::from_secs(5)),
            },
        )
        .unwrap()
        .require_success()
        .unwrap_err()
        .to_string();
        for count in 1..=2 {
            let output = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"))
                .env("PATH", fixture.0.path())
                .args(["setup", "--directory"])
                .arg(&store)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, [] as [u8; 0]);
            assert!(String::from_utf8_lossy(&output.stderr).contains(&expected));
            let attempts = fs::read_dir(&store)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>();
            assert_eq!(attempts.len(), count);
            for attempt in attempts {
                assert_eq!(fs::read(attempt.join("archive.gz")).unwrap(), b"partial");
                if block_stderr {
                    assert!(attempt.join("download.stderr").is_dir());
                    let record_error = ic_host_fs::durable::write_bytes(
                        &attempt.join("download.stderr"),
                        b"download diagnostic",
                    )
                    .unwrap_err();
                    assert!(
                        String::from_utf8_lossy(&output.stderr).contains(&record_error.to_string())
                    );
                } else {
                    assert_eq!(
                        fs::read(attempt.join("download.stderr")).unwrap(),
                        b"download diagnostic"
                    );
                }
                assert_eq!(
                    fs::read_to_string(attempt.join("failure.txt")).unwrap(),
                    expected
                );
                assert!(!attempt.join("pocket-ic").exists());
            }
        }
    }
}

#[test]
fn successful_download_stops_when_stderr_cannot_be_recorded() {
    let fixture = Fixture::new();
    let store = fixture.0.path().canonicalize().unwrap().join("store");
    write_executable_script(
        &fixture.0.path().join("curl"),
        "#!/bin/sh\nwhile [ \"$1\" != --output ]; do shift; done\nprintf partial > \"$2\"\n/bin/mkdir \"${2%/*}/download.stderr\"\nprintf 'download diagnostic' >&2\n",
    ).expect("write executable test script");
    let output = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"))
        .env("PATH", fixture.0.path())
        .args(["setup", "--directory"])
        .arg(&store)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, [] as [u8; 0]);
    let attempts = fs::read_dir(&store)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    assert_eq!(attempts.len(), 1);
    let attempt = &attempts[0];
    assert_eq!(fs::read(attempt.join("archive.gz")).unwrap(), b"partial");
    let expected =
        ic_host_fs::durable::write_bytes(&attempt.join("download.stderr"), b"download diagnostic")
            .unwrap_err();
    assert_eq!(
        fs::read_to_string(attempt.join("failure.txt")).unwrap(),
        expected.to_string()
    );
    assert!(!attempt.join("pocket-ic").exists());
}

#[test]
fn line_break_store_paths_are_refused_before_setup_or_launch() {
    for line_break in ['\n', '\r'] {
        let fixture = Fixture::new();
        let marker = fixture.0.path().join("download-ran");
        write_executable_script(
            &fixture.0.path().join("curl"),
            "#!/bin/sh\nprintf ran > \"$DOWNLOAD_MARKER\"\nexit 23\n",
        )
        .expect("write executable test script");
        let absolute = fixture.0.path().canonicalize().unwrap();
        let ancestor = absolute.join(format!("ancestor{line_break}"));
        fs::create_dir(&ancestor).unwrap();
        for (cwd, directory) in [
            (
                absolute.clone(),
                absolute.join(format!("store{line_break}")),
            ),
            (ancestor, PathBuf::from("store")),
        ] {
            for action in ["setup", "check", "run"] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"));
                command
                    .current_dir(&cwd)
                    .env_remove("POCKET_IC_BIN")
                    .env_remove("IC_TESTKIT_POCKET_IC_URL")
                    .env("PATH", &absolute)
                    .env("DOWNLOAD_MARKER", &marker)
                    .args([action, "--directory"])
                    .arg(&directory);
                if action == "run" {
                    command.args(["--", "/bin/sh", "-c", "exit 99"]);
                }
                let output = command.output().unwrap();
                assert_eq!(output.status.code(), Some(1));
                assert_eq!(output.stdout, [] as [u8; 0]);
                assert!(!marker.exists(), "invalid directory reached downloader");
                assert!(!cwd.join(&directory).exists());
                assert!(String::from_utf8_lossy(&output.stderr).contains("line breaks"));
            }
        }
    }
}

#[test]
fn offline_check_and_run_refuse_missing_setup_without_executing_tools() {
    let fixture = Fixture::new();
    let missing = fixture.0.path().join("missing");
    for action in ["check", "run"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"));
        command
            .env_remove("POCKET_IC_BIN")
            .env_remove("IC_TESTKIT_POCKET_IC_URL")
            .env("PATH", &missing)
            .arg(action)
            .arg("--directory")
            .arg(&missing);
        if action == "run" {
            command.args(["--", "/bin/sh", "-c", "exit 99"]);
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!missing.exists());
    }
}

#[test]
fn interruption_terminates_the_owned_server_command_and_descendants() {
    let fixture = Fixture::new();
    let mut command = fixture.runner();
    command
        .env("COMMAND_PID_FILE", fixture.0.path().join("command.pid"))
        .env(
            "DESCENDANT_PID_FILE",
            fixture.0.path().join("descendant.pid"),
        );
    let stdout = fixture.0.path().join("stdout");
    let stderr = fixture.0.path().join("stderr");
    command
        .args(["run", "--server-stdout"])
        .arg(&stdout)
        .arg("--server-stderr")
        .arg(&stderr);
    let diagnostic = fixture.0.path().join("runner-diagnostic");
    let mut runner = Running(command.args(["--", "/bin/sh", "-c", "printf '%s' \"$$\" > \"$COMMAND_PID_FILE\"; sleep 30 & printf '%s' \"$!\" > \"$DESCENDANT_PID_FILE\"; wait"]).stdout(Stdio::null()).stderr(fs::File::create(&diagnostic).unwrap()).spawn().unwrap());
    wait_until(|| {
        fixture.0.path().join("descendant.pid").exists()
            && fs::metadata(fixture.0.path().join("descendant.pid"))
                .unwrap()
                .len()
                > 0
    });
    let server = fixture.pid("server.pid");
    let child = fixture.pid("command.pid");
    let descendant = fixture.pid("descendant.pid");
    // SAFETY: this is the live child owned by this test; SIGTERM is supported.
    assert_eq!(
        unsafe { libc::kill(i32::try_from(runner.0.id()).unwrap(), libc::SIGTERM) },
        0
    );
    let mut status = None;
    wait_until(|| {
        status = runner.0.try_wait().unwrap();
        status.is_some()
    });
    assert_eq!(status.unwrap().code(), Some(143));
    wait_until(|| stopped(server) && stopped(child) && stopped(descendant));
    assert!(fs::read(stdout).unwrap().ends_with(b"stdout-end"));
    assert!(fs::read(stderr).unwrap().ends_with(b"stderr-end"));
    let diagnostic = fs::read_to_string(diagnostic).unwrap();
    assert!(diagnostic.contains("server stdout:"));
    assert!(diagnostic.contains("server stderr:"));
}
