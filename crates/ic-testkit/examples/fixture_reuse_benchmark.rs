//! Workload for the fixture_reuse_benchmark_driver example; not an ordinary test.
//! See docs/fixture-reuse-benchmark.md for the workload and measurement boundaries.

mod fixture_reuse_protocol;
mod fixture_reuse_report;
use fixture_reuse_protocol::{Mode, WorkerArguments};
use std::{
    error::Error,
    fmt, fs, io,
    num::NonZeroUsize,
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use candid::Principal;
use fixture_reuse_report::{Phases, Sample, WorkerReport};
use ic_testkit::pic::{
    BaselinePoolOutcome, BaselinePoolTimings, BaselinePreparationStage, CachedPocketIcBaseline,
    CachedPocketIcBaselinePool, CandidCallExt, CanisterRestoreReceipt, CanisterSnapshotTarget,
    CycleResetPolicy, FailureDisposition, FixtureRecipeId, PocketIc, PocketIcBaselineRecipe,
    PocketIcBuilder, PocketIcInstance, PocketIcStartupConfig, PreparedBaseline, ReadinessReceipt,
    ResetReceipt, ResetRequirements, SnapshotRestoreFunding, ValidationReceipt,
};
use serde::Serialize;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct Recipe {
    id: FixtureRecipeId,
    requirements: ResetRequirements,
    server_url: String,
    wasm: Arc<[u8]>,
    state_bytes: u32,
}

#[derive(Debug)]
struct RecipeError(Box<dyn Error + Send + Sync>);

impl fmt::Display for RecipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for RecipeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

fn recipe_error(error: impl Error + Send + Sync + 'static) -> RecipeError {
    RecipeError(Box::new(error))
}

impl Recipe {
    // Fresh construction intentionally does not pay for snapshot capture.
    fn create(
        &self,
    ) -> std::result::Result<(PocketIcInstance<PocketIc>, [Principal; 2]), RecipeError> {
        let pic = PocketIcStartupConfig::connect(&self.server_url, TIMEOUT)
            .build_instance(
                PocketIcBuilder::new()
                    .with_application_subnet()
                    .with_max_request_time_ms(Some(30_000)),
            )
            .map_err(recipe_error)?;
        let ids = [
            pic.instance.create_canister(),
            pic.instance.create_canister(),
        ];
        for id in ids {
            pic.instance.add_cycles(id, 5_000_000_000_000);
            pic.instance
                .install_canister(id, self.wasm.to_vec(), vec![], None);
            pic.instance
                .update_candid::<(), _>(id, "fixture_seed_state", (self.state_bytes,))
                .map_err(recipe_error)?;
        }
        Ok((pic, ids))
    }

    fn validate_state(
        &self,
        pic: &PocketIc,
        ids: &[Principal; 2],
        mutated: bool,
    ) -> std::result::Result<(), RecipeError> {
        let first = (self.state_bytes > 0).then_some(if mutated { 255 } else { 42 });
        let expected = (u64::from(mutated), u64::from(self.state_bytes), first);
        for id in ids {
            let state: (u64, u64, Option<u8>) = pic
                .query_candid(*id, "fixture_state", ())
                .map_err(recipe_error)?;
            if state != expected {
                return Err(recipe_error(io::Error::other(format!(
                    "fixture state mismatch: expected {expected:?}, got {state:?}"
                ))));
            }
        }
        Ok(())
    }

    fn exercise(&self, pic: &PocketIc, ids: &[Principal; 2]) -> Result<()> {
        for id in ids {
            pic.update_candid::<(), _>(*id, "fixture_mutate_state", ())?;
        }
        self.validate_state(pic, ids, true)?;
        Ok(())
    }
}

impl PocketIcBaselineRecipe for Recipe {
    type Metadata = [Principal; 2];
    type Error = RecipeError;

    fn id(&self) -> &FixtureRecipeId {
        &self.id
    }

    fn reset_requirements(&self) -> &ResetRequirements {
        &self.requirements
    }

    fn build(&self) -> std::result::Result<CachedPocketIcBaseline<Self::Metadata>, Self::Error> {
        let (pic, ids) = self.create()?;
        CachedPocketIcBaseline::capture(
            pic,
            ids.map(|id| CanisterSnapshotTarget::new(id, None)),
            ids,
        )
        .map_err(recipe_error)
    }

    fn restore_canisters(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> std::result::Result<CanisterRestoreReceipt, Self::Error> {
        baseline
            .restore(SnapshotRestoreFunding::Preserve)
            .map_err(recipe_error)?;
        CanisterRestoreReceipt::try_from_baseline(baseline, CycleResetPolicy::PreserveCurrent)
            .map_err(recipe_error)
    }

    fn reset_non_snapshot_state(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> std::result::Result<ResetReceipt, Self::Error> {
        // This workload mutates only captured canister state. Time and cycle
        // balances are deliberately not asserted to rewind.
        Ok(ResetReceipt::empty())
    }

    fn drive_to_readiness(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> std::result::Result<ReadinessReceipt, Self::Error> {
        ReadinessReceipt::try_new("synchronous-calls-complete").map_err(recipe_error)
    }

    fn validate(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
        _preparation: &PreparedBaseline,
    ) -> std::result::Result<ValidationReceipt, Self::Error> {
        self.validate_state(baseline.pocket_ic(), baseline.metadata(), false)?;
        ValidationReceipt::try_new(self.id.clone(), "seeded-state-restored").map_err(recipe_error)
    }

    fn classify_failure(
        &self,
        _stage: BaselinePreparationStage,
        _error: &Self::Error,
    ) -> FailureDisposition {
        // Recovery would conceal a failed measurement. Stop this benchmark.
        FailureDisposition::Fatal
    }
}

impl From<BaselinePoolTimings> for Phases {
    fn from(timings: BaselinePoolTimings) -> Self {
        Self {
            wait_ms: ms(timings.wait()),
            build_ms: timings.build().map(ms),
            restore_ms: timings.restore().map(ms),
            reset_ms: timings.reset().map(ms),
            readiness_ms: timings.readiness().map(ms),
            validation_ms: timings.validation().map(ms),
            acquisition_ms: ms(timings.total()),
        }
    }
}

#[derive(Serialize)]
struct WorkloadReport {
    #[serde(flatten)]
    measurements: WorkerReport,
    server_startup_ms: f64,
    cold_preparation: Vec<Phases>,
    preparation_wall_ms: f64,
    pool_teardown_ms: f64,
    server_teardown_ms: f64,
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn fresh_sample(recipe: &Recipe, iteration: usize) -> Result<Sample> {
    let started = Instant::now();
    let (pic, ids) = recipe.create()?;
    let build_ms = Some(ms(started.elapsed()));
    let validation_started = Instant::now();
    recipe.validate_state(&pic.instance, &ids, false)?;
    let phases = Phases {
        build_ms,
        validation_ms: Some(ms(validation_started.elapsed())),
        acquisition_ms: ms(started.elapsed()),
        ..Phases::default()
    };
    let body = Instant::now();
    recipe.exercise(&pic.instance, &ids)?;
    let body_ms = ms(body.elapsed());
    let release = Instant::now();
    drop(pic);
    Ok(Sample {
        iteration,
        phases,
        body_ms,
        release_ms: ms(release.elapsed()),
        total_ms: ms(started.elapsed()),
    })
}

fn pooled_sample(
    pool: &CachedPocketIcBaselinePool<Recipe>,
    recipe: &Recipe,
    iteration: usize,
) -> Result<Sample> {
    let started = Instant::now();
    let (fixture, outcome) = pool.acquire()?;
    if !matches!(outcome, BaselinePoolOutcome::Restored { .. }) {
        return Err(io::Error::other("measured pooled acquisition was not a warm restore").into());
    }
    let phases = outcome.timings().into();
    let body = Instant::now();
    recipe.exercise(fixture.pocket_ic(), fixture.metadata())?;
    let body_ms = ms(body.elapsed());
    let release = Instant::now();
    drop(fixture);
    Ok(Sample {
        iteration,
        phases,
        body_ms,
        release_ms: ms(release.elapsed()),
        total_ms: ms(started.elapsed()),
    })
}

fn run_tasks(
    pool: &CachedPocketIcBaselinePool<Recipe>,
    recipe: &Recipe,
    mode: Mode,
    iterations: NonZeroUsize,
    workers: NonZeroUsize,
) -> Result<(Vec<Sample>, f64)> {
    let barrier = Barrier::new(workers.get() + 1);
    thread::scope(|scope| -> Result<_> {
        let handles = (0..workers.get())
            .map(|worker| {
                let barrier = &barrier;
                scope.spawn(move || -> Result<Vec<Sample>> {
                    barrier.wait();
                    (worker..iterations.get())
                        .step_by(workers.get())
                        .map(|iteration| {
                            if mode == Mode::Fresh {
                                fresh_sample(recipe, iteration)
                            } else {
                                pooled_sample(pool, recipe, iteration)
                            }
                        })
                        .collect()
                })
            })
            .collect::<Vec<_>>();
        let started = Instant::now();
        barrier.wait();
        let mut samples = Vec::new();
        for handle in handles {
            samples.extend(
                handle
                    .join()
                    .map_err(|_| io::Error::other("benchmark worker panicked"))??,
            );
        }
        Ok((samples, ms(started.elapsed())))
    })
}

fn main() -> Result<()> {
    let WorkerArguments {
        mode,
        capacity,
        iterations,
        workers,
        state_bytes,
        wasm,
        server,
    } = WorkerArguments::try_from(std::env::args_os().skip(1).collect::<Vec<_>>())?;
    let startup = Instant::now();
    let server = PocketIcStartupConfig::spawn(&server, TIMEOUT).start_managed_server()?;
    let server_startup_ms = ms(startup.elapsed());
    let recipe = Recipe {
        id: FixtureRecipeId::try_new("benchmark/two-stateful-canisters/v1")?,
        requirements: ResetRequirements::try_new(CycleResetPolicy::PreserveCurrent, [])?,
        server_url: server.url().to_owned(),
        wasm: fs::read(&wasm)?.into(),
        state_bytes,
    };
    let pool = CachedPocketIcBaselinePool::new(capacity, recipe.clone());
    let fixture_started = Instant::now();
    let preparation = Instant::now();
    let mut cold = Vec::new();
    if mode == Mode::Pooled {
        // Hold every lease until all slots exist; do not accidentally measure
        // an initial cold build as a warm acquisition.
        let mut held = Vec::new();
        for _ in 0..capacity.get() {
            let (fixture, outcome) = pool.acquire()?;
            if !matches!(outcome, BaselinePoolOutcome::Built { .. }) {
                return Err(io::Error::other("initial slot was not built").into());
            }
            cold.push(Phases::from(outcome.timings()));
            // The first timed restore must prove it undoes a real mutation too.
            recipe.exercise(fixture.pocket_ic(), fixture.metadata())?;
            held.push(fixture);
        }
        drop(held);
    }
    let preparation_wall_ms = ms(preparation.elapsed());
    let (mut samples, wall_ms) = run_tasks(&pool, &recipe, mode, iterations, workers)?;
    samples.sort_by_key(|sample| sample.iteration);
    let teardown = Instant::now();
    drop(pool);
    let pool_teardown_ms = ms(teardown.elapsed());
    let fixture_wall_ms = ms(fixture_started.elapsed());
    let teardown = Instant::now();
    drop(server);
    let server_teardown_ms = ms(teardown.elapsed());
    let report = WorkloadReport {
        measurements: WorkerReport {
            mode: mode.as_str().into(),
            capacity: if mode == Mode::Fresh {
                workers.get()
            } else {
                capacity.get()
            },
            iterations: iterations.get(),
            workers: workers.get(),
            state_bytes_per_canister: state_bytes,
            wall_ms,
            fixture_wall_ms,
            samples,
        },
        server_startup_ms,
        cold_preparation: cold,
        preparation_wall_ms,
        pool_teardown_ms,
        server_teardown_ms,
    };
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
