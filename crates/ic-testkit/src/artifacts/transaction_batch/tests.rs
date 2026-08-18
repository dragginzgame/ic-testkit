use std::{cell::Cell, fs};

use super::{
    ArtifactCacheBatchContractError, ArtifactCacheBatchFailure, ArtifactCacheBatchFailurePhase,
    ArtifactCacheBatchOutcomeEntry, LabeledArtifactCacheSpec, build_artifact_caches_batch,
};
use crate::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache,
    test_support::unique_temp_directory,
};

#[test]
fn labeled_transactions_build_then_reuse_in_order() {
    let root = unique_temp_directory("artifact-cache-batch");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        LabeledArtifactCacheSpec::new(
            "first",
            ArtifactCacheSpec::new(&root.join("cache"), "first", "recipe/v1")
                .with_coordination_scope("shared-builder")
                .with_input("input", &input)
                .with_output("output", &root.join("first.out")),
        ),
        LabeledArtifactCacheSpec::new(
            "second",
            ArtifactCacheSpec::new(&root.join("cache"), "second", "recipe/v1")
                .with_coordination_scope("shared-builder")
                .with_input("input", &input)
                .with_output("output", &root.join("second.out")),
        ),
    ];
    let mut built_labels = Vec::new();
    let built = build_artifact_caches_batch(&specs, |label, transaction| {
        built_labels.push(label.to_owned());
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{label}"),
        )
        .expect("write batch output");
        Ok::<(), &'static str>(())
    })
    .expect("valid labeled batch");

    assert!(built.is_success());
    assert_eq!(built.entries().len(), 2);
    assert_eq!(built.entries()[0].label(), "first");
    assert_eq!(built.entries()[1].label(), "second");
    assert_eq!(built_labels, ["first", "second"]);
    assert!(built.outcomes().all(|entry| {
        matches!(entry.outcome(), ArtifactCacheOutcome::Built(_))
            && entry.entry_elapsed() <= built.total()
    }));
    let built_metrics = built.metrics();
    assert_eq!(built_metrics.entries(), 2);
    assert_eq!(built_metrics.succeeded(), 2);
    assert_eq!(built_metrics.failed(), 0);
    assert_eq!(built_metrics.built(), 2);
    assert_eq!(built_metrics.reused(), 0);
    assert!(built_metrics.successful_timings().caller_build().is_some());

    let reused = build_artifact_caches_batch(&specs, |_label, _transaction| {
        Err::<(), _>("unexpected cache miss")
    })
    .expect("valid reused batch");
    assert!(reused.is_success());
    assert!(reused.outcomes().all(|entry| entry.outcome().is_reused()));
    let reused_metrics = reused.metrics();
    assert_eq!(reused_metrics.built(), 0);
    assert_eq!(reused_metrics.reused(), 2);
    assert!(reused_metrics.successful_timings().caller_build().is_none());
    fs::remove_dir_all(root).expect("remove artifact batch fixture");
}

#[test]
fn callback_failure_retains_label_timings_and_later_transaction() {
    let root = unique_temp_directory("artifact-cache-batch-builder-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        if label == "second" {
            return Err("synthetic builder failure".to_owned());
        }
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{label}"),
        )
        .expect("write successful batch output");
        Ok(())
    })
    .expect("valid labeled batch");

    assert!(!report.is_success());
    assert_eq!(report.entries().len(), 3);
    assert_eq!(
        report
            .outcomes()
            .map(ArtifactCacheBatchOutcomeEntry::label)
            .collect::<Vec<_>>(),
        ["first", "third"]
    );
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    let failure = &failures[0];
    assert_eq!(failure.index(), 1);
    assert_eq!(failure.label(), "second");
    assert_eq!(failure.entry_elapsed(), failure.timings().total());
    assert_eq!(
        failure.failure().phase(),
        ArtifactCacheBatchFailurePhase::Callback
    );
    assert!(failure.timings().callback().is_some());
    assert!(failure.timings().cleanup().is_some());
    assert!(failure.timings().commit().is_none());
    assert!(matches!(
        failure.failure(),
        ArtifactCacheBatchFailure::Build {
            cleanup_error: None,
            ..
        }
    ));
    assert!(!root.join("second.out").exists());
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-third");
    let metrics = report.metrics();
    assert_eq!(metrics.succeeded(), 2);
    assert_eq!(metrics.failed(), 1);
    assert_eq!(metrics.built(), 2);

    let preparation = prepare_artifact_cache(specs[1].spec()).expect("prepare failed entry again");
    let ArtifactCachePreparation::Build(transaction) = preparation else {
        panic!("failed batch entry must not be published");
    };
    transaction.abort().expect("abort verification transaction");
    fs::remove_dir_all(root).expect("remove artifact batch failure fixture");
}

#[test]
fn preparation_failure_is_labeled_timed_and_does_not_stop_later_entries() {
    let root = unique_temp_directory("artifact-cache-batch-cache-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        LabeledArtifactCacheSpec::new(
            "invalid",
            ArtifactCacheSpec::new(&root.join("cache"), "", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("invalid.out")),
        ),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{label}"),
        )
        .expect("write successful batch output");
        Ok::<(), &'static str>(())
    })
    .expect("valid labeled batch");

    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    let failure = &failures[0];
    assert_eq!(failure.index(), 1);
    assert_eq!(failure.label(), "invalid");
    assert_eq!(
        failure.failure().phase(),
        ArtifactCacheBatchFailurePhase::Preparation
    );
    assert!(failure.timings().callback().is_none());
    assert!(failure.timings().cleanup().is_none());
    assert!(failure.timings().commit().is_none());
    assert!(matches!(
        failure.failure(),
        ArtifactCacheBatchFailure::Cache { .. }
    ));
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-third");
    fs::remove_dir_all(root).expect("remove artifact cache failure fixture");
}

#[test]
fn commit_failure_is_labeled_timed_and_does_not_stop_later_entries() {
    let root = unique_temp_directory("artifact-cache-batch-commit-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        if label != "second" {
            fs::write(
                transaction
                    .output_path("output")
                    .expect("batch output path"),
                format!("output-{label}"),
            )
            .expect("write successful batch output");
        }
        Ok::<(), &'static str>(())
    })
    .expect("valid labeled batch");

    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    let failure = &failures[0];
    assert_eq!(failure.label(), "second");
    assert_eq!(
        failure.failure().phase(),
        ArtifactCacheBatchFailurePhase::Commit
    );
    assert!(failure.timings().callback().is_some());
    assert!(failure.timings().cleanup().is_none());
    assert!(failure.timings().commit().is_some());
    assert!(matches!(
        failure.failure(),
        ArtifactCacheBatchFailure::Cache { .. }
    ));
    assert!(!root.join("second.out").exists());
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-third");
    fs::remove_dir_all(root).expect("remove artifact cache commit failure fixture");
}

#[test]
fn batch_rejects_empty_or_duplicate_labels_before_work() {
    let root = unique_temp_directory("artifact-cache-batch-label-contract");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let callback_ran = Cell::new(false);
    let empty = [artifact_spec(&root, &input, "", "empty.out")];
    let error = build_artifact_caches_batch(&empty, |_label, _transaction| {
        callback_ran.set(true);
        Ok::<(), &'static str>(())
    })
    .expect_err("empty label must fail");
    assert_eq!(
        error,
        ArtifactCacheBatchContractError::EmptyLabel { index: 0 }
    );
    assert!(!callback_ran.get());

    let duplicate = [
        artifact_spec(&root, &input, "same", "first.out"),
        LabeledArtifactCacheSpec::new(
            "same",
            ArtifactCacheSpec::new(&root.join("cache"), "other", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("second.out")),
        ),
    ];
    let error = build_artifact_caches_batch(&duplicate, |_label, _transaction| {
        callback_ran.set(true);
        Ok::<(), &'static str>(())
    })
    .expect_err("duplicate label must fail");
    assert_eq!(
        error,
        ArtifactCacheBatchContractError::DuplicateLabel {
            label: "same".to_owned(),
            first_index: 0,
            duplicate_index: 1,
        }
    );
    assert!(!callback_ran.get());
    fs::remove_dir_all(root).expect("remove artifact cache label fixture");
}

fn artifact_spec(
    root: &std::path::Path,
    input: &std::path::Path,
    label: &str,
    output: &str,
) -> LabeledArtifactCacheSpec {
    LabeledArtifactCacheSpec::new(
        label,
        ArtifactCacheSpec::new(&root.join("cache"), label, "recipe/v1")
            .with_input("input", input)
            .with_output("output", &root.join(output)),
    )
}
