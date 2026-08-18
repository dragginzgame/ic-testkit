use std::{
    collections::HashSet,
    path::PathBuf,
    time::{Duration, Instant},
};

use super::wasm_cache::{
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceOutcome,
    SharedIncrementalTargetPrunePolicy, WasmBuildBatchInputResolver, WasmBuildCacheMode,
    WasmBuildError, WasmBuildOutcome, WasmBuildProgressConfig, WasmBuildProgressEvent,
    WasmBuildSpec, build_wasm_canisters_cached_in_batch,
    build_wasm_canisters_cached_in_batch_with_progress,
};

/// Orchestration shared by every entry in one independent Wasm build batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildBatchConfig {
    shared_incremental_maintenance: Option<SharedIncrementalTargetMaintenanceConfig>,
}

/// Ordered outcomes and failures from a collect-all Wasm build batch.
#[derive(Debug)]
pub struct WasmBuildBatchReport {
    results: Vec<Result<WasmBuildOutcome, WasmBuildError>>,
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
    /// One independent build failed.
    BuildFailed {
        /// Zero-based position in the supplied specification slice.
        index: usize,
    },
}

impl WasmBuildBatchReport {
    /// Per-specification results in the supplied order.
    pub fn results(&self) -> &[Result<WasmBuildOutcome, WasmBuildError>] {
        &self.results
    }

    /// Consume the report and return its ordered per-specification results.
    #[must_use]
    pub fn into_results(self) -> Vec<Result<WasmBuildOutcome, WasmBuildError>> {
        self.results
    }

    /// Successful outcomes with their specification indexes.
    pub fn outcomes(&self) -> impl Iterator<Item = (usize, &WasmBuildOutcome)> {
        self.results
            .iter()
            .enumerate()
            .filter_map(|(index, result)| result.as_ref().ok().map(|outcome| (index, outcome)))
    }

    /// Failures with their specification indexes.
    pub fn failures(&self) -> impl Iterator<Item = (usize, &WasmBuildError)> {
        self.results
            .iter()
            .enumerate()
            .filter_map(|(index, result)| result.as_ref().err().map(|error| (index, error)))
    }

    /// Integrated shared-target maintenance outcomes with their build indexes.
    ///
    /// Batch-owned maintenance contributes at most one outcome for each
    /// distinct configured shared-target path.
    pub fn shared_incremental_maintenance_outcomes(
        &self,
    ) -> impl Iterator<Item = (usize, &SharedIncrementalTargetMaintenanceOutcome)> {
        self.outcomes().filter_map(|(index, outcome)| {
            outcome
                .record()
                .shared_incremental_maintenance()
                .map(|maintenance| (index, maintenance))
        })
    }

    /// Complete wall-clock time for the sequential collect-all batch.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }

    /// Whether every specification completed successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.results.iter().all(Result::is_ok)
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

impl std::fmt::Display for WasmBuildBatchReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let succeeded = self.results.iter().filter(|result| result.is_ok()).count();
        let reused = self
            .results
            .iter()
            .filter_map(|result| result.as_ref().ok())
            .filter(|outcome| outcome.is_reused())
            .count();
        write!(
            formatter,
            "builds={} succeeded={} failed={} built={} reused={} total={:?}",
            self.results.len(),
            succeeded,
            self.results.len().saturating_sub(succeeded),
            succeeded.saturating_sub(reused),
            reused,
            self.total,
        )
    }
}

/// Build every Wasm specification as an independent Cargo invocation.
///
/// Specifications run sequentially and every result is retained. Each entry
/// keeps its own package set, profile arguments, feature resolution,
/// fingerprint, locks, and cache policy. Packages are never combined into one
/// Cargo command because doing so can unify shared dependency features.
#[must_use]
pub fn build_wasm_canisters_cached_batch(specs: &[WasmBuildSpec]) -> WasmBuildBatchReport {
    build_wasm_canisters_cached_batch_with_config(specs, WasmBuildBatchConfig::new())
}

/// Build an independent Wasm batch with shared batch orchestration.
///
/// Batch-owned maintenance is attached only to the first specification for
/// each distinct configured shared-target path. Isolated specifications are
/// unaffected. An entry mixing batch-owned and per-spec integrated maintenance
/// reports an indexed error without preventing later entries from running.
#[must_use]
pub fn build_wasm_canisters_cached_batch_with_config(
    specs: &[WasmBuildSpec],
    config: WasmBuildBatchConfig,
) -> WasmBuildBatchReport {
    let mut resolver = WasmBuildBatchInputResolver::new(specs);
    build_wasm_batch(specs, config, |spec, index| {
        build_wasm_canisters_cached_in_batch(spec, index, &mut resolver)
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
) -> WasmBuildBatchReport
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
) -> WasmBuildBatchReport
where
    F: FnMut(WasmBuildBatchProgressEvent),
{
    let count = specs.len();
    let mut resolver = WasmBuildBatchInputResolver::new(specs);
    build_wasm_batch(specs, batch_config, |spec, index| {
        observer(WasmBuildBatchProgressEvent::BuildStarted {
            index,
            total: count,
        });
        let result = build_wasm_canisters_cached_in_batch_with_progress(
            spec,
            index,
            &mut resolver,
            progress_config,
            |event| observer(WasmBuildBatchProgressEvent::BuildProgress { index, event }),
        );
        observer(match result {
            Ok(_) => WasmBuildBatchProgressEvent::BuildFinished { index },
            Err(_) => WasmBuildBatchProgressEvent::BuildFailed { index },
        });
        result
    })
}

fn build_wasm_batch<F>(
    specs: &[WasmBuildSpec],
    config: WasmBuildBatchConfig,
    mut build: F,
) -> WasmBuildBatchReport
where
    F: FnMut(&WasmBuildSpec, usize) -> Result<WasmBuildOutcome, WasmBuildError>,
{
    let started = Instant::now();
    let mut results = Vec::with_capacity(specs.len());
    let mut maintenance = BatchMaintenanceTracker::new(config.shared_incremental_maintenance);
    for (index, spec) in specs.iter().enumerate() {
        if config.shared_incremental_maintenance.is_some()
            && spec.shared_incremental_target_maintenance().is_some()
        {
            results.push(Err(batch_maintenance_ownership_error()));
            continue;
        }
        let configured = maintenance.prepare_spec(spec);
        results.push(build(configured.as_ref().unwrap_or(spec), index));
    }
    WasmBuildBatchReport {
        results,
        total: started.elapsed(),
    }
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

#[cfg(test)]
mod tests;
