use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Command, ExitCode},
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};

use ic_testkit::pic::PocketIcStartupConfig;

mod provisioning;

static INTERRUPTED: AtomicI32 = AtomicI32::new(0);
const USAGE: &str = "usage: ic-testkit-server setup|check [--directory DIRECTORY]\n       ic-testkit-server run [--directory DIRECTORY] [--ttl SECONDS] [--startup-timeout SECONDS] [--server-stdout NEW-FILE --server-stderr NEW-FILE] -- COMMAND [ARG...]";

enum Arguments {
    Setup(PathBuf),
    Check(PathBuf),
    Run(Box<RunArguments>),
}

struct RunArguments {
    directory: Option<PathBuf>,
    timeout: Duration,
    ttl: Option<Duration>,
    output_files: Option<(PathBuf, PathBuf)>,
    command: Command,
}

extern "C" fn interrupted(signal: libc::c_int) {
    INTERRUPTED.store(signal, Ordering::Relaxed);
}

struct Signals {
    previous: Vec<(libc::c_int, libc::sigaction)>,
}

impl Signals {
    fn install() -> io::Result<Self> {
        let mut signals = Self {
            previous: Vec::new(),
        };
        for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
            // SAFETY: sigaction is a C structure; zero is valid initialization.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = interrupted as *const () as usize;
            // SAFETY: action's signal mask and previous action are writable.
            let mut previous = unsafe { std::mem::zeroed() };
            if unsafe { libc::sigemptyset(&raw mut action.sa_mask) } != 0
                || unsafe { libc::sigaction(signal, &raw const action, &raw mut previous) } != 0
            {
                return Err(io::Error::last_os_error());
            }
            signals.previous.push((signal, previous));
        }
        Ok(signals)
    }
}

impl Drop for Signals {
    fn drop(&mut self) {
        for (signal, previous) in &self.previous {
            // SAFETY: restore only the actions saved by this CLI process.
            unsafe { libc::sigaction(*signal, previous, std::ptr::null_mut()) };
        }
    }
}

fn parse(mut arguments: impl Iterator<Item = OsString>) -> Result<Arguments, String> {
    let action = arguments.next().ok_or(USAGE)?;
    if action == "setup" || action == "check" {
        let directory = match arguments.next() {
            None => PathBuf::from(provisioning::DEFAULT_DIRECTORY),
            Some(flag) if flag == "--directory" => PathBuf::from(
                arguments
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or(USAGE)?,
            ),
            _ => return Err(USAGE.to_owned()),
        };
        if arguments.next().is_some() {
            return Err(USAGE.to_owned());
        }
        return Ok(if action == "setup" {
            Arguments::Setup(directory)
        } else {
            Arguments::Check(directory)
        });
    }
    if action != "run" {
        return Err(USAGE.to_owned());
    }
    let mut directory = None;
    let mut timeout = Duration::from_secs(30);
    let mut ttl = None;
    let mut selected_timeout = false;
    let mut stdout = None;
    let mut stderr = None;
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            let executable = arguments
                .next()
                .filter(|value| !value.is_empty())
                .ok_or(USAGE)?;
            let mut command = Command::new(executable);
            command.args(arguments);
            let output_files = match (stdout, stderr) {
                (Some(stdout), Some(stderr)) => Some((stdout, stderr)),
                (None, None) => None,
                _ => {
                    return Err(
                        "--server-stdout and --server-stderr must be selected together".to_owned(),
                    );
                }
            };
            return Ok(Arguments::Run(Box::new(RunArguments {
                directory,
                timeout,
                ttl,
                output_files,
                command,
            })));
        }
        if argument == "--directory" && directory.is_none() {
            directory = Some(PathBuf::from(
                arguments
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or(USAGE)?,
            ));
            continue;
        }
        if argument == "--server-stdout" || argument == "--server-stderr" {
            let selection = if argument == "--server-stdout" {
                &mut stdout
            } else {
                &mut stderr
            };
            if selection.is_some() {
                return Err(USAGE.to_owned());
            }
            *selection = Some(PathBuf::from(
                arguments
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or(USAGE)?,
            ));
            continue;
        }
        let is_ttl = argument == "--ttl";
        if !(is_ttl && ttl.is_none() || argument == "--startup-timeout" && !selected_timeout) {
            return Err(USAGE.to_owned());
        }
        let seconds = arguments
            .next()
            .and_then(|value| value.into_string().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .ok_or("timeouts must be positive whole seconds")?;
        if is_ttl {
            ttl = Some(Duration::from_secs(seconds));
        } else {
            timeout = Duration::from_secs(seconds);
            selected_timeout = true;
        }
    }
    Err(USAGE.to_owned())
}

pub fn main() -> ExitCode {
    let arguments = match parse(std::env::args_os().skip(1)) {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let arguments = match arguments {
        Arguments::Setup(directory) => return report_provisioning(provisioning::setup(&directory)),
        Arguments::Check(directory) => return report_provisioning(provisioning::check(&directory)),
        Arguments::Run(arguments) => arguments,
    };
    let RunArguments {
        directory,
        timeout,
        ttl,
        output_files,
        mut command,
    } = *arguments;
    let result = (|| {
        let _signals = Signals::install()?;
        let environment_selected = std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_some()
            || std::env::var_os("POCKET_IC_BIN").is_some();
        let mut config = if directory.is_some() || !environment_selected {
            if environment_selected {
                return Err(
                    "--directory conflicts with an explicit server URL or POCKET_IC_BIN".into(),
                );
            }
            let binary = provisioning::check(
                &directory.unwrap_or_else(|| PathBuf::from(provisioning::DEFAULT_DIRECTORY)),
            )?;
            command.env("POCKET_IC_BIN", &binary);
            PocketIcStartupConfig::spawn(binary, timeout)
        } else {
            PocketIcStartupConfig::from_env(timeout)?
        };
        if let Some(ttl) = ttl {
            if config.server_url().is_some() {
                return Err("--ttl requires an owned server".into());
            }
            config = config.with_server_hard_ttl(ttl);
        }
        if let Some((stdout, stderr)) = output_files {
            config = config.with_server_output_files(stdout, stderr);
        }
        Ok::<_, Box<dyn std::error::Error>>(
            config.run_command(&mut command, || INTERRUPTED.load(Ordering::Relaxed) != 0)?,
        )
    })();
    let signal = INTERRUPTED.load(Ordering::Relaxed);
    if signal != 0 {
        return ExitCode::from(u8::try_from(128 + signal).unwrap_or(1));
    }
    match result {
        Ok(status) => {
            use std::os::unix::process::ExitStatusExt as _;
            ExitCode::from(
                u8::try_from(
                    status
                        .code()
                        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1)),
                )
                .unwrap_or(1),
            )
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn report_provisioning(result: Result<PathBuf, Box<dyn std::error::Error>>) -> ExitCode {
    use std::io::Write as _;
    use std::os::unix::ffi::OsStrExt as _;
    match result {
        Ok(path) => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            if stdout
                .write_all(path.as_os_str().as_bytes())
                .and_then(|()| stdout.write_all(b"\n"))
                .is_err()
            {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
