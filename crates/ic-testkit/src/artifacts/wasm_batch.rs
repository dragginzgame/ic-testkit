use std::{
    collections::HashSet,
    path::PathBuf,
    time::{Duration, Instant},
};

use super::wasm_cache::{
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceOutcome,
    SharedIncrementalTargetPrunePolicy, WasmBuildCacheMode, WasmBuildError, WasmBuildOutcome,
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildSpec, build_wasm_canisters_cached,
    build_wasm_canisters_cached_with_progress,
};

/// Orchestration shared by every entry in one independent Wasm build batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildBatchConfig {
    shared_incremental_maintenance: Option<SharedIncrementalTargetMaintenanceConfig>,
}

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

    /// Integrated shared-target maintenance outcomes with their build indexes.
    ///
    /// Batch-owned maintenance contributes at most one outcome for each
    /// distinct configured shared-target path.
    pub fn shared_incremental_maintenance_outcomes(
        &self,
    ) -> impl Iterator<Item = (usize, &SharedIncrementalTargetMaintenanceOutcome)> {
        self.outcomes
            .iter()
            .enumerate()
            .filter_map(|(index, outcome)| {
                outcome
                    .record()
                    .shared_incremental_maintenance()
                    .map(|maintenance| (index, maintenance))
            })
    }
}

impl WasmBuildBatchConfig {
    /// Create batch orchestration without batch-owned target maintenance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            shared_incremental_maintenance: None,
        }
    }

    /// Maintain each distinct shared target once through its first batch entry.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance(
        mut self,
        config: SharedIncrementalTargetMaintenanceConfig,
    ) -> Self {
        self.shared_incremental_maintenance = Some(config);
        self
    }

    /// Strictly maintain each distinct shared target at most once per interval.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance_at_most_every(
        self,
        policy: SharedIncrementalTargetPrunePolicy,
        minimum_interval: Duration,
    ) -> Self {
        self.with_shared_incremental_target_maintenance(
            SharedIncrementalTargetMaintenanceConfig::new(policy, minimum_interval),
        )
    }

    /// Batch-owned shared-target maintenance, when configured.
    #[must_use]
    pub const fn shared_incremental_target_maintenance(
        self,
    ) -> Option<SharedIncrementalTargetMaintenanceConfig> {
        self.shared_incremental_maintenance
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
    build_wasm_canisters_cached_batch_with_config(specs, WasmBuildBatchConfig::new())
}

/// Build an independent Wasm batch with shared batch orchestration.
///
/// Batch-owned maintenance is attached only to the first specification for
/// each distinct configured shared-target path. Isolated specifications are
/// unaffected. Mixing batch-owned and per-spec integrated maintenance is
/// rejected before any build starts because policy ownership would otherwise
/// be ambiguous.
pub fn build_wasm_canisters_cached_batch_with_config(
    specs: &[WasmBuildSpec],
    config: WasmBuildBatchConfig,
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError> {
    build_wasm_batch(specs, config, |spec, _index| {
        build_wasm_canisters_cached(spec)
    })
}

/// Build an independent Wasm batch while forwarding structured progress.
///
/// The same observation configuration is applied to every entry. Batch events
/// identify the originating specification without altering the standalone
/// build semantics.
pub fn build_wasm_canisters_cached_batch_with_progress<F>(
    specs: &[WasmBuildSpec],
    config: WasmBuildProgressConfig,
    observer: F,
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError>
where
    F: FnMut(WasmBuildBatchProgressEvent),
{
    build_wasm_canisters_cached_batch_with_config_and_progress(
        specs,
        WasmBuildBatchConfig::new(),
        config,
        observer,
    )
}

/// Build a configured independent Wasm batch while forwarding structured progress.
pub fn build_wasm_canisters_cached_batch_with_config_and_progress<F>(
    specs: &[WasmBuildSpec],
    batch_config: WasmBuildBatchConfig,
    progress_config: WasmBuildProgressConfig,
    mut observer: F,
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError>
where
    F: FnMut(WasmBuildBatchProgressEvent),
{
    let count = specs.len();
    build_wasm_batch(specs, batch_config, |spec, index| {
        observer(WasmBuildBatchProgressEvent::BuildStarted {
            index,
            total: count,
        });
        let outcome = build_wasm_canisters_cached_with_progress(spec, progress_config, |event| {
            observer(WasmBuildBatchProgressEvent::BuildProgress { index, event });
        })?;
        observer(WasmBuildBatchProgressEvent::BuildFinished { index });
        Ok(outcome)
    })
}

fn build_wasm_batch<F>(
    specs: &[WasmBuildSpec],
    config: WasmBuildBatchConfig,
    mut build: F,
) -> Result<WasmBuildBatchOutcome, WasmBuildBatchError>
where
    F: FnMut(&WasmBuildSpec, usize) -> Result<WasmBuildOutcome, WasmBuildError>,
{
    let started = Instant::now();
    if let Some(failed_index) = config.shared_incremental_maintenance.and_then(|_| {
        specs
            .iter()
            .position(|spec| spec.shared_incremental_target_maintenance().is_some())
    }) {
        return Err(WasmBuildBatchError {
            failed_index,
            completed: Vec::new(),
            total: started.elapsed(),
            source: batch_maintenance_ownership_error(),
        });
    }
    let mut outcomes = Vec::with_capacity(specs.len());
    let mut maintenance = BatchMaintenanceTracker::new(config.shared_incremental_maintenance);
    for (index, spec) in specs.iter().enumerate() {
        let configured = maintenance.prepare_spec(spec);
        match build(configured.as_ref().unwrap_or(spec), index) {
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

struct BatchMaintenanceTracker {
    config: Option<SharedIncrementalTargetMaintenanceConfig>,
    configured_targets: HashSet<PathBuf>,
}

impl BatchMaintenanceTracker {
    fn new(config: Option<SharedIncrementalTargetMaintenanceConfig>) -> Self {
        Self {
            config,
            configured_targets: HashSet::new(),
        }
    }

    fn prepare_spec(&mut self, spec: &WasmBuildSpec) -> Option<WasmBuildSpec> {
        let config = self.config?;
        debug_assert!(spec.shared_incremental_target_maintenance().is_none());
        let WasmBuildCacheMode::SharedIncremental { target_dir } = spec.cache_mode() else {
            return None;
        };
        if !self.configured_targets.insert(target_dir.clone()) {
            return None;
        }
        Some(
            spec.clone()
                .with_shared_incremental_target_maintenance(config),
        )
    }
}

fn batch_maintenance_ownership_error() -> WasmBuildError {
    WasmBuildError::InvalidSpec {
        message:
            "batch-owned shared-target maintenance cannot be combined with per-spec maintenance"
                .to_owned(),
    }
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
mod tests;
