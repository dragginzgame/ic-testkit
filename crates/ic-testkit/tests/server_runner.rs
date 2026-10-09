#![cfg(unix)]

mod support;

use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let fixture = Self(std::env::temp_dir().join(format!(
            "ic-testkit-server-runner-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&fixture.0).unwrap();
        support::executable::write_executable_script(
            &fixture.0.join("server"),
            format!(
                "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'pocket-ic-server {}\\n'; exit 0; fi\nprintf '%s\\n' \"$$\" > \"$SERVER_PID_FILE\"\nif [ \"$1\" = --hard-ttl ]; then printf '%s' \"$2\" > \"$TTL_FILE\"; shift 2; fi\n[ \"$1\" = --port-file ] || exit 99\ndd if=/dev/zero bs=1024 count=20 2>/dev/null\nprintf stdout-end\ndd if=/dev/zero bs=1024 count=20 >&2 2>/dev/null\nprintf stderr-end >&2\nprintf '34567\\n' > \"$2\"\nexec sleep 30\n",
                ic_testkit::pocket_ic::LATEST_SERVER_VERSION
            ),
        );
        fixture
    }
    fn runner(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ic-testkit-server"));
        command
            .env_remove("IC_TESTKIT_POCKET_IC_URL")
            .env("POCKET_IC_BIN", self.0.join("server"))
            .env("SERVER_PID_FILE", self.0.join("server.pid"))
            .env("TTL_FILE", self.0.join("ttl"));
        command
    }
    fn pid(&self, file: &str) -> u32 {
        fs::read_to_string(self.0.join(file))
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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
    assert_eq!(fs::read_to_string(fixture.0.join("ttl")).unwrap(), "17");
    assert!(stopped(fixture.pid("server.pid")));
}

#[test]
fn runner_retains_complete_server_streams_after_command_failure() {
    let fixture = Fixture::new();
    let stdout = fixture.0.join("stdout");
    let stderr = fixture.0.join("stderr");
    let output = fixture
        .runner()
        .current_dir(&fixture.0)
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
    assert!(!fixture.0.join("server.pid").exists());
    let output = fixture
        .runner()
        .env("IC_TESTKIT_POCKET_IC_URL", "http://127.0.0.1:45678/")
        .args(["run", "--ttl", "1", "--", "/bin/sh", "-c", "exit 0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--ttl requires an owned server"));
}

#[test]
fn invalid_configuration_and_version_failure_never_run_the_command() {
    let fixture = Fixture::new();
    let marker = fixture.0.join("command-ran");
    for args in [
        vec!["run", "--ttl", "0", "--", "/bin/sh"],
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
    support::executable::write_executable_script(
        &fixture.0.join("failed-version"),
        format!(
            "#!/bin/sh\nprintf 'pocket-ic-server {}\\n'\nexit 23\n",
            ic_testkit::pocket_ic::LATEST_SERVER_VERSION
        ),
    );
    let output = fixture
        .runner()
        .env("POCKET_IC_BIN", fixture.0.join("failed-version"))
        .args(["run", "--", "/bin/sh", "-c", "touch \"$1\"", "test"])
        .arg(&marker)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!marker.exists());
    assert!(!fixture.0.join("server.pid").exists());
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
        .env("DESCENDANT_PID_FILE", fixture.0.join("descendant.pid"))
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
    use ic_testkit::pic::{PocketIcStartupConfig, PocketIcStartupError};

    let fixture = Fixture::new();
    let binary = fixture.0.join("server");
    support::executable::write_executable_script(
        &binary,
        "#!/bin/sh\nprintf '%s' \"$$\" > \"$0.pid\"\nprintf '34567\\n' > \"$2\"\nwhile [ ! -s \"$0.descendant.pid\" ]; do sleep 0.02; done\nprintf 'server stopped\\n'\nprintf 'failure detail\\n' >&2\nexit 42\n",
    );
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
    let PocketIcStartupError::ServerExited {
        server_binary,
        status,
        stdout,
        stderr,
        ..
    } = error
    else {
        panic!("expected the owned server exit, got {error:?}");
    };
    assert_eq!(server_binary, binary);
    assert_eq!(status.code(), Some(42));
    assert_eq!(stdout, "server stopped\n");
    assert_eq!(stderr, "failure detail\n");
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
    use ic_testkit::pic::{PocketIcBuilder, PocketIcBuilderExt, PocketIcStartupConfig};
    assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_some());
    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .try_build(PocketIcStartupConfig::from_env(Duration::from_secs(30)).unwrap())
        .unwrap();
    assert_eq!(
        pic.get_server_url().as_str(),
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
fn offline_check_and_run_refuse_missing_setup_without_executing_tools() {
    let fixture = Fixture::new();
    let missing = fixture.0.join("missing");
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
        .env("COMMAND_PID_FILE", fixture.0.join("command.pid"))
        .env("DESCENDANT_PID_FILE", fixture.0.join("descendant.pid"));
    let stdout = fixture.0.join("stdout");
    let stderr = fixture.0.join("stderr");
    command
        .args(["run", "--server-stdout"])
        .arg(&stdout)
        .arg("--server-stderr")
        .arg(&stderr);
    let mut runner = Running(command.args(["--", "/bin/sh", "-c", "printf '%s' \"$$\" > \"$COMMAND_PID_FILE\"; sleep 30 & printf '%s' \"$!\" > \"$DESCENDANT_PID_FILE\"; wait"]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap());
    wait_until(|| {
        fixture.0.join("descendant.pid").exists()
            && fs::metadata(fixture.0.join("descendant.pid"))
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
}
