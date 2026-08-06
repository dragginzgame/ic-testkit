use super::{
    BatchMaintenanceTracker, WasmBuildBatchConfig, build_wasm_canisters_cached_batch,
    build_wasm_canisters_cached_batch_with_config,
};
use crate::artifacts::{
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceFailureMode,
    SharedIncrementalTargetPrunePolicy, WasmBuildError, WasmBuildSpec,
};
use std::{path::Path, time::Duration};

#[test]
fn empty_independent_batch_succeeds_without_work() {
    let outcome = build_wasm_canisters_cached_batch(&[]).expect("empty batch");
    assert!(outcome.outcomes().is_empty());
}

#[test]
fn invalid_independent_spec_reports_its_batch_index() {
    let specs = [WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &[],
        "debug",
    )];
    let error = build_wasm_canisters_cached_batch(&specs).expect_err("invalid batch entry");
    assert_eq!(error.failed_index(), 0);
    assert!(error.completed().is_empty());
}

#[test]
fn batch_maintenance_configures_each_shared_target_once() {
    let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
        SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(1024),
        Duration::from_secs(60),
    )
    .with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
    let batch = WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance);
    let mut tracker = BatchMaintenanceTracker::new(batch.shared_incremental_target_maintenance());
    let first = WasmBuildSpec::new(Path::new("."), Path::new("exact-a"), &["a"], "debug")
        .with_shared_incremental_target("shared-a");
    let second = WasmBuildSpec::new(Path::new("."), Path::new("exact-b"), &["b"], "debug")
        .with_shared_incremental_target("shared-a");
    let other = WasmBuildSpec::new(Path::new("."), Path::new("exact-c"), &["c"], "debug")
        .with_shared_incremental_target("shared-b");
    let isolated = WasmBuildSpec::new(Path::new("."), Path::new("exact-d"), &["d"], "debug");

    let prepared_first = tracker
        .prepare_spec(&first)
        .expect("first shared target must own maintenance");
    assert_eq!(
        prepared_first.shared_incremental_target_maintenance(),
        Some(maintenance)
    );
    assert!(tracker.prepare_spec(&second).is_none());
    assert!(tracker.prepare_spec(&other).is_some());
    assert!(tracker.prepare_spec(&isolated).is_none());
}

#[test]
fn batch_maintenance_rejects_per_spec_policy_ownership() {
    let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
        SharedIncrementalTargetPrunePolicy::new(),
        Duration::from_secs(60),
    );
    let spec = WasmBuildSpec::new(Path::new("."), Path::new("exact"), &["fixture"], "debug")
        .with_shared_incremental_target("shared")
        .with_shared_incremental_target_maintenance(maintenance);
    let batch = WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance);

    let error = build_wasm_canisters_cached_batch_with_config(&[spec], batch)
        .expect_err("ambiguous maintenance ownership must fail before building");
    assert_eq!(error.failed_index(), 0);
    assert!(error.completed().is_empty());
    let (_, source) = error.into_parts();
    assert!(
        matches!(source, WasmBuildError::InvalidSpec { message } if message.contains("cannot be combined"))
    );
}
