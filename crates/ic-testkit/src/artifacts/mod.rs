//! Host-side artifact discovery, freshness, and Wasm build helpers.
//!
//! These functions keep integration-test artifacts in caller-selected target
//! directories and contain no application-specific package or profile policy.

mod cache_fs;
mod digest;
mod icp;
mod transaction;
mod wasm;
mod wasm_cache;
mod workspace;

#[cfg(test)]
mod test_support;

pub use cache_fs::{ArtifactCacheMaintenance, ArtifactCachePrunePolicy, ArtifactCachePruneReport};
pub use digest::InputDigest;
pub use icp::{
    WatchedInputSnapshot, icp_artifact_ready_for_build, icp_artifact_ready_with_snapshot,
};
pub use transaction::{
    ArtifactBuildTransaction, ArtifactCacheArtifact, ArtifactCacheError, ArtifactCacheOutcome,
    ArtifactCachePreparation, ArtifactCacheRecord, ArtifactCacheSpec, ArtifactCacheTimings,
    ArtifactOutputValidation, prepare_artifact_cache, prune_artifact_cache,
};
pub use wasm::{build_wasm_canisters, read_wasm, wasm_artifacts_ready, wasm_path};
pub use wasm_cache::{
    WasmBuildCacheMaintenance, WasmBuildCachePrunePolicy, WasmBuildCachePruneReport,
    WasmBuildError, WasmBuildOutcome, WasmBuildPhase, WasmBuildRecord, WasmBuildSpec,
    WasmBuildTimings, WasmInputResolutionTimings, build_wasm_canisters_cached,
    prune_wasm_build_cache,
};
pub use workspace::{test_target_dir, workspace_root_for};
