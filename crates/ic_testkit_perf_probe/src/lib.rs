use ic_cdk::{query, update};
use ic_testkit::performance::Performance;
use std::cell::RefCell;

thread_local! {
    static FIXTURE_STATE: RefCell<(u64, Vec<u8>)> = const { RefCell::new((0, Vec::new())) };
}

// Deterministic snapshot-contained state for the opt-in fixture benchmark.
#[update]
fn fixture_seed_state(bytes: u32) {
    FIXTURE_STATE.with_borrow_mut(|state| *state = (0, vec![42; bytes as usize]));
}

#[update]
fn fixture_mutate_state() {
    FIXTURE_STATE.with_borrow_mut(|state| {
        state.0 += 1;
        if let Some(first) = state.1.first_mut() {
            *first = 255;
        }
    });
}

// A named result makes the CDK encode one record instead of expanding a tuple
// into multiple Candid return values. Testkit's typed calls use decode_one.
type FixtureState = (u64, u64, Option<u8>);

#[query]
fn fixture_state() -> FixtureState {
    FIXTURE_STATE.with_borrow(|state| (state.0, state.1.len() as u64, state.1.first().copied()))
}

#[query]
fn ping() -> &'static str {
    "pong"
}

#[update]
fn benchmark_once() -> u64 {
    Performance::measure("probe/benchmark_once:start");

    let mut acc = 0_u64;
    for n in 0..1_000 {
        acc = acc.wrapping_add(n * 3);
    }

    Performance::measure("probe/benchmark_once:end");
    acc
}

#[update]
fn benchmark_start_then_trap() {
    Performance::measure("probe/benchmark_start_then_trap:start");
    ic_cdk::trap("intentional perf probe trap");
}
