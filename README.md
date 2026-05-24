<div align="center">
# ic-testkit

**A small wrapper and helper layer around `pocket-ic` for Internet Computer canister tests.**

[![Crates.io](https://img.shields.io/crates/v/ic-testkit.svg)](https://crates.io/crates/ic-testkit)
[![Docs.rs](https://docs.rs/ic-testkit/badge.svg)](https://docs.rs/ic-testkit)
[![Downloads](https://img.shields.io/crates/d/ic-testkit.svg)](https://crates.io/crates/ic-testkit)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](Cargo.toml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88.0-blue.svg)](Cargo.toml)
[![Internal Rust](https://img.shields.io/badge/internal%20rust-1.95.0-orange.svg)](README.md#toolchains)
[![Edition](https://img.shields.io/badge/edition-2024-purple.svg)](Cargo.toml)
[![PocketIC](https://img.shields.io/badge/PocketIC-13.0-green.svg)](Cargo.toml)
[![Repository](https://img.shields.io/badge/GitHub-dragginzgame%2Fic--testkit-black.svg)](https://github.com/dragginzgame/ic-testkit)

<p>

<img src="images/cave.png" alt="ic-testkit banner" width="640">
</div>

`ic-testkit` is a wrapper around
[`pocket-ic`](https://crates.io/crates/pocket-ic), the core local IC testing
runtime this crate builds on. It does not replace `pocket-ic`; it adds a small,
opinionated host-side layer for test suites that want typed Candid calls,
install helpers, diagnostics, serialized PocketIC startup, cached baselines,
deterministic fake principals, and wasm artifact utilities.

If you need the underlying IC simulator/runtime itself, start with
[`pocket-ic`](https://crates.io/crates/pocket-ic). Use `ic-testkit` when you
want reusable Rust test harness conveniences on top of it.

It is intentionally application-neutral. Bring your own init payloads, method
names, readiness checks, fixture graph, and product-specific test policy.

## Install

```toml
[dev-dependencies]
ic-testkit = "0.0.1"
```

> [!WARNING]
> Do not use - some of this may be hallucinations, our best agents are currently auditing the code.

## Quick Start

Use `PicSerialGuard` when a test owns a PocketIC instance. It serializes
PocketIC usage across processes, which helps avoid shared server/resource
exhaustion in larger test runs.

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

`Pic` wraps common update/query calls with Candid encoding and decoding. The
error includes the canister id and method name when PocketIC rejects the call.

```rust
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

Use the `_as` variants when the caller matters:

```rust
use candid::Principal;

let caller = ic_testkit::Fake::principal(7);
let ledger_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();

let balance: u128 = pic
    .query_call_as(ledger_id, caller, "balance_of", (caller,))
    .unwrap();
```

## Installing Wasm

For one-off tests, install a prebuilt wasm into a fresh PocketIC instance:

```rust
use ic_testkit::{artifacts, pic::install_prebuilt_canister};

#[test]
fn installs_a_prebuilt_canister() {
    let workspace = artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    let target = artifacts::test_target_dir(&workspace, "pic-wasm");
    let wasm = artifacts::read_wasm(
        &target,
        "counter_canister",
        artifacts::WasmBuildProfile::Release,
    );

    let fixture = install_prebuilt_canister(wasm, vec![]);
    fixture.pic().tick();
}
```

For existing `Pic` instances, use the lower-level helper:

```rust
let canister_id = pic.create_and_install_with_args(
    wasm,
    candid::encode_one(()).unwrap(),
    1_000_000_000_000,
);
```

If PocketIC reports install-code rate limiting, retry while advancing PocketIC
time between attempts:

```rust
use std::time::Duration;

let result = pic.retry_install_code_ok(5, Duration::from_secs(5), || {
    pic.try_create_and_install_with_args(wasm.clone(), vec![], 1_000_000_000_000)
        .map_err(|err| err.to_string())
});
```

## Artifact Helpers

Build wasm packages into a dedicated target directory:

```rust
use ic_testkit::artifacts::{self, WasmBuildProfile};

let workspace = artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"));
let target = artifacts::test_target_dir(&workspace, "pic-wasm");

artifacts::build_wasm_canisters(
    &workspace,
    &target,
    &["counter_canister"],
    WasmBuildProfile::Release,
    &[],
);

assert!(artifacts::wasm_artifacts_ready(
    &target,
    &["counter_canister"],
    WasmBuildProfile::Release,
));
```

Check generated `.icp` artifacts against watched inputs:

```rust
let ready = artifacts::icp_artifact_ready_for_build(
    &workspace,
    ".icp/local/canisters/counter/counter.wasm.gz",
    &["Cargo.toml", "src"],
);
```

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

## Cached Baselines

For expensive multi-canister setup, `CachedPicBaseline` can snapshot canisters
once and restore them between tests. If the cached PocketIC instance has died,
`restore_or_rebuild_cached_pic_baseline` rebuilds instead of reusing a broken
instance.

Use this when setup time dominates the test and the fixture can be restored from
PocketIC snapshots. Keep application-specific topology and readiness logic in
your own test harness.

## What This Adds Over `pocket-ic`

- `Pic`, a narrow wrapper for the PocketIC operations used by this crate
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

## Boundaries

This crate does not define application init payloads, endpoint names, role
models, readiness polling, canister graph topology, attestation policy, or broad
self-test orchestration. Those belong in the application or framework that owns
the canisters being tested.

## Toolchains

- MSRV: Rust 1.88
- internal build/test lane: Rust 1.95

## Local Checks

```sh
make test
cargo +1.88.0 check
cargo publish --dry-run
```
