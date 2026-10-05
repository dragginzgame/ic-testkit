//! Validates the operator's workload selection before launching any tools.

use std::{ffi::OsString, num::NonZeroUsize, path::PathBuf};

use super::Error;

pub(super) const HELP: &str = "Fixture reuse benchmark (Linux/macOS)
Required: --server PATH (exact PocketIC 16 binary)
Options:
  --iterations N       Total tasks per mode/run (default: 100)
  --workers N          Same concurrency in all modes (default: 2)
  --repeats N          Rotating repeats (default: 3)
  --modes MODE...      fresh and/or pooled (default: fresh pooled)
  --capacities N...    Pool capacities (default: 1 2)
  --state-bytes N      Heap bytes per canister, fitting u32 (default: 1048576)
  --profile PROFILE   Workload host profile: release or dev (default: release)
  --output PATH        Save provenance, raw runs, and summaries as JSON
  --help               Show this help
Builds use the selected lockfile and prepared offline caches; no server download.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode {
    Fresh,
    Pooled,
}

impl Mode {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Pooled => "pooled",
        }
    }
}

pub(super) struct Options {
    pub(super) server: PathBuf,
    pub(super) iterations: NonZeroUsize,
    pub(super) workers: NonZeroUsize,
    pub(super) repeats: NonZeroUsize,
    pub(super) modes: Vec<Mode>,
    pub(super) capacities: Vec<NonZeroUsize>,
    pub(super) state_bytes: u32,
    pub(super) profile: String,
    pub(super) output: Option<PathBuf>,
}

pub(super) struct Case {
    pub(super) label: String,
    pub(super) mode: Mode,
    pub(super) capacity: NonZeroUsize,
}

impl Options {
    pub(super) fn benchmark_cases(&self) -> Vec<Case> {
        self.modes
            .iter()
            .flat_map(|mode| match mode {
                Mode::Fresh => vec![Case {
                    label: "fresh".into(),
                    mode: *mode,
                    capacity: self.workers,
                }],
                Mode::Pooled => self
                    .capacities
                    .iter()
                    .map(|capacity| Case {
                        label: format!("pooled-{capacity}"),
                        mode: *mode,
                        capacity: *capacity,
                    })
                    .collect(),
            })
            .collect()
    }
}

fn positive(value: &OsString) -> Result<NonZeroUsize, Error> {
    value
        .to_str()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| {
            Error::InvalidArgument(format!(
                "expected a positive integer, got {}",
                value.display()
            ))
        })
}

fn parse_modes(values: &[OsString]) -> Result<Vec<Mode>, Error> {
    let modes = values
        .iter()
        .map(|value| match value.to_str() {
            Some("fresh") => Ok(Mode::Fresh),
            Some("pooled") => Ok(Mode::Pooled),
            _ => Err(Error::InvalidArgument(format!(
                "invalid mode: {}",
                value.display()
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (index, value) in modes.iter().enumerate() {
        if modes[..index].contains(value) {
            return Err(Error::InvalidArgument(
                "each mode must be selected only once".into(),
            ));
        }
    }
    Ok(modes)
}

pub(super) fn parse_arguments(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Option<Options>, Error> {
    let mut options = Options {
        server: PathBuf::new(),
        iterations: NonZeroUsize::new(100).expect("positive default"),
        workers: NonZeroUsize::new(2).expect("positive default"),
        repeats: NonZeroUsize::new(3).expect("positive default"),
        modes: vec![Mode::Fresh, Mode::Pooled],
        capacities: vec![
            NonZeroUsize::new(1).expect("positive default"),
            NonZeroUsize::new(2).expect("positive default"),
        ],
        state_bytes: 1024 * 1024,
        profile: "release".into(),
        output: None,
    };
    let mut args = args.into_iter().peekable();
    while let Some(option) = args.next() {
        let name = option
            .to_str()
            .ok_or_else(|| Error::InvalidArgument("option is not UTF-8".into()))?;
        if matches!(name, "--help" | "-h") {
            return Ok(None);
        }
        if matches!(name, "--modes" | "--capacities") {
            let mut values = Vec::new();
            while args
                .peek()
                .is_some_and(|value| !value.to_string_lossy().starts_with("--"))
            {
                values.push(args.next().expect("peeked value"));
            }
            if values.is_empty() {
                return Err(Error::InvalidArgument(format!(
                    "{name} requires at least one value"
                )));
            }
            if name == "--capacities" {
                options.capacities = values.iter().map(positive).collect::<Result<_, _>>()?;
                for (index, value) in options.capacities.iter().enumerate() {
                    if options.capacities[..index].contains(value) {
                        return Err(Error::InvalidArgument(
                            "each capacity must be selected only once".into(),
                        ));
                    }
                }
            } else {
                options.modes = parse_modes(&values)?;
            }
            continue;
        }
        if !matches!(
            name,
            "--server"
                | "--iterations"
                | "--workers"
                | "--repeats"
                | "--state-bytes"
                | "--profile"
                | "--output"
        ) {
            return Err(Error::InvalidArgument(format!("unknown option: {name}")));
        }
        let value = args
            .next()
            .filter(|value| !value.to_string_lossy().starts_with("--"))
            .ok_or_else(|| Error::InvalidArgument(format!("{name} requires a value")))?;
        match name {
            "--server" => options.server = value.into(),
            "--iterations" => options.iterations = positive(&value)?,
            "--workers" => options.workers = positive(&value)?,
            "--repeats" => options.repeats = positive(&value)?,
            "--output" => options.output = Some(value.into()),
            "--state-bytes" => {
                options.state_bytes = value
                    .to_str()
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| Error::InvalidArgument("state-bytes must fit u32".into()))?;
            }
            "--profile" => {
                if !matches!(value.to_str(), Some("release" | "dev")) {
                    return Err(Error::InvalidArgument(
                        "profile must be release or dev".into(),
                    ));
                }
                options.profile = value.to_string_lossy().into_owned();
            }
            _ => unreachable!("validated option"),
        }
    }
    if options.server.as_os_str().is_empty() || options.workers > options.iterations {
        return Err(Error::InvalidArgument(
            "server is required; workers must not exceed iterations".into(),
        ));
    }
    Ok(Some(options))
}

#[cfg(test)]
mod tests {
    use super::parse_arguments;
    use crate::fixture_reuse_driver::Error;

    #[test]
    fn capacity_sweep_preserves_order_and_one_fresh_control() {
        let options = parse_arguments(
            [
                "--server",
                "/unused/pocket-ic",
                "--iterations",
                "8",
                "--workers",
                "8",
                "--modes",
                "pooled",
                "fresh",
                "--capacities",
                "1",
                "2",
                "4",
                "8",
            ]
            .map(Into::into),
        )
        .unwrap()
        .unwrap();
        let cases = options
            .benchmark_cases()
            .into_iter()
            .map(|case| (case.label, case.mode.as_str(), case.capacity.get()))
            .collect::<Vec<_>>();
        assert_eq!(
            cases,
            [
                ("pooled-1".into(), "pooled", 1),
                ("pooled-2".into(), "pooled", 2),
                ("pooled-4".into(), "pooled", 4),
                ("pooled-8".into(), "pooled", 8),
                ("fresh".into(), "fresh", 8)
            ]
        );
    }

    #[test]
    fn default_and_fresh_only_selection() {
        let options = parse_arguments(["--server", "/unused/pocket-ic"].map(Into::into))
            .unwrap()
            .unwrap();
        assert_eq!(
            options
                .benchmark_cases()
                .iter()
                .map(|case| case.label.as_str())
                .collect::<Vec<_>>(),
            ["fresh", "pooled-1", "pooled-2"]
        );
        let options = parse_arguments(
            [
                "--server",
                "/unused/pocket-ic",
                "--modes",
                "fresh",
                "--capacities",
                "8",
            ]
            .map(Into::into),
        )
        .unwrap()
        .unwrap();
        assert_eq!(options.benchmark_cases().len(), 1);
        assert_eq!(options.benchmark_cases()[0].capacity.get(), 2);
    }

    #[test]
    fn invalid_selections_fail_before_launching_tools() {
        for selection in [
            vec!["--capacities", "0"],
            vec!["--capacities", "-1"],
            vec!["--capacities", "2", "2"],
            vec!["--modes", "pooled", "pooled"],
            vec!["--workers", "101"],
            vec!["--state-bytes", "4294967296"],
            vec!["--profile", "custom"],
            vec!["--modes"],
            vec!["--unknown"],
        ] {
            let args = ["--server", "/unused/pocket-ic"]
                .into_iter()
                .chain(selection)
                .map(Into::into);
            assert!(matches!(
                parse_arguments(args),
                Err(Error::InvalidArgument(_))
            ));
        }
        assert!(
            parse_arguments(["--server", "/unused", "--state-bytes", "0"].map(Into::into)).is_ok()
        );
    }
}
