//! Validates workload reports at the process boundary and projects summaries.
//! The workload remains the encoder/owner of raw measurements; no data migration.

use std::collections::BTreeMap;

use crate::{
    fixture_reuse_report::{Phases, WorkerReport},
    statistics::summarize_samples,
};
use serde::Serialize;

use super::{
    Error,
    arguments::{Case, Options},
};

/// Borrow collected measurements when encoding the final report.
#[derive(Serialize)]
pub(super) struct DriverReport<'a> {
    pub(super) format: &'static str,
    pub(super) provenance: &'a serde_json::Value,
    pub(super) runs: &'a BTreeMap<String, Vec<MeasuredRun>>,
    pub(super) summary: &'a BTreeMap<String, Summary>,
}

#[derive(Serialize)]
pub(super) struct MeasuredRun {
    #[serde(flatten)]
    metrics: WorkerReport,
    pub(super) sampled_peak_tree_rss_bytes: u64,
    pub(super) peak_tree_process_count: usize,
    pub(super) rss_samples: usize,
}

impl MeasuredRun {
    pub(super) fn from_worker(
        metrics: WorkerReport,
        options: &Options,
        case: &Case,
        rss: (u64, usize, usize),
    ) -> Result<Self, Error> {
        if metrics.mode != case.mode
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
        for value in [
            metrics.server_startup_ms,
            metrics.preparation_wall_ms,
            metrics.pool_teardown_ms,
            metrics.server_teardown_ms,
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(Error::InvalidData("invalid worker lifecycle duration"));
            }
        }
        for phases in metrics
            .cold_preparation
            .iter()
            .chain(metrics.samples.iter().map(|sample| &sample.phases))
        {
            if [
                Some(phases.wait_ms),
                phases.build_ms,
                phases.restore_ms,
                phases.reset_ms,
                phases.readiness_ms,
                phases.validation_ms,
                Some(phases.acquisition_ms),
            ]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite() || value < 0.0)
            {
                return Err(Error::InvalidData("invalid worker phase duration"));
            }
        }
        for (iteration, sample) in metrics.samples.iter().enumerate() {
            if sample.iteration != iteration
                || [sample.body_ms, sample.release_ms, sample.total_ms]
                    .into_iter()
                    .any(|value| !value.is_finite() || value < 0.0)
            {
                return Err(Error::InvalidData(
                    "invalid worker task identity, phases or durations",
                ));
            }
        }
        Ok(Self {
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

// Complete floating-point sample statistics belong to this private report owner.
fn distribution(mut values: Vec<f64>) -> Result<Option<Distribution>, Error> {
    Ok(summarize_samples(&mut values, 95, 100)
        .map_err(Error::Statistics)?
        .map(|statistics| Distribution {
            count: statistics.count,
            mean: statistics.mean,
            p50: statistics.median,
            p95: statistics.quantile,
        }))
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
pub(super) fn summarize(runs: &[MeasuredRun]) -> Result<Summary, Error> {
    let samples = || runs.iter().flat_map(|run| &run.metrics.samples);
    let phase = |select: fn(&Phases) -> Option<f64>| {
        distribution(
            samples()
                .filter_map(|sample| select(&sample.phases))
                .collect(),
        )
    };
    let phases_ms = [
        ("wait_ms", phase(|p| Some(p.wait_ms))?),
        ("build_ms", phase(|p| p.build_ms)?),
        ("restore_ms", phase(|p| p.restore_ms)?),
        ("reset_ms", phase(|p| p.reset_ms)?),
        ("readiness_ms", phase(|p| p.readiness_ms)?),
        ("validation_ms", phase(|p| p.validation_ms)?),
        ("acquisition_ms", phase(|p| Some(p.acquisition_ms))?),
    ]
    .into_iter()
    .map(|(name, distribution)| (name.into(), distribution))
    .collect();
    Ok(Summary {
        phases_ms,
        body_ms: distribution(samples().map(|sample| sample.body_ms).collect())?,
        release_ms: distribution(samples().map(|sample| sample.release_ms).collect())?,
        iteration_ms: distribution(samples().map(|sample| sample.total_ms).collect())?,
        work_wall_ms: distribution(runs.iter().map(|run| run.metrics.wall_ms).collect())?,
        fixture_wall_ms: distribution(
            runs.iter().map(|run| run.metrics.fixture_wall_ms).collect(),
        )?,
        iterations_per_second: distribution(
            runs.iter()
                .map(|run| run.metrics.iterations as f64 * 1000.0 / run.metrics.wall_ms)
                .collect(),
        )?,
        sampled_peak_tree_rss_bytes: distribution(
            runs.iter()
                .map(|run| run.sampled_peak_tree_rss_bytes as f64)
                .collect(),
        )?,
    })
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
            "cold_preparation": [], "preparation_wall_ms": 1.0,
            "pool_teardown_ms": 0.5, "server_teardown_ms": 0.5,
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
            serde_json::from_value(Value::Object(worker_report())).unwrap(),
            &options,
            &options.benchmark_cases()[0],
            (1024, 2, 3),
        )
        .unwrap();
        let encoded = serde_json::to_value(&run).unwrap();
        assert_eq!(encoded["server_startup_ms"], 6.0);
        assert_eq!(encoded["sampled_peak_tree_rss_bytes"], 1024);
        let mut second = serde_json::from_value::<crate::fixture_reuse_report::WorkerReport>(
            Value::Object(worker_report()),
        )
        .unwrap();
        second.samples[0].body_ms = 3.0;
        second.samples[0].total_ms = 6.0;
        second.wall_ms = 6.0;
        second.fixture_wall_ms = 7.0;
        let second = MeasuredRun::from_worker(
            second,
            &options,
            &options.benchmark_cases()[0],
            (2048, 2, 3),
        )
        .unwrap();
        let summary = summarize(&[run, second]).unwrap();
        assert!(summary.phases_ms["restore_ms"].is_none());
        assert_eq!(summary.phases_ms["build_ms"].as_ref().unwrap().count, 2);
        assert_eq!(summary.body_ms.as_ref().unwrap().count, 2);
        assert_eq!(summary.body_ms.as_ref().unwrap().mean, 2.0);
        assert_eq!(summary.iteration_ms.as_ref().unwrap().mean, 5.0);
        assert_eq!(summary.work_wall_ms.as_ref().unwrap().mean, 5.0);
        assert_eq!(summary.fixture_wall_ms.as_ref().unwrap().mean, 6.0);
        assert_eq!(
            summary.sampled_peak_tree_rss_bytes.as_ref().unwrap().mean,
            1536.0
        );
        let stats = distribution(vec![4.0, 1.0, 3.0, 2.0]).unwrap().unwrap();
        assert_eq!(stats.mean, 2.5);
        assert_eq!(stats.p50, 2.5);
        assert_eq!(stats.p95, 4.0);
        assert_eq!(
            distribution((1..=20).map(f64::from).collect())
                .unwrap()
                .unwrap()
                .p95,
            19.0
        );
    }

    #[test]
    fn summary_refuses_nonfinite_derived_rates_with_the_statistics_cause() {
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
        let mut metrics: crate::fixture_reuse_report::WorkerReport =
            serde_json::from_value(Value::Object(worker_report())).unwrap();
        metrics.wall_ms = f64::from_bits(1);
        let run =
            MeasuredRun::from_worker(metrics, &options, &options.benchmark_cases()[0], (1, 1, 1))
                .unwrap();
        let Err(error) = summarize(&[run]) else {
            panic!("infinite derived rate must refuse");
        };
        assert!(matches!(
            error,
            Error::Statistics(crate::statistics::SampleStatisticsError::NonFiniteSample { .. })
        ));
        assert!(
            std::error::Error::source(&error)
                .unwrap()
                .is::<crate::statistics::SampleStatisticsError>()
        );
        assert!(matches!(
            distribution(vec![f64::MAX; 3]),
            Err(Error::Statistics(
                crate::statistics::SampleStatisticsError::NonFiniteMean
            ))
        ));
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
        let admit = |raw| {
            let metrics = serde_json::from_value(Value::Object(raw))?;
            MeasuredRun::from_worker(metrics, &options, &case, (1, 1, 1))
        };
        let mut raw = worker_report();
        raw.insert("mode".into(), json!("pooled"));
        assert!(matches!(admit(raw), Err(Error::InvalidData(_))));
        for mode in [json!("unknown"), json!("Fresh"), json!(null), json!(1)] {
            let mut raw = worker_report();
            raw.insert("mode".into(), mode);
            assert!(matches!(admit(raw), Err(Error::Json(_))));
        }
        for field in [
            "workers",
            "capacity",
            "iterations",
            "state_bytes_per_canister",
        ] {
            let mut raw = worker_report();
            raw.insert(field.into(), json!(2));
            assert!(matches!(admit(raw), Err(Error::InvalidData(_))));
        }
        let mut raw = worker_report();
        raw.insert("samples".into(), json!([]));
        assert!(matches!(admit(raw), Err(Error::InvalidData(_))));
        for phase in [
            "wait_ms",
            "build_ms",
            "restore_ms",
            "reset_ms",
            "readiness_ms",
            "validation_ms",
            "acquisition_ms",
        ] {
            let mut raw = worker_report();
            raw["samples"][0]["phases"]
                .as_object_mut()
                .unwrap()
                .remove(phase);
            assert!(admit(raw).is_err());
        }
        let mut raw = worker_report();
        raw["samples"][0]["phases"]["unexpected"] = json!(0);
        assert!(admit(raw).is_err());
        let mut raw = worker_report();
        raw["samples"][0]["phases"]["wait_ms"] = Value::Null;
        assert!(admit(raw).is_err());
        let mut raw = worker_report();
        raw.insert("rss_samples".into(), json!(1));
        assert!(matches!(admit(raw), Err(Error::Json(_))));
        for field in [
            "server_startup_ms",
            "preparation_wall_ms",
            "pool_teardown_ms",
            "server_teardown_ms",
        ] {
            let mut raw = worker_report();
            raw.remove(field);
            assert!(matches!(admit(raw), Err(Error::Json(_))));
            let mut raw = worker_report();
            raw.insert(field.into(), json!(-1));
            assert!(matches!(admit(raw), Err(Error::InvalidData(_))));
        }
        let mut raw = worker_report();
        raw.remove("cold_preparation");
        assert!(matches!(admit(raw), Err(Error::Json(_))));
        let mut raw = worker_report();
        raw["cold_preparation"] = json!([raw["samples"][0]["phases"]]);
        raw["cold_preparation"][0]["build_ms"] = json!(-1);
        assert!(matches!(admit(raw), Err(Error::InvalidData(_))));
        let mut raw = worker_report();
        raw["samples"][0]["unexpected"] = json!(0);
        assert!(matches!(admit(raw), Err(Error::Json(_))));
        let mut raw = worker_report();
        raw["samples"][0]["body_ms"] = json!(-1);
        assert!(matches!(admit(raw), Err(Error::InvalidData(_))));
    }
}
