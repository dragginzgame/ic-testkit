use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{self, Read as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};

use ic_host_process::child::{CleanupError, OwnedChild};
use pocket_ic::{PocketIc, PocketIcBuilder};

use super::transport;

const STARTUP_POLL_INTERVAL: Duration = Duration::from_millis(20);
const SERVER_OUTPUT_LIMIT: usize = 16 * 1024;
// PocketIC publishes a decimal u16 and a newline. Leave whitespace room
// without allowing readiness polling to read an arbitrary-size file.
const SERVER_PORT_FILE_LIMIT: usize = 64;

static STARTUP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Explicit bounded source and policy for one PocketIC startup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PocketIcStartupConfig {
    source: PocketIcStartupSource,
    timeout: Duration,
    server_hard_ttl: Option<Duration>,
    server_idle_ttl: Option<Duration>,
    server_output_files: Option<(PathBuf, PathBuf)>,
}

/// Caller-owned PocketIC server process with bounded startup and output capture.
///
/// Dropping the handle terminates and waits for the managed child. On Unix,
/// teardown also terminates descendants remaining in its owned process group.
/// Callers may create several instances through [`Self::url`] and
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
/// number of omitted bytes when truncation occurred. Unreadable streams and
/// paths replaced with non-regular files are omitted.
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

/// Original startup failure, bounded server output and typed cleanup failures.
///
/// Match [`Self::failure`] to classify the original cause. Cleanup diagnostics
/// never replace it, and command/server failures retain separate ownership.
/// Output is an inert bounded snapshot; it grants no process authority.
#[derive(Debug)]
pub struct PocketIcStartupError {
    failure: Box<PocketIcStartupFailure>,
    output: PocketIcManagedServerOutput,
    command_cleanup: Option<Box<CleanupError>>,
    server_cleanup: Option<Box<CleanupError>>,
}

impl PocketIcStartupError {
    fn new(failure: PocketIcStartupFailure) -> Self {
        Self {
            failure: Box::new(failure),
            output: PocketIcManagedServerOutput::default(),
            command_cleanup: None,
            server_cleanup: None,
        }
    }

    /// Original cause, unaffected by subsequent cleanup failure.
    #[must_use]
    pub fn failure(&self) -> &PocketIcStartupFailure {
        &self.failure
    }

    /// Bounded output from the server owned by this operation, if any.
    #[must_use]
    pub const fn output(&self) -> &PocketIcManagedServerOutput {
        &self.output
    }

    /// Failure cleaning the command's owned process group/direct child.
    #[must_use]
    pub fn command_cleanup(&self) -> Option<&CleanupError> {
        self.command_cleanup.as_deref()
    }

    /// Failure cleaning the managed server's owned process group/direct child.
    #[must_use]
    pub fn server_cleanup(&self) -> Option<&CleanupError> {
        self.server_cleanup.as_deref()
    }

    fn with_cleanup(
        mut self,
        command: Option<CleanupError>,
        server: Option<(PocketIcManagedServerOutput, Option<CleanupError>)>,
    ) -> Self {
        self.command_cleanup = command.map(Box::new);
        if let Some((output, cleanup)) = server {
            self.output = output;
            self.server_cleanup = cleanup.map(Box::new);
        }
        self
    }
}

/// Original failure, independent of output capture and cleanup outcomes.
#[non_exhaustive]
#[derive(Debug)]
pub enum PocketIcStartupFailure {
    /// Neither the shared server URL nor an explicit executable was configured.
    NotConfigured,
    /// A selected environment value was empty or was not valid Unicode.
    InvalidEnvironment { variable: &'static str },
    /// The bounded version probe failed, including nonzero exit or timeout.
    ServerVersionProbe {
        source: ic_host_process::tool::ToolError,
    },
    /// The selected executable does not report the qualified server identity.
    ServerVersionMismatch { expected: String, observed: Vec<u8> },
    /// Command execution failed.
    CommandRun { program: PathBuf, source: io::Error },
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
    /// The managed PocketIC server exited during construction or command execution.
    ServerExited {
        server_binary: PathBuf,
        status: ExitStatus,
        elapsed: Duration,
    },
    /// The managed server did not publish a usable port before the deadline.
    ReadinessTimeout {
        server_binary: PathBuf,
        timeout: Duration,
    },
    /// The managed server published an invalid or oversized port-file value.
    InvalidServerPort {
        server_binary: PathBuf,
        value: String,
    },
    /// PocketIC instance creation did not finish before the startup deadline.
    InstanceCreationTimeout { timeout: Duration },
    /// Spawning the bounded builder worker failed.
    BuilderThreadSpawn { source: io::Error },
    /// Upstream PocketIC construction panicked before returning an instance.
    BuilderPanicked { message: String },
    /// The bounded builder worker ended without returning a result.
    BuilderDisconnected,
    /// The command completed, but the owned server could not be cleaned up.
    ServerCleanup { command_status: ExitStatus },
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
    /// terminates the child on timeout, and reads bounded stdout/stderr prefixes.
    /// Instance creation is also bounded. Upstream panics remain structured.
    ///
    /// This deadline covers construction only. Dropping the returned instance
    /// uses PocketIC's synchronous HTTP deletion, which has no request deadline
    /// in PocketIC 16. An operation's maximum request time does not bound drop.
    fn try_build(self, config: PocketIcStartupConfig) -> Result<PocketIc, PocketIcStartupError>;
}

impl PocketIcStartupConfig {
    /// Select the shared environment contract without discovery or downloads.
    ///
    /// `IC_TESTKIT_POCKET_IC_URL` takes precedence over `POCKET_IC_BIN`. An
    /// explicitly empty or invalid selected value fails rather than falling
    /// back. URL mode never launches a version probe or claims server ownership.
    /// Binary mode resolves the explicit path and checks `--version` against
    /// Testkit's supported protocol range using the shared bounded capture
    /// engine. This is version qualification, not executable-byte admission;
    /// prepare and verify the binary with `ic-testkit-server setup` / `check`.
    /// The probe has its own `timeout`; subsequent startup has the same budget.
    pub fn from_env(timeout: Duration) -> Result<Self, PocketIcStartupError> {
        Self::from_environment(timeout, |name| std::env::var_os(name))
    }

    fn from_environment(
        timeout: Duration,
        mut variable: impl FnMut(&str) -> Option<std::ffi::OsString>,
    ) -> Result<Self, PocketIcStartupError> {
        if let Some(value) = variable("IC_TESTKIT_POCKET_IC_URL") {
            let server_url = value
                .into_string()
                .ok()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    PocketIcStartupError::new(PocketIcStartupFailure::InvalidEnvironment {
                        variable: "IC_TESTKIT_POCKET_IC_URL",
                    })
                })?;
            let config = Self::connect(&server_url, timeout);
            config.validate()?;
            let parsed = server_url.parse().map_err(|error| {
                PocketIcStartupError::new(PocketIcStartupFailure::InvalidServerUrl {
                    server_url: server_url.clone(),
                    message: format!("{error}"),
                })
            })?;
            let _ = PocketIcBuilder::new().with_server_url(parsed);
            return Ok(config);
        }
        let value = variable("POCKET_IC_BIN")
            .ok_or_else(|| PocketIcStartupError::new(PocketIcStartupFailure::NotConfigured))?;
        if value.is_empty() {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::InvalidEnvironment {
                    variable: "POCKET_IC_BIN",
                },
            ));
        }
        let path = PathBuf::from(value);
        let binary = fs::canonicalize(&path).map_err(|source| {
            PocketIcStartupError::new(PocketIcStartupFailure::Io {
                operation: "resolve configured PocketIC executable",
                path,
                source,
            })
        })?;
        let config = Self::spawn(&binary, timeout);
        config.validate()?;
        let evidence = ic_host_process::tool::capture_group_command(
            Command::new(&binary).arg("--version"),
            ic_host_process::tool::OutputLimits {
                stdout_bytes: SERVER_OUTPUT_LIMIT,
                stderr_bytes: SERVER_OUTPUT_LIMIT,
                timeout,
            },
        )
        .map_err(|source| {
            PocketIcStartupError::new(PocketIcStartupFailure::ServerVersionProbe { source })
        })?;
        let expected = "pocket-ic-server >=16.0.0,<17.0.0 (stable)".to_owned();
        if !super::supports_pocket_ic_server(&evidence.stdout) {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::ServerVersionMismatch {
                    expected,
                    observed: evidence.stdout,
                },
            ));
        }
        Ok(config)
    }

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
            server_hard_ttl: None,
            server_idle_ttl: None,
            server_output_files: None,
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
            server_hard_ttl: None,
            server_idle_ttl: None,
            server_output_files: None,
        }
    }

    /// Set the hard lifetime passed to an `ic-testkit`-managed server.
    #[must_use]
    pub const fn with_server_hard_ttl(mut self, hard_ttl: Duration) -> Self {
        self.server_hard_ttl = Some(hard_ttl);
        self
    }

    /// Set the operation-idle lifetime of a managed server, independently of
    /// its hard lifetime. Requires spawn mode and at least one whole second.
    /// Without this selection, PocketIC uses its own idle default.
    /// Fractional seconds are discarded when constructing the server argument.
    #[must_use]
    pub const fn with_server_idle_ttl(mut self, idle_ttl: Duration) -> Self {
        self.server_idle_ttl = Some(idle_ttl);
        self
    }

    /// Explicit managed operation-idle lifetime, or PocketIC's default.
    #[must_use]
    pub const fn server_idle_ttl(&self) -> Option<Duration> {
        self.server_idle_ttl
    }

    /// Capture complete raw server streams in two caller-owned new files.
    ///
    /// Requires spawn mode. Both parent directories must exist and remain under
    /// caller control; relative paths resolve at startup. Existing files,
    /// symlinks and special files are refused, without truncating them. New files
    /// have Unix mode 0600. Created output survives success, startup failure,
    /// cancellation and server teardown, including a partially prepared pair.
    /// The caller owns retention, disk budget and path presentation. Without
    /// this selection, output remains temporary and is removed during cleanup.
    /// Public output/error excerpts still read at most 16 KiB per stream.
    #[must_use]
    pub fn with_server_output_files(
        mut self,
        stdout: impl Into<PathBuf>,
        stderr: impl Into<PathBuf>,
    ) -> Self {
        self.server_output_files = Some((stdout.into(), stderr.into()));
        self
    }

    /// Complete startup deadline.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Explicit managed server hard lifetime, or `None` when disabled.
    #[must_use]
    pub const fn server_hard_ttl(&self) -> Option<Duration> {
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
    /// bounded by [`Self::timeout`]. No hard TTL is passed by default; an
    /// explicit [`Self::with_server_hard_ttl`] value is passed to the child.
    /// [`Self::with_server_idle_ttl`] independently selects the operation-idle
    /// lifetime; otherwise PocketIC retains its own idle default.
    /// Readiness requires a nonzero decimal port followed by a newline in a
    /// regular UTF-8 file of at most 64 bytes; oversized files fail with bounded
    /// diagnostics. Non-regular port files fail with [`PocketIcStartupFailure::Io`]
    /// and [`io::ErrorKind::InvalidData`] without waiting for a FIFO writer.
    /// The returned handle terminates the child on drop; use its URL with
    /// [`Self::connect`] to construct bounded instances.
    pub fn start_managed_server(self) -> Result<PocketIcManagedServer, PocketIcStartupError> {
        self.validate()?;
        let PocketIcStartupSource::Spawn { server_binary } = self.source else {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::InvalidConfiguration {
                    message: "starting a managed PocketIC server requires a spawn configuration"
                        .to_owned(),
                },
            ));
        };
        let started = Instant::now();
        let deadline = startup_deadline(started, self.timeout)?;
        let (server, url) = ManagedServer::start(
            server_binary,
            self.server_hard_ttl,
            self.server_idle_ttl,
            self.server_output_files,
            deadline,
            self.timeout,
            started,
        )?;
        Ok(PocketIcManagedServer { server, url })
    }

    /// Run a command with `IC_TESTKIT_POCKET_IC_URL` set to this server.
    ///
    /// Spawn mode retains the managed server until command completion; connect
    /// mode borrows the external server and never terminates it. The command's
    /// IO and other environment selections remain caller-owned. Cancellation
    /// is polled after bounded startup and every 20 ms while the command runs.
    /// It returns an [`io::ErrorKind::Interrupted`] error after cleanup.
    /// If the owned server exits while the command is pending, the command is
    /// terminated and the server's status and bounded diagnostics are returned
    /// as [`PocketIcStartupFailure::ServerExited`]. External servers are not monitored.
    ///
    /// On Unix the command starts in a new owned process group. Completion,
    /// cancellation and observation failures terminate remaining group members
    /// before reaping the leader, using the same lifecycle engine as managed
    /// servers. This does not impose a command deadline or sandbox descendants
    /// that deliberately leave the owned group. Other hosts own the direct child.
    pub fn run_command(
        self,
        command: &mut Command,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<ExitStatus, PocketIcStartupError> {
        self.validate()?;
        let program = PathBuf::from(command.get_program());
        let command_error = |source| {
            PocketIcStartupError::new(PocketIcStartupFailure::CommandRun {
                program: program.clone(),
                source,
            })
        };
        if cancelled() {
            return Err(command_error(io::Error::from(io::ErrorKind::Interrupted)));
        }
        let (server, url) = if let Some(url) = self.server_url() {
            (None, url.to_owned())
        } else {
            let server = self.start_managed_server()?;
            let url = server.url().to_owned();
            (Some(server.server), url)
        };
        let mut server = server;
        if cancelled() {
            return Err(finalize_failure(
                command_error(io::Error::from(io::ErrorKind::Interrupted)),
                None,
                server,
            ));
        }
        command.env("IC_TESTKIT_POCKET_IC_URL", url);
        let mut owned_child = match OwnedChild::spawn(command) {
            Ok(child) => child,
            Err(source) => {
                return Err(finalize_failure(
                    PocketIcStartupError::new(PocketIcStartupFailure::Io {
                        operation: "spawn command with PocketIC server",
                        path: PathBuf::from(command.get_program()),
                        source,
                    }),
                    None,
                    server,
                ));
            }
        };
        let result = loop {
            if cancelled() {
                break Err(command_error(io::Error::from(io::ErrorKind::Interrupted)));
            }
            match owned_child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {
                    if let Some(managed) = server.as_mut() {
                        match managed.try_wait() {
                            Ok(Some(status)) => break Err(managed.exit_failure(status)),
                            Ok(None) => {}
                            Err(error) => break Err(error),
                        }
                    }
                    thread::sleep(STARTUP_POLL_INTERVAL);
                }
                Err(source) => break Err(command_error(source)),
            }
        };
        match result {
            Err(error) => Err(finalize_failure(error, Some(&mut owned_child), server)),
            Ok(status) => {
                if let Some(server) = server {
                    let (output, cleanup) = server.terminate_and_capture();
                    if let Some(cleanup) = cleanup {
                        return Err(PocketIcStartupError::new(
                            PocketIcStartupFailure::ServerCleanup {
                                command_status: status,
                            },
                        )
                        .with_cleanup(None, Some((output, Some(cleanup)))));
                    }
                }
                Ok(status)
            }
        }
    }

    fn validate(&self) -> Result<(), PocketIcStartupError> {
        if self.server_idle_ttl.is_some()
            && (self.server_url().is_some()
                || self.server_idle_ttl.is_some_and(|ttl| ttl.as_secs() == 0))
        {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::InvalidConfiguration {
                    message: "PocketIC server idle TTL requires spawn mode and at least one second"
                        .to_owned(),
                },
            ));
        }
        if self.server_url().is_some() && self.server_output_files.is_some() {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::InvalidConfiguration {
                    message: "server output files require a spawn configuration".to_owned(),
                },
            ));
        }
        if self.timeout.is_zero() {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::InvalidConfiguration {
                    message: "PocketIC startup timeout must be greater than zero".to_owned(),
                },
            ));
        }
        if matches!(&self.source, PocketIcStartupSource::Spawn { .. })
            && self
                .server_hard_ttl
                .is_some_and(|hard_ttl| hard_ttl.as_secs() == 0)
        {
            return Err(PocketIcStartupError::new(
                PocketIcStartupFailure::InvalidConfiguration {
                    message: "PocketIC server hard TTL must be at least one second".to_owned(),
                },
            ));
        }
        Ok(())
    }
}

impl PocketIcManagedServer {
    /// OS process ID of the owned server child, for caller-managed monitoring.
    ///
    /// This identifies the server process, not its descendants, and does not
    /// establish that it is still running. The OS may reuse the ID after the
    /// child exits and is reaped. Dropping this handle terminates and waits for
    /// the child; retaining the ID does not retain server ownership.
    #[must_use]
    pub fn process_id(&self) -> u32 {
        self.server
            .child
            .as_ref()
            .expect("managed server handle must own its child")
            .id()
    }

    /// Loopback URL published by the managed server.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Current bounded stdout and stderr captured from the managed server.
    ///
    /// This reads at most the first 16 KiB from each retained output file,
    /// renders it as lossy UTF-8, and adds an omitted-byte suffix when truncated.
    /// The diagnostic read is bounded; files can grow while the server runs.
    #[must_use]
    pub fn output(&self) -> PocketIcManagedServerOutput {
        self.server.capture()
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
                    config.server_idle_ttl,
                    config.server_output_files,
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
    started.checked_add(timeout).ok_or_else(|| {
        PocketIcStartupError::new(PocketIcStartupFailure::InvalidConfiguration {
            message: "PocketIC startup timeout exceeds the platform clock range".to_owned(),
        })
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
            return Err(finalize_failure(
                PocketIcStartupError::new(PocketIcStartupFailure::InvalidServerUrl {
                    server_url: server_url.to_owned(),
                    message: error.to_string(),
                }),
                None,
                server,
            ));
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
        return Err(finalize_failure(
            PocketIcStartupError::new(PocketIcStartupFailure::BuilderThreadSpawn { source }),
            None,
            server,
        ));
    }
    await_builder(receiver, deadline, timeout, &mut server)
}

fn await_builder(
    receiver: mpsc::Receiver<Result<PocketIc, String>>,
    deadline: Instant,
    timeout: Duration,
    server: &mut Option<ManagedServer>,
) -> Result<PocketIc, PocketIcStartupError> {
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(finalize_failure(
                PocketIcStartupError::new(PocketIcStartupFailure::InstanceCreationTimeout {
                    timeout,
                }),
                None,
                server.take(),
            ));
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
                    match managed.try_wait() {
                        Ok(Some(status)) => return Err(managed.exited_error(status)),
                        Ok(None) => managed.reap_in_background(),
                        Err(error) => return Err(finalize_failure(error, None, Some(managed))),
                    }
                }
                return Ok(pocket_ic);
            }
            Ok(Err(message)) => {
                return Err(finalize_failure(
                    PocketIcStartupError::new(PocketIcStartupFailure::BuilderPanicked { message }),
                    None,
                    server.take(),
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(finalize_failure(
                    PocketIcStartupError::new(PocketIcStartupFailure::BuilderDisconnected),
                    None,
                    server.take(),
                ));
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(managed) = server.as_mut() {
                    let error = match managed.try_wait() {
                        Ok(Some(status)) => Some(managed.exit_failure(status)),
                        Ok(None) => None,
                        Err(error) => Some(error),
                    };
                    if let Some(error) = error {
                        return Err(finalize_failure(error, None, server.take()));
                    }
                }
            }
        }
    }
}

/// Finalize an owned operation once, preserving the primary failure.
fn finalize_failure(
    error: PocketIcStartupError,
    command: Option<&mut OwnedChild>,
    server: Option<ManagedServer>,
) -> PocketIcStartupError {
    error.with_cleanup(
        command.and_then(|child| child.terminate().err()),
        server.map(ManagedServer::terminate_and_capture),
    )
}

struct ManagedServer {
    child: Option<OwnedChild>,
    binary: PathBuf,
    files: StartupFiles,
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
        hard_ttl: Option<Duration>,
        idle_ttl: Option<Duration>,
        output_files: Option<(PathBuf, PathBuf)>,
        deadline: Instant,
        timeout: Duration,
        started: Instant,
    ) -> Result<(Self, String), PocketIcStartupError> {
        let (files, stdout, stderr) = StartupFiles::create(output_files)?;
        let mut command = Command::new(&binary);
        if let Some(hard_ttl) = hard_ttl {
            command
                .arg("--hard-ttl")
                .arg(hard_ttl.as_secs().to_string());
        }
        if let Some(idle_ttl) = idle_ttl {
            command.arg("--ttl").arg(idle_ttl.as_secs().to_string());
        }
        command
            .arg("--port-file")
            .arg(&files.port)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        let child = OwnedChild::spawn(&mut command).map_err(|source| {
            PocketIcStartupError::new(PocketIcStartupFailure::ServerSpawn {
                server_binary: binary.clone(),
                source,
            })
        })?;
        let mut server = Self {
            child: Some(child),
            binary,
            files,
            started,
        };

        loop {
            match server.try_wait() {
                Ok(Some(status)) => return Err(server.exited_error(status)),
                Ok(None) => {}
                Err(error) => return Err(finalize_failure(error, None, Some(server))),
            }
            let now = Instant::now();
            if now >= deadline {
                let error = PocketIcStartupError::new(PocketIcStartupFailure::ReadinessTimeout {
                    server_binary: server.binary.clone(),
                    timeout,
                });
                return Err(finalize_failure(error, None, Some(server)));
            }
            match server.read_port() {
                Ok(PortFileState::Pending) => {}
                Ok(PortFileState::Ready(port)) => {
                    return Ok((server, format!("http://127.0.0.1:{port}/")));
                }
                Ok(PortFileState::Invalid(value)) => {
                    let error =
                        PocketIcStartupError::new(PocketIcStartupFailure::InvalidServerPort {
                            server_binary: server.binary.clone(),
                            value,
                        });
                    return Err(finalize_failure(error, None, Some(server)));
                }
                Err(error) => return Err(finalize_failure(error, None, Some(server))),
            }
            thread::sleep(
                deadline
                    .saturating_duration_since(now)
                    .min(STARTUP_POLL_INTERVAL),
            );
        }
    }

    fn try_wait(&mut self) -> Result<Option<ExitStatus>, PocketIcStartupError> {
        let child = self
            .child
            .as_mut()
            .expect("managed server child must remain present");
        child.try_wait().map_err(|source| {
            PocketIcStartupError::new(PocketIcStartupFailure::Io {
                operation: "inspect PocketIC server child",
                path: self.binary.clone(),
                source,
            })
        })
    }

    fn read_port(&self) -> Result<PortFileState, PocketIcStartupError> {
        let port_path = &self.files.port;
        let mut contents = String::new();
        match open_regular_startup_file(port_path).and_then(|file| {
            file.take((SERVER_PORT_FILE_LIMIT + 1) as u64)
                .read_to_string(&mut contents)
        }) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PortFileState::Pending);
            }
            Err(source) => {
                return Err(PocketIcStartupError::new(PocketIcStartupFailure::Io {
                    operation: "read PocketIC server port file",
                    path: port_path.clone(),
                    source,
                }));
            }
        }
        if contents.len() > SERVER_PORT_FILE_LIMIT {
            return Ok(PortFileState::Invalid(format!(
                "{} (port file exceeds {SERVER_PORT_FILE_LIMIT} bytes)",
                contents.trim()
            )));
        }
        if !contents.contains('\n') {
            return Ok(PortFileState::Pending);
        }
        let value = contents.trim().to_owned();
        match value.parse::<u16>() {
            Ok(port) if port != 0 => Ok(PortFileState::Ready(port)),
            _ => Ok(PortFileState::Invalid(value)),
        }
    }

    fn exit_failure(&self, status: ExitStatus) -> PocketIcStartupError {
        PocketIcStartupError::new(PocketIcStartupFailure::ServerExited {
            server_binary: self.binary.clone(),
            status,
            elapsed: self.started.elapsed(),
        })
    }

    fn exited_error(self, status: ExitStatus) -> PocketIcStartupError {
        let error = self.exit_failure(status);
        finalize_failure(error, None, Some(self))
    }

    fn terminate_and_capture(mut self) -> (PocketIcManagedServerOutput, Option<CleanupError>) {
        let cleanup = self
            .child
            .take()
            .and_then(|mut child| child.terminate().err());
        (self.capture(), cleanup)
    }

    fn capture(&self) -> PocketIcManagedServerOutput {
        PocketIcManagedServerOutput {
            stdout: read_bounded_lossy(&self.files.stdout),
            stderr: read_bounded_lossy(&self.files.stderr),
        }
    }

    fn reap_in_background(self) {
        let _ = thread::Builder::new()
            .name("ic-testkit-pocket-ic-server-reaper".to_owned())
            .spawn(move || {
                // Keep child and files under one owner, including if spawning
                // this thread fails. On Unix, Drop terminates the group before reaping.
                let mut server = self;
                if let Some(child) = server.child.as_mut() {
                    let _ = child.wait();
                }
            });
    }
}

struct StartupFiles {
    directory: PathBuf,
    port: PathBuf,
    stdout: PathBuf,
    stderr: PathBuf,
}

impl StartupFiles {
    fn create(
        output_files: Option<(PathBuf, PathBuf)>,
    ) -> Result<(Self, File, File), PocketIcStartupError> {
        let output_files = output_files
            .map(|(stdout, stderr)| {
                Ok::<_, PocketIcStartupError>((
                    std::path::absolute(&stdout)
                        .map_err(|source| startup_file_error("resolve", &stdout, source))?,
                    std::path::absolute(&stderr)
                        .map_err(|source| startup_file_error("resolve", &stderr, source))?,
                ))
            })
            .transpose()?;
        loop {
            let sequence = STARTUP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let base = std::env::temp_dir().join(format!(
                "ic-testkit-pocket-ic-startup-{}-{sequence}",
                std::process::id()
            ));
            let mut directory = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                directory.mode(0o700);
            }
            match directory.create(&base) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => return Err(startup_file_error("create", &base, source)),
            }
            let (stdout_path, stderr_path) =
                output_files.unwrap_or_else(|| (base.join("stdout"), base.join("stderr")));
            let files = Self {
                port: base.join("port"),
                stdout: stdout_path,
                stderr: stderr_path,
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
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path)
}

fn startup_file_error(
    operation: &'static str,
    path: &Path,
    source: io::Error,
) -> PocketIcStartupError {
    PocketIcStartupError::new(PocketIcStartupFailure::Io {
        operation,
        path: path.to_owned(),
        source,
    })
}

fn read_bounded_lossy(path: &Path) -> String {
    let Ok(file) = open_regular_startup_file(path) else {
        return String::new();
    };
    let length = file.metadata().map_or(0, |metadata| metadata.len());
    let Ok(bytes) = ic_host_artifacts::artifact::read_reader(
        file.take(SERVER_OUTPUT_LIMIT as u64),
        SERVER_OUTPUT_LIMIT,
    ) else {
        return String::new();
    };
    let mut output = String::from_utf8_lossy(&bytes).into_owned();
    let omitted = length.saturating_sub(bytes.len() as u64);
    if omitted > 0 {
        let _ = write!(output, "\n<truncated {omitted} bytes>");
    }
    output
}

fn open_regular_startup_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    // Bound opening a replaced FIFO as well as reading file contents. Inspect
    // the opened file, rather than a path that can change before open completes.
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK);
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PocketIC startup reader requires a regular file",
        ));
    }
    Ok(file)
}

impl std::fmt::Display for PocketIcStartupFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => formatter.write_str("configure IC_TESTKIT_POCKET_IC_URL or POCKET_IC_BIN; prepare a verified binary with make install-server"),
            Self::InvalidEnvironment { variable } => write!(formatter, "invalid selected environment value: {variable}"),
            Self::ServerVersionProbe { source } => write!(formatter, "PocketIC version probe failed: {source}"),
            Self::ServerVersionMismatch { expected, observed } => write!(formatter, "PocketIC version mismatch: expected {expected:?}, observed {:?}", String::from_utf8_lossy(observed)),
            Self::CommandRun { program, source } => write!(formatter, "command {} failed: {source}", program.display()),
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
            } => write!(
                formatter,
                "PocketIC server {} exited with {status} after {elapsed:?}",
                server_binary.display()
            ),
            Self::ReadinessTimeout {
                server_binary,
                timeout,
            } => write!(
                formatter,
                "PocketIC server {} was not ready within {timeout:?}",
                server_binary.display()
            ),
            Self::InvalidServerPort {
                server_binary,
                value,
            } => write!(
                formatter,
                "PocketIC server {} published invalid port {value:?}",
                server_binary.display()
            ),
            Self::InstanceCreationTimeout { timeout } => {
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
            Self::ServerCleanup { command_status } => write!(formatter, "PocketIC server cleanup failed after command exited with {command_status}"),
            Self::BuilderDisconnected => {
                formatter.write_str("PocketIC builder worker disconnected without a result")
            }
        }
    }
}

impl std::error::Error for PocketIcStartupFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ServerVersionProbe { source } => Some(source),
            Self::Io { source, .. }
            | Self::CommandRun { source, .. }
            | Self::ServerSpawn { source, .. }
            | Self::BuilderThreadSpawn { source } => Some(source),
            _ => None,
        }
    }
}

impl std::fmt::Display for PocketIcStartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.failure.as_ref(), formatter)?;
        if !self.output.stdout.is_empty() {
            write!(formatter, "; server stdout: {}", self.output.stdout)?;
        }
        if !self.output.stderr.is_empty() {
            write!(formatter, "; server stderr: {}", self.output.stderr)?;
        }
        for (owner, cleanup) in [
            ("command", self.command_cleanup()),
            ("server", self.server_cleanup()),
        ] {
            if let Some(error) = cleanup {
                write!(formatter, "; {owner} cleanup also failed: {error}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for PocketIcStartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        std::error::Error::source(self.failure.as_ref())
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
        PocketIcBuilderExt as _, PocketIcStartupConfig, PocketIcStartupError,
        PocketIcStartupFailure, StartupFiles,
    };
    use pocket_ic::PocketIcBuilder;

    #[cfg(unix)]
    use crate::test_executable::write_executable_script;
    #[cfg(unix)]
    use std::{
        io::Write as _,
        os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _},
        process::Command,
        sync::mpsc,
    };

    #[test]
    fn original_failure_output_and_both_typed_cleanup_reports_survive_projection() {
        use std::error::Error as _;
        for failure in [
            PocketIcStartupFailure::ReadinessTimeout {
                server_binary: "server".into(),
                timeout: Duration::from_secs(1),
            },
            PocketIcStartupFailure::InstanceCreationTimeout {
                timeout: Duration::from_secs(1),
            },
            PocketIcStartupFailure::InvalidServerPort {
                server_binary: "server".into(),
                value: "invalid".into(),
            },
            PocketIcStartupFailure::BuilderPanicked {
                message: "original-panic".into(),
            },
            PocketIcStartupFailure::BuilderDisconnected,
            PocketIcStartupFailure::BuilderThreadSpawn {
                source: std::io::Error::from_raw_os_error(9),
            },
            PocketIcStartupFailure::Io {
                operation: "original-io",
                path: "port".into(),
                source: std::io::Error::from_raw_os_error(9),
            },
        ] {
            let primary_text = failure.to_string();
            let output = super::PocketIcManagedServerOutput {
                stdout: "original-stdout".into(),
                stderr: "original-stderr".into(),
            };
            let command = super::CleanupError {
                status: None,
                group_error: None,
                term_error: Some(std::io::Error::from_raw_os_error(4)),
                kill_error: Some(std::io::Error::from_raw_os_error(5)),
                wait_error: None,
            };
            let server = super::CleanupError {
                status: None,
                group_error: Some(std::io::Error::from_raw_os_error(1)),
                term_error: None,
                kill_error: None,
                wait_error: Some(std::io::Error::from_raw_os_error(10)),
            };
            let error = PocketIcStartupError::new(failure)
                .with_cleanup(Some(command), Some((output, Some(server))));
            assert_eq!(error.failure().to_string(), primary_text);
            assert_eq!(error.output().stdout(), "original-stdout");
            assert_eq!(error.output().stderr(), "original-stderr");
            let command = error.command_cleanup().unwrap();
            let server = error.server_cleanup().unwrap();
            assert_eq!(command.kill_error.as_ref().unwrap().raw_os_error(), Some(5));
            assert_eq!(command.term_error.as_ref().unwrap().raw_os_error(), Some(4));
            assert_eq!(server.group_error.as_ref().unwrap().raw_os_error(), Some(1));
            assert_eq!(server.wait_error.as_ref().unwrap().raw_os_error(), Some(10));
            if matches!(
                error.failure(),
                PocketIcStartupFailure::Io { .. }
                    | PocketIcStartupFailure::BuilderThreadSpawn { .. }
            ) {
                assert_eq!(
                    error
                        .source()
                        .unwrap()
                        .downcast_ref::<std::io::Error>()
                        .unwrap()
                        .raw_os_error(),
                    Some(9)
                );
            }
            let displayed = error.to_string();
            assert!(displayed.starts_with(&primary_text));
            for marker in [
                "original-stdout",
                "original-stderr",
                "command cleanup also failed",
                "server cleanup also failed",
            ] {
                assert!(displayed.contains(marker));
            }
        }
        let error = PocketIcStartupError::new(PocketIcStartupFailure::NotConfigured);
        assert_eq!(error.to_string(), error.failure().to_string());
        assert!(error.command_cleanup().is_none());
        assert!(error.server_cleanup().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn builder_failures_finalize_the_owned_server_and_keep_output() {
        for disconnected in [false, true] {
            let script = TestServerScript::new(
                "builder-failure",
                "#!/bin/sh\nprintf '%s\n%s\n' \"$$\" \"$2\"\nprintf 'builder-diagnostic' >&2\nprintf '34567\n' > \"$2\"\nexec sleep 30\n",
            );
            let managed = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
                .start_managed_server()
                .unwrap();
            let pid = managed.process_id();
            let directory = managed.server.files.directory.clone();
            let (sender, receiver) = mpsc::channel();
            if !disconnected {
                sender
                    .send(Err("original-builder-panic".to_owned()))
                    .unwrap();
            }
            drop(sender);
            let error = super::await_builder(
                receiver,
                Instant::now() + Duration::from_secs(2),
                Duration::from_secs(2),
                &mut Some(managed.server),
            )
            .err()
            .unwrap();
            assert!(matches!(
                (disconnected, error.failure()),
                (true, PocketIcStartupFailure::BuilderDisconnected)
                    | (false, PocketIcStartupFailure::BuilderPanicked { .. })
            ));
            assert_eq!(error.output().stderr(), "builder-diagnostic");
            assert!(error.server_cleanup().is_none());
            assert!(!directory.exists());
            assert_eq!(process_state(pid), None);
        }
    }

    #[cfg(unix)]
    #[test]
    fn port_io_and_command_spawn_failures_finalize_the_owned_server() {
        for port_io in [true, false] {
            let port = if port_io {
                "mkfifo \"$2\""
            } else {
                "printf '34567\n' > \"$2\""
            };
            let script = TestServerScript::new(
                "owned-io-failure",
                &format!(
                    "#!/bin/sh\nprintf '%s\n%s\n' \"$$\" \"$2\"\nprintf 'original-server-diagnostic' >&2\n{port}\nexec sleep 30\n"
                ),
            );
            let config = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2));
            let error = if port_io {
                config.start_managed_server().err().unwrap()
            } else {
                config
                    .run_command(&mut Command::new("/missing/ic-testkit-command"), || false)
                    .unwrap_err()
            };
            let PocketIcStartupFailure::Io { source, .. } = error.failure() else {
                panic!("expected original IO failure: {error:?}");
            };
            assert_eq!(
                source.kind(),
                if port_io {
                    std::io::ErrorKind::InvalidData
                } else {
                    std::io::ErrorKind::NotFound
                }
            );
            assert_eq!(error.output().stderr(), "original-server-diagnostic");
            let mut lines = error.output().stdout().lines();
            let pid = lines.next().unwrap().parse().unwrap();
            let port = PathBuf::from(lines.next().unwrap());
            assert_eq!(process_state(pid), None);
            assert!(!port.parent().unwrap().exists());
            assert!(error.server_cleanup().is_none());
        }
    }

    #[cfg(unix)]
    #[test]
    fn command_cancellation_before_startup_has_no_spawn_effects() {
        let error = PocketIcStartupConfig::spawn("/missing/server", Duration::from_secs(1))
            .run_command(&mut Command::new("/missing/command"), || true)
            .unwrap_err();
        assert!(
            matches!(error.failure(), PocketIcStartupFailure::CommandRun { source, .. } if source.kind() == std::io::ErrorKind::Interrupted)
        );
        assert_eq!(error.output().stdout(), "");
        assert_eq!(error.output().stderr(), "");
        assert!(error.command_cleanup().is_none());
        assert!(error.server_cleanup().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_callback_panic_still_reaps_the_owned_command() {
        let script = TestServerScript::new(
            "cancel-panic",
            "#!/bin/sh\nprintf '%s' \"$$\" > \"$1\"\nexec sleep 30\n",
        );
        let pid_file = script.path().with_extension("pid");
        let result = std::panic::catch_unwind(|| {
            PocketIcStartupConfig::connect("http://127.0.0.1:12345/", Duration::from_secs(1))
                .run_command(Command::new(script.path()).arg(&pid_file), || {
                    assert!(
                        !fs::read_to_string(&pid_file).is_ok_and(|value| !value.is_empty()),
                        "caller cancellation failed"
                    );
                    false
                })
        });
        assert!(result.is_err());
        let pid = fs::read_to_string(&pid_file).unwrap().parse().unwrap();
        assert!(process_state(pid).is_none_or(|state| state == 'Z'));
        fs::remove_file(pid_file).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn environment_selection_prefers_urls_and_fails_closed() {
        use std::os::unix::ffi::OsStringExt as _;

        let timeout = Duration::from_secs(1);
        let config = PocketIcStartupConfig::from_environment(timeout, |name| {
            Some(
                if name == "IC_TESTKIT_POCKET_IC_URL" {
                    "http://127.0.0.1:12345/"
                } else {
                    "/missing/server"
                }
                .into(),
            )
        })
        .unwrap();
        assert_eq!(config.server_url(), Some("http://127.0.0.1:12345/"));
        assert!(config.server_binary().is_none());
        assert!(matches!(
            PocketIcStartupConfig::from_environment(timeout, |_| None)
                .as_ref()
                .map_err(PocketIcStartupError::failure),
            Err(PocketIcStartupFailure::NotConfigured)
        ));
        assert!(matches!(
            PocketIcStartupConfig::from_environment(timeout, |_| Some("".into()))
                .as_ref()
                .map_err(PocketIcStartupError::failure),
            Err(PocketIcStartupFailure::InvalidEnvironment {
                variable: "IC_TESTKIT_POCKET_IC_URL"
            })
        ));
        assert!(matches!(
            PocketIcStartupConfig::from_environment(timeout, |_| Some("bad URL".into()))
                .as_ref()
                .map_err(PocketIcStartupError::failure),
            Err(PocketIcStartupFailure::InvalidServerUrl { .. })
        ));
        assert!(matches!(
            PocketIcStartupConfig::from_environment(timeout, |_| Some(
                std::ffi::OsString::from_vec(vec![0xff])
            ))
            .as_ref()
            .map_err(PocketIcStartupError::failure),
            Err(PocketIcStartupFailure::InvalidEnvironment { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn version_capture_cleans_wrapper_descendants_on_exit_and_timeout() {
        for timeout in [false, true] {
            let qualified = format!(
                "printf 'pocket-ic-server {}\\n'",
                pocket_ic::LATEST_SERVER_VERSION
            );
            let ending = if timeout { "wait" } else { &qualified };
            let script = TestServerScript::new(
                "version-group",
                &format!(
                    "#!/bin/sh\nsleep 30 >/dev/null 2>&1 &\nprintf '%s' \"$!\" > \"$0.pid\"\n{ending}\n"
                ),
            );
            let result = PocketIcStartupConfig::from_environment(Duration::from_secs(1), |name| {
                (name == "POCKET_IC_BIN").then(|| script.path().into_os_string())
            });
            if timeout {
                assert!(matches!(
                    result.as_ref().map_err(PocketIcStartupError::failure),
                    Err(PocketIcStartupFailure::ServerVersionProbe { .. })
                ));
            } else {
                assert!(result.is_ok());
            }
            let pid_file = script.path().with_extension("pid");
            let pid = fs::read_to_string(&pid_file).unwrap().parse().unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            while process_state(pid).is_some_and(|state| state != 'Z') {
                assert!(
                    Instant::now() < deadline,
                    "version wrapper descendant remained alive"
                );
                std::thread::sleep(Duration::from_millis(20));
            }
            fs::remove_file(pid_file).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn owned_builder_panic_retains_server_output_and_reaps_child() {
        let script = TestServerScript::new(
            "builder-panic",
            "#!/bin/sh\nprintf '%s' \"$$\" > \"$0.pid\"\nprintf 'builder stdout'\nprintf 'builder stderr' >&2\nprintf '34567\\n' > \"$2\"\nexec sleep 30\n",
        );
        let error = PocketIcBuilder::new()
            .try_build(PocketIcStartupConfig::spawn(
                script.path(),
                Duration::from_secs(2),
            ))
            .err()
            .unwrap();
        let PocketIcStartupFailure::BuilderPanicked { message } = error.failure() else {
            panic!("expected builder failure, got {error:?}");
        };
        assert_ne!(message, "");
        assert_eq!(error.output().stdout(), "builder stdout");
        assert_eq!(error.output().stderr(), "builder stderr");
        assert!(error.server_cleanup().is_none());
        let pid_file = script.path().with_extension("pid");
        let pid = fs::read_to_string(&pid_file).unwrap().parse().unwrap();
        assert_eq!(process_state(pid), None);
        fs::remove_file(pid_file).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn environment_binary_selection_uses_bounded_shared_version_capture() {
        let qualified = format!(
            "printf 'pocket-ic-server {}\\n'",
            pocket_ic::LATEST_SERVER_VERSION
        );
        let failed = format!("{qualified}; exit 23");
        for (label, body, expected) in [
            ("qualified", qualified.as_str(), 0),
            ("compatible-16.0", "printf 'pocket-ic-server 16.0.0\\n'", 0),
            ("compatible-patch", "printf 'pocket-ic-server 16.0.1\\n'", 0),
            ("wrong-version", "printf 'pocket-ic-server 15.0.0\\n'", 1),
            ("future-major", "printf 'pocket-ic-server 17.0.0\\n'", 1),
            ("malformed-version", "printf 'pocket-ic-server 16.1\\n'", 1),
            ("failed-version", failed.as_str(), 2),
            ("invalid-utf8", "printf '\\377'", 1),
            ("version-timeout", "exec sleep 30", 2),
        ] {
            let script = TestServerScript::new(
                label,
                &format!("#!/bin/sh\n[ \"$1\" = --version ] || exit 99\n{body}\n"),
            );
            let result =
                PocketIcStartupConfig::from_environment(Duration::from_millis(200), |name| {
                    (name == "POCKET_IC_BIN").then(|| script.path().into_os_string())
                });
            match (
                expected,
                result.as_ref().map_err(PocketIcStartupError::failure),
            ) {
                (0, Ok(config)) => {
                    let binary = script.path().canonicalize().unwrap();
                    assert_eq!(config.server_binary(), Some(binary.as_path()));
                }
                (1, Err(PocketIcStartupFailure::ServerVersionMismatch { .. }))
                | (2, Err(PocketIcStartupFailure::ServerVersionProbe { .. })) => {}
                (_, result) => panic!("unexpected {label} result: {result:?}"),
            }
        }
    }

    #[cfg(unix)]
    fn process_state(pid: u32) -> Option<char> {
        // Both supported Unix hosts provide this ps field. A zombie has stopped
        // running but may remain visible until its parent reaps it.
        let output = Command::new("/bin/ps")
            .args(["-p", &pid.to_string(), "-o", "stat="])
            .output()
            .expect("inspect managed test process state");
        assert!(
            output.status.success()
                || (output.status.code() == Some(1)
                    && output.stdout.is_empty()
                    && output.stderr.is_empty()),
            "process-state inspection failed: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8(output.stdout)
            .expect("process state is ASCII")
            .trim()
            .chars()
            .next()
    }

    #[cfg(unix)]
    #[test]
    fn reading_large_sparse_server_output_is_bounded() {
        let (files, _, _) = StartupFiles::create(None).expect("allocate startup files");
        let mut file = fs::File::create(&files.stdout).expect("create sparse log");
        file.write_all(b"server started\n").expect("write prefix");
        let size = 8_u64 * 1024 * 1024 * 1024;
        file.set_len(size).expect("extend sparse log");
        let output = super::read_bounded_lossy(&files.stdout);
        assert!(output.starts_with("server started\n"));
        assert!(output.ends_with(&format!(
            "<truncated {} bytes>",
            size - super::SERVER_OUTPUT_LIMIT as u64
        )));
        assert!(output.len() < super::SERVER_OUTPUT_LIMIT + 100);
    }

    #[cfg(unix)]
    #[test]
    fn startup_readers_reject_fifos_without_waiting_for_a_writer() {
        let (files, stdout, stderr) = StartupFiles::create(None).expect("allocate startup files");
        drop((stdout, stderr));
        fs::remove_file(&files.stdout).unwrap();
        fs::remove_file(&files.stderr).unwrap();
        assert!(
            Command::new("mkfifo")
                .args([&files.port, &files.stdout, &files.stderr])
                .status()
                .expect("create FIFO startup files")
                .success()
        );
        let server = super::ManagedServer {
            child: None,
            binary: PathBuf::from("unused-server"),
            files,
            started: Instant::now(),
        };
        for path in [
            &server.files.port,
            &server.files.stdout,
            &server.files.stderr,
        ] {
            // A delayed writer bounds a blocked read and records whether the
            // reader needed it. Completion cancels the writer; with no reader,
            // a nonblocking open fails and is retried if the reader starts late.
            let fifo = path.clone();
            let (stop_writer, stopped) = mpsc::channel();
            let writer = std::thread::spawn(move || {
                loop {
                    match stopped.recv_timeout(Duration::from_millis(200)) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return false,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                    if fs::OpenOptions::new()
                        .write(true)
                        .custom_flags(libc::O_NONBLOCK)
                        .open(&fifo)
                        .is_ok()
                    {
                        return true;
                    }
                }
            });
            let result = if path == &server.files.port {
                Some(server.read_port())
            } else {
                assert_eq!(super::read_bounded_lossy(path), "");
                None
            };
            let _ = stop_writer.send(());
            assert!(
                !writer.join().expect("join delayed FIFO writer"),
                "startup reader waited for a writer: {}",
                path.display(),
            );
            if let Some(result) = result {
                assert!(matches!(
                    result.as_ref().map_err(PocketIcStartupError::failure),
                    Err(PocketIcStartupFailure::Io { source, .. })
                        if source.kind() == std::io::ErrorKind::InvalidData
                ));
            }
        }
    }

    #[test]
    fn port_file_readiness_preserves_partial_writes_and_rejects_oversized_contents() {
        let (files, _, _) = StartupFiles::create(None).expect("allocate startup files");
        let server = super::ManagedServer {
            child: None,
            binary: PathBuf::from("unused-server"),
            files,
            started: Instant::now(),
        };
        assert!(matches!(
            server.read_port().unwrap(),
            super::PortFileState::Pending
        ));
        for contents in ["", "34567"] {
            fs::write(&server.files.port, contents).unwrap();
            assert!(matches!(
                server.read_port().unwrap(),
                super::PortFileState::Pending
            ));
        }
        for (contents, expected) in [("1\n", 1), ("65535\n", 65535), (" 34567\r\n", 34567)] {
            fs::write(&server.files.port, contents).unwrap();
            assert!(matches!(
                server.read_port().unwrap(),
                super::PortFileState::Ready(port) if port == expected
            ));
        }
        for contents in ["0\n", "65536\n", "invalid\n", "1\n2\n"] {
            fs::write(&server.files.port, contents).unwrap();
            assert!(matches!(
                server.read_port().unwrap(),
                super::PortFileState::Invalid(_)
            ));
        }
        fs::write(&server.files.port, [0xff, b'\n']).unwrap();
        assert!(matches!(
            server.read_port().as_ref().map_err(PocketIcStartupError::failure),
            Err(PocketIcStartupFailure::Io { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidData
        ));
        for contents in ["1\n".to_owned() + &" ".repeat(128), "0".repeat(128)] {
            fs::write(&server.files.port, contents).unwrap();
            assert!(
                matches!(
                    server.read_port().unwrap(),
                    super::PortFileState::Invalid(_)
                ),
                "oversized port contents must fail even without a newline",
            );
        }
        // A large backing file must not enlarge the returned diagnostic.
        fs::File::options()
            .write(true)
            .open(&server.files.port)
            .unwrap()
            .set_len(1024 * 1024)
            .unwrap();
        assert!(matches!(
            server.read_port().unwrap(),
            super::PortFileState::Invalid(value) if value.len() < 256
        ));
    }

    #[test]
    fn startup_config_requires_positive_bounds() {
        for config in [
            PocketIcStartupConfig::spawn("pocket-ic", Duration::from_secs(1))
                .with_server_idle_ttl(Duration::from_millis(1)),
            PocketIcStartupConfig::connect("http://127.0.0.1:1/", Duration::from_secs(1))
                .with_server_idle_ttl(Duration::from_secs(120)),
        ] {
            assert!(matches!(
                config
                    .validate()
                    .as_ref()
                    .map_err(PocketIcStartupError::failure),
                Err(PocketIcStartupFailure::InvalidConfiguration { .. })
            ));
        }
        let error = PocketIcStartupConfig::connect("http://127.0.0.1:1/", Duration::ZERO)
            .validate()
            .expect_err("zero startup timeout must fail");
        assert!(matches!(
            error.failure(),
            PocketIcStartupFailure::InvalidConfiguration { .. }
        ));

        let error = PocketIcStartupConfig::spawn("pocket-ic", Duration::from_secs(1))
            .with_server_hard_ttl(Duration::from_millis(1))
            .validate()
            .expect_err("subsecond server hard TTL must fail");
        assert!(matches!(
            error.failure(),
            PocketIcStartupFailure::InvalidConfiguration { .. }
        ));
    }

    #[test]
    fn managed_server_hard_ttl_is_opt_in() {
        let default = PocketIcStartupConfig::spawn("pocket-ic", Duration::from_secs(1));
        assert_eq!(default.server_idle_ttl(), None);
        assert_eq!(
            default
                .clone()
                .with_server_idle_ttl(Duration::from_secs(120))
                .server_idle_ttl(),
            Some(Duration::from_secs(120))
        );
        assert_eq!(default.server_hard_ttl(), None);

        let explicit = default.with_server_hard_ttl(Duration::from_secs(17));
        assert_eq!(explicit.server_hard_ttl(), Some(Duration::from_secs(17)));
    }

    #[test]
    fn startup_files_leave_the_server_owned_port_path_absent() {
        let (files, stdout, stderr) = StartupFiles::create(None).expect("allocate startup files");
        let directory = files.directory.clone();

        assert!(directory.is_dir());
        assert!(!files.port.exists());
        assert!(files.stdout.is_file());
        assert!(files.stderr.is_file());
        #[cfg(unix)]
        {
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
    fn caller_output_files_survive_startup_command_and_cancellation_cleanup() {
        for outcome in [
            "timeout",
            "startup-exit",
            "server-exit",
            "command-exit",
            "cancel",
            "success",
        ] {
            let (owner, _, _) = StartupFiles::create(None).unwrap();
            let stdout = owner.directory.join("retained-stdout");
            let stderr = owner.directory.join("retained-stderr");
            let ending = match outcome {
                "timeout" => "exec sleep 30",
                "startup-exit" => "exit 41",
                "server-exit" => {
                    "printf '34567\\n' > \"$2\"; while [ ! -s \"$0.command\" ]; do sleep 0.02; done; exit 42"
                }
                _ => "printf '34567\\n' > \"$2\"; exec sleep 30",
            };
            let script = TestServerScript::new(
                outcome,
                &format!(
                    "#!/bin/sh\nprintf '%s\\n%s\\n' \"$$\" \"$2\" > \"$0.pid\"\ndd if=/dev/zero bs=1024 count=20 2>/dev/null\nprintf raw-stdout-end\ndd if=/dev/zero bs=1024 count=20 >&2 2>/dev/null\nprintf raw-stderr-end >&2\n{ending}\n"
                ),
            );
            let pid_file = script.path().with_extension("pid");
            let command_file = script.path().with_extension("command");
            let config = PocketIcStartupConfig::spawn(
                script.path(),
                Duration::from_millis(if outcome == "timeout" { 1000 } else { 2000 }),
            )
            .with_server_output_files(&stdout, &stderr);
            if outcome == "timeout" || outcome == "startup-exit" {
                let error = config.start_managed_server().err().unwrap();
                match (outcome, error.failure()) {
                    ("timeout", PocketIcStartupFailure::ReadinessTimeout { .. }) => {
                        assert!(error.output().stdout().contains("truncated"));
                        assert!(!error.output().stderr().contains("raw-stderr-end"));
                    }
                    ("startup-exit", PocketIcStartupFailure::ServerExited { status, .. }) => {
                        assert_eq!(status.code(), Some(41));
                    }
                    (_, error) => panic!("unexpected startup result: {error:?}"),
                }
            } else {
                let command_end = match outcome {
                    "command-exit" => "exit 37",
                    "success" => "exit 0",
                    _ => "exec sleep 30",
                };
                let result = config.run_command(
                    Command::new("/bin/sh")
                        .args([
                            "-c",
                            &format!("printf '%s' \"$$\" > \"$1\"; {command_end}"),
                            "fixture",
                        ])
                        .arg(&command_file),
                    || {
                        outcome == "cancel"
                            && fs::metadata(&command_file).is_ok_and(|m| m.len() > 0)
                    },
                );
                let result = result.as_ref().map_err(PocketIcStartupError::failure);
                match (outcome, result) {
                    ("command-exit", Ok(status)) => assert_eq!(status.code(), Some(37)),
                    ("success", Ok(status)) => assert!(status.success()),
                    ("server-exit", Err(PocketIcStartupFailure::ServerExited { status, .. })) => {
                        assert_eq!(status.code(), Some(42));
                    }
                    ("cancel", Err(PocketIcStartupFailure::CommandRun { source, .. })) => {
                        assert_eq!(source.kind(), std::io::ErrorKind::Interrupted);
                    }
                    (_, result) => panic!("unexpected command result: {result:?}"),
                }
                let pid = fs::read_to_string(&command_file).unwrap().parse().unwrap();
                assert!(process_state(pid).is_none_or(|state| state == 'Z'));
                fs::remove_file(command_file).unwrap();
            }
            for (path, suffix) in [(&stdout, b"raw-stdout-end"), (&stderr, b"raw-stderr-end")] {
                let bytes = fs::read(path).unwrap();
                assert_eq!(bytes.len(), 20 * 1024 + suffix.len());
                assert!(bytes.ends_with(suffix));
                assert_eq!(
                    fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
            let report = fs::read_to_string(&pid_file).unwrap();
            let mut lines = report.lines();
            assert_eq!(process_state(lines.next().unwrap().parse().unwrap()), None);
            assert!(
                !PathBuf::from(lines.next().unwrap())
                    .parent()
                    .unwrap()
                    .exists()
            );
            fs::remove_file(pid_file).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn caller_output_files_refuse_existing_entries_and_keep_partial_preparation() {
        use std::os::unix::fs::symlink;

        let (owner, _, _) = StartupFiles::create(None).unwrap();
        let stdout = owner.directory.join("retained-stdout");
        let stderr = owner.directory.join("retained-stderr");
        let unrelated = owner.directory.join("unrelated");
        fs::write(&unrelated, b"original").unwrap();
        symlink(&unrelated, &stderr).unwrap();
        let error = StartupFiles::create(Some((stdout.clone(), stderr.clone())))
            .err()
            .unwrap();
        assert!(
            matches!(error.failure(), PocketIcStartupFailure::Io { source, .. } if source.kind() == std::io::ErrorKind::AlreadyExists)
        );
        assert!(stdout.is_file());
        assert_eq!(fs::read(&unrelated).unwrap(), b"original");
        fs::write(&stdout, b"retained attempt").unwrap();
        assert!(StartupFiles::create(Some((stdout.clone(), stderr.clone()))).is_err());
        assert_eq!(fs::read(&stdout).unwrap(), b"retained attempt");
        fs::remove_file(stderr.clone()).unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(&stderr)
                .status()
                .unwrap()
                .success()
        );
        fs::remove_file(stdout.clone()).unwrap();
        assert!(StartupFiles::create(Some((stdout.clone(), stderr.clone()))).is_err());
        assert!(stdout.is_file());
        let error =
            PocketIcStartupConfig::connect("http://127.0.0.1:12345/", Duration::from_secs(1))
                .with_server_output_files(&stdout, &stderr)
                .run_command(&mut Command::new("/missing/command"), || false)
                .unwrap_err();
        assert!(matches!(
            error.failure(),
            PocketIcStartupFailure::InvalidConfiguration { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn managed_startup_reports_an_exited_server_with_bounded_output() {
        let script = TestServerScript::new(
            "exit",
            "#!/bin/sh\nif [ \"$1\" != \"--port-file\" ] || [ -e \"$2\" ]; then exit 97; fi\nprintf 'synthetic server stdout'\nprintf 'synthetic bind failure' >&2\nexit 23\n",
        );

        let result = PocketIcBuilder::new().with_application_subnet().try_build(
            PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2)),
        );

        let error = result.err().unwrap();
        let PocketIcStartupFailure::ServerExited {
            server_binary,
            status,
            ..
        } = error.failure()
        else {
            panic!("expected a structured server exit, got {error:?}");
        };
        assert_eq!(server_binary, &script.path());
        assert_eq!(status.code(), Some(23));
        assert_eq!(error.output().stdout(), "synthetic server stdout");
        assert_eq!(error.output().stderr(), "synthetic bind failure");
    }

    #[cfg(unix)]
    #[test]
    fn managed_startup_rejects_oversized_port_files_and_cleans_up() {
        let script = TestServerScript::new(
            "oversized-port",
            "#!/bin/sh\nprintf '%s\\n%s\\n' \"$$\" \"$2\"\nprintf '34567\\n%064s' '' > \"$2.pending\"\nmv \"$2.pending\" \"$2\"\nexec sleep 30\n",
        );
        let result = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
            .start_managed_server();
        let error = result.err().unwrap();
        let PocketIcStartupFailure::InvalidServerPort { value, .. } = error.failure() else {
            panic!("oversized port publication must fail readiness");
        };
        assert!(value.contains("port file exceeds"));
        assert!(value.len() < 256);
        let mut lines = error.output().stdout().lines();
        let pid = lines.next().unwrap().parse::<u32>().unwrap();
        let port_path = PathBuf::from(lines.next().unwrap());
        assert!(!port_path.parent().unwrap().exists());
        assert_eq!(process_state(pid), None, "failed server must be reaped");
    }

    #[cfg(unix)]
    #[test]
    fn managed_startup_terminates_a_server_that_never_becomes_ready() {
        let script = TestServerScript::new(
            "timeout",
            "#!/bin/sh\nif [ \"$1\" != \"--port-file\" ] || [ -e \"$2\" ]; then exit 97; fi\nexec sleep 30\n",
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
        let error = result.err().unwrap();
        assert!(
            matches!(error.failure(), PocketIcStartupFailure::ReadinessTimeout {
            server_binary, timeout: actual_timeout,
        } if server_binary == &script.path() && *actual_timeout == timeout)
        );
        assert!(error.server_cleanup().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn managed_server_handle_exposes_process_id_url_output_and_raii_ownership() {
        let script = TestServerScript::new(
            "handle",
            "#!/bin/sh\nif [ \"$1\" != \"--port-file\" ] || [ -e \"$2\" ]; then echo 'unexpected managed server arguments' >&2; exit 97; fi\nprintf 'managed server ready: %s' \"$$\"\nprintf '34567\\n' > \"$2\"\nexec sleep 30\n",
        );

        let server = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
            .start_managed_server()
            .expect("start caller-owned managed server");

        assert_eq!(server.url(), "http://127.0.0.1:34567/");
        assert_eq!(
            server.output().stdout(),
            format!("managed server ready: {}", server.process_id())
        );
        assert_eq!(server.output().stderr(), "");
        let pid = server.process_id();
        assert!(process_state(pid).is_some_and(|state| state != 'Z'));
        drop(server);
        assert_eq!(process_state(pid), None, "owned server must be reaped");
    }

    #[cfg(unix)]
    #[test]
    fn managed_server_cleans_descendants_on_drop_timeout_exit_and_background_reap() {
        for mode in ["drop", "timeout", "exit", "background"] {
            let publish = if matches!(mode, "drop" | "background") {
                "printf '34567\\n' > \"$2\"\n"
            } else {
                ""
            };
            let finish = if mode == "background" {
                // The caller removes the port only after handing off to the
                // reaper, so slow native hosts cannot miss this ready server.
                "while [ -e \"$2\" ]; do sleep 0.01; done\nexit 23\n"
            } else if mode == "exit" {
                "sleep 0.03\nexit 23\n"
            } else {
                "exec sleep 30\n"
            };
            let script = TestServerScript::new(
                mode,
                &format!("#!/bin/sh\nsleep 30 &\nprintf '%s' \"$!\"\n{publish}{finish}"),
            );
            let result = PocketIcStartupConfig::spawn(script.path(), Duration::from_millis(300))
                .start_managed_server();
            let output = match result {
                Ok(server) => {
                    let output = server.output().stdout().to_owned();
                    if mode == "background" {
                        let port = server.server.files.port.clone();
                        server.server.reap_in_background();
                        fs::remove_file(port).expect("release the background server after handoff");
                    } else {
                        drop(server);
                    }
                    output
                }
                Err(error) => {
                    match error.failure() {
                        PocketIcStartupFailure::ReadinessTimeout { .. } => {
                            assert_eq!(mode, "timeout");
                            assert!(error.server_cleanup().is_none());
                        }
                        PocketIcStartupFailure::ServerExited { status, .. } => {
                            assert_eq!(mode, "exit");
                            assert_eq!(status.code(), Some(23));
                        }
                        _ => panic!("unexpected {mode} startup result: {error}"),
                    }
                    error.output().stdout().to_owned()
                }
            };
            let pid = output
                .parse::<u32>()
                .expect("server published its descendant PID");
            let deadline = Instant::now() + Duration::from_secs(2);
            while process_state(pid).is_some_and(|state| state != 'Z') {
                assert!(
                    Instant::now() < deadline,
                    "{mode} left its descendant running"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn managed_server_passes_an_explicit_hard_ttl() {
        let script = TestServerScript::new(
            "hard-ttl",
            "#!/bin/sh\nif [ \"$1\" != \"--hard-ttl\" ] || [ \"$2\" != \"17\" ] || [ \"$3\" != \"--port-file\" ] || [ -e \"$4\" ]; then exit 97; fi\nprintf '34567\\n' > \"$4\"\nexec sleep 30\n",
        );

        let server = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
            .with_server_hard_ttl(Duration::from_secs(17))
            .start_managed_server()
            .expect("start managed server with an explicit hard TTL");

        assert_eq!(server.url(), "http://127.0.0.1:34567/");
    }

    #[test]
    #[ignore = "requires POCKET_IC_BIN=<caller-provided PocketIC server binary>"]
    fn caller_provided_server_publishes_port_constructs_instance_and_cleans_up() {
        let binary = std::env::var_os("POCKET_IC_BIN")
            .map(PathBuf::from)
            .expect("set POCKET_IC_BIN to the exact server binary");
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
        let files = &server.server.files;
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
            write_executable_script(&path, contents);
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
