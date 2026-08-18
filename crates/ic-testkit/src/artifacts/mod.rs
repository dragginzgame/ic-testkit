//! Host-side artifact discovery, freshness, and Wasm build helpers.
//!
//! These functions keep integration-test artifacts in caller-selected target
//! directories and contain no application-specific package or profile policy.

mod batch;
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
    ArtifactCacheBatchFailedEntry, ArtifactCacheBatchFailure, ArtifactCacheBatchMetrics,
    ArtifactCacheBatchReport, build_artifact_caches_batch,
};
pub use wasm::{read_wasm, wasm_artifacts_ready, wasm_path};
pub use wasm_batch::{
    WasmBuildBatchConfig, WasmBuildBatchFailure, WasmBuildBatchMetrics,
    WasmBuildBatchProgressEvent, WasmBuildBatchReport, build_wasm_canisters_cached_batch,
    build_wasm_canisters_cached_batch_with_config,
    build_wasm_canisters_cached_batch_with_config_and_progress,
    build_wasm_canisters_cached_batch_with_progress,
};
pub use wasm_cache::{
    CargoBuildInput, ResolvedCargoBuildInputs, SharedIncrementalTargetInspection,
    SharedIncrementalTargetMaintenance, SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode, SharedIncrementalTargetMaintenanceOutcome,
    SharedIncrementalTargetPrunePolicy, WasmBuildCacheMode, WasmBuildError, WasmBuildOutcome,
    WasmBuildOutputStream, WasmBuildPhase, WasmBuildProgressConfig, WasmBuildProgressEvent,
    WasmBuildProgressOutcome, WasmBuildProgressPhase, WasmBuildRecord, WasmBuildSpec,
    WasmBuildTimings, WasmInputResolutionTimings, build_wasm_canisters_cached,
    build_wasm_canisters_cached_with_progress, inspect_shared_incremental_target,
    maintain_shared_incremental_target, maintain_shared_incremental_target_at_most_every,
    prune_wasm_build_cache, resolve_cargo_build_inputs,
};
pub use workspace::{test_target_dir, workspace_root_for};
