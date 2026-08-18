use std::time::{Duration, Instant};

use super::transaction::{
    ArtifactBuildTransaction, ArtifactCacheError, ArtifactCacheOutcome, ArtifactCachePreparation,
    ArtifactCacheSpec, ArtifactCacheTimings, prepare_artifact_cache,
};

/// Ordered outcomes and failures from a collect-all artifact transaction batch.
#[derive(Debug)]
pub struct ArtifactCacheBatchReport<E> {
    results: Vec<Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>>,
    total: Duration,
}

/// Failure from one entry in a collect-all artifact transaction batch.
#[derive(Debug)]
pub enum ArtifactCacheBatchFailure<E> {
    /// Cache preparation or commit failed.
    Cache {
        /// Cache preparation or commit failure.
        source: Box<ArtifactCacheError>,
    },
    /// The caller's population callback failed.
    Build {
        /// Caller population failure.
        source: Box<E>,
        /// Failure from synchronously aborting the active transaction.
        cleanup_error: Option<Box<ArtifactCacheError>>,
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

impl<E> ArtifactCacheBatchReport<E> {
    /// Per-specification results in the supplied order.
    pub fn results(&self) -> &[Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>] {
        &self.results
    }

    /// Consume the report and return its ordered per-specification results.
    #[must_use]
    pub fn into_results(self) -> Vec<Result<ArtifactCacheOutcome, ArtifactCacheBatchFailure<E>>> {
        self.results
    }

    /// Successful outcomes with their specification indexes.
    pub fn outcomes(&self) -> impl Iterator<Item = (usize, &ArtifactCacheOutcome)> {
        self.results
            .iter()
            .enumerate()
            .filter_map(|(index, result)| result.as_ref().ok().map(|outcome| (index, outcome)))
    }

    /// Failures with their specification indexes.
    pub fn failures(&self) -> impl Iterator<Item = (usize, &ArtifactCacheBatchFailure<E>)> {
        self.results
            .iter()
            .enumerate()
            .filter_map(|(index, result)| result.as_ref().err().map(|error| (index, error)))
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

    /// Aggregate outcome counters and successful-acquisition timings.
    #[must_use]
    pub fn metrics(&self) -> ArtifactCacheBatchMetrics {
        let mut metrics = ArtifactCacheBatchMetrics {
            entries: self.results.len(),
            total: self.total,
            ..ArtifactCacheBatchMetrics::default()
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

impl<E> ArtifactCacheBatchFailure<E> {
    /// Cleanup failure after a caller population error, when one occurred.
    #[must_use]
    pub fn cleanup_error(&self) -> Option<&ArtifactCacheError> {
        match self {
            Self::Build { cleanup_error, .. } => cleanup_error.as_deref(),
            Self::Cache { .. } => None,
        }
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

/// Build every independent transactional artifact specification.
///
/// The callback runs only for misses and receives one live transaction at a
/// time. Sequential acquisition avoids self-deadlock when entries share a
/// coordination scope. Every result is retained, and a callback error aborts
/// and synchronously cleans that transaction before the next entry begins.
///
/// This operation is not atomic across specifications. A caller requiring one
/// all-or-nothing multi-output publication should declare those outputs on one
/// [`ArtifactCacheSpec`] instead.
#[must_use]
pub fn build_artifact_caches_batch<E, F>(
    specs: &[ArtifactCacheSpec],
    mut populate: F,
) -> ArtifactCacheBatchReport<E>
where
    F: FnMut(usize, &ArtifactBuildTransaction) -> Result<(), E>,
{
    let started = Instant::now();
    let mut results = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        let result = match prepare_artifact_cache(spec) {
            Ok(ArtifactCachePreparation::Reused(record)) => {
                Ok(ArtifactCacheOutcome::Reused(record))
            }
            Ok(ArtifactCachePreparation::Build(transaction)) => {
                if let Err(source) = populate(index, &transaction) {
                    let cleanup_error = transaction.abort().err().map(Box::new);
                    Err(ArtifactCacheBatchFailure::Build {
                        source: Box::new(source),
                        cleanup_error,
                    })
                } else {
                    transaction
                        .commit()
                        .map_err(|source| ArtifactCacheBatchFailure::Cache {
                            source: Box::new(source),
                        })
                }
            }
            Err(source) => Err(ArtifactCacheBatchFailure::Cache {
                source: Box::new(source),
            }),
        };
        results.push(result);
    }
    ArtifactCacheBatchReport {
        results,
        total: started.elapsed(),
    }
}

impl<E: std::fmt::Display> std::fmt::Display for ArtifactCacheBatchFailure<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cache { source } => write!(formatter, "artifact cache failed: {source}"),
            Self::Build {
                source,
                cleanup_error,
            } => {
                write!(formatter, "artifact builder failed: {source}")?;
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
            Self::Cache { source } => Some(source.as_ref()),
            Self::Build { source, .. } => Some(source.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests;
