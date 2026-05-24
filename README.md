# ic-testkit

<p align="center">
  <a href="https://crates.io/crates/ic-testkit"><img src="https://img.shields.io/crates/v/ic-testkit.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/ic-testkit"><img src="https://docs.rs/ic-testkit/badge.svg" alt="Docs.rs"></a>
  <a href="https://crates.io/crates/ic-testkit"><img src="https://img.shields.io/crates/d/ic-testkit.svg" alt="Downloads"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/MSRV-1.88.0-blue.svg" alt="MSRV"></a>
  <a href="README.md#toolchains"><img src="https://img.shields.io/badge/internal%20rust-1.95.0-orange.svg" alt="Internal Rust"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/edition-2024-purple.svg" alt="Rust edition"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/PocketIC-13.0-green.svg" alt="PocketIC"></a>
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://img.shields.io/badge/GitHub-dragginzgame%2Fic--testkit-black.svg" alt="Repository"></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-testkit/main/images/cave.png" alt="ic-testkit banner" width="640">
</p>

`ic-testkit` is a small wrapper and helper layer around [`pocket-ic`](https://crates.io/crates/pocket-ic), the local Internet Computer testing runtime this crate stands on. It does not replace `pocket-ic`; it adds reusable Rust test-harness conveniences on top of it.

Use `pocket-ic` directly when you want the underlying simulator/runtime API. Use `ic-testkit` when you want typed Candid calls, install helpers, serialized PocketIC startup, cached baselines, deterministic fake principals, wasm artifact utilities, and compact benchmark reporting.

`ic-testkit` is intentionally application-neutral. Bring your own init payloads, method names, canister graph, readiness checks, labels, thresholds, and CI policy.

## Install

```toml
[dev-dependencies]
ic-testkit = "0.1.1"
```

> [!WARNING]
> Do not use - some of this may be hallucinations, our best agents are currently auditing the code.

## Quick Start

Use `PicSerialGuard` when a test owns a PocketIC instance. It serializes PocketIC usage across processes, which helps avoid shared server/resource exhaustion in larger test runs.

```rust
use ic_testkit::pic::{acquire_pic_serial_guard, pic};

#[test]
fn starts_a_pic_instance() {
    let _guard = acquire_pic_serial_guard();
    let pic = pic();

    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 1_000_000_000_000);
    pic.tick();
}
```

## Calling Canisters

`Pic` wraps common update/query calls with Candid encoding and decoding. Rejections include the canister id and method name.

```rust,no_run
use ic_testkit::pic::{acquire_pic_serial_guard, pic};

#[test]
fn calls_a_counter_canister() {
    let _guard = acquire_pic_serial_guard();
    let pic = pic();
    // Supply this from your own harness. It should install a wasm module and
    // return the installed canister id.
    let counter = install_counter(&pic);

    let _: () = pic.update_call(counter, "increment", ()).unwrap();
    let value: u64 = pic.query_call(counter, "get", ()).unwrap();

    assert_eq!(value, 1);
}
```

Use the `_as` variants when caller identity matters:

```rust,no_run
let caller = ic_testkit::Fake::principal(7);
let ledger_id = ic_testkit::Fake::principal(100);

let balance: u128 = pic
    .query_call_as(ledger_id, caller, "balance_of", (caller,))
    .unwrap();
```

## Installing Wasm

For one-off tests, install a prebuilt wasm into a fresh PocketIC instance:

```rust,no_run
use ic_testkit::{artifacts, pic::install_prebuilt_canister};

#[test]
fn installs_a_prebuilt_canister() {
    let workspace = artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    let target = artifacts::test_target_dir(&workspace, "pic-wasm");
    let wasm = artifacts::read_wasm(&target, "counter_canister", "release");

    let fixture = install_prebuilt_canister(wasm, vec![]);
    fixture.pic().tick();
}
```

For an existing `Pic`, use the lower-level helper:

```rust,no_run
// `pic` is an ic_testkit::pic::Pic from your test setup.
// `wasm` is a Vec<u8> containing the compiled canister wasm.
let canister_id = pic.create_and_install_with_args(
    wasm,
    candid::encode_one(()).unwrap(),
    1_000_000_000_000,
);
```

If PocketIC reports install-code rate limiting, retry while advancing PocketIC time between attempts:

```rust,no_run
use std::time::Duration;

// `pic` is an ic_testkit::pic::Pic from your test setup.
// `wasm` is a Vec<u8> containing the compiled canister wasm.
let result = pic.retry_install_code_ok(5, Duration::from_secs(5), || {
    pic.try_create_and_install_with_args(wasm.clone(), vec![], 1_000_000_000_000)
        .map_err(|err| err.to_string())
});
```

## Artifact Helpers

Build wasm packages into a dedicated target directory:

```rust,no_run
use ic_testkit::artifacts;

let workspace = artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"));
let target = artifacts::test_target_dir(&workspace, "pic-wasm");

artifacts::build_wasm_canisters(
    &workspace,
    &target,
    &["counter_canister"],
    &["--release"],
    &[],
);

assert!(artifacts::wasm_artifacts_ready(
    &target,
    &["counter_canister"],
    "release",
));
```

Check generated `.icp` artifacts against watched inputs:

```rust,no_run
use ic_testkit::artifacts;

let workspace = artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"));

let ready = artifacts::icp_artifact_ready_for_build(
    &workspace,
    ".icp/local/canisters/counter/counter.wasm.gz",
    &["Cargo.toml", "src"],
);
```

## Benchmark Reports

`ic_testkit::benchmark` turns compact canister log markers into parsed events, paired spans, aggregate rows, comparison rows, CSV files, and a Markdown summary.

The default marker prefix is `ICTK`. The compact marker shape is:

```text
ICTK|<label>:start|<instructions>|<heap_bytes>|<memory_bytes>|<total_allocation>
ICTK|<label>:end|<instructions>|<heap_bytes>|<memory_bytes>|<total_allocation>
```

Example:

```text
ICTK|app/myfunc/something:start|100|200|300|400
ICTK|app/myfunc/something:end|150|260|390|430
```

Parse, pair, and aggregate captured logs:

```rust
use ic_testkit::benchmark::{
    aggregate_benchmark_spans, pair_benchmark_spans, parse_benchmark_events,
    BenchmarkParserConfig,
};

let logs = "\
ICTK|app/myfunc/something:start|100|200|300|400
ICTK|app/myfunc/something:end|150|260|390|430
";

let parsed = parse_benchmark_events(logs, &BenchmarkParserConfig::default());
let spans = pair_benchmark_spans(&parsed.events);
let aggregates = aggregate_benchmark_spans(&spans.spans);

assert_eq!(aggregates.rows[0].span_label, "app/myfunc/something");
```

If the harness captures stdout and stderr separately, use `parse_benchmark_events_from_captured_output(stdout, stderr, config)` to preserve the source stream for each marker. Separate streams do not carry global ordering, so this helper parses stdout first, then stderr. If a span can start on one stream and end on the other, capture combined output and use `parse_benchmark_events`.

The report writer emits:

- `raw-events.csv`
- `spans.csv`
- `suite-aggregates.csv`
- `all-aggregates.csv`
- `malformed-markers.csv`
- `unpaired-markers.csv`
- `invalid-spans.csv`
- `bench-summary.md`
- `metadata.json`

The Markdown summary is optimized for quick cost review. Rows include the span label, run count, average instruction cost in billions, memory deltas in human units, and optional percentage changes against the previous run, for example `0.2342B (+34%)` and `+4.0 MB (-23%)`.

Run helpers create directories such as `reports/runs/2026-05-24T162600Z-a1b2c3d-0001/` and discover the latest compatible previous run:

```rust,no_run
use ic_testkit::benchmark::{
    find_latest_previous_run, next_benchmark_run_directory,
};

let run = next_benchmark_run_directory(
    "reports/runs",
    "2026-05-24T162600Z",
    Some("a1b2c3d4e5f6"),
)?;

let previous = find_latest_previous_run(
    "reports/runs",
    &run.directory_name,
    Some("make benchmark"),
)?;
# Ok::<(), std::io::Error>(())
```

## Canister-Side Markers

Call `Performance::measure` around the region under measurement:

```rust,no_run
use ic_testkit::performance::Performance;

Performance::measure("app/myfunc/something:start");
// code under measurement
Performance::measure("app/myfunc/something:end");
```

The helper prints the compact `ICTK|...` line with the IC CDK call-context instruction counter, Wasm linear memory size, stable memory size, and a `total_allocation` slot. `total_allocation` is currently emitted as `0` because the IC CDK does not expose Rust allocator total allocation.

This repository includes a real PocketIC fixture canister under `canisters/test/perf_probe` so the marker path can be tested end to end:

```sh
make test-canisters
```

## Cached Baselines

For expensive multi-canister setup, `CachedPicBaseline` can snapshot canisters once and restore them between tests. If the cached PocketIC instance has died, `restore_or_rebuild_cached_pic_baseline` rebuilds instead of reusing a broken instance.

Use cached baselines when setup time dominates the test and the fixture can be restored from PocketIC snapshots. Keep application-specific topology and readiness logic in your own test harness.

## Deterministic Test Identities

`Fake` gives stable principals and account-like values from numeric seeds:

```rust
use ic_testkit::Fake;

let alice = Fake::principal(1);
let bob = Fake::principal(2);
let account = Fake::account(42);

assert_ne!(alice, bob);
assert_eq!(account.owner, Fake::principal(42));
```

## What This Adds Over `pocket-ic`

- `Pic`, a narrow wrapper for common PocketIC operations
- typed startup errors for common PocketIC launch failures
- `PicSerialGuard` for cross-process PocketIC serialization
- Candid query/update helpers with contextual errors
- generic wasm install helpers and install-code retry helpers
- canister status/log diagnostics
- standalone prebuilt-wasm fixtures
- cached snapshot baselines
- deterministic fake principals and accounts
- wasm path/build/readiness helpers
- watched-input freshness checks for generated `.icp` artifacts
- compact benchmark marker parsing, pairing, aggregation, comparison, and report writing
- optional canister-side `Performance::measure` marker emission
- an in-repo PocketIC fixture canister for testing the benchmark marker path

## Boundaries

This crate does not define application init payloads, endpoint names, role models, readiness polling, canister graph topology, benchmark labels, threshold policy, CI failure policy, or broad self-test orchestration. Those belong in the application or framework that owns the canisters being tested.

## Toolchains

- MSRV: Rust 1.88
- internal build/test lane: Rust 1.95

## Local Checks

```sh
make test
make test-canisters
make build-test-canisters
make release-check
```
