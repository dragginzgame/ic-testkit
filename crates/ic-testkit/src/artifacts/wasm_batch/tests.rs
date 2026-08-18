use super::{
    BatchMaintenanceTracker, LabeledWasmBuildSpec, WasmBuildBatchConfig,
    WasmBuildBatchContractError, WasmBuildBatchProgressEvent, build_wasm_canisters_cached_batch,
    build_wasm_canisters_cached_batch_with_config, build_wasm_canisters_cached_batch_with_progress,
};
use crate::artifacts::{
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceFailureMode,
    SharedIncrementalTargetPrunePolicy, WasmBuildBatchFailure, WasmBuildError,
    WasmBuildFailurePhase, WasmBuildProgressConfig, WasmBuildSpec,
};
use std::{path::Path, time::Duration};

#[cfg(unix)]
use crate::artifacts::test_support::{unique_temp_directory, write_executable_script};

#[test]
fn empty_independent_batch_succeeds_without_work() {
    let report = build_wasm_canisters_cached_batch(&[]).expect("empty labeled batch");
    assert!(report.is_success());
    assert_eq!(report.outcomes().count(), 0);
}

#[test]
fn batch_retains_every_indexed_failure() {
    let specs = [
        LabeledWasmBuildSpec::new(
            "root",
            WasmBuildSpec::new(Path::new("."), Path::new("target"), &[], "debug"),
        ),
        LabeledWasmBuildSpec::new(
            "worker",
            WasmBuildSpec::new(Path::new("."), Path::new("target"), &["fixture"], ""),
        ),
    ];

    let report = build_wasm_canisters_cached_batch(&specs).expect("valid labeled batch");

    assert!(!report.is_success());
    assert_eq!(report.entries().len(), 2);
    assert_eq!(report.entries()[0].label(), "root");
    assert_eq!(report.entries()[1].label(), "worker");
    assert_eq!(
        report
            .failures()
            .map(WasmBuildBatchFailure::label)
            .collect::<Vec<_>>(),
        ["root", "worker"]
    );
    assert!(
        report
            .failures()
            .all(|failure| failure.entry_elapsed() <= report.total())
    );
    assert!(report.failures().all(|failure| {
        failure.phase() == WasmBuildFailurePhase::Specification
            && failure.timings().total() <= failure.entry_elapsed()
    }));
    assert_eq!(report.outcomes().count(), 0);
}

#[cfg(unix)]
#[test]
fn batch_failure_retains_partial_metadata_timing() {
    let root = unique_temp_directory("wasm-batch-failure-timings");
    let cargo = root.join("cargo.sh");
    write_executable_script(
        &cargo,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'cargo 1.0.0'; exit 0; fi\nsleep 0.02\necho 'synthetic metadata failure' >&2\nexit 23\n",
    );
    let specs = [LabeledWasmBuildSpec::new(
        "metadata-failure",
        WasmBuildSpec::new(&root, &root.join("target"), &["fixture"], "debug")
            .with_cargo_program(&cargo),
    )];

    let report = build_wasm_canisters_cached_batch(&specs).expect("valid labeled batch");
    let failure = report.failures().next().expect("metadata failure");

    assert_eq!(failure.phase(), WasmBuildFailurePhase::CargoMetadata);
    assert!(failure.timings().input_resolution().tool_identity() > Duration::ZERO);
    assert!(failure.timings().input_resolution().cargo_metadata() > Duration::ZERO);
    assert_eq!(failure.timings().cargo_build(), None);
    assert!(failure.timings().total() <= failure.entry_elapsed());

    std::fs::remove_dir_all(root).expect("remove failure timing fixture");
}

#[test]
fn batch_rejects_invalid_labels_before_progress_or_build_work() {
    let invalid = WasmBuildSpec::new(Path::new("."), Path::new("target"), &[], "debug");
    let empty = [LabeledWasmBuildSpec::new("", invalid.clone())];
    let mut progress_events = 0;
    let empty_error = build_wasm_canisters_cached_batch_with_progress(
        &empty,
        WasmBuildProgressConfig::new(),
        |_| progress_events += 1,
    )
    .expect_err("empty label must reject the batch");
    assert_eq!(
        empty_error,
        WasmBuildBatchContractError::EmptyLabel { index: 0 }
    );
    assert_eq!(progress_events, 0);

    let duplicate = [
        LabeledWasmBuildSpec::new("same", invalid.clone()),
        LabeledWasmBuildSpec::new("same", invalid),
    ];
    assert_eq!(
        build_wasm_canisters_cached_batch(&duplicate)
            .expect_err("duplicate labels must reject the batch"),
        WasmBuildBatchContractError::DuplicateLabel {
            label: "same".to_owned(),
            first_index: 0,
            duplicate_index: 1,
        }
    );
}

#[test]
fn batch_progress_retains_the_caller_label() {
    let specs = [LabeledWasmBuildSpec::new(
        "root",
        WasmBuildSpec::new(Path::new("."), Path::new("target"), &[], "debug"),
    )];
    let mut labels = Vec::new();

    let report = build_wasm_canisters_cached_batch_with_progress(
        &specs,
        WasmBuildProgressConfig::new(),
        |event| match event {
            WasmBuildBatchProgressEvent::BuildStarted { label, .. }
            | WasmBuildBatchProgressEvent::BuildProgress { label, .. }
            | WasmBuildBatchProgressEvent::BuildFinished { label, .. }
            | WasmBuildBatchProgressEvent::BuildFailed { label, .. } => labels.push(label),
        },
    )
    .expect("valid progress batch");

    assert!(!report.is_success());
    assert_eq!(labels, ["root", "root"]);
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
    let spec = LabeledWasmBuildSpec::new(
        "fixture",
        WasmBuildSpec::new(Path::new("."), Path::new("exact"), &["fixture"], "debug")
            .with_shared_incremental_target("shared")
            .with_shared_incremental_target_maintenance(maintenance),
    );
    let batch = WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance);

    let report =
        build_wasm_canisters_cached_batch_with_config(&[spec], batch).expect("valid labeled batch");
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(report.entries().len(), 1);
    assert_eq!(failures.len(), 1);
    let failure = failures[0];
    assert_eq!(failure.index(), 0);
    assert_eq!(failure.label(), "fixture");
    assert_eq!(failure.entry_elapsed(), report.entries()[0].entry_elapsed());
    assert_eq!(failure.phase(), WasmBuildFailurePhase::Specification);
    assert!(
        matches!(failure.error(), WasmBuildError::InvalidSpec { message } if message.contains("cannot be combined"))
    );
}
