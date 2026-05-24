use ic_cdk::{query, update};
use ic_testkit::performance::Performance;

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
