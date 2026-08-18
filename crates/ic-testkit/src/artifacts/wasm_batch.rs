use std::{
    collections::HashSet,
    path::PathBuf,
    time::{Duration, Instant},
};

use super::{
    batch::{indexed_failures, indexed_outcomes},
    wasm_cache::{
        SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceOutcome,
        SharedIncrementalTargetPrunePolicy, WasmBuildBatchInputMetrics,
        WasmBuildBatchInputResolver, WasmBuildCacheMode, WasmBuildError, WasmBuildOutcome,
        WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildSpec, WasmBuildTimings,
        build_wasm_canisters_cached_in_batch, build_wasm_canisters_cached_in_batch_with_progress,
    },
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
    entry_elapsed: Vec<Duration>,
    input_resolution: WasmBuildBatchInputMetrics,
    total: Duration,
}

/// Aggregate counters and successful-acquisition timings for a Wasm build batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildBatchMetrics {
    specifications: usize,
    succeeded: usize,
    failed: usize,
    built: usize,
    reused: usize,
    input_resolution_runs: usize,
    input_resolution_reuses: usize,
    successful_timings: WasmBuildTimings,
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

    /// Wall-clock time retained for every entry, including failed entries.
    ///
    /// Values use the supplied specification order and include batch-owned
    /// validation plus the complete acquisition attempt for that entry.
    #[must_use]
    pub fn entry_elapsed(&self) -> &[Duration] {
        &self.entry_elapsed
    }

    /// Consume the report and return its ordered per-specification results.
    #[must_use]
    pub fn into_results(self) -> Vec<Result<WasmBuildOutcome, WasmBuildError>> {
        self.results
    }

    /// Successful outcomes with their specification indexes.
    pub fn outcomes(&self) -> impl Iterator<Item = (usize, &WasmBuildOutcome)> {
        indexed_outcomes(&self.results)
    }

    /// Failures with their specification indexes.
    pub fn failures(&self) -> impl Iterator<Item = (usize, &WasmBuildError)> {
        indexed_failures(&self.results)
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

    /// Aggregate outcome, input-resolution reuse, and timing counters.
    #[must_use]
    pub fn metrics(&self) -> WasmBuildBatchMetrics {
        let mut metrics = WasmBuildBatchMetrics {
            specifications: self.results.len(),
            input_resolution_runs: self.input_resolution.runs,
            input_resolution_reuses: self.input_resolution.reuses,
            total: self.total,
            ..WasmBuildBatchMetrics::default()
        };
        for result in &self.results {
            match result {
                Ok(outcome) => {
                    metrics.succeeded += 1;
                    if outcome.is_reused() {
                        metrics.reused += 1;
                    } else {
                        metrics.built += 1;
                    }
                    metrics.successful_timings = metrics
                        .successful_timings
                        .saturating_add(outcome.record().timings());
                }
                Err(_) => metrics.failed += 1,
            }
        }
        metrics
    }
}

impl WasmBuildBatchMetrics {
    /// Number of supplied specifications.
    #[must_use]
    pub const fn specifications(self) -> usize {
        self.specifications
    }

    /// Number of successful specifications.
    #[must_use]
    pub const fn succeeded(self) -> usize {
        self.succeeded
    }

    /// Number of failed specifications.
    #[must_use]
    pub const fn failed(self) -> usize {
        self.failed
    }

    /// Number of newly built Wasm artifact sets.
    #[must_use]
    pub const fn built(self) -> usize {
        self.built
    }

    /// Number of Wasm artifact sets reused from the exact cache.
    #[must_use]
    pub const fn reused(self) -> usize {
        self.reused
    }

    /// Number of workspace/toolchain input-resolution snapshots performed.
    #[must_use]
    pub const fn input_resolution_runs(self) -> usize {
        self.input_resolution_runs
    }

    /// Number of specifications resolved by reusing another batch snapshot.
    #[must_use]
    pub const fn input_resolution_reuses(self) -> usize {
        self.input_resolution_reuses
    }

    /// Sum of timings from successful acquisitions.
    #[must_use]
    pub const fn successful_timings(self) -> WasmBuildTimings {
        self.successful_timings
    }

    /// Complete wall-clock time for the sequential batch.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
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
        let metrics = self.metrics();
        write!(
            formatter,
            "builds={} succeeded={} failed={} built={} reused={} input_resolution_runs={} input_resolution_reuses={} successful_timings=({}) total={:?}",
            metrics.specifications(),
            metrics.succeeded(),
            metrics.failed(),
            metrics.built(),
            metrics.reused(),
            metrics.input_resolution_runs(),
            metrics.input_resolution_reuses(),
            metrics.successful_timings(),
            metrics.total(),
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
    let mut report = build_wasm_batch(specs, config, |spec, index| {
        build_wasm_canisters_cached_in_batch(spec, index, &mut resolver)
    });
    report.input_resolution = resolver.metrics();
    report
}

/// Build an independent Wasm batch while forwarding structured progress.
///
/// The same observation configuration is applied to every entry. Batch events
/// identify the originating specification without altering the standalone
/// build semantics.
#[must_use]
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
#[must_use]
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
    let mut report = build_wasm_batch(specs, batch_config, |spec, index| {
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
    });
    report.input_resolution = resolver.metrics();
    report
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
    let mut entry_elapsed = Vec::with_capacity(specs.len());
    let mut maintenance = BatchMaintenanceTracker::new(config.shared_incremental_maintenance);
    for (index, spec) in specs.iter().enumerate() {
        let entry_started = Instant::now();
        if config.shared_incremental_maintenance.is_some()
            && spec.shared_incremental_target_maintenance().is_some()
        {
            results.push(Err(batch_maintenance_ownership_error()));
            entry_elapsed.push(entry_started.elapsed());
            continue;
        }
        let configured = maintenance.prepare_spec(spec);
        results.push(build(configured.as_ref().unwrap_or(spec), index));
        entry_elapsed.push(entry_started.elapsed());
    }
    WasmBuildBatchReport {
        results,
        entry_elapsed,
        input_resolution: WasmBuildBatchInputMetrics::default(),
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
