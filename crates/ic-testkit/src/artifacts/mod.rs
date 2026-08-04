//! Host-side artifact discovery, freshness, and Wasm build helpers.
//!
//! These functions keep integration-test artifacts in caller-selected target
//! directories and contain no application-specific package or profile policy.

mod icp;
mod wasm;
mod workspace;

pub use icp::{
    WatchedInputSnapshot, icp_artifact_ready_for_build, icp_artifact_ready_with_snapshot,
};
pub use wasm::{build_wasm_canisters, read_wasm, wasm_artifacts_ready, wasm_path};
pub use workspace::{test_target_dir, workspace_root_for};
