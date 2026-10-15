mod support;

use std::{
    num::NonZeroUsize,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use candid::Principal;
use ic_testkit::pic::{
    BaselinePoolError, BaselinePoolOutcome, BaselinePoolPreparationError, BaselinePreparationStage,
    CachedPocketIcBaselinePool, ControllerSnapshotError, FixtureRecipeId, InstallSpec, PocketIc,
    PocketIcBuilder, PocketIcInstance, PocketIcStartupConfig, PocketIcStartupFailure,
    RebuildReason, SnapshotRestoreFunding, StandaloneCanisterFixture, StandaloneCanisterRecipe,
};

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

type SingleCanisterPool =
    CachedPocketIcBaselinePool<StandaloneCanisterRecipe<fn() -> StandaloneCanisterFixture>>;

static STANDALONE_RESTORE_POOL: LazyLock<SingleCanisterPool> = LazyLock::new(|| {
    single_canister_pool(
        NonZeroUsize::new(1).unwrap(),
        build_empty_standalone_fixture as fn() -> _,
    )
});
static STANDALONE_CAPACITY_POOL: LazyLock<SingleCanisterPool> = LazyLock::new(|| {
    single_canister_pool(
        NonZeroUsize::new(1).unwrap(),
        build_empty_standalone_fixture as fn() -> _,
    )
});
static STANDALONE_PANIC_POOL: LazyLock<SingleCanisterPool> = LazyLock::new(|| {
    single_canister_pool(
        NonZeroUsize::new(1).unwrap(),
        build_counted_empty_standalone_fixture as fn() -> _,
    )
});

fn single_canister_pool<B>(
    capacity: NonZeroUsize,
    build: B,
) -> CachedPocketIcBaselinePool<StandaloneCanisterRecipe<B>>
where
    B: Fn() -> StandaloneCanisterFixture + Send + Sync + 'static,
{
    CachedPocketIcBaselinePool::new(
        capacity,
        StandaloneCanisterRecipe::new(
            FixtureRecipeId::try_new("test/single-canister").unwrap(),
            SnapshotRestoreFunding::Preserve,
            build,
        ),
    )
}

static STANDALONE_PANIC_BUILDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
enum Command {
    Initialize,
    Probe,
    Drop,
}

#[derive(Debug)]
enum Event {
    Ready {
        worker: usize,
    },
    Initialized {
        worker: usize,
        canister_id: Principal,
        balance: u128,
    },
    Probed {
        worker: usize,
    },
    Dropped {
        worker: usize,
    },
}

#[test]
fn upstream_supports_two_overlapping_instances() {
    assert_two_instances_overlap(move || {
        support::startup_config()
            .build_instance(pocket_ic::PocketIcBuilder::new().with_application_subnet())
            .expect("build explicitly configured upstream instance")
    });
}

#[test]
fn startup_configuration_returns_a_typed_startup_error() {
    let missing_binary = std::env::temp_dir().join(format!(
        "ic-testkit-missing-pocket-ic-{}",
        std::process::id()
    ));
    let result = PocketIcStartupConfig::spawn(&missing_binary, Duration::from_secs(1))
        .build_instance(PocketIcBuilder::new().with_application_subnet());

    let error = result.err().unwrap();
    assert!(
        matches!(error.failure(), PocketIcStartupFailure::ServerSpawn { server_binary, .. } if server_binary == &missing_binary)
    );
}

#[test]
fn standalone_accepts_a_caller_built_instance_and_preserves_it_in_parts() {
    let caller_built = support::startup_config()
        .build_instance(
            PocketIcBuilder::new()
                .with_application_subnet()
                .with_ii_subnet(),
        )
        .expect("build explicitly configured custom topology");
    let fixture = StandaloneCanisterFixture::install(
        caller_built,
        InstallSpec::new(EMPTY_WASM.to_vec(), vec![], 0),
    );
    let (owned, canister_id) = fixture.into_parts();
    let pocket_ic = &owned.instance;

    pocket_ic
        .canister_status(canister_id, None)
        .expect("PocketIC returned by into_parts should remain usable");
}

#[test]
fn bounded_standalone_pool_restores_and_reuses_one_slot() {
    let (fixture, outcome) = STANDALONE_RESTORE_POOL
        .acquire()
        .expect("first standalone pool fixture should capture");
    assert!(matches!(&outcome, BaselinePoolOutcome::Built { .. }));
    let timings = outcome.timings();
    assert!(timings.build().is_some());
    assert!(timings.restore().is_none());
    let first_instance = fixture.pocket_ic().instance_id();
    let canister_id = *fixture.metadata();
    fixture
        .pocket_ic()
        .uninstall_canister(canister_id, None)
        .expect("test should mutate the leased canister");
    assert!(
        fixture
            .pocket_ic()
            .canister_status(canister_id, None)
            .expect("mutated canister status should remain readable")
            .module_hash
            .is_none(),
        "test mutation should remove the installed module",
    );
    drop(fixture);

    let (fixture, outcome) = STANDALONE_RESTORE_POOL
        .acquire()
        .expect("cached standalone pool fixture should restore");
    assert!(matches!(&outcome, BaselinePoolOutcome::Restored { .. }));
    assert!(matches!(outcome, BaselinePoolOutcome::Restored { .. }));
    let timings = outcome.timings();
    assert!(timings.build().is_none());
    assert!(timings.restore().is_some());
    assert_eq!(fixture.pocket_ic().instance_id(), first_instance);
    assert!(
        fixture
            .pocket_ic()
            .canister_status(canister_id, None)
            .expect("restored canister status should remain readable")
            .module_hash
            .is_some(),
        "snapshot restore should recover the installed module",
    );
    drop(fixture);

    let (fixture, outcome) = STANDALONE_RESTORE_POOL
        .acquire()
        .expect("standalone pool snapshot should remain reusable");
    assert!(
        matches!(outcome, BaselinePoolOutcome::Restored { .. }),
        "third lease should reuse the same pool slot"
    );
    assert_eq!(fixture.pocket_ic().instance_id(), first_instance);
}

#[test]
fn bounded_standalone_pool_allows_capacity_scoped_overlap() {
    let capacity = NonZeroUsize::new(2).unwrap();
    let pool = Arc::new(single_canister_pool(
        capacity,
        build_empty_standalone_fixture,
    ));
    assert_eq!(pool.capacity(), capacity);
    let (first, first_outcome) = pool
        .acquire()
        .expect("first overlapping fixture should capture");
    assert!(
        !matches!(first_outcome, BaselinePoolOutcome::Restored { .. }),
        "first pool slot should be new"
    );

    let (ready_tx, ready_rx) = mpsc::channel();
    let worker_pool = Arc::clone(&pool);
    let worker = thread::spawn(move || {
        let (second, second_outcome) = worker_pool
            .acquire()
            .expect("second overlapping fixture should capture");
        ready_tx
            .send((
                (
                    second.pocket_ic().get_server_url(),
                    second.pocket_ic().instance_id(),
                ),
                second_outcome,
            ))
            .expect("overlap result receiver should remain live");
    });

    let (second_instance, second_outcome) = ready_rx
        .recv_timeout(OPERATION_TIMEOUT)
        .expect("second pool slot should not wait for the first lease");
    assert!(
        !matches!(second_outcome, BaselinePoolOutcome::Restored { .. }),
        "second pool slot should be new"
    );
    assert_ne!(
        (
            first.pocket_ic().get_server_url(),
            first.pocket_ic().instance_id()
        ),
        second_instance
    );

    drop(first);
    worker.join().expect("overlap worker should exit cleanly");
}

#[test]
fn bounded_standalone_pool_waits_when_capacity_is_exhausted() {
    let (first, first_outcome) = STANDALONE_CAPACITY_POOL
        .acquire()
        .expect("first capacity-limited fixture should capture");
    assert!(
        !matches!(first_outcome, BaselinePoolOutcome::Restored { .. }),
        "first pool slot should be new"
    );
    let first_instance = first.pocket_ic().instance_id();

    let (attempting_tx, attempting_rx) = mpsc::channel();
    let (acquired_tx, acquired_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        attempting_tx
            .send(())
            .expect("capacity test coordinator should remain live");
        let (second, outcome) = STANDALONE_CAPACITY_POOL
            .acquire()
            .expect("waiting fixture should restore after release");
        acquired_tx
            .send((second.pocket_ic().instance_id(), outcome))
            .expect("capacity result receiver should remain live");
    });

    attempting_rx
        .recv_timeout(OPERATION_TIMEOUT)
        .expect("worker should begin the capacity-limited acquisition");
    match acquired_rx.recv_timeout(Duration::from_millis(100)) {
        Err(RecvTimeoutError::Timeout) => {}
        Ok(_) => panic!("a second lease must not exceed pool capacity"),
        Err(RecvTimeoutError::Disconnected) => {
            panic!("capacity worker exited before the first lease was released")
        }
    }

    drop(first);
    let (second_instance, outcome) = acquired_rx
        .recv_timeout(OPERATION_TIMEOUT)
        .expect("waiting acquisition should proceed after release");
    assert!(matches!(&outcome, BaselinePoolOutcome::Restored { .. }));
    assert!(
        outcome.timings().wait() >= Duration::from_millis(50),
        "structured timings should include capacity wait: {:?}",
        outcome.timings(),
    );
    assert_eq!(second_instance, first_instance);
    worker.join().expect("capacity worker should exit cleanly");
}

#[test]
fn bounded_standalone_pool_rebuilds_after_a_leased_test_panics() {
    let (fixture, outcome) = STANDALONE_PANIC_POOL
        .acquire()
        .expect("first panic-test fixture should capture");
    assert!(!matches!(outcome, BaselinePoolOutcome::Restored { .. }));
    drop(fixture);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let (_fixture, outcome) = STANDALONE_PANIC_POOL
            .acquire()
            .expect("panic-test fixture should restore");
        assert!(matches!(outcome, BaselinePoolOutcome::Restored { .. }));
        panic!("synthetic pooled-test panic");
    }));
    assert!(panic.is_err(), "the test panic must keep unwinding");

    let (fixture, outcome) = STANDALONE_PANIC_POOL
        .acquire()
        .expect("the invalidated standalone slot should rebuild");
    assert!(matches!(
        &outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::UnwindWhileLeased,
            ..
        }
    ));
    let timings = outcome.timings();
    assert!(timings.stale_teardown().is_some());
    assert!(timings.build().is_some());
    assert_eq!(STANDALONE_PANIC_BUILDS.load(Ordering::SeqCst), 2);
    drop(fixture);
}

#[test]
fn structured_standalone_acquisition_error_retains_build_timings() {
    let pool = single_canister_pool(
        NonZeroUsize::new(1).unwrap(),
        build_deleted_standalone_fixture,
    );
    let Err(error) = pool.acquire() else {
        panic!("capturing a deleted fixture canister must fail");
    };

    let timings = error.timings();
    assert!(timings.build().is_some());
    assert!(timings.restore().is_none());
    assert!(timings.total() >= timings.build().unwrap());
    assert!(matches!(
        error,
        BaselinePoolError::Preparation {
            error: BaselinePoolPreparationError::Recipe {
                stage: BaselinePreparationStage::Build,
                ..
            },
            ..
        }
    ));
}

#[test]
fn failed_standalone_restore_is_timed_and_rebuilt_on_the_next_acquisition() {
    let builds = Arc::new(AtomicUsize::new(0));
    let builder_count = Arc::clone(&builds);
    let pool = single_canister_pool(NonZeroUsize::new(1).unwrap(), move || {
        builder_count.fetch_add(1, Ordering::SeqCst);
        build_empty_standalone_fixture()
    });
    let (fixture, outcome) = pool
        .acquire()
        .expect("standalone fixture should build before restore failure");
    assert!(matches!(outcome, BaselinePoolOutcome::Built { .. }));
    assert_eq!(builds.load(Ordering::SeqCst), 1);
    drop(fixture);
    let (fixture, outcome) = pool
        .acquire()
        .expect("owned builder baseline should restore");
    assert!(matches!(outcome, BaselinePoolOutcome::Restored { .. }));
    assert_eq!(builds.load(Ordering::SeqCst), 1);
    fixture
        .pocket_ic()
        .stop_canister(*fixture.metadata(), None)
        .expect("stop fixture canister before deleting it");
    fixture
        .pocket_ic()
        .delete_canister(*fixture.metadata(), None)
        .expect("delete fixture canister before restore");
    drop(fixture);

    let Err(error) = pool.acquire() else {
        panic!("restoring a deleted fixture canister must fail");
    };
    let timings = error.timings();
    assert!(timings.restore().is_some());
    assert!(timings.build().is_none());
    assert!(timings.stale_teardown().is_some());
    assert!(matches!(
        error,
        BaselinePoolError::Preparation {
            error: BaselinePoolPreparationError::Recipe {
                stage: BaselinePreparationStage::RestoreCanisters,
                source,
            },
            ..
        } if matches!(source.get_ref().and_then(|error| error.downcast_ref::<ControllerSnapshotError>()),
            Some(ControllerSnapshotError::RestoreFailed { .. }))
    ));

    let (fixture, outcome) = pool
        .acquire()
        .expect("partially restored standalone slot should rebuild next");
    assert!(matches!(&outcome, BaselinePoolOutcome::Built { .. }));
    let timings = outcome.timings();
    assert!(timings.stale_teardown().is_none());
    assert!(timings.build().is_some());
    assert_eq!(builds.load(Ordering::SeqCst), 2);
    drop(fixture);
}

#[test]
fn single_canister_recipe_applies_funding_before_warm_restore() {
    let recipe = StandaloneCanisterRecipe::new(
        FixtureRecipeId::try_new("test/funded-canister").unwrap(),
        SnapshotRestoreFunding::TopUpTo {
            minimum_cycles: 1_000_000_000_000_000_000,
        },
        build_empty_standalone_fixture,
    );
    let pool = CachedPocketIcBaselinePool::new(NonZeroUsize::new(1).unwrap(), recipe);
    let (baseline, first) = pool.acquire().unwrap();
    assert!(matches!(first, BaselinePoolOutcome::Built { .. }));
    let instance = baseline.pocket_ic().instance_id();
    let initial_cycles = baseline.pocket_ic().cycle_balance(*baseline.metadata());
    baseline
        .pocket_ic()
        .uninstall_canister(*baseline.metadata(), None)
        .unwrap();
    drop(baseline);

    let (baseline, restored) = pool.acquire().unwrap();
    assert!(matches!(restored, BaselinePoolOutcome::Restored { .. }));
    assert_eq!(baseline.pocket_ic().instance_id(), instance);
    assert!(baseline.pocket_ic().cycle_balance(*baseline.metadata()) > initial_cycles);
    assert!(
        baseline
            .pocket_ic()
            .canister_status(*baseline.metadata(), None)
            .unwrap()
            .module_hash
            .is_some()
    );
}

#[test]
#[ignore = "requires POCKET_IC_BIN and stops a dedicated test-owned server"]
fn standalone_dead_transport_preserves_failed_recovery_and_can_build_again() {
    let server = PocketIcStartupConfig::spawn(
        std::env::var_os("POCKET_IC_BIN")
            .expect("prepare POCKET_IC_BIN for dedicated-server recovery"),
        READY_TIMEOUT,
    )
    .start_managed_server()
    .unwrap();
    let url = server.url().to_owned();
    let builds = Arc::new(AtomicUsize::new(0));
    let builder_count = Arc::clone(&builds);
    let pool = single_canister_pool(NonZeroUsize::new(1).unwrap(), move || {
        match builder_count.fetch_add(1, Ordering::SeqCst) {
            0 => StandaloneCanisterFixture::install(
                PocketIcStartupConfig::connect(&url, READY_TIMEOUT)
                    .build_instance(PocketIcBuilder::new().with_application_subnet())
                    .unwrap(),
                InstallSpec::new(EMPTY_WASM.to_vec(), vec![], 0),
            ),
            1 => build_deleted_standalone_fixture(),
            _ => build_empty_standalone_fixture(),
        }
    });
    drop(pool.acquire().unwrap().0);
    drop(server);

    let Err(BaselinePoolError::RecoveryFailed {
        original,
        rebuild,
        timings,
    }) = pool.acquire()
    else {
        panic!("dead transport and failed capture must retain both causes");
    };
    assert!(ic_testkit::pic::is_dead_pocket_ic_transport_error(
        original.as_ref()
    ));
    assert!(matches!(
        rebuild.as_ref(),
        BaselinePoolPreparationError::Recipe { stage: BaselinePreparationStage::Build, source }
            if matches!(source.get_ref().and_then(|error| error.downcast_ref::<ControllerSnapshotError>()),
                Some(ControllerSnapshotError::CaptureFailed { .. }))
    ));
    assert!(timings.restore().is_some());
    assert!(timings.build().is_some());
    assert!(timings.stale_teardown().is_some());
    assert_eq!(builds.load(Ordering::SeqCst), 2);

    let (fixture, outcome) = pool.acquire().unwrap();
    assert!(matches!(outcome, BaselinePoolOutcome::Built { .. }));
    assert!(outcome.timings().restore().is_none());
    assert!(outcome.timings().stale_teardown().is_none());
    assert_eq!(builds.load(Ordering::SeqCst), 3);
    drop(fixture);
}

fn build_empty_standalone_fixture() -> StandaloneCanisterFixture {
    StandaloneCanisterFixture::install(
        support::pocket_ic(),
        InstallSpec::new(EMPTY_WASM.to_vec(), vec![], 0),
    )
}

fn build_counted_empty_standalone_fixture() -> StandaloneCanisterFixture {
    STANDALONE_PANIC_BUILDS.fetch_add(1, Ordering::SeqCst);
    build_empty_standalone_fixture()
}

fn build_deleted_standalone_fixture() -> StandaloneCanisterFixture {
    let fixture = build_empty_standalone_fixture();
    fixture
        .pocket_ic()
        .stop_canister(fixture.canister_id(), None)
        .expect("stop fixture canister before deleting it");
    fixture
        .pocket_ic()
        .delete_canister(fixture.canister_id(), None)
        .expect("delete fixture canister before snapshot capture");
    fixture
}

fn assert_two_instances_overlap<F>(build: F)
where
    F: Fn() -> PocketIcInstance<PocketIc> + Send + Sync + 'static,
{
    let build = Arc::new(build);
    let (event_tx, event_rx) = mpsc::channel();
    let mut start_txs = Vec::new();
    let mut command_txs = Vec::new();
    let mut workers = Vec::new();

    for worker in 0..2 {
        let (start_tx, start_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();
        let worker_events = event_tx.clone();
        let worker_build = Arc::clone(&build);

        start_txs.push(start_tx);
        command_txs.push(command_tx);
        workers.push(thread::spawn(move || {
            run_worker(
                worker,
                worker_build.as_ref(),
                start_rx,
                command_rx,
                worker_events,
            );
        }));
    }
    drop(event_tx);

    for start in start_txs {
        start.send(()).expect("worker should await start release");
    }

    let result = coordinate_workers(&event_rx, &command_txs);

    // Closing command channels is the cooperative cancellation path. A ready
    // worker drops its retained instance, allowing a peer blocked by an
    // accidentally reintroduced ownership lock to progress and then exit.
    drop(command_txs);

    let join_result = workers
        .into_iter()
        .map(thread::JoinHandle::join)
        .collect::<Vec<_>>();

    if let Err(message) = result {
        panic!("{message}");
    }
    for joined in join_result {
        joined.expect("PocketIC concurrency worker should exit cleanly");
    }
}

fn coordinate_workers(
    events: &Receiver<Event>,
    commands: &[Sender<Command>],
) -> Result<(), String> {
    let ready_deadline = Instant::now() + READY_TIMEOUT;
    let mut ready = [false; 2];
    while !ready.iter().all(|is_ready| *is_ready) {
        let event = recv_until(
            events,
            ready_deadline,
            "both PocketIC instances to be ready",
        )?;
        let Event::Ready { worker } = event else {
            return Err(format!(
                "expected Ready while acquiring instances, got {event:?}"
            ));
        };
        ready[worker] = true;
    }

    for command in commands {
        command
            .send(Command::Initialize)
            .map_err(|_| "worker exited before initialization".to_string())?;
    }

    let mut states = [None, None];
    while states.iter().any(Option::is_none) {
        match recv_operation(events, "both PocketIC instances to initialize")? {
            Event::Initialized {
                worker,
                canister_id,
                balance,
            } => states[worker] = Some((canister_id, balance)),
            event => return Err(format!("expected Initialized, got {event:?}")),
        }
    }

    let (first_id, first_balance) = states[0].expect("worker zero initialized");
    let (second_id, second_balance) = states[1].expect("worker one initialized");
    if first_id != second_id {
        return Err(format!(
            "expected equivalent fresh instances to allocate the same first canister id, got {first_id} and {second_id}"
        ));
    }
    if first_balance == second_balance {
        return Err("independent instances unexpectedly reported identical test state".to_string());
    }

    commands[0]
        .send(Command::Drop)
        .map_err(|_| "first worker exited before drop".to_string())?;
    match recv_operation(events, "the first PocketIC instance to drop")? {
        Event::Dropped { worker: 0 } => {}
        event => return Err(format!("expected worker zero to drop, got {event:?}")),
    }

    commands[1]
        .send(Command::Probe)
        .map_err(|_| "second worker exited before post-drop probe".to_string())?;
    match recv_operation(events, "the surviving PocketIC instance to respond")? {
        Event::Probed { worker: 1 } => {}
        event => return Err(format!("expected worker one probe, got {event:?}")),
    }

    commands[1]
        .send(Command::Drop)
        .map_err(|_| "second worker exited before drop".to_string())?;
    match recv_operation(events, "the second PocketIC instance to drop")? {
        Event::Dropped { worker: 1 } => Ok(()),
        event => Err(format!("expected worker one to drop, got {event:?}")),
    }
}

fn run_worker<F>(
    worker: usize,
    build: &F,
    start: Receiver<()>,
    commands: Receiver<Command>,
    events: Sender<Event>,
) where
    F: Fn() -> PocketIcInstance<PocketIc>,
{
    if start.recv().is_err() {
        return;
    }

    let owned = build();
    let pocket_ic = &owned.instance;
    if events.send(Event::Ready { worker }).is_err() {
        return;
    }

    let mut canister_id = None;
    while let Ok(command) = commands.recv() {
        match command {
            Command::Initialize => {
                let id = pocket_ic.create_canister();
                pocket_ic.install_canister(id, EMPTY_WASM.to_vec(), vec![], None);
                let balance = pocket_ic.add_cycles(id, (worker as u128 + 1) * 1_000_000);
                canister_id = Some(id);
                if events
                    .send(Event::Initialized {
                        worker,
                        canister_id: id,
                        balance,
                    })
                    .is_err()
                {
                    return;
                }
            }
            Command::Probe => {
                let id = canister_id.expect("worker must initialize before probing");
                pocket_ic
                    .canister_status(id, None)
                    .expect("surviving PocketIC instance should remain usable");
                if events.send(Event::Probed { worker }).is_err() {
                    return;
                }
            }
            Command::Drop => {
                drop(owned);
                let _ = events.send(Event::Dropped { worker });
                return;
            }
        }
    }
}

fn recv_until(events: &Receiver<Event>, deadline: Instant, context: &str) -> Result<Event, String> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    match events.recv_timeout(remaining) {
        Ok(event) => Ok(event),
        Err(RecvTimeoutError::Timeout) => Err(format!("timed out waiting for {context}")),
        Err(RecvTimeoutError::Disconnected) => {
            Err(format!("workers disconnected while waiting for {context}"))
        }
    }
}

fn recv_operation(events: &Receiver<Event>, context: &str) -> Result<Event, String> {
    match events.recv_timeout(OPERATION_TIMEOUT) {
        Ok(event) => Ok(event),
        Err(RecvTimeoutError::Timeout) => Err(format!("timed out waiting for {context}")),
        Err(RecvTimeoutError::Disconnected) => {
            Err(format!("workers disconnected while waiting for {context}"))
        }
    }
}
