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

## Install

```toml
[dev-dependencies]
ic-testkit = "0.1.1"
```

> [!WARNING]
> Do not use - some of this may be hallucinations, our best agents are currently auditing the code.

## Quick Start

Use `PicSerialGuard` when a test owns a PocketIC instance. `Pic` then provides a small Candid-aware wrapper for common calls.

```rust,no_run
use ic_testkit::pic::{acquire_pic_serial_guard, pic};

#[test]
fn calls_a_counter_canister() {
    let _guard = acquire_pic_serial_guard();
    let pic = pic();
    let counter = install_counter(&pic);

    let _: () = pic.update_call(counter, "increment", ()).unwrap();
    let value: u64 = pic.query_call(counter, "get", ()).unwrap();

    assert_eq!(value, 1);
}
```

Use `update_call_as` and `query_call_as` when caller identity matters.

## Installing Wasm

Install a prebuilt wasm into a fresh PocketIC instance:

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

For an existing `Pic`, use `create_and_install_with_args` or `try_create_and_install_with_args`. If PocketIC reports install-code rate limiting, `retry_install_code_ok` retries while advancing PocketIC time.

## Artifact Helpers

Build wasm packages into a dedicated target directory and check expected artifacts:

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

There are also helpers for reading wasm files and checking generated `.icp` artifacts against watched inputs.

## Benchmark Reports

`ic_testkit::benchmark` turns compact canister log markers into parsed events, paired spans, aggregate rows, CSV files, and a Markdown summary. The default marker prefix is `ICTK`:

```text
ICTK|<label>:start|<instructions>|<heap_bytes>|<memory_bytes>|<total_allocation>
ICTK|<label>:end|<instructions>|<heap_bytes>|<memory_bytes>|<total_allocation>
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

The report writer emits CSV artifacts for raw events, spans, aggregates, malformed/unpaired/invalid markers, and comparisons, plus `bench-summary.md` and `metadata.json`. Run helpers create directories such as `reports/runs/2026-05-24T162600Z-a1b2c3d-0001/` and discover compatible previous runs.

## Canister-Side Markers

Call `Performance::measure` around the region under measurement:

```rust,no_run
use ic_testkit::performance::Performance;

Performance::measure("app/myfunc/something:start");
// code under measurement
Performance::measure("app/myfunc/something:end");
```

The helper prints the compact `ICTK|...` line with the IC CDK call-context instruction counter, Wasm linear memory size, stable memory size, and a `total_allocation` slot. The in-repo `canisters/test/perf_probe` fixture tests this end to end.

## Cached Baselines

For expensive setup, `CachedPicBaseline` can snapshot canisters once and restore them between tests. If the cached PocketIC instance has died, `restore_or_rebuild_cached_pic_baseline` rebuilds instead of reusing a broken instance.

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

- `Pic` Candid query/update helpers with contextual errors
- `PicSerialGuard` for cross-process PocketIC serialization
- generic wasm install helpers, retry helpers, diagnostics, and standalone fixtures
- cached snapshot baselines for expensive test setup
- deterministic fake principals and accounts
- wasm path/build/readiness helpers, including generated `.icp` freshness checks
- compact benchmark marker parsing, aggregation, comparison, and report writing
- canister-side `Performance::measure` marker emission

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
