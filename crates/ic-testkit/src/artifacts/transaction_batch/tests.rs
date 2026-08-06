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
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            b"first",
        )
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
