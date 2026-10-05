//! Validates workload reports at the process boundary and projects summaries.
//! The workload remains the encoder/owner of raw measurements; no data migration.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{
    Error,
    arguments::{Case, Options},
};

const PHASES: [&str; 7] = [
    "wait_ms",
    "build_ms",
    "restore_ms",
    "reset_ms",
    "readiness_ms",
    "validation_ms",
    "acquisition_ms",
];

#[derive(Deserialize)]
struct Sample {
    iteration: usize,
    phases: BTreeMap<String, Option<f64>>,
    body_ms: f64,
    release_ms: f64,
    total_ms: f64,
}

#[derive(Deserialize)]
struct Metrics {
    mode: String,
    capacity: usize,
    iterations: usize,
    workers: usize,
    state_bytes_per_canister: u32,
    wall_ms: f64,
    fixture_wall_ms: f64,
    samples: Vec<Sample>,
}

#[derive(Serialize)]
pub(super) struct MeasuredRun {
    #[serde(flatten)]
    raw: Map<String, Value>,
    #[serde(skip)]
    metrics: Metrics,
    pub(super) sampled_peak_tree_rss_bytes: u64,
    pub(super) peak_tree_process_count: usize,
    pub(super) rss_samples: usize,
}

impl MeasuredRun {
    pub(super) fn from_worker(
        raw: Map<String, Value>,
        options: &Options,
        case: &Case,
        rss: (u64, usize, usize),
    ) -> Result<Self, Error> {
        let metrics: Metrics = serde_json::from_value(Value::Object(raw.clone()))?;
        if metrics.mode != case.mode.as_str()
            || metrics.capacity != case.capacity.get()
            || metrics.iterations != options.iterations.get()
            || metrics.workers != options.workers.get()
            || metrics.state_bytes_per_canister != options.state_bytes
        {
            return Err(Error::InvalidData(
                "worker report does not match requested workload",
            ));
        }
        if metrics.samples.len() != metrics.iterations
            || metrics.samples.is_empty()
            || !metrics.wall_ms.is_finite()
            || metrics.wall_ms <= 0.0
            || !metrics.fixture_wall_ms.is_finite()
            || metrics.fixture_wall_ms < metrics.wall_ms
        {
            return Err(Error::InvalidData("invalid worker task count or wall time"));
        }
        for (iteration, sample) in metrics.samples.iter().enumerate() {
            if sample.iteration != iteration
                || sample.phases.len() != PHASES.len()
                || !PHASES
                    .iter()
                    .all(|phase| sample.phases.contains_key(*phase))
                || sample.phases["wait_ms"].is_none()
                || sample.phases["acquisition_ms"].is_none()
                || sample
                    .phases
                    .values()
                    .flatten()
                    .chain([&sample.body_ms, &sample.release_ms, &sample.total_ms])
                    .any(|value| !value.is_finite() || *value < 0.0)
            {
                return Err(Error::InvalidData(
                    "invalid worker task identity, phases or durations",
                ));
            }
        }
        if [
            "sampled_peak_tree_rss_bytes",
            "peak_tree_process_count",
            "rss_samples",
        ]
        .iter()
        .any(|key| raw.contains_key(*key))
        {
            return Err(Error::InvalidData(
                "worker report contains driver-owned sampling fields",
            ));
        }
        Ok(Self {
            raw,
            metrics,
            sampled_peak_tree_rss_bytes: rss.0,
            peak_tree_process_count: rss.1,
            rss_samples: rss.2,
        })
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Distribution {
    pub(super) count: usize,
    pub(super) mean: f64,
    pub(super) p50: f64,
    pub(super) p95: f64,
}

// Distribution values are approximate floating-point observations, as in the
// original report; sample counts themselves remain exact integers.
#[allow(clippy::cast_precision_loss)]
fn distribution(mut values: Vec<f64>) -> Option<Distribution> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let count = values.len();
    let mean = values.iter().map(|value| value / count as f64).sum();
    let middle = count / 2;
    let p50 = if count.is_multiple_of(2) {
        values[middle].mul_add(0.5, values[middle - 1] * 0.5)
    } else {
        values[middle]
    };
    let p95 = values[count - count / 20 - 1];
    Some(Distribution {
        count,
        mean,
        p50,
        p95,
    })
}

#[derive(Serialize)]
pub(super) struct Summary {
    pub(super) phases_ms: BTreeMap<String, Option<Distribution>>,
    body_ms: Option<Distribution>,
    release_ms: Option<Distribution>,
    iteration_ms: Option<Distribution>,
    pub(super) work_wall_ms: Option<Distribution>,
    pub(super) fixture_wall_ms: Option<Distribution>,
    pub(super) iterations_per_second: Option<Distribution>,
    pub(super) sampled_peak_tree_rss_bytes: Option<Distribution>,
}

#[allow(clippy::cast_precision_loss)]
pub(super) fn summarize(runs: &[MeasuredRun]) -> Summary {
    let samples = runs
        .iter()
        .flat_map(|run| &run.metrics.samples)
        .collect::<Vec<_>>();
    let phases_ms = PHASES
        .iter()
        .map(|phase| {
            (
                (*phase).into(),
                distribution(
                    samples
                        .iter()
                        .filter_map(|sample| sample.phases[*phase])
                        .collect(),
                ),
            )
        })
        .collect();
    Summary {
        phases_ms,
        body_ms: distribution(samples.iter().map(|sample| sample.body_ms).collect()),
        release_ms: distribution(samples.iter().map(|sample| sample.release_ms).collect()),
        iteration_ms: distribution(samples.iter().map(|sample| sample.total_ms).collect()),
        work_wall_ms: distribution(runs.iter().map(|run| run.metrics.wall_ms).collect()),
        fixture_wall_ms: distribution(runs.iter().map(|run| run.metrics.fixture_wall_ms).collect()),
        iterations_per_second: distribution(
            runs.iter()
                .map(|run| run.metrics.iterations as f64 * 1000.0 / run.metrics.wall_ms)
                .collect(),
        ),
        sampled_peak_tree_rss_bytes: distribution(
            runs.iter()
                .map(|run| run.sampled_peak_tree_rss_bytes as f64)
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{MeasuredRun, distribution, summarize};
    use crate::fixture_reuse_driver::{Error, arguments::parse_arguments};
    use serde_json::{Map, Value, json};

    fn worker_report() -> Map<String, Value> {
        json!({
            "mode": "fresh", "capacity": 1, "iterations": 1, "workers": 1, "state_bytes_per_canister": 0,
            "wall_ms": 4.0, "fixture_wall_ms": 5.0, "server_startup_ms": 6.0,
            "samples": [{"iteration": 0, "phases": {"wait_ms": 0.0, "build_ms": 1.0, "restore_ms": null,
                "reset_ms": null, "readiness_ms": null, "validation_ms": 1.0, "acquisition_ms": 2.0},
                "body_ms": 1.0, "release_ms": 1.0, "total_ms": 4.0}],
        }).as_object().unwrap().clone()
    }

    #[test]
    fn summary_preserves_null_phases_raw_data_and_percentiles() {
        let options = parse_arguments(
            [
                "--server",
                "/unused",
                "--modes",
                "fresh",
                "--workers",
                "1",
                "--iterations",
                "1",
                "--state-bytes",
                "0",
            ]
            .map(Into::into),
        )
        .unwrap()
        .unwrap();
        let run = MeasuredRun::from_worker(
            worker_report(),
            &options,
            &options.benchmark_cases()[0],
            (1024, 2, 3),
        )
        .unwrap();
        let encoded = serde_json::to_value(&run).unwrap();
        assert_eq!(encoded["server_startup_ms"], 6.0);
        assert_eq!(encoded["sampled_peak_tree_rss_bytes"], 1024);
        let summary = summarize(&[run]);
        assert!(summary.phases_ms["restore_ms"].is_none());
        assert_eq!(summary.phases_ms["build_ms"].as_ref().unwrap().count, 1);
        let stats = distribution(vec![4.0, 1.0, 3.0, 2.0]).unwrap();
        assert_eq!(stats.mean, 2.5);
        assert_eq!(stats.p50, 2.5);
        assert_eq!(stats.p95, 4.0);
        assert_eq!(
            distribution((1..=20).map(f64::from).collect()).unwrap().p95,
            19.0
        );
    }

    #[test]
    fn worker_boundary_rejects_mismatched_incomplete_and_invalid_reports() {
        let options = parse_arguments(
            [
                "--server",
                "/unused",
                "--modes",
                "fresh",
                "--workers",
                "1",
                "--iterations",
                "1",
                "--state-bytes",
                "0",
            ]
            .map(Into::into),
        )
        .unwrap()
        .unwrap();
        let case = options.benchmark_cases().remove(0);
        for field in [
            "workers",
            "capacity",
            "iterations",
            "state_bytes_per_canister",
        ] {
            let mut raw = worker_report();
            raw.insert(field.into(), json!(2));
            assert!(matches!(
                MeasuredRun::from_worker(raw, &options, &case, (1, 1, 1)),
                Err(Error::InvalidData(_))
            ));
        }
        let mut raw = worker_report();
        raw.insert("samples".into(), json!([]));
        assert!(matches!(
            MeasuredRun::from_worker(raw, &options, &case, (1, 1, 1)),
            Err(Error::InvalidData(_))
        ));
        let mut raw = worker_report();
        raw["samples"][0]["body_ms"] = json!(-1);
        assert!(matches!(
            MeasuredRun::from_worker(raw, &options, &case, (1, 1, 1)),
            Err(Error::InvalidData(_))
        ));
    }
}
