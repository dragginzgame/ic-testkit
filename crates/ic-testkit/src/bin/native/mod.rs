use std::{
    ffi::OsString,
    io,
    process::{Command, ExitCode},
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};

use ic_testkit::pic::PocketIcStartupConfig;

static INTERRUPTED: AtomicI32 = AtomicI32::new(0);
const USAGE: &str =
    "usage: ic-testkit-server run [--ttl SECONDS] [--startup-timeout SECONDS] -- COMMAND [ARG...]";

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

fn parse(
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<(Duration, Option<Duration>, Command), String> {
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("run")) {
        return Err(USAGE.to_owned());
    }
    let mut timeout = Duration::from_secs(30);
    let mut ttl = None;
    let mut selected_timeout = false;
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            let executable = arguments
                .next()
                .filter(|value| !value.is_empty())
                .ok_or(USAGE)?;
            let mut command = Command::new(executable);
            command.args(arguments);
            return Ok((timeout, ttl, command));
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
    let (timeout, ttl, mut command) = match parse(std::env::args_os().skip(1)) {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let result = (|| {
        let _signals = Signals::install()?;
        let mut config = PocketIcStartupConfig::from_env(timeout)?;
        if let Some(ttl) = ttl {
            if config.server_url().is_some() {
                return Err("--ttl requires an owned server selected by POCKET_IC_BIN".into());
            }
            config = config.with_server_hard_ttl(ttl);
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
