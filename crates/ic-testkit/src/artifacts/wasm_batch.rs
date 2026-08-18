use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::{Duration, Instant},
};

use super::wasm_cache::{
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceOutcome,
    SharedIncrementalTargetPrunePolicy, WasmBuildBatchInputMetrics, WasmBuildBatchInputResolver,
    WasmBuildCacheMode, WasmBuildError, WasmBuildOutcome, WasmBuildProgressConfig,
    WasmBuildProgressEvent, WasmBuildSpec, WasmBuildTimings, build_wasm_canisters_cached_in_batch,
    build_wasm_canisters_cached_in_batch_with_progress,
};

/// Orchestration shared by every entry in one independent Wasm build batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildBatchConfig {
    shared_incremental_maintenance: Option<SharedIncrementalTargetMaintenanceConfig>,
}

/// Caller-labeled specification for one exact Wasm batch entry.
///
/// The label is report and progress identity only; it does not alter the
/// underlying exact Wasm fingerprint or cache key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabeledWasmBuildSpec {
    label: String,
    spec: WasmBuildSpec,
}

/// Ordered outcomes and failures from a collect-all Wasm build batch.
#[derive(Debug)]
pub struct WasmBuildBatchReport {
    entries: Vec<WasmBuildBatchEntry>,
    input_resolution: WasmBuildBatchInputMetrics,
    total: Duration,
}

/// One ordered caller-labeled result from a Wasm build batch.
#[derive(Debug)]
pub struct WasmBuildBatchEntry {
    index: usize,
    label: String,
    result: Result<WasmBuildOutcome, WasmBuildError>,
    entry_elapsed: Duration,
}

/// One successful Wasm batch entry.
#[derive(Clone, Copy, Debug)]
pub struct WasmBuildBatchOutcomeEntry<'a> {
    index: usize,
    label: &'a str,
    outcome: &'a WasmBuildOutcome,
    entry_elapsed: Duration,
}

/// One failed Wasm batch entry with its retained wall-clock time.
#[derive(Clone, Copy, Debug)]
pub struct WasmBuildBatchFailure<'a> {
    index: usize,
    label: &'a str,
    error: &'a WasmBuildError,
    entry_elapsed: Duration,
}

/// One integrated shared-target maintenance outcome from a Wasm batch.
#[derive(Clone, Copy, Debug)]
pub struct WasmBuildBatchMaintenanceEntry<'a> {
    index: usize,
    label: &'a str,
    outcome: &'a SharedIncrementalTargetMaintenanceOutcome,
}

/// Structural error that prevents a labeled Wasm batch from starting.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildBatchContractError {
    /// An entry label was empty.
    EmptyLabel {
        /// Zero-based position of the invalid entry.
        index: usize,
    },
    /// Two entries used the same label.
    DuplicateLabel {
        /// Duplicated caller label.
        label: String,
        /// Position where the label first appeared.
        first_index: usize,
        /// Position where the label was repeated.
        duplicate_index: usize,
    },
}

impl LabeledWasmBuildSpec {
    /// Attach a caller-owned stable label to one Wasm build specification.
    #[must_use]
    pub fn new(label: impl Into<String>, spec: WasmBuildSpec) -> Self {
        Self {
            label: label.into(),
            spec,
        }
    }

    /// Caller-owned report and progress label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Underlying exact Wasm build specification.
    #[must_use]
    pub const fn spec(&self) -> &WasmBuildSpec {
        &self.spec
    }

    /// Consume the entry into its label and Wasm build specification.
    #[must_use]
    pub fn into_parts(self) -> (String, WasmBuildSpec) {
        (self.label, self.spec)
    }
}

impl WasmBuildBatchEntry {
    /// Zero-based position in the supplied labeled specification slice.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Caller-owned stable label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Structured success or failure for this entry.
    pub const fn result(&self) -> Result<&WasmBuildOutcome, &WasmBuildError> {
        self.result.as_ref()
    }

    /// Successful Wasm outcome, when this entry succeeded.
    #[must_use]
    pub fn outcome(&self) -> Option<&WasmBuildOutcome> {
        self.result.as_ref().ok()
    }

    /// Structured build failure, when this entry failed.
    #[must_use]
    pub fn error(&self) -> Option<&WasmBuildError> {
        self.result.as_ref().err()
    }

    /// Complete wall-clock time retained for this entry.
    #[must_use]
    pub const fn entry_elapsed(&self) -> Duration {
        self.entry_elapsed
    }

    /// Whether this entry completed successfully.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    /// Consume the entry into its ordered identity, result, and wall time.
    pub fn into_parts(
        self,
    ) -> (
        usize,
        String,
        Result<WasmBuildOutcome, WasmBuildError>,
        Duration,
    ) {
        (self.index, self.label, self.result, self.entry_elapsed)
    }
}

impl<'a> WasmBuildBatchOutcomeEntry<'a> {
    /// Zero-based position in the supplied labeled specification slice.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Caller-owned stable label.
    #[must_use]
    pub const fn label(self) -> &'a str {
        self.label
    }

    /// Successful Wasm build outcome.
    #[must_use]
    pub const fn outcome(self) -> &'a WasmBuildOutcome {
        self.outcome
    }

    /// Complete wall-clock time retained for this successful entry.
    #[must_use]
    pub const fn entry_elapsed(self) -> Duration {
        self.entry_elapsed
    }
}

impl<'a> WasmBuildBatchFailure<'a> {
    /// Zero-based position in the supplied specification slice.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Caller-owned stable label.
    #[must_use]
    pub const fn label(self) -> &'a str {
        self.label
    }

    /// Structured acquisition failure.
    #[must_use]
    pub const fn error(self) -> &'a WasmBuildError {
        self.error
    }

    /// Complete wall-clock time retained for this failed entry.
    #[must_use]
    pub const fn entry_elapsed(self) -> Duration {
        self.entry_elapsed
    }
}

impl<'a> WasmBuildBatchMaintenanceEntry<'a> {
    /// Zero-based position in the supplied labeled specification slice.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Caller-owned stable label.
    #[must_use]
    pub const fn label(self) -> &'a str {
        self.label
    }

    /// Structured shared-target maintenance outcome.
    #[must_use]
    pub const fn outcome(self) -> &'a SharedIncrementalTargetMaintenanceOutcome {
        self.outcome
    }
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
        /// Caller-owned stable label.
        label: String,
        /// Total number of supplied specifications.
        total: usize,
    },
    /// Progress forwarded from one independent build.
    BuildProgress {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
        /// Event emitted by that build.
        event: WasmBuildProgressEvent,
    },
    /// One independent build completed successfully.
    BuildFinished {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
    },
    /// One independent build failed.
    BuildFailed {
        /// Zero-based position in the supplied specification slice.
        index: usize,
        /// Caller-owned stable label.
        label: String,
    },
}

impl WasmBuildBatchReport {
    /// Ordered labeled entries.
    #[must_use]
    pub fn entries(&self) -> &[WasmBuildBatchEntry] {
        &self.entries
    }

    /// Consume the report into its ordered labeled entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<WasmBuildBatchEntry> {
        self.entries
    }

    /// Structured successful entries with labels and wall-clock times.
    pub fn outcomes(&self) -> impl Iterator<Item = WasmBuildBatchOutcomeEntry<'_>> {
        self.entries.iter().filter_map(|entry| {
            entry.outcome().map(|outcome| WasmBuildBatchOutcomeEntry {
                index: entry.index,
                label: &entry.label,
                outcome,
                entry_elapsed: entry.entry_elapsed,
            })
        })
    }

    /// Structured failed entries with labels and wall-clock times.
    pub fn failures(&self) -> impl Iterator<Item = WasmBuildBatchFailure<'_>> {
        self.entries.iter().filter_map(|entry| {
            entry.error().map(|error| WasmBuildBatchFailure {
                index: entry.index,
                label: &entry.label,
                error,
                entry_elapsed: entry.entry_elapsed,
            })
        })
    }

    /// Labeled integrated shared-target maintenance outcomes.
    ///
    /// Batch-owned maintenance contributes at most one outcome for each
    /// distinct configured shared-target path.
    pub fn shared_incremental_maintenance_outcomes(
        &self,
    ) -> impl Iterator<Item = WasmBuildBatchMaintenanceEntry<'_>> {
        self.outcomes().filter_map(|entry| {
            entry
                .outcome
                .record()
                .shared_incremental_maintenance()
                .map(|outcome| WasmBuildBatchMaintenanceEntry {
                    index: entry.index,
                    label: entry.label,
                    outcome,
                })
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
        self.entries.iter().all(WasmBuildBatchEntry::is_success)
    }

    /// Aggregate outcome, input-resolution reuse, and timing counters.
    #[must_use]
    pub fn metrics(&self) -> WasmBuildBatchMetrics {
        let mut metrics = WasmBuildBatchMetrics {
            specifications: self.entries.len(),
            input_resolution_runs: self.input_resolution.runs,
            input_resolution_reuses: self.input_resolution.reuses,
            total: self.total,
            ..WasmBuildBatchMetrics::default()
        };
        for entry in &self.entries {
            match &entry.result {
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
pub fn build_wasm_canisters_cached_batch(
    specs: &[LabeledWasmBuildSpec],
) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError> {
    build_wasm_canisters_cached_batch_with_config(specs, WasmBuildBatchConfig::new())
}

/// Build an independent Wasm batch with shared batch orchestration.
///
/// Batch-owned maintenance is attached only to the first specification for
/// each distinct configured shared-target path. Isolated specifications are
/// unaffected. An entry mixing batch-owned and per-spec integrated maintenance
/// reports an indexed error without preventing later entries from running.
pub fn build_wasm_canisters_cached_batch_with_config(
    specs: &[LabeledWasmBuildSpec],
    config: WasmBuildBatchConfig,
) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError> {
    validate_batch_labels(specs)?;
    let build_specs = specs
        .iter()
        .map(|labeled| labeled.spec.clone())
        .collect::<Vec<_>>();
    let mut resolver = WasmBuildBatchInputResolver::new(&build_specs);
    let mut report = build_wasm_batch(specs, config, |spec, index| {
        build_wasm_canisters_cached_in_batch(spec, index, &mut resolver)
    });
    report.input_resolution = resolver.metrics();
    Ok(report)
}

/// Build an independent Wasm batch while forwarding structured progress.
///
/// The same observation configuration is applied to every entry. Batch events
/// identify the originating specification without altering the standalone
/// build semantics.
pub fn build_wasm_canisters_cached_batch_with_progress<F>(
    specs: &[LabeledWasmBuildSpec],
    config: WasmBuildProgressConfig,
    observer: F,
) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError>
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
    specs: &[LabeledWasmBuildSpec],
    batch_config: WasmBuildBatchConfig,
    progress_config: WasmBuildProgressConfig,
    mut observer: F,
) -> Result<WasmBuildBatchReport, WasmBuildBatchContractError>
where
    F: FnMut(WasmBuildBatchProgressEvent),
{
    validate_batch_labels(specs)?;
    let count = specs.len();
    let build_specs = specs
        .iter()
        .map(|labeled| labeled.spec.clone())
        .collect::<Vec<_>>();
    let mut resolver = WasmBuildBatchInputResolver::new(&build_specs);
    let mut report = build_wasm_batch(specs, batch_config, |spec, index| {
        let label = specs[index].label.clone();
        observer(WasmBuildBatchProgressEvent::BuildStarted {
            index,
            label: label.clone(),
            total: count,
        });
        let result = build_wasm_canisters_cached_in_batch_with_progress(
            spec,
            index,
            &mut resolver,
            progress_config,
            |event| {
                observer(WasmBuildBatchProgressEvent::BuildProgress {
                    index,
                    label: label.clone(),
                    event,
                });
            },
        );
        observer(match result {
            Ok(_) => WasmBuildBatchProgressEvent::BuildFinished { index, label },
            Err(_) => WasmBuildBatchProgressEvent::BuildFailed { index, label },
        });
        result
    });
    report.input_resolution = resolver.metrics();
    Ok(report)
}

fn build_wasm_batch<F>(
    specs: &[LabeledWasmBuildSpec],
    config: WasmBuildBatchConfig,
    mut build: F,
) -> WasmBuildBatchReport
where
    F: FnMut(&WasmBuildSpec, usize) -> Result<WasmBuildOutcome, WasmBuildError>,
{
    let started = Instant::now();
    let mut entries = Vec::with_capacity(specs.len());
    let mut maintenance = BatchMaintenanceTracker::new(config.shared_incremental_maintenance);
    for (index, labeled) in specs.iter().enumerate() {
        let entry_started = Instant::now();
        let spec = &labeled.spec;
        if config.shared_incremental_maintenance.is_some()
            && spec.shared_incremental_target_maintenance().is_some()
        {
            entries.push(WasmBuildBatchEntry {
                index,
                label: labeled.label.clone(),
                result: Err(batch_maintenance_ownership_error()),
                entry_elapsed: entry_started.elapsed(),
            });
            continue;
        }
        let configured = maintenance.prepare_spec(spec);
        let result = build(configured.as_ref().unwrap_or(spec), index);
        entries.push(WasmBuildBatchEntry {
            index,
            label: labeled.label.clone(),
            result,
            entry_elapsed: entry_started.elapsed(),
        });
    }
    WasmBuildBatchReport {
        entries,
        input_resolution: WasmBuildBatchInputMetrics::default(),
        total: started.elapsed(),
    }
}

fn validate_batch_labels(
    specs: &[LabeledWasmBuildSpec],
) -> Result<(), WasmBuildBatchContractError> {
    let mut labels = HashMap::with_capacity(specs.len());
    for (index, labeled) in specs.iter().enumerate() {
        if labeled.label.is_empty() {
            return Err(WasmBuildBatchContractError::EmptyLabel { index });
        }
        if let Some(first_index) = labels.get(labeled.label.as_str()) {
            return Err(WasmBuildBatchContractError::DuplicateLabel {
                label: labeled.label.clone(),
                first_index: *first_index,
                duplicate_index: index,
            });
        }
        labels.insert(labeled.label.as_str(), index);
    }
    Ok(())
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

impl std::fmt::Display for WasmBuildBatchContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLabel { index } => {
                write!(formatter, "Wasm batch label at index {index} is empty")
            }
            Self::DuplicateLabel {
                label,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "Wasm batch label {label:?} at index {duplicate_index} duplicates index {first_index}",
            ),
        }
    }
}

impl std::error::Error for WasmBuildBatchContractError {}

#[cfg(test)]
mod tests;
