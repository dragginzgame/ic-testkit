//! Repository-local worker invocation; callers retain workload and report admission.

use std::{
    ffi::{OsStr, OsString},
    io,
    num::NonZeroUsize,
    path::PathBuf,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Fresh,
    Pooled,
}

impl Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Pooled => "pooled",
        }
    }

    pub fn parse(value: &OsStr) -> Option<Self> {
        match value.to_str()? {
            "fresh" => Some(Self::Fresh),
            "pooled" => Some(Self::Pooled),
            _ => None,
        }
    }
}

pub struct WorkerArguments {
    pub mode: Mode,
    pub capacity: NonZeroUsize,
    pub iterations: NonZeroUsize,
    pub workers: NonZeroUsize,
    pub state_bytes: u32,
    pub wasm: PathBuf,
    pub server: PathBuf,
}

fn invalid_arguments() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "expected MODE CAPACITY ITERATIONS WORKERS STATE_BYTES WASM SERVER; positive counts, fresh/pooled mode, and workers <= iterations",
    )
}

fn number<T: FromStr>(value: &OsStr) -> io::Result<T> {
    value
        .to_str()
        .and_then(|value| value.parse().ok())
        .ok_or_else(invalid_arguments)
}

impl TryFrom<Vec<OsString>> for WorkerArguments {
    type Error = io::Error;

    fn try_from(args: Vec<OsString>) -> io::Result<Self> {
        let [
            mode,
            capacity,
            iterations,
            workers,
            state_bytes,
            wasm,
            server,
        ]: [OsString; 7] = args.try_into().map_err(|_| invalid_arguments())?;
        let result = Self {
            mode: Mode::parse(&mode).ok_or_else(invalid_arguments)?,
            capacity: number(&capacity)?,
            iterations: number(&iterations)?,
            workers: number(&workers)?,
            state_bytes: number(&state_bytes)?,
            wasm: wasm.into(),
            server: server.into(),
        };
        if result.workers > result.iterations {
            return Err(invalid_arguments());
        }
        Ok(result)
    }
}

impl From<WorkerArguments> for [OsString; 7] {
    fn from(args: WorkerArguments) -> Self {
        [
            args.mode.as_str().into(),
            args.capacity.to_string().into(),
            args.iterations.to_string().into(),
            args.workers.to_string().into(),
            args.state_bytes.to_string().into(),
            args.wasm.into_os_string(),
            args.server.into_os_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, WorkerArguments};
    use std::{ffi::OsString, io};

    #[test]
    fn worker_invocation_admits_both_modes_and_zero_state() {
        for mode in [Mode::Fresh, Mode::Pooled] {
            let args = [
                mode.as_str(),
                "2",
                "4",
                "2",
                "0",
                "probe wasm",
                "server path",
            ]
            .map(OsString::from);
            let parsed = WorkerArguments::try_from(args.to_vec()).unwrap();
            assert_eq!(parsed.mode, mode);
            assert_eq!(parsed.capacity.get(), 2);
            assert_eq!(parsed.iterations.get(), 4);
            assert_eq!(parsed.workers.get(), 2);
            assert_eq!(parsed.state_bytes, 0);
            assert_eq!(<[OsString; 7]>::from(parsed), args);
        }
    }

    #[test]
    fn invalid_worker_invocations_refuse_before_work_starts() {
        for args in [
            vec!["fresh", "2", "4", "2", "0", "wasm"],
            vec!["fresh", "2", "4", "2", "0", "wasm", "server", "extra"],
            vec!["unknown", "2", "4", "2", "0", "wasm", "server"],
            vec!["fresh", "0", "4", "2", "0", "wasm", "server"],
            vec!["fresh", "2", "0", "2", "0", "wasm", "server"],
            vec!["fresh", "2", "4", "0", "0", "wasm", "server"],
            vec!["fresh", "2", "4", "5", "0", "wasm", "server"],
            vec!["fresh", "2", "4", "2", "4294967296", "wasm", "server"],
        ] {
            let error =
                WorkerArguments::try_from(args.into_iter().map(OsString::from).collect::<Vec<_>>())
                    .err()
                    .unwrap();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        }
    }

    #[cfg(unix)]
    #[test]
    fn worker_paths_preserve_native_non_utf8_bytes() {
        use std::os::unix::ffi::OsStringExt as _;
        let mut args = ["fresh", "1", "1", "1", "0", "wasm", "server"].map(OsString::from);
        args[5] = OsString::from_vec(vec![b'w', 255]);
        args[6] = OsString::from_vec(vec![b's', 254]);
        assert_eq!(
            <[OsString; 7]>::from(WorkerArguments::try_from(args.to_vec()).unwrap()),
            args
        );
    }
}
