use std::time::{Duration, Instant};

use super::transaction::{
    ArtifactBuildTransaction, ArtifactCacheError, ArtifactCacheOutcome, ArtifactCachePreparation,
    ArtifactCacheSpec, prepare_artifact_cache,
};

/// Successful outcomes from sequential independent artifact transactions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactCacheBatchOutcome {
    outcomes: Vec<ArtifactCacheOutcome>,
    total: Duration,
}

/// Failure from a sequential independent artifact transaction batch.
#[derive(Debug)]
pub enum ArtifactCacheBatchError<E> {
    /// Cache preparation or commit failed.
    Cache {
        failed_index: usize,
        completed: Vec<ArtifactCacheOutcome>,
        total: Duration,
        source: Box<ArtifactCacheError>,
    },
    /// The caller's population callback failed.
    Build {
        failed_index: usize,
        completed: Vec<ArtifactCacheOutcome>,
        total: Duration,
        source: Box<E>,
        cleanup_error: Option<Box<ArtifactCacheError>>,
    },
}

impl ArtifactCacheBatchOutcome {
    /// Successful outcomes in specification order.
    #[must_use]
    pub fn outcomes(&self) -> &[ArtifactCacheOutcome] {
        &self.outcomes
    }

    /// Consume the report and return its ordered outcomes.
    #[must_use]
    pub fn into_outcomes(self) -> Vec<ArtifactCacheOutcome> {
        self.outcomes
    }

    /// Complete wall-clock time for the sequential batch.
    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }
}

impl<E> ArtifactCacheBatchError<E> {
    /// Zero-based index of the failed independent specification.
    #[must_use]
    pub const fn failed_index(&self) -> usize {
        match self {
            Self::Cache { failed_index, .. } | Self::Build { failed_index, .. } => *failed_index,
        }
    }

    /// Successful outcomes completed before the failure.
    #[must_use]
    pub fn completed(&self) -> &[ArtifactCacheOutcome] {
        match self {
            Self::Cache { completed, .. } | Self::Build { completed, .. } => completed,
        }
    }

    /// Wall-clock time through the failure.
    #[must_use]
    pub const fn total(&self) -> Duration {
        match self {
            Self::Cache { total, .. } | Self::Build { total, .. } => *total,
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

impl std::fmt::Display for ArtifactCacheBatchOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reused = self
            .outcomes
            .iter()
            .filter(|outcome| outcome.is_reused())
            .count();
        write!(
            formatter,
            "entries={} built={} reused={} total={:?}",
            self.outcomes.len(),
            self.outcomes.len().saturating_sub(reused),
            reused,
            self.total,
        )
    }
}

/// Build or reuse multiple independent transactional artifact sets.
///
/// The callback runs only for misses and receives one live transaction at a
/// time. Sequential acquisition avoids self-deadlock when entries share a
/// coordination scope. A callback error aborts and synchronously cleans that
/// transaction before returning. Completed entries remain valid when a later
/// entry fails.
///
/// This operation is not atomic across specifications. A caller requiring one
/// all-or-nothing multi-output publication should declare those outputs on one
/// [`ArtifactCacheSpec`] instead.
pub fn build_artifact_caches_batch<E, F>(
    specs: &[ArtifactCacheSpec],
    mut populate: F,
) -> Result<ArtifactCacheBatchOutcome, ArtifactCacheBatchError<E>>
where
    F: FnMut(usize, &ArtifactBuildTransaction) -> Result<(), E>,
{
    let started = Instant::now();
    let mut outcomes = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        let preparation = match prepare_artifact_cache(spec) {
            Ok(preparation) => preparation,
            Err(source) => {
                return Err(ArtifactCacheBatchError::Cache {
                    failed_index: index,
                    completed: outcomes,
                    total: started.elapsed(),
                    source: Box::new(source),
                });
            }
        };
        let outcome = match preparation {
            ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
            ArtifactCachePreparation::Build(transaction) => {
                if let Err(source) = populate(index, &transaction) {
                    let cleanup_error = transaction.abort().err().map(Box::new);
                    return Err(ArtifactCacheBatchError::Build {
                        failed_index: index,
                        completed: outcomes,
                        total: started.elapsed(),
                        source: Box::new(source),
                        cleanup_error,
                    });
                }
                match transaction.commit() {
                    Ok(outcome) => outcome,
                    Err(source) => {
                        return Err(ArtifactCacheBatchError::Cache {
                            failed_index: index,
                            completed: outcomes,
                            total: started.elapsed(),
                            source: Box::new(source),
                        });
                    }
                }
            }
        };
        outcomes.push(outcome);
    }
    Ok(ArtifactCacheBatchOutcome {
        outcomes,
        total: started.elapsed(),
    })
}

impl<E: std::fmt::Display> std::fmt::Display for ArtifactCacheBatchError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cache {
                failed_index,
                completed,
                source,
                ..
            } => write!(
                formatter,
                "artifact cache batch entry {failed_index} failed after {} successful entry/entries: {source}",
                completed.len(),
            ),
            Self::Build {
                failed_index,
                completed,
                source,
                cleanup_error,
                ..
            } => {
                write!(
                    formatter,
                    "artifact cache batch builder {failed_index} failed after {} successful entry/entries: {source}",
                    completed.len(),
                )?;
                if let Some(cleanup_error) = cleanup_error {
                    write!(formatter, "; cleanup also failed: {cleanup_error}")?;
                }
                Ok(())
            }
        }
    }
}

impl<E> std::error::Error for ArtifactCacheBatchError<E>
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
mod tests {
    use super::{ArtifactCacheBatchError, build_artifact_caches_batch};
    use crate::artifacts::{
        ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache,
        test_support::unique_temp_directory,
    };
    use std::fs;

    #[test]
    fn independent_transactions_build_then_reuse_in_order() {
        let root = unique_temp_directory("artifact-cache-batch");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write batch input");
        let specs = [
            ArtifactCacheSpec::new(&root.join("cache"), "first", "recipe/v1")
                .with_coordination_scope("shared-builder")
                .with_input("input", &input)
                .with_output("output", &root.join("first.out")),
            ArtifactCacheSpec::new(&root.join("cache"), "second", "recipe/v1")
                .with_coordination_scope("shared-builder")
                .with_input("input", &input)
                .with_output("output", &root.join("second.out")),
        ];
        let mut built_indices = Vec::new();
        let built = build_artifact_caches_batch(&specs, |index, transaction| {
            built_indices.push(index);
            fs::write(
                transaction
                    .output_path("output")
                    .expect("batch output path"),
                format!("output-{index}"),
            )
            .expect("write batch output");
            Ok::<(), &'static str>(())
        })
        .expect("build independent batch");

        assert_eq!(built_indices, [0, 1]);
        assert!(
            built
                .outcomes()
                .iter()
                .all(|outcome| matches!(outcome, ArtifactCacheOutcome::Built(_)))
        );

        let reused = build_artifact_caches_batch(&specs, |_index, _transaction| {
            Err::<(), _>("unexpected cache miss")
        })
        .expect("reuse independent batch");
        assert!(
            reused
                .outcomes()
                .iter()
                .all(ArtifactCacheOutcome::is_reused)
        );
        fs::remove_dir_all(root).expect("remove artifact batch fixture");
    }

    #[test]
    fn builder_failure_aborts_current_transaction_and_reports_completed_prefix() {
        let root = unique_temp_directory("artifact-cache-batch-failure");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write batch input");
        let specs = [
            ArtifactCacheSpec::new(&root.join("cache"), "first", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("first.out")),
            ArtifactCacheSpec::new(&root.join("cache"), "second", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("second.out")),
        ];
        let result = build_artifact_caches_batch(&specs, |index, transaction| {
            if index == 1 {
                return Err("synthetic builder failure");
            }
            fs::write(transaction.output_path("output").unwrap(), b"first")
                .expect("write successful prefix output");
            Ok(())
        });

        let ArtifactCacheBatchError::Build {
            failed_index,
            completed,
            cleanup_error,
            ..
        } = result.expect_err("second builder must fail")
        else {
            panic!("expected caller builder failure");
        };
        assert_eq!(failed_index, 1);
        assert_eq!(completed.len(), 1);
        assert!(cleanup_error.is_none());
        assert!(!root.join("second.out").exists());
        let preparation = prepare_artifact_cache(&specs[1]).expect("prepare failed entry again");
        let ArtifactCachePreparation::Build(transaction) = preparation else {
            panic!("failed batch entry must not be published");
        };
        transaction.abort().expect("abort verification transaction");
        fs::remove_dir_all(root).expect("remove artifact batch failure fixture");
    }
}
