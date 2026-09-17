//! Host-side artifact discovery, freshness, and Wasm build helpers.
//!
//! These functions keep integration-test artifacts in caller-selected target
//! directories and contain no application-specific package or profile policy.
//! Independent Wasm batches remain sequential; [`WasmBuildSession`] adds
//! explicit cross-call resolution reuse, while [`WasmBuildInputSnapshot`]
//! prepares a fixed specification set for concurrent readers. Both require a
//! caller-held source lease.
//!
//! Successful Wasm and generic artifact records retain their read-only exact
//! paths against cache replacement and pruning. Retain the record (or a clone)
//! through consumption; paths and artifact descriptors alone carry no ownership.
//! Batch reports own their successful records even when later entries fail.
//!
//! ```no_run
//! use ic_testkit::artifacts::{
//!     ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec,
//!     WasmBuildSpec, build_wasm_canisters_cached, prepare_artifact_cache,
//! };
//! use std::path::Path;
//!
//! fn build_deployable(spec: &WasmBuildSpec, post_cache: &Path, destination: &Path)
//!     -> Result<Vec<u8>, Box<dyn std::error::Error>>
//! {
//!     let wasm = build_wasm_canisters_cached(spec)?;
//!     let input = &wasm.record().artifacts()[0];
//!     let post_spec = ArtifactCacheSpec::new(post_cache, "deploy", "copy/v1")
//!         .with_input("wasm", input)
//!         .with_output("deploy", destination);
//!     let deploy = match prepare_artifact_cache(&post_spec)? {
//!         ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
//!         ArtifactCachePreparation::Build(transaction) => {
//!             // A real post-link recipe can write transformed bytes to
//!             // transaction.output_path("deploy") instead of copying.
//!             transaction.import_output("deploy", input)?;
//!             transaction.commit()?
//!         }
//!     };
//!     drop(wasm); // The post-link transaction has finished consuming the input.
//!     let bytes = std::fs::read(deploy.record().artifacts()[0].path())?;
//!     drop(deploy); // Reading is complete; maintenance may now reclaim it.
//!     Ok(bytes)
//! }
//! ```

mod cache_fs;
mod digest;
mod icp;
mod tool;
mod transaction;
mod transaction_batch;
mod wasm;
mod wasm_batch;
mod wasm_cache;
mod workspace;

#[cfg(test)]
mod test_support;

pub use cache_fs::{ArtifactCacheMaintenance, ArtifactCachePrunePolicy, ArtifactCachePruneReport};
pub use digest::InputDigest;
pub use icp::{
    WatchedInputSnapshot, icp_artifact_ready_for_build, icp_artifact_ready_with_snapshot,
};
pub use tool::resolve_executable;
pub use transaction::{
    ArtifactBuildTransaction, ArtifactCacheArtifact, ArtifactCacheError, ArtifactCacheOutcome,
    ArtifactCachePreparation, ArtifactCacheRecord, ArtifactCacheSpec, ArtifactCacheTimings,
    ArtifactOutputValidation, prepare_artifact_cache, prune_artifact_cache,
};
pub use transaction_batch::{
    ArtifactCacheBatchContractError, ArtifactCacheBatchEntry, ArtifactCacheBatchFailedEntry,
    ArtifactCacheBatchFailure, ArtifactCacheBatchFailurePhase, ArtifactCacheBatchFailureTimings,
    ArtifactCacheBatchMetrics, ArtifactCacheBatchOutcomeEntry, ArtifactCacheBatchReport,
    LabeledArtifactCacheSpec, build_artifact_caches_batch,
};
pub use wasm::{read_wasm, wasm_artifacts_ready, wasm_path};
pub use wasm_batch::{
    LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildBatchContractError, WasmBuildBatchEntry,
    WasmBuildBatchFailure, WasmBuildBatchMaintenanceEntry, WasmBuildBatchMetrics,
    WasmBuildBatchOutcomeEntry, WasmBuildBatchProgressEvent, WasmBuildBatchReport,
    WasmBuildFailureDetails, WasmBuildInputSnapshot, WasmBuildInputSnapshotMetrics,
    WasmBuildSession, WasmBuildSessionMetrics, build_wasm_canisters_cached_batch,
    build_wasm_canisters_cached_batch_with_config,
    build_wasm_canisters_cached_batch_with_config_and_progress,
    build_wasm_canisters_cached_batch_with_progress,
};
pub use wasm_cache::{
    CargoBuildInput, ResolvedCargoBuildInputs, SharedIncrementalTargetInspection,
    SharedIncrementalTargetMaintenance, SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode, SharedIncrementalTargetMaintenanceOutcome,
    SharedIncrementalTargetPrunePolicy, WasmBuildCacheMode, WasmBuildError, WasmBuildFailurePhase,
    WasmBuildFailureTimings, WasmBuildOutcome, WasmBuildOutputStream, WasmBuildPhase,
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildProgressOutcome,
    WasmBuildProgressPhase, WasmBuildRecord, WasmBuildSpec, WasmBuildTimings,
    WasmInputResolutionTimings, build_wasm_canisters_cached,
    build_wasm_canisters_cached_with_progress, inspect_shared_incremental_target,
    maintain_shared_incremental_target, maintain_shared_incremental_target_at_most_every,
    prune_wasm_build_cache, resolve_cargo_build_inputs,
};
pub use workspace::{test_target_dir, workspace_root_for};
