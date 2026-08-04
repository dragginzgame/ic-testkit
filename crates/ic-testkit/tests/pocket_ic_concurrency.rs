use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use candid::Principal;
use ic_testkit::pic::{
    CachedPocketIcBaseline, InstallSpec, PocketIc, PocketIcBuilder, StandaloneCanisterFixture,
    restore_or_rebuild_cached_pocket_ic_baseline,
};

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

static BASELINE_A: Mutex<Option<CachedPocketIcBaseline<()>>> = Mutex::new(None);
static BASELINE_B: Mutex<Option<CachedPocketIcBaseline<()>>> = Mutex::new(None);

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
        pocket_ic::PocketIcBuilder::new()
            .with_application_subnet()
            .build()
    });
}

#[test]
fn standalone_accepts_a_caller_built_instance_and_preserves_it_in_parts() {
    let caller_built = PocketIcBuilder::new()
        .with_application_subnet()
        .with_ii_subnet()
        .build();
    let fixture = StandaloneCanisterFixture::install(
        caller_built,
        InstallSpec::new(EMPTY_WASM.to_vec(), vec![], 0),
    );
    let (pocket_ic, canister_id) = fixture.into_parts();

    pocket_ic
        .canister_status(canister_id, None)
        .expect("PocketIC returned by into_parts should remain usable");
}

#[test]
fn cached_baseline_guards_are_scoped_to_their_own_slots() {
    let (baseline_a, cache_hit) = restore_or_rebuild_cached_pocket_ic_baseline(
        &BASELINE_A,
        build_empty_cached_baseline,
        |_| {},
    );
    assert!(!cache_hit, "baseline A should be built for this test");

    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let worker = thread::spawn(move || {
        let fresh = PocketIc::new();
        let canister_id = fresh.create_canister();
        fresh
            .canister_status(canister_id, None)
            .expect("fresh instance should remain usable beside baseline A");

        let (_baseline_b, cache_hit) = restore_or_rebuild_cached_pocket_ic_baseline(
            &BASELINE_B,
            build_empty_cached_baseline,
            |_| {},
        );
        assert!(!cache_hit, "baseline B should have an independent slot");

        if ready_tx.send(()).is_ok() {
            let _ = release_rx.recv();
        }
    });

    let result = ready_rx
        .recv_timeout(READY_TIMEOUT)
        .map_err(|err| format!("fresh instance or independent baseline was blocked: {err}"));

    // Release the retained baseline and cancellation channel before joining so
    // an accidentally introduced shared lock can unwind instead of hanging.
    drop(baseline_a);
    drop(release_tx);
    let join_result = worker.join();

    BASELINE_A
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    BASELINE_B
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();

    if let Err(message) = result {
        panic!("{message}");
    }
    join_result.expect("cached-baseline concurrency worker should exit cleanly");
}

fn build_empty_cached_baseline() -> CachedPocketIcBaseline<()> {
    CachedPocketIcBaseline::capture(
        PocketIc::new(),
        Principal::anonymous(),
        std::iter::empty::<Principal>(),
        (),
    )
    .expect("empty cached baseline should capture")
}

fn assert_two_instances_overlap<F>(build: F)
where
    F: Fn() -> PocketIc + Send + Sync + 'static,
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
    F: Fn() -> PocketIc,
{
    if start.recv().is_err() {
        return;
    }

    let pocket_ic = build();
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
                drop(pocket_ic);
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
