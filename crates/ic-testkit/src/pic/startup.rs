use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

use pocket_ic::{PocketIc, PocketIcBuilder};

use super::transport;

const DEFAULT_SERVER_HARD_TTL: Duration = Duration::from_secs(10 * 60);
const STARTUP_POLL_INTERVAL: Duration = Duration::from_millis(20);
const SERVER_OUTPUT_LIMIT: usize = 16 * 1024;

static STARTUP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Explicit bounded source and policy for one PocketIC startup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PocketIcStartupConfig {
    source: PocketIcStartupSource,
    timeout: Duration,
    server_hard_ttl: Duration,
}

/// Caller-owned PocketIC server process with bounded startup and output capture.
///
/// Dropping the handle terminates and waits for the managed child. Callers may
/// create several instances through [`Self::url`] and
/// [`PocketIcStartupConfig::connect`] while retaining explicit server ownership.
/// The handle is process-local and does not coordinate ownership across Cargo
/// or test-runner processes; use an externally owned server with bounded
/// connect mode for that topology.
/// The handle owns no binary discovery, download, cache, or compatibility policy.
pub struct PocketIcManagedServer {
    server: ManagedServer,
    url: String,
}

/// Bounded lossy UTF-8 output captured from a managed PocketIC server.
///
/// Each stream retains at most the first 16 KiB. A textual suffix reports the
/// number of omitted bytes when truncation occurred.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PocketIcManagedServerOutput {
    stdout: String,
    stderr: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PocketIcStartupSource {
    Spawn { server_binary: PathBuf },
    Connect { server_url: String },
}

/// Structured failure from bounded PocketIC construction.
#[non_exhaustive]
#[derive(Debug)]
pub enum PocketIcStartupError {
    /// The caller supplied a zero timeout or unusable hard TTL.
    InvalidConfiguration { message: String },
    /// A caller-provided existing server URL could not be parsed.
    InvalidServerUrl { server_url: String, message: String },
    /// Preparing or inspecting bounded startup files failed.
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    /// The configured PocketIC server process could not be spawned.
    ServerSpawn {
        server_binary: PathBuf,
        source: io::Error,
    },
    /// The managed PocketIC server exited before instance construction completed.
    ServerExited {
        server_binary: PathBuf,
        status: ExitStatus,
        elapsed: Duration,
        stdout: String,
        stderr: String,
    },
    /// The managed server did not publish a usable port before the deadline.
    ReadinessTimeout {
        server_binary: PathBuf,
        timeout: Duration,
        stdout: String,
        stderr: String,
        termination_error: Option<String>,
    },
    /// The managed server published an invalid port-file value.
    InvalidServerPort {
        server_binary: PathBuf,
        value: String,
        stdout: String,
        stderr: String,
    },
    /// PocketIC instance creation did not finish before the startup deadline.
    InstanceCreationTimeout {
        timeout: Duration,
        stdout: String,
        stderr: String,
        termination_error: Option<String>,
    },
    /// Spawning the bounded builder worker failed.
    BuilderThreadSpawn { source: io::Error },
    /// Upstream PocketIC construction panicked before returning an instance.
    BuilderPanicked { message: String },
    /// The bounded builder worker ended without returning a result.
    BuilderDisconnected,
}

/// Fallible construction at PocketIC's panicking builder boundary.
///
/// Startup is explicit: callers either provide an existing server URL or let
/// `ic-testkit` spawn and monitor one exact server binary. This prevents the
/// upstream builder from hiding an unobservable child process.
pub trait PocketIcBuilderExt {
    /// Build one PocketIC instance within the configured deadline.
    ///
    /// Managed server startup detects child exit while awaiting the port file,
    /// terminates the child on timeout, and captures bounded stdout/stderr.
    /// Instance creation is also bounded. Upstream panics remain structured.
    fn try_build(self, config: PocketIcStartupConfig) -> Result<PocketIc, PocketIcStartupError>;
}

impl PocketIcStartupConfig {
    /// Spawn and monitor one exact PocketIC server binary.
    ///
    /// Startup allocates a unique private temporary directory while leaving
    /// the `--port-file` path absent for PocketIC to create.
    #[must_use]
    pub fn spawn(server_binary: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            source: PocketIcStartupSource::Spawn {
                server_binary: server_binary.into(),
            },
            timeout,
            server_hard_ttl: DEFAULT_SERVER_HARD_TTL,
        }
    }

    /// Connect to a caller-owned existing PocketIC server.
    ///
    /// The URL is applied to the builder explicitly, so this mode never lets
    /// the upstream builder spawn a hidden server child.
    #[must_use]
    pub fn connect(server_url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            source: PocketIcStartupSource::Connect {
                server_url: server_url.into(),
            },
            timeout,
            server_hard_ttl: DEFAULT_SERVER_HARD_TTL,
        }
    }

    /// Set the hard lifetime passed to an `ic-testkit`-managed server.
    #[must_use]
    pub const fn with_server_hard_ttl(mut self, hard_ttl: Duration) -> Self {
        self.server_hard_ttl = hard_ttl;
        self
    }

    /// Complete startup deadline.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Managed server hard lifetime.
    #[must_use]
    pub const fn server_hard_ttl(&self) -> Duration {
        self.server_hard_ttl
    }

    /// Managed server binary, when this configuration spawns one.
    #[must_use]
    pub fn server_binary(&self) -> Option<&Path> {
        match &self.source {
            PocketIcStartupSource::Spawn { server_binary } => Some(server_binary),
            PocketIcStartupSource::Connect { .. } => None,
        }
    }

    /// Existing caller-owned server URL, when configured.
    #[must_use]
    pub fn server_url(&self) -> Option<&str> {
        match &self.source {
            PocketIcStartupSource::Connect { server_url } => Some(server_url),
            PocketIcStartupSource::Spawn { .. } => None,
        }
    }

    /// Start a caller-owned managed server without constructing an instance.
    ///
    /// This requires a configuration created by [`Self::spawn`]. Readiness is
    /// bounded by [`Self::timeout`], and the configured hard TTL is still
    /// passed to the child. The returned handle terminates the child on drop;
    /// use its URL with [`Self::connect`] to construct bounded instances.
    pub fn start_managed_server(self) -> Result<PocketIcManagedServer, PocketIcStartupError> {
        self.validate()?;
        let PocketIcStartupSource::Spawn { server_binary } = self.source else {
            return Err(PocketIcStartupError::InvalidConfiguration {
                message: "starting a managed PocketIC server requires a spawn configuration"
                    .to_owned(),
            });
        };
        let started = Instant::now();
        let deadline = startup_deadline(started, self.timeout)?;
        let (server, url) = ManagedServer::start(
            server_binary,
            self.server_hard_ttl,
            deadline,
            self.timeout,
            started,
        )?;
        Ok(PocketIcManagedServer { server, url })
    }

    fn validate(&self) -> Result<(), PocketIcStartupError> {
        if self.timeout.is_zero() {
            return Err(PocketIcStartupError::InvalidConfiguration {
                message: "PocketIC startup timeout must be greater than zero".to_owned(),
            });
        }
        if matches!(&self.source, PocketIcStartupSource::Spawn { .. })
            && self.server_hard_ttl.as_secs() == 0
        {
            return Err(PocketIcStartupError::InvalidConfiguration {
                message: "PocketIC server hard TTL must be at least one second".to_owned(),
            });
        }
        Ok(())
    }
}

impl PocketIcManagedServer {
    /// Loopback URL published by the managed server.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Current bounded stdout and stderr captured from the managed server.
    ///
    /// This reads a snapshot of each retained output file. Each stream is
    /// limited to 16 KiB and carries an omitted-byte suffix when truncated.
    #[must_use]
    pub fn output(&self) -> PocketIcManagedServerOutput {
        self.server.capture().into()
    }
}

impl PocketIcManagedServerOutput {
    /// Bounded lossy UTF-8 standard output.
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Bounded lossy UTF-8 standard error.
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }
}

impl PocketIcBuilderExt for PocketIcBuilder {
    fn try_build(self, config: PocketIcStartupConfig) -> Result<PocketIc, PocketIcStartupError> {
        config.validate()?;
        let started = Instant::now();
        let deadline = startup_deadline(started, config.timeout)?;
        match config.source {
            PocketIcStartupSource::Connect { server_url } => {
                build_bounded(self, &server_url, deadline, config.timeout, None)
            }
            PocketIcStartupSource::Spawn { server_binary } => {
                let (server, server_url) = ManagedServer::start(
                    server_binary,
                    config.server_hard_ttl,
                    deadline,
                    config.timeout,
                    started,
                )?;
                build_bounded(self, &server_url, deadline, config.timeout, Some(server))
            }
        }
    }
}

fn startup_deadline(started: Instant, timeout: Duration) -> Result<Instant, PocketIcStartupError> {
    started
        .checked_add(timeout)
        .ok_or_else(|| PocketIcStartupError::InvalidConfiguration {
            message: "PocketIC startup timeout exceeds the platform clock range".to_owned(),
        })
}

fn build_bounded(
    builder: PocketIcBuilder,
    server_url: &str,
    deadline: Instant,
    timeout: Duration,
    mut server: Option<ManagedServer>,
) -> Result<PocketIc, PocketIcStartupError> {
    let builder = match server_url.parse() {
        Ok(server_url) => builder.with_server_url(server_url),
        Err(error) => {
            return Err(PocketIcStartupError::InvalidServerUrl {
                server_url: server_url.to_owned(),
                message: error.to_string(),
            });
        }
    };
    let (sender, receiver) = mpsc::sync_channel(1);
    if let Err(source) = thread::Builder::new()
        .name("ic-testkit-pocket-ic-startup".to_owned())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| builder.build()))
                .map_err(|payload| transport::panic_payload_to_string(payload.as_ref()));
            let _ = sender.send(result);
        })
    {
        return Err(PocketIcStartupError::BuilderThreadSpawn { source });
    }

    loop {
        let now = Instant::now();
        if now >= deadline {
            let captured = server.take().map_or_else(
                CapturedServer::default,
                ManagedServer::terminate_and_capture,
            );
            return Err(PocketIcStartupError::InstanceCreationTimeout {
                timeout,
                stdout: captured.stdout,
                stderr: captured.stderr,
                termination_error: captured.termination_error,
            });
        }
        let remaining = deadline.saturating_duration_since(now);
        let wait = if server.is_some() {
            remaining.min(STARTUP_POLL_INTERVAL)
        } else {
            remaining
        };
        match receiver.recv_timeout(wait) {
            Ok(Ok(pocket_ic)) => {
                if let Some(mut managed) = server.take() {
                    if let Some(status) = managed.try_wait()? {
                        return Err(managed.exited_error(status));
                    }
                    managed.reap_in_background();
                }
                return Ok(pocket_ic);
            }
            Ok(Err(message)) => {
                if let Some(server) = server.take() {
                    let _ = server.terminate_and_capture();
                }
                return Err(PocketIcStartupError::BuilderPanicked { message });
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(server) = server.take() {
                    let _ = server.terminate_and_capture();
                }
                return Err(PocketIcStartupError::BuilderDisconnected);
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(managed) = &mut server
                    && let Some(status) = managed.try_wait()?
                {
                    return Err(server
                        .take()
                        .expect("managed server must remain present")
                        .exited_error(status));
                }
            }
        }
    }
}

struct ManagedServer {
    child: Option<Child>,
    binary: PathBuf,
    files: Option<StartupFiles>,
    started: Instant,
}

enum PortFileState {
    Pending,
    Ready(u16),
    Invalid(String),
}

impl ManagedServer {
    fn start(
        binary: PathBuf,
        hard_ttl: Duration,
        deadline: Instant,
        timeout: Duration,
        started: Instant,
    ) -> Result<(Self, String), PocketIcStartupError> {
        let (files, stdout, stderr) = StartupFiles::create()?;
        let mut command = Command::new(&binary);
        command
            .arg("--hard-ttl")
            .arg(hard_ttl.as_secs().to_string())
            .arg("--port-file")
            .arg(&files.port)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt as _;
            command.process_group(0);
        }
        let child = command
            .spawn()
            .map_err(|source| PocketIcStartupError::ServerSpawn {
                server_binary: binary.clone(),
                source,
            })?;
        let mut server = Self {
            child: Some(child),
            binary,
            files: Some(files),
            started,
        };

        loop {
            if let Some(status) = server.try_wait()? {
                return Err(server.exited_error(status));
            }
            let now = Instant::now();
            if now >= deadline {
                let binary = server.binary.clone();
                let captured = server.terminate_and_capture();
                return Err(PocketIcStartupError::ReadinessTimeout {
                    server_binary: binary,
                    timeout,
                    stdout: captured.stdout,
                    stderr: captured.stderr,
                    termination_error: captured.termination_error,
                });
            }
            match server.read_port()? {
                PortFileState::Pending => {}
                PortFileState::Ready(port) => {
                    return Ok((server, format!("http://127.0.0.1:{port}/")));
                }
                PortFileState::Invalid(value) => {
                    let binary = server.binary.clone();
                    let captured = server.terminate_and_capture();
                    return Err(PocketIcStartupError::InvalidServerPort {
                        server_binary: binary,
                        value,
                        stdout: captured.stdout,
                        stderr: captured.stderr,
                    });
                }
            }
            thread::sleep(
                deadline
                    .saturating_duration_since(now)
                    .min(STARTUP_POLL_INTERVAL),
            );
        }
    }

    fn try_wait(&mut self) -> Result<Option<ExitStatus>, PocketIcStartupError> {
        self.child
            .as_mut()
            .expect("managed server child must remain present")
            .try_wait()
            .map_err(|source| PocketIcStartupError::Io {
                operation: "inspect PocketIC server child",
                path: self.binary.clone(),
                source,
            })
    }

    fn read_port(&self) -> Result<PortFileState, PocketIcStartupError> {
        let port_path = &self
            .files
            .as_ref()
            .expect("managed server startup files must remain present")
            .port;
        let contents = match fs::read_to_string(port_path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PortFileState::Pending);
            }
            Err(source) => {
                return Err(PocketIcStartupError::Io {
                    operation: "read PocketIC server port file",
                    path: port_path.clone(),
                    source,
                });
            }
        };
        if !contents.contains('\n') {
            return Ok(PortFileState::Pending);
        }
        let value = contents.trim().to_owned();
        match value.parse::<u16>() {
            Ok(port) if port != 0 => Ok(PortFileState::Ready(port)),
            _ => Ok(PortFileState::Invalid(value)),
        }
    }

    fn exited_error(mut self, status: ExitStatus) -> PocketIcStartupError {
        let elapsed = self.started.elapsed();
        let binary = self.binary.clone();
        self.child.take();
        let captured = self.capture();
        PocketIcStartupError::ServerExited {
            server_binary: binary,
            status,
            elapsed,
            stdout: captured.stdout,
            stderr: captured.stderr,
        }
    }

    fn terminate_and_capture(mut self) -> CapturedServer {
        let termination_error = match self.child.take() {
            Some(mut child) => terminate_child(&mut child),
            None => None,
        };
        let mut captured = self.capture();
        captured.termination_error = termination_error;
        captured
    }

    fn capture(&self) -> CapturedServer {
        let files = self
            .files
            .as_ref()
            .expect("managed server startup files must remain present");
        CapturedServer {
            stdout: read_bounded_lossy(&files.stdout),
            stderr: read_bounded_lossy(&files.stderr),
            termination_error: None,
        }
    }

    fn reap_in_background(mut self) {
        let child = ServerChildGuard {
            child: self.child.take(),
        };
        let files = self.files.take();
        let _ = thread::Builder::new()
            .name("ic-testkit-pocket-ic-server-reaper".to_owned())
            .spawn(move || {
                let mut child = child;
                if let Some(mut process) = child.child.take() {
                    let _ = process.wait();
                }
                drop(files);
            });
    }
}

impl Drop for ManagedServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = terminate_child(&mut child);
        }
    }
}

struct ServerChildGuard {
    child: Option<Child>,
}

impl Drop for ServerChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = terminate_child(&mut child);
        }
    }
}

fn terminate_child(child: &mut Child) -> Option<String> {
    match child.try_wait() {
        Ok(Some(_)) => None,
        Ok(None) => child
            .kill()
            .and_then(|()| child.wait().map(|_| ()))
            .err()
            .map(|error| error.to_string()),
        Err(inspect_error) => {
            let termination_error = child.kill().and_then(|()| child.wait().map(|_| ())).err();
            termination_error.map(|termination_error| {
                format!(
                    "failed to inspect child before termination: {inspect_error}; termination also failed: {termination_error}"
                )
            })
        }
    }
}

#[derive(Default)]
struct CapturedServer {
    stdout: String,
    stderr: String,
    termination_error: Option<String>,
}

impl From<CapturedServer> for PocketIcManagedServerOutput {
    fn from(captured: CapturedServer) -> Self {
        Self {
            stdout: captured.stdout,
            stderr: captured.stderr,
        }
    }
}

struct StartupFiles {
    directory: PathBuf,
    port: PathBuf,
    stdout: PathBuf,
    stderr: PathBuf,
}

impl StartupFiles {
    fn create() -> Result<(Self, File, File), PocketIcStartupError> {
        loop {
            let sequence = STARTUP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let base = std::env::temp_dir().join(format!(
                "ic-testkit-pocket-ic-startup-{}-{sequence}",
                std::process::id()
            ));
            let mut directory = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt as _;
                directory.mode(0o700);
            }
            match directory.create(&base) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => return Err(startup_file_error("create", &base, source)),
            }
            let files = Self {
                port: base.join("port"),
                stdout: base.join("stdout"),
                stderr: base.join("stderr"),
                directory: base,
            };
            let stdout = create_new_file(&files.stdout)
                .map_err(|source| startup_file_error("create", &files.stdout, source))?;
            let stderr = create_new_file(&files.stderr)
                .map_err(|source| startup_file_error("create", &files.stderr, source))?;
            return Ok((files, stdout, stderr));
        }
    }
}

impl Drop for StartupFiles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn create_new_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn startup_file_error(
    operation: &'static str,
    path: &Path,
    source: io::Error,
) -> PocketIcStartupError {
    PocketIcStartupError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

fn read_bounded_lossy(path: &Path) -> String {
    let Ok(bytes) = fs::read(path) else {
        return String::new();
    };
    let retained = bytes.len().min(SERVER_OUTPUT_LIMIT);
    let mut output = String::from_utf8_lossy(&bytes[..retained]).into_owned();
    let omitted = bytes.len().saturating_sub(retained);
    if omitted > 0 {
        let _ = write!(output, "\n<truncated {omitted} bytes>");
    }
    output
}

impl std::fmt::Display for PocketIcStartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfiguration { message } => formatter.write_str(message),
            Self::InvalidServerUrl {
                server_url,
                message,
            } => write!(
                formatter,
                "invalid PocketIC server URL {server_url:?}: {message}"
            ),
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::ServerSpawn {
                server_binary,
                source,
            } => write!(
                formatter,
                "failed to spawn PocketIC server {}: {source}",
                server_binary.display()
            ),
            Self::ServerExited {
                server_binary,
                status,
                elapsed,
                stderr,
                ..
            } => write!(
                formatter,
                "PocketIC server {} exited with {status} after {elapsed:?}: {stderr}",
                server_binary.display()
            ),
            Self::ReadinessTimeout {
                server_binary,
                timeout,
                ..
            } => write!(
                formatter,
                "PocketIC server {} was not ready within {timeout:?}",
                server_binary.display()
            ),
            Self::InvalidServerPort {
                server_binary,
                value,
                ..
            } => write!(
                formatter,
                "PocketIC server {} published invalid port {value:?}",
                server_binary.display()
            ),
            Self::InstanceCreationTimeout { timeout, .. } => {
                write!(formatter, "PocketIC instance creation exceeded {timeout:?}")
            }
            Self::BuilderThreadSpawn { source } => {
                write!(
                    formatter,
                    "failed to spawn PocketIC builder worker: {source}"
                )
            }
            Self::BuilderPanicked { message } => {
                write!(formatter, "PocketIC startup panicked: {message}")
            }
            Self::BuilderDisconnected => {
                formatter.write_str("PocketIC builder worker disconnected without a result")
            }
        }
    }
}

impl std::error::Error for PocketIcStartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. }
            | Self::ServerSpawn { source, .. }
            | Self::BuilderThreadSpawn { source } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, Instant},
    };

    use super::{
        PocketIcBuilderExt as _, PocketIcStartupConfig, PocketIcStartupError, StartupFiles,
    };
    use pocket_ic::PocketIcBuilder;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt as _;

    #[test]
    fn startup_config_requires_positive_bounds() {
        let error = PocketIcStartupConfig::connect("http://127.0.0.1:1/", Duration::ZERO)
            .validate()
            .expect_err("zero startup timeout must fail");
        assert!(matches!(
            error,
            PocketIcStartupError::InvalidConfiguration { .. }
        ));

        let error = PocketIcStartupConfig::spawn("pocket-ic", Duration::from_secs(1))
            .with_server_hard_ttl(Duration::from_millis(1))
            .validate()
            .expect_err("subsecond server hard TTL must fail");
        assert!(matches!(
            error,
            PocketIcStartupError::InvalidConfiguration { .. }
        ));
    }

    #[test]
    fn startup_files_leave_the_server_owned_port_path_absent() {
        let (files, stdout, stderr) = StartupFiles::create().expect("allocate startup files");
        let directory = files.directory.clone();

        assert!(directory.is_dir());
        assert!(!files.port.exists());
        assert!(files.stdout.is_file());
        assert!(files.stderr.is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;

            let mode = fs::metadata(&directory)
                .expect("inspect private startup directory")
                .permissions()
                .mode();
            assert_eq!(mode & 0o077, 0);
        }

        drop(stdout);
        drop(stderr);
        drop(files);
        assert!(!directory.exists());
    }

    #[cfg(unix)]
    #[test]
    fn managed_startup_reports_an_exited_server_with_bounded_output() {
        let script = TestServerScript::new(
            "exit",
            "#!/bin/sh\nif [ -e \"$4\" ]; then exit 97; fi\nprintf 'synthetic server stdout'\nprintf 'synthetic bind failure' >&2\nexit 23\n",
        );

        let result = PocketIcBuilder::new().with_application_subnet().try_build(
            PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2)),
        );

        let Err(PocketIcStartupError::ServerExited {
            server_binary,
            status,
            stdout,
            stderr,
            ..
        }) = result
        else {
            panic!("an exited managed server must return a structured exit error");
        };
        assert_eq!(server_binary, script.path());
        assert_eq!(status.code(), Some(23));
        assert_eq!(stdout, "synthetic server stdout");
        assert_eq!(stderr, "synthetic bind failure");
    }

    #[cfg(unix)]
    #[test]
    fn managed_startup_terminates_a_server_that_never_becomes_ready() {
        let script = TestServerScript::new(
            "timeout",
            "#!/bin/sh\nif [ -e \"$4\" ]; then exit 97; fi\nexec sleep 30\n",
        );
        let timeout = Duration::from_millis(100);
        let started = Instant::now();

        let result = PocketIcBuilder::new()
            .with_application_subnet()
            .try_build(PocketIcStartupConfig::spawn(script.path(), timeout));

        assert!(
            started.elapsed() < Duration::from_secs(2),
            "bounded startup should not wait for the sleeping child"
        );
        assert!(matches!(
            result,
            Err(PocketIcStartupError::ReadinessTimeout {
                server_binary,
                timeout: actual_timeout,
                termination_error: None,
                ..
            }) if server_binary == script.path() && actual_timeout == timeout
        ));
    }

    #[cfg(unix)]
    #[test]
    fn managed_server_handle_exposes_url_output_and_raii_ownership() {
        let script = TestServerScript::new(
            "handle",
            "#!/bin/sh\nif [ -e \"$4\" ]; then echo 'port path already exists' >&2; exit 97; fi\nprintf 'managed server ready'\nprintf '34567\\n' > \"$4\"\nexec sleep 30\n",
        );

        let server = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
            .start_managed_server()
            .expect("start caller-owned managed server");

        assert_eq!(server.url(), "http://127.0.0.1:34567/");
        assert_eq!(server.output().stdout(), "managed server ready");
        assert_eq!(server.output().stderr(), "");
        #[cfg(target_os = "linux")]
        let child_process = PathBuf::from(format!(
            "/proc/{}",
            server
                .server
                .child
                .as_ref()
                .expect("managed handle must own its child")
                .id()
        ));
        #[cfg(target_os = "linux")]
        assert!(child_process.exists());
        drop(server);
        #[cfg(target_os = "linux")]
        assert!(!child_process.exists());
    }

    #[test]
    #[ignore = "requires IC_TESTKIT_POCKET_IC_SERVER=<caller-provided PocketIC server binary>"]
    fn caller_provided_server_publishes_port_constructs_instance_and_cleans_up() {
        let binary = std::env::var_os("IC_TESTKIT_POCKET_IC_SERVER")
            .map(PathBuf::from)
            .expect("set IC_TESTKIT_POCKET_IC_SERVER to the exact server binary");
        let one_shot_sequence = super::STARTUP_FILE_SEQUENCE.load(super::Ordering::Relaxed);
        let one_shot_directory = std::env::temp_dir().join(format!(
            "ic-testkit-pocket-ic-startup-{}-{one_shot_sequence}",
            std::process::id()
        ));
        let one_shot = PocketIcBuilder::new()
            .with_application_subnet()
            .try_build(
                PocketIcStartupConfig::spawn(&binary, Duration::from_secs(30))
                    .with_server_hard_ttl(Duration::from_secs(1)),
            )
            .expect("one-shot managed spawn must construct an instance");
        assert!(one_shot_directory.is_dir());
        drop(one_shot);
        let cleanup_deadline = Instant::now() + Duration::from_secs(3);
        while one_shot_directory.exists() && Instant::now() < cleanup_deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!one_shot_directory.exists());

        let server = PocketIcStartupConfig::spawn(&binary, Duration::from_secs(30))
            .with_server_hard_ttl(Duration::from_secs(60))
            .start_managed_server()
            .expect("caller-provided PocketIC server must publish its port");
        let files = server
            .server
            .files
            .as_ref()
            .expect("managed server must retain startup files");
        let startup_directory = files.directory.clone();

        assert!(files.port.is_file());
        let pocket_ic = PocketIcBuilder::new()
            .with_application_subnet()
            .try_build(PocketIcStartupConfig::connect(
                server.url(),
                Duration::from_secs(30),
            ))
            .expect("construct instance through caller-provided server");

        drop(pocket_ic);
        drop(server);
        assert!(!startup_directory.exists());
    }

    #[cfg(unix)]
    struct TestServerScript {
        path: PathBuf,
    }

    #[cfg(unix)]
    impl TestServerScript {
        fn new(label: &str, contents: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "ic-testkit-pocket-ic-{label}-{}-{}",
                std::process::id(),
                super::STARTUP_FILE_SEQUENCE.fetch_add(1, super::Ordering::Relaxed),
            ));
            fs::write(&path, contents).expect("write synthetic PocketIC server script");
            let mut permissions = fs::metadata(&path)
                .expect("read synthetic server script metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions)
                .expect("make synthetic server script executable");
            Self { path }
        }

        fn path(&self) -> PathBuf {
            self.path.clone()
        }
    }

    #[cfg(unix)]
    impl Drop for TestServerScript {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }
}
