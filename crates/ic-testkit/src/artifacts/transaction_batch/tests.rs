use super::{ArtifactCacheBatchFailure, build_artifact_caches_batch};
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
    });

    assert!(built.is_success());
    assert_eq!(built.entry_elapsed().len(), 2);
    assert_eq!(built_indices, [0, 1]);
    assert!(
        built
            .outcomes()
            .all(|(_index, outcome)| matches!(outcome, ArtifactCacheOutcome::Built(_)))
    );
    let built_metrics = built.metrics();
    assert_eq!(built_metrics.entries(), 2);
    assert_eq!(built_metrics.succeeded(), 2);
    assert_eq!(built_metrics.failed(), 0);
    assert_eq!(built_metrics.built(), 2);
    assert_eq!(built_metrics.reused(), 0);
    assert!(built_metrics.successful_timings().caller_build().is_some());

    let reused = build_artifact_caches_batch(&specs, |_index, _transaction| {
        Err::<(), _>("unexpected cache miss")
    });
    assert!(reused.is_success());
    assert!(
        reused
            .outcomes()
            .all(|(_index, outcome)| outcome.is_reused())
    );
    let reused_metrics = reused.metrics();
    assert_eq!(reused_metrics.built(), 0);
    assert_eq!(reused_metrics.reused(), 2);
    assert!(reused_metrics.successful_timings().caller_build().is_none());
    fs::remove_dir_all(root).expect("remove artifact batch fixture");
}

#[test]
fn builder_failure_is_retained_and_later_transaction_runs() {
    let root = unique_temp_directory("artifact-cache-batch-builder-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |index, transaction| {
        if index == 1 {
            return Err("synthetic builder failure".to_owned());
        }
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{index}"),
        )
        .expect("write successful batch output");
        Ok(())
    });

    assert!(!report.is_success());
    assert_eq!(report.results().len(), 3);
    assert_eq!(report.entry_elapsed().len(), 3);
    assert_eq!(
        report
            .outcomes()
            .map(|(index, _outcome)| index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].index(), 1);
    assert_eq!(failures[0].entry_elapsed(), report.entry_elapsed()[1]);
    assert!(matches!(
        failures[0].failure(),
        ArtifactCacheBatchFailure::Build {
            cleanup_error: None,
            ..
        }
    ));
    assert!(!root.join("second.out").exists());
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-2");
    let metrics = report.metrics();
    assert_eq!(metrics.succeeded(), 2);
    assert_eq!(metrics.failed(), 1);
    assert_eq!(metrics.built(), 2);

    let preparation = prepare_artifact_cache(&specs[1]).expect("prepare failed entry again");
    let ArtifactCachePreparation::Build(transaction) = preparation else {
        panic!("failed batch entry must not be published");
    };
    transaction.abort().expect("abort verification transaction");
    fs::remove_dir_all(root).expect("remove artifact batch failure fixture");
}

#[test]
fn cache_failure_is_indexed_and_does_not_stop_later_entries() {
    let root = unique_temp_directory("artifact-cache-batch-cache-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        ArtifactCacheSpec::new(&root.join("cache"), "", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("invalid.out")),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |index, transaction| {
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{index}"),
        )
        .expect("write successful batch output");
        Ok::<(), &'static str>(())
    });

    assert_eq!(
        report
            .failures()
            .map(|entry| {
                assert!(matches!(
                    entry.failure(),
                    ArtifactCacheBatchFailure::Cache { .. }
                ));
                entry.index()
            })
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-2");
    fs::remove_dir_all(root).expect("remove artifact cache failure fixture");
}

#[test]
fn commit_failure_is_indexed_and_does_not_stop_later_entries() {
    let root = unique_temp_directory("artifact-cache-batch-commit-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |index, transaction| {
        if index != 1 {
            fs::write(
                transaction
                    .output_path("output")
                    .expect("batch output path"),
                format!("output-{index}"),
            )
            .expect("write successful batch output");
        }
        Ok::<(), &'static str>(())
    });

    assert_eq!(
        report
            .failures()
            .map(|entry| {
                assert!(matches!(
                    entry.failure(),
                    ArtifactCacheBatchFailure::Cache { .. }
                ));
                entry.index()
            })
            .collect::<Vec<_>>(),
        [1]
    );
    assert!(!root.join("second.out").exists());
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-2");
    fs::remove_dir_all(root).expect("remove artifact cache commit failure fixture");
}

fn artifact_spec(
    root: &std::path::Path,
    input: &std::path::Path,
    namespace: &str,
    output: &str,
) -> ArtifactCacheSpec {
    ArtifactCacheSpec::new(&root.join("cache"), namespace, "recipe/v1")
        .with_input("input", input)
        .with_output("output", &root.join(output))
}
