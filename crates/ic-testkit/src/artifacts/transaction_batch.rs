use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use super::transaction::{
    ArtifactBuildTransaction, ArtifactCacheError, ArtifactCacheOutcome, ArtifactCachePreparation,
    ArtifactCacheSpec, ArtifactCacheTimings, prepare_artifact_cache,
};

/// Caller-labeled specification for one generic artifact batch entry.
///
/// The label is report and callback identity, not artifact-cache identity. The
/// caller must keep it stable anywhere reports are composed across stages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabeledArtifactCacheSpec {
    label: String,
    spec: ArtifactCacheSpec,
}

/// Ordered labeled entries from a collect-all artifact transaction batch.
#[derive(Debug)]
pub struct ArtifactCacheBatchReport<E> {
    entries: Vec<ArtifactCacheBatchEntry<E>>,
    total: Duration,
}

/// One ordered labeled result from a generic artifact batch.
#[derive(Debug)]
pub struct ArtifactCacheBatchEntry<E> {
    index: usize,
    label: String,
    result: Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>,
    entry_elapsed: Duration,
}

/// One successful generic artifact batch entry.
#[derive(Clone, Copy, Debug)]
pub struct ArtifactCacheBatchOutcomeEntry<'a> {
    index: usize,
    label: &'a str,
    outcome: &'a ArtifactCacheOutcome,
    entry_elapsed: Duration,
}

/// One failed generic artifact batch entry.
#[derive(Debug)]
pub struct ArtifactCacheBatchFailedEntry<'a, E> {
    index: usize,
    label: &'a str,
    failure: &'a ArtifactCacheBatchFailure<E>,
    entry_elapsed: Duration,
}

/// Structural error that prevents a labeled artifact batch from starting.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactCacheBatchContractError {
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

/// Primary phase in which a generic artifact batch entry failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactCacheBatchFailurePhase {
    /// Cache preparation failed before the population callback ran.
    Preparation,
    /// The caller's population callback failed.
    Callback,
    /// Transaction commit failed after successful population.
    Commit,
}

/// Partial phase timings retained for one failed artifact batch entry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCacheBatchFailureTimings {
    preparation: Duration,
    callback: Option<Duration>,
    cleanup: Option<Duration>,
    commit: Option<Duration>,
    total: Duration,
}

/// Failure from one entry in a collect-all artifact transaction batch.
#[derive(Debug)]
pub enum ArtifactCacheBatchFailure<E> {
    /// Cache preparation or commit failed.
    Cache {
        /// Failed cache phase.
        phase: ArtifactCacheBatchFailurePhase,
        /// Cache preparation or commit failure.
        source: Box<ArtifactCacheError>,
        /// Phase timings completed before the failure returned.
        timings: ArtifactCacheBatchFailureTimings,
    },
    /// The caller's population callback failed.
    Build {
        /// Caller population failure.
        source: Box<E>,
        /// Failure from synchronously aborting the active transaction.
        cleanup_error: Option<Box<ArtifactCacheError>>,
        /// Preparation, callback, and explicit cleanup timings.
        timings: ArtifactCacheBatchFailureTimings,
    },
}

/// Aggregate counters and successful-acquisition timings for an artifact batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCacheBatchMetrics {
    entries: usize,
    succeeded: usize,
    failed: usize,
    built: usize,
    reused: usize,
    successful_timings: ArtifactCacheTimings,
    total: Duration,
}

impl LabeledArtifactCacheSpec {
    /// Attach a caller-owned stable label to one artifact specification.
    #[must_use]
    pub fn new(label: impl Into<String>, spec: ArtifactCacheSpec) -> Self {
        Self {
            label: label.into(),
            spec,
        }
    }

    /// Caller-owned report and callback label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Underlying exact artifact specification.
    #[must_use]
    pub const fn spec(&self) -> &ArtifactCacheSpec {
        &self.spec
    }

    /// Consume the entry into its label and artifact specification.
    #[must_use]
    pub fn into_parts(self) -> (String, ArtifactCacheSpec) {
        (self.label, self.spec)
    }
}

impl<E> ArtifactCacheBatchReport<E> {
    /// Ordered labeled entries.
    #[must_use]
    pub fn entries(&self) -> &[ArtifactCacheBatchEntry<E>] {
        &self.entries
    }

    /// Consume the report into its ordered labeled entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<ArtifactCacheBatchEntry<E>> {
        self.entries
    }

    /// Structured successful entries with labels and wall-clock times.
    pub fn outcomes(&self) -> impl Iterator<Item = ArtifactCacheBatchOutcomeEntry<'_>> {
        self.entries.iter().filter_map(|entry| {
            entry
                .outcome()
                .map(|outcome| ArtifactCacheBatchOutcomeEntry {
                    index: entry.index,
                    label: &entry.label,
                    outcome,
                    entry_elapsed: entry.entry_elapsed,
                })
        })
    }

    /// Structured failed entries with labels and partial phase timings.
    pub fn failures(&self) -> impl Iterator<Item = ArtifactCacheBatchFailedEntry<'_, E>> {
        self.entries.iter().filter_map(|entry| {
            entry
                .failure()
                .map(|failure| ArtifactCacheBatchFailedEntry {
                    index: entry.index,
                    label: &entry.label,
                    failure,
                    entry_elapsed: entry.entry_elapsed,
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
        self.entries.iter().all(ArtifactCacheBatchEntry::is_success)
    }

    /// Aggregate outcome counters and successful-acquisition timings.
    #[must_use]
    pub fn metrics(&self) -> ArtifactCacheBatchMetrics {
        let mut metrics = ArtifactCacheBatchMetrics {
            entries: self.entries.len(),
            total: self.total,
            ..ArtifactCacheBatchMetrics::default()
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

impl<E> ArtifactCacheBatchEntry<E> {
    /// Zero-based position in the supplied specification slice.
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
    pub const fn result(&self) -> Result<&ArtifactCacheOutcome, &ArtifactCacheBatchFailure<E>> {
        self.result.as_ref()
    }

    /// Successful artifact outcome, when this entry succeeded.
    #[must_use]
    pub fn outcome(&self) -> Option<&ArtifactCacheOutcome> {
        self.result.as_ref().ok()
    }

    /// Structured batch failure, when this entry failed.
    #[must_use]
    pub fn failure(&self) -> Option<&ArtifactCacheBatchFailure<E>> {
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
        Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>,
        Duration,
    ) {
        (self.index, self.label, self.result, self.entry_elapsed)
    }
}

impl<'a> ArtifactCacheBatchOutcomeEntry<'a> {
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

    /// Successful artifact outcome.
    #[must_use]
    pub const fn outcome(self) -> &'a ArtifactCacheOutcome {
        self.outcome
    }

    /// Complete wall-clock time retained for this successful entry.
    #[must_use]
    pub const fn entry_elapsed(self) -> Duration {
        self.entry_elapsed
    }
}

impl<'a, E> ArtifactCacheBatchFailedEntry<'a, E> {
    /// Zero-based position in the supplied specification slice.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Caller-owned stable label.
    #[must_use]
    pub const fn label(&self) -> &'a str {
        self.label
    }

    /// Structured cache or caller-build failure.
    #[must_use]
    pub const fn failure(&self) -> &'a ArtifactCacheBatchFailure<E> {
        self.failure
    }

    /// Partial phase timings retained with the failure.
    #[must_use]
    pub const fn timings(&self) -> ArtifactCacheBatchFailureTimings {
        self.failure.timings()
    }

    /// Complete wall-clock time retained for this failed entry.
    #[must_use]
    pub const fn entry_elapsed(&self) -> Duration {
        self.entry_elapsed
    }
}

impl<E> ArtifactCacheBatchFailure<E> {
    /// Primary phase in which this entry failed.
    #[must_use]
    pub const fn phase(&self) -> ArtifactCacheBatchFailurePhase {
        match self {
            Self::Cache { phase, .. } => *phase,
            Self::Build { .. } => ArtifactCacheBatchFailurePhase::Callback,
        }
    }

    /// Partial phase timings completed before the failure returned.
    #[must_use]
    pub const fn timings(&self) -> ArtifactCacheBatchFailureTimings {
        match self {
            Self::Cache { timings, .. } | Self::Build { timings, .. } => *timings,
        }
    }

    /// Cleanup failure after a caller population error, when one occurred.
    #[must_use]
    pub fn cleanup_error(&self) -> Option<&ArtifactCacheError> {
        match self {
            Self::Build { cleanup_error, .. } => cleanup_error.as_deref(),
            Self::Cache { .. } => None,
        }
    }
}

impl ArtifactCacheBatchFailureTimings {
    /// Time spent in cache preparation before the failure path continued.
    #[must_use]
    pub const fn preparation(self) -> Duration {
        self.preparation
    }

    /// Time spent in the population callback, when it ran.
    #[must_use]
    pub const fn callback(self) -> Option<Duration> {
        self.callback
    }

    /// Time spent explicitly aborting after a callback failure, when attempted.
    #[must_use]
    pub const fn cleanup(self) -> Option<Duration> {
        self.cleanup
    }

    /// Time spent committing, including commit-owned failure cleanup, when attempted.
    #[must_use]
    pub const fn commit(self) -> Option<Duration> {
        self.commit
    }

    /// Complete wall-clock time retained for the failed entry.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl ArtifactCacheBatchMetrics {
    /// Number of supplied specifications.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
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

    /// Number of newly built artifact sets.
    #[must_use]
    pub const fn built(self) -> usize {
        self.built
    }

    /// Number of artifact sets reused from the exact cache.
    #[must_use]
    pub const fn reused(self) -> usize {
        self.reused
    }

    /// Sum of timings from successful acquisitions.
    #[must_use]
    pub const fn successful_timings(self) -> ArtifactCacheTimings {
        self.successful_timings
    }

    /// Complete wall-clock time for the sequential batch.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl<E> std::fmt::Display for ArtifactCacheBatchReport<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let metrics = self.metrics();
        write!(
            formatter,
            "entries={} succeeded={} failed={} built={} reused={} successful_timings=({}) total={:?}",
            metrics.entries(),
            metrics.succeeded(),
            metrics.failed(),
            metrics.built(),
            metrics.reused(),
            metrics.successful_timings(),
            metrics.total(),
        )
    }
}

/// Build every independent caller-labeled artifact specification.
///
/// Labels must be nonempty and unique; the complete batch is rejected before
/// work begins otherwise. The callback runs only for misses and receives the
/// stable label plus one live transaction at a time. Sequential acquisition
/// avoids self-deadlock when entries share a coordination scope. Every
/// independent result is retained, and a callback error aborts and
/// synchronously cleans that transaction before the next entry begins.
///
/// This operation is not atomic across specifications. A caller requiring one
/// all-or-nothing multi-output publication should declare those outputs on one
/// [`ArtifactCacheSpec`] instead.
pub fn build_artifact_caches_batch<E, F>(
    specs: &[LabeledArtifactCacheSpec],
    mut populate: F,
) -> Result<ArtifactCacheBatchReport<E>, ArtifactCacheBatchContractError>
where
    F: FnMut(&str, &ArtifactBuildTransaction) -> Result<(), E>,
{
    validate_batch_labels(specs)?;
    let started = Instant::now();
    let mut entries = Vec::with_capacity(specs.len());
    for (index, labeled) in specs.iter().enumerate() {
        let entry_started = Instant::now();
        let preparation_started = Instant::now();
        let preparation = prepare_artifact_cache(&labeled.spec);
        let preparation_elapsed = preparation_started.elapsed();
        let (result, entry_elapsed) = match preparation {
            Ok(ArtifactCachePreparation::Reused(record)) => (
                Ok(ArtifactCacheOutcome::Reused(record)),
                entry_started.elapsed(),
            ),
            Ok(ArtifactCachePreparation::Build(transaction)) => {
                let callback_started = Instant::now();
                let callback_result = populate(&labeled.label, &transaction);
                let callback_elapsed = callback_started.elapsed();
                if let Err(source) = callback_result {
                    let cleanup_started = Instant::now();
                    let cleanup_error = transaction.abort().err().map(Box::new);
                    let cleanup_elapsed = cleanup_started.elapsed();
                    let entry_elapsed = entry_started.elapsed();
                    (
                        Err(ArtifactCacheBatchFailure::Build {
                            source: Box::new(source),
                            cleanup_error,
                            timings: ArtifactCacheBatchFailureTimings {
                                preparation: preparation_elapsed,
                                callback: Some(callback_elapsed),
                                cleanup: Some(cleanup_elapsed),
                                commit: None,
                                total: entry_elapsed,
                            },
                        }),
                        entry_elapsed,
                    )
                } else {
                    let commit_started = Instant::now();
                    match transaction.commit() {
                        Ok(outcome) => (Ok(outcome), entry_started.elapsed()),
                        Err(source) => {
                            let commit_elapsed = commit_started.elapsed();
                            let entry_elapsed = entry_started.elapsed();
                            (
                                Err(ArtifactCacheBatchFailure::Cache {
                                    phase: ArtifactCacheBatchFailurePhase::Commit,
                                    source: Box::new(source),
                                    timings: ArtifactCacheBatchFailureTimings {
                                        preparation: preparation_elapsed,
                                        callback: Some(callback_elapsed),
                                        cleanup: None,
                                        commit: Some(commit_elapsed),
                                        total: entry_elapsed,
                                    },
                                }),
                                entry_elapsed,
                            )
                        }
                    }
                }
            }
            Err(source) => {
                let entry_elapsed = entry_started.elapsed();
                (
                    Err(ArtifactCacheBatchFailure::Cache {
                        phase: ArtifactCacheBatchFailurePhase::Preparation,
                        source: Box::new(source),
                        timings: ArtifactCacheBatchFailureTimings {
                            preparation: preparation_elapsed,
                            callback: None,
                            cleanup: None,
                            commit: None,
                            total: entry_elapsed,
                        },
                    }),
                    entry_elapsed,
                )
            }
        };
        entries.push(ArtifactCacheBatchEntry {
            index,
            label: labeled.label.clone(),
            result,
            entry_elapsed,
        });
    }
    Ok(ArtifactCacheBatchReport {
        entries,
        total: started.elapsed(),
    })
}

fn validate_batch_labels(
    specs: &[LabeledArtifactCacheSpec],
) -> Result<(), ArtifactCacheBatchContractError> {
    let mut labels = HashMap::with_capacity(specs.len());
    for (index, labeled) in specs.iter().enumerate() {
        if labeled.label.is_empty() {
            return Err(ArtifactCacheBatchContractError::EmptyLabel { index });
        }
        if let Some(first_index) = labels.get(labeled.label.as_str()) {
            return Err(ArtifactCacheBatchContractError::DuplicateLabel {
                label: labeled.label.clone(),
                first_index: *first_index,
                duplicate_index: index,
            });
        }
        labels.insert(labeled.label.as_str(), index);
    }
    Ok(())
}

impl std::fmt::Display for ArtifactCacheBatchContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLabel { index } => {
                write!(formatter, "artifact batch label at index {index} is empty")
            }
            Self::DuplicateLabel {
                label,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "artifact batch label {label:?} at index {duplicate_index} duplicates index {first_index}",
            ),
        }
    }
}

impl std::error::Error for ArtifactCacheBatchContractError {}

impl std::fmt::Display for ArtifactCacheBatchFailurePhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Preparation => "preparation",
            Self::Callback => "callback",
            Self::Commit => "commit",
        })
    }
}

impl std::fmt::Display for ArtifactCacheBatchFailureTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} preparation={:?} callback={:?} cleanup={:?} commit={:?}",
            self.total, self.preparation, self.callback, self.cleanup, self.commit,
        )
    }
}

impl<E: std::fmt::Display> std::fmt::Display for ArtifactCacheBatchFailure<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cache {
                phase,
                source,
                timings,
            } => write!(
                formatter,
                "artifact cache {phase} failed: {source}; timings=({timings})",
            ),
            Self::Build {
                source,
                cleanup_error,
                timings,
            } => {
                write!(
                    formatter,
                    "artifact callback failed: {source}; timings=({timings})"
                )?;
                if let Some(cleanup_error) = cleanup_error {
                    write!(formatter, "; cleanup also failed: {cleanup_error}")?;
                }
                Ok(())
            }
        }
    }
}

impl<E> std::error::Error for ArtifactCacheBatchFailure<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cache { source, .. } => Some(source.as_ref()),
            Self::Build { source, .. } => Some(source.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests;
