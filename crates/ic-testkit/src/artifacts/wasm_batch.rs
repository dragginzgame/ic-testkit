use std::time::{Duration, Instant};

use super::wasm_cache::{
    WasmBuildError, WasmBuildOutcome, WasmBuildProgressConfig, WasmBuildProgressEvent,
    WasmBuildSpec, build_wasm_canisters_cached, build_wasm_canisters_cached_with_progress,
};

/// Successful outcomes from an independent sequence of exact Wasm builds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBuildBatchOutcome {
    outcomes: Vec<WasmBuildOutcome>,
    total: Duration,
}

/// Structured progress for an independent sequence of exact Wasm builds.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildBatchProgressEvent {
    /// One independently resolved build specification is about to start.
    BuildStarted {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Total number of supplied specifications.
        total: usize,
    },
    /// Progress forwarded from one independent build.
    BuildProgress {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Event emitted by that build.
        event: WasmBuildProgressEvent,
    },
    /// One independent build completed successfully.
    BuildFinished {
        /// Zero-based position in the supplied specification slice.
        index: usize,
    },
}

/// Failure from an independent Wasm build batch.
#[derive(Debug)]
pub struct WasmBuildBatchError {
    failed_index: usize,
    completed: Vec<WasmBuildOutcome>,
    total: Duration,
    source: WasmBuildError,
}

impl WasmBuildBatchOutcome {
    /// Successful outcomes in specification order.
    #[must_use]
    pub fn outcomes(&self) -> &[WasmBuildOutcome] {
        &self.outcomes
    }

    /// Consume the report and return its ordered outcomes.
    #[must_use]
    pub fn into_outcomes(self) -> Vec<WasmBuildOutcome> {
        self.outcomes
    }

    /// Complete wall-clock time for the sequential batch.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }
}

impl WasmBuildBatchError {
    /// Zero-based index of the failed independent specification.
    #[must_use]
    pub const fn failed_index(&self) -> usize {
        self.failed_index
    }

    /// Successful outcomes completed before the failure.
    #[must_use]
    pub fn completed(&self) -> &[WasmBuildOutcome] {
        &self.completed
    }

    /// Consume the failure and return the successful prefix and root cause.
    #[must_use]
    pub fn into_parts(self) -> (Vec<WasmBuildOutcome>, WasmBuildError) {
        (self.completed, self.source)
    }

    /// Wall-clock time through the failure.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }
}

impl std::fmt::Display for WasmBuildBatchOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reused = self
            .outcomes
            .iter()
            .filter(|outcome| outcome.is_reused())
            .count();
        write!(
            formatter,
            "builds={} built={} reused={} total={:?}",
            self.outcomes.len(),
            self.outcomes.len().saturating_sub(reused),
            reused,
            self.total,
        )
    }
}

/// Build or reuse multiple Wasm specifications as independent Cargo invocations.
///
/// Specifications run sequentially and fail fast. Each entry retains its own
/// package set, profile arguments, feature resolution, fingerprint, locks, and
/// cache policy. The implementation deliberately never combines packages into
/// one Cargo command, because doing so can unify shared dependency features.
/// Completed outcomes remain valid when a later entry fails.
pub fn build_wasm_canisters_cached_batch(
    specs: &[WasmBuildSpec],
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError> {
    build_wasm_batch(specs, |spec, _index| build_wasm_canisters_cached(spec))
}

/// Build an independent Wasm batch while forwarding structured progress.
///
/// The same observation configuration is applied to every entry. Batch events
/// identify the originating specification without altering the standalone
/// build semantics.
pub fn build_wasm_canisters_cached_batch_with_progress<F>(
    specs: &[WasmBuildSpec],
    config: WasmBuildProgressConfig,
    mut observer: F,
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError>
where
    F: FnMut(WasmBuildBatchProgressEvent),
{
    let count = specs.len();
    build_wasm_batch(specs, |spec, index| {
        observer(WasmBuildBatchProgressEvent::BuildStarted {
            index,
            total: count,
        });
        let outcome = build_wasm_canisters_cached_with_progress(spec, config, |event| {
            observer(WasmBuildBatchProgressEvent::BuildProgress { index, event });
        })?;
        observer(WasmBuildBatchProgressEvent::BuildFinished { index });
        Ok(outcome)
    })
}

fn build_wasm_batch<F>(
    specs: &[WasmBuildSpec],
    mut build: F,
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError>
where
    F: FnMut(&WasmBuildSpec, usize) -> Result<WasmBuildOutcome, WasmBuildError>,
{
    let started = Instant::now();
    let mut outcomes = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        match build(spec, index) {
            Ok(outcome) => outcomes.push(outcome),
            Err(source) => {
                return Err(WasmBuildBatchError {
                    failed_index: index,
                    completed: outcomes,
                    total: started.elapsed(),
                    source,
                });
            }
        }
    }
    Ok(WasmBuildBatchOutcome {
        outcomes,
        total: started.elapsed(),
    })
}

impl std::fmt::Display for WasmBuildBatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "independent Wasm build {} failed after {} successful build(s): {}",
            self.failed_index,
            self.completed.len(),
            self.source,
        )
    }
}

impl std::error::Error for WasmBuildBatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::build_wasm_canisters_cached_batch;
    use crate::artifacts::WasmBuildSpec;
    use std::path::Path;

    #[test]
    fn empty_independent_batch_succeeds_without_work() {
        let outcome = build_wasm_canisters_cached_batch(&[]).expect("empty batch");
        assert!(outcome.outcomes().is_empty());
    }

    #[test]
    fn invalid_independent_spec_reports_its_batch_index() {
        let specs = [WasmBuildSpec::new(
            Path::new("."),
            Path::new("target"),
            &[],
            "debug",
        )];
        let error = build_wasm_canisters_cached_batch(&specs).expect_err("invalid batch entry");
        assert_eq!(error.failed_index(), 0);
        assert!(error.completed().is_empty());
    }
}
