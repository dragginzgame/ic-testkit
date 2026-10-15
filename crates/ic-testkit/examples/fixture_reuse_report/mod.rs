//! Shared workload report fields; execution, admission and summaries remain caller-owned.

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phases {
    pub(super) wait_ms: f64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) build_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) restore_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) reset_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) readiness_ms: Option<f64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(super) validation_ms: Option<f64>,
    pub(super) acquisition_ms: f64,
}

#[derive(Deserialize, Serialize)]
pub struct Sample {
    pub(super) iteration: usize,
    pub(super) phases: Phases,
    pub(super) body_ms: f64,
    pub(super) release_ms: f64,
    pub(super) total_ms: f64,
}

#[derive(Deserialize, Serialize)]
pub struct WorkerReport {
    pub(super) mode: String,
    pub(super) capacity: usize,
    pub(super) iterations: usize,
    pub(super) workers: usize,
    pub(super) state_bytes_per_canister: u32,
    pub(super) wall_ms: f64,
    pub(super) fixture_wall_ms: f64,
    pub(super) samples: Vec<Sample>,
}
