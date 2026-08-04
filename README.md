# ic-testkit

[PocketIC upstream wishlist](POCKET-IC.md)

<p align="center">
  <a href="https://crates.io/crates/ic-testkit"><img src="https://img.shields.io/crates/v/ic-testkit.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/ic-testkit"><img src="https://docs.rs/ic-testkit/badge.svg" alt="Docs.rs"></a>
  <a href="https://crates.io/crates/ic-testkit"><img src="https://img.shields.io/crates/d/ic-testkit.svg" alt="Downloads"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/MSRV-1.88.0-blue.svg" alt="MSRV"></a>
  <a href="README.md#toolchains"><img src="https://img.shields.io/badge/internal%20rust-1.96.0-orange.svg" alt="Internal Rust"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/edition-2024-purple.svg" alt="Rust edition"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/PocketIC-15.0-green.svg" alt="PocketIC"></a>
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://img.shields.io/badge/GitHub-dragginzgame%2Fic--testkit-black.svg" alt="Repository"></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-testkit/main/images/cave.png" alt="ic-testkit banner" width="640">
</p>

`ic-testkit` is a small helper layer around [`pocket-ic`](https://crates.io/crates/pocket-ic), the local Internet Computer testing runtime this crate stands on. It re-exports PocketIC's native types and adds reusable Rust test-harness conveniences without wrapping or replacing the simulator API.

Use PocketIC's inherent methods for simulator operations. Use `ic-testkit` when you want typed Candid calls, fixture installation with contextual diagnostics, cached baselines, deterministic fake principals, wasm artifact utilities, and compact benchmark reporting.

## Install

```toml
[dev-dependencies]
ic-testkit = "0.2"
```

## Quick Start

Each test normally creates and directly owns one fresh `PocketIc`. Import
`CandidCallExt` for typed Candid calls; all simulator operations remain the
upstream type's inherent methods.

```rust,no_run
use ic_testkit::pic::{CandidCallExt, PocketIc};

#[test]
fn calls_a_counter_canister() {
    let pocket_ic = PocketIc::new();
    let counter = install_counter(&pocket_ic);

    let _: () = pocket_ic.update_candid(counter, "increment", ()).unwrap();
    let value: u64 = pocket_ic.query_candid(counter, "get", ()).unwrap();

    assert_eq!(value, 1);
}
```

Use `update_candid_as` and `query_candid_as` when caller identity matters. In
tests that should fail immediately on rejection, transport, or Candid codec
errors, use `update_candid_or_panic`, `query_candid_or_panic`,
`update_candid_as_or_panic`, or `query_candid_as_or_panic`. These helpers only
unwrap the outer `CandidCallError`; application-level return values such as
`Result<T, E>` are returned unchanged.

When PocketIC rejects a call, `CandidCallError::reject_response()` preserves
the complete upstream `RejectResponse`; rejection is not classified as a
transport failure.

## PocketIC Server Binary

PocketIC remains the authority for server-binary discovery, `POCKET_IC_BIN`,
downloads, and its cache. `ic-testkit` does not maintain a second downloader or
cache policy. Use `PocketIcBuilder::with_server_binary` when a harness needs an
explicit binary.

Use `PocketIc::new()` for the default application subnet. For a custom topology,
configure the re-exported upstream builder and call its native `build()` method:

```rust,no_run
use ic_testkit::pic::PocketIcBuilder;

let pocket_ic = PocketIcBuilder::new()
    .with_application_subnet()
    .with_ii_subnet()
    .build();
```

For benchmark metadata, `ic_testkit::pic::LATEST_SERVER_VERSION` exposes the
server version expected by the PocketIC client and `PocketIc::get_server_url()`
exposes the active endpoint. PocketIC 15 does not expose the resolved server
binary path or digest from a built instance. A benchmark requiring that
provenance should select an explicit path with `with_server_binary`, record and
hash that caller-owned file, and then build the instance. ic-testkit does not
guess the path from environment or cache conventions.

There is no crate-level PocketIC ownership lock. If a heavy E2E target exceeds
CI capacity, tune that target through the test runner, starting conservatively
when needed:

```bash
cargo test --test pocket_ic_e2e -- --test-threads=1
```

Ordinary unit tests should remain parallel. Raising or lowering this value is
downstream capacity tuning, not an ic-testkit correctness requirement.

## Installing Wasm

Build the exact PocketIC instance required by the test, then move it into the
standalone fixture installer:

```rust,no_run
use ic_testkit::{
    artifacts,
    pic::{InstallSpec, PocketIcBuilder, StandaloneCanisterFixture},
};

#[test]
fn installs_a_prebuilt_canister() {
    let workspace = artifacts::workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    let target = artifacts::test_target_dir(&workspace, "pic-wasm");
    let wasm = artifacts::read_wasm(&target, "counter_canister", "release");

    let pocket_ic = PocketIcBuilder::new()
        .with_application_subnet()
        .build();
    let fixture = StandaloneCanisterFixture::install(
        pocket_ic,
        InstallSpec::new(wasm, vec![], 0),
    );
    fixture.pocket_ic().tick();
}
```

The builder may select a custom topology or an exact server binary before
construction. Use `try_install` when installation failure should remain typed;
`StandaloneCanisterInstallError` returns both the instance and the underlying
`CanisterInstallError` for inspection or recovery:

```rust,no_run
use candid::{Principal, encode_one};
use ic_testkit::pic::{InstallSpec, PocketIc, StandaloneCanisterFixture};

let fixture = StandaloneCanisterFixture::try_install(
    PocketIc::new(),
    InstallSpec::new(counter_wasm, encode_one(()).unwrap(), 1_000_000_000_000)
        .install_sender(Principal::anonymous())
        .label("counter"),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For an existing `PocketIc`, import `CanisterInstallExt` and use
`create_and_install_with_args` or
`try_create_and_install_with_args`. Use `InstallSpec` when you want an explicit
install sender, a diagnostic label, or sequential batch installs:

```rust,no_run
use candid::encode_one;
use ic_testkit::pic::{CanisterInstallExt, InstallSpec, PocketIc};

fn install_pair(pocket_ic: &PocketIc, first_wasm: Vec<u8>, second_wasm: Vec<u8>) {
    let ids = pocket_ic.create_and_install_many([
        InstallSpec::new(first_wasm, encode_one(()).unwrap(), 1_000_000_000_000)
            .label("first"),
        InstallSpec::new(second_wasm, encode_one(()).unwrap(), 1_000_000_000_000)
            .label("second"),
    ]);

    assert_eq!(ids.len(), 2);
}
```

Batch installs are sequential. If one install fails, earlier installs remain in
the PocketIC instance, the failed canister may also exist with the id exposed by
`CanisterInstallError::canister_id()`, and later installs are not attempted. If
PocketIC reports install-code rate limiting, one `RetryPolicy` defines the
exact maximum attempt count and simulated cooldown. The operation returns
PocketIC's `RejectResponse`; retry classification compares its structured
`error_code` and returns the original response unchanged:

```rust,no_run
use std::time::Duration;
use ic_testkit::pic::{CanisterInstallExt, RejectResponse, RetryPolicy};

let result: Result<(), RejectResponse> = pocket_ic.retry_install_code(
    RetryPolicy::new(3, Duration::from_secs(60)),
    || install_again(),
);
```

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

Use `BenchmarkAggregateRow::is_all_suites()` to distinguish the cross-suite
aggregate from an authored suite that is literally named `ALL`.

The report writer emits CSV artifacts for raw events, spans, aggregates,
malformed/unpaired/invalid markers, and comparisons, plus `bench-summary.md`
and `metadata.json`. Run helpers derive paths such as
`reports/runs/2026-05-24T162600Z-a1b2c3d-0001/`, write reports, and discover
compatible previous runs. The path remains caller-owned: concurrent writers
must use unique paths or synchronize access to the same path.

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

For expensive setup, `CachedPocketIcBaseline` can snapshot canisters once and restore them between tests. If the cached PocketIC instance has died, `restore_or_rebuild_cached_pocket_ic_baseline` rebuilds instead of reusing a broken instance.

```rust,no_run
use std::sync::Mutex;

use candid::Principal;
use ic_testkit::pic::{
    CachedPocketIcBaseline, CandidCallExt, PocketIc,
    restore_or_rebuild_cached_pocket_ic_baseline,
};

struct BaselineMetadata {
    controller_id: Principal,
    canister_id: Principal,
}

static BASELINE: Mutex<Option<CachedPocketIcBaseline<BaselineMetadata>>> = Mutex::new(None);

fn baseline_for_test() {
    let (baseline, cache_hit) = restore_or_rebuild_cached_pocket_ic_baseline(
        &BASELINE,
        || build_baseline_once(),
        |baseline| {
            baseline
                .restore(baseline.metadata().controller_id)
                .expect("restore cached snapshot set");
        },
    );

    if cache_hit {
        baseline.pocket_ic().tick();
    }

    let canister_id = baseline.metadata().canister_id;
    let value: u64 = baseline
        .pocket_ic()
        .query_candid_or_panic(canister_id, "get", ());
    assert_eq!(value, 0);
}

fn build_baseline_once() -> CachedPocketIcBaseline<BaselineMetadata> {
    let (pocket_ic, controller_id, canister_id) = build_expensive_fixture();

    CachedPocketIcBaseline::capture(
        pocket_ic,
        controller_id,
        [canister_id],
        BaselineMetadata {
            controller_id,
            canister_id,
        },
    )
    .expect("snapshot capture must be available")
}

fn build_expensive_fixture() -> (PocketIc, Principal, Principal) {
    unimplemented!("install the canisters needed by this test suite")
}
```

Snapshot sets reject duplicate canister ids before capture, store entries in
deterministic order, return `ControllerSnapshotError`, and remove snapshots
already captured if a later canister fails.

## Deterministic Test Identities

`Fake` gives stable principals from numeric seeds:

```rust
use ic_testkit::Fake;

let alice = Fake::principal(1);
let bob = Fake::principal(2);

assert_ne!(alice, bob);
assert_eq!(alice, Fake::principal(1));
```

## What This Adds Over `pocket-ic`

- direct re-exports of `PocketIc` and `PocketIcBuilder`
- `CandidCallExt` query/update helpers with structured rejections, contextual errors, and panic variants
- generic wasm install helpers, retry helpers, diagnostics, and standalone fixtures
- cached snapshot baselines for expensive test setup
- deterministic fake principals
- wasm path/build/readiness helpers, including generated `.icp` freshness checks
- compact benchmark marker parsing, aggregation, comparison, and report writing
- canister-side `Performance::measure` marker emission

## Migrating From 0.1

| 0.1 API | 0.2 API |
| --- | --- |
| `Pic` | re-exported `PocketIc` |
| `pic()` | `PocketIc::new()` |
| `PicBuilder` | re-exported `PocketIcBuilder`; call `build()` directly |
| `PicSerialGuard` and acquisition helpers | remove them; each test owns an independent instance |
| `Pic::query_call` / `update_call` | `CandidCallExt::query_candid` / `update_candid` |
| `PicCallError` | `CandidCallError` with structured `CanisterReject` responses |
| `PicInstallError` | `CanisterInstallError` |
| `PicRuntimeConfig` and binary preflight helpers | upstream `PocketIcBuilder`, `POCKET_IC_BIN`, and PocketIC's cache |
| `CachedPicBaseline` | `CachedPocketIcBaseline` |
| `fixture.pic()` | `fixture.pocket_ic()` |
| `retry_install_code_ok` / `retry_install_code_err` | `retry_install_code(RetryPolicy, operation)` |
| snapshot capture returning `Option` | ordered capture returning `Result<_, ControllerSnapshotError>` |

### 0.2.2 hard cut

| Removed API | Replacement |
| --- | --- |
| `install_prebuilt_canister*` free functions | `StandaloneCanisterFixture::install(caller_built_pocket_ic, InstallSpec)` |
| `try_install_prebuilt_canister*` free functions | `StandaloneCanisterFixture::try_install(caller_built_pocket_ic, InstallSpec)` |
| `StandaloneCanisterFixtureError` | `StandaloneCanisterInstallError`, which retains the caller's `PocketIc` and the `CanisterInstallError` |
| `PocketIcStartError` | upstream `PocketIc::new()` / `PocketIcBuilder::build()` behavior |
| string-returning `retry_install_code` operation | operation returning `Result<T, RejectResponse>` |

## Boundaries

This crate does not define application init payloads, endpoint names, role models, readiness polling, canister graph topology, benchmark labels, threshold policy, CI failure policy, or broad self-test orchestration. Those belong in the application or framework that owns the canisters being tested.

## Toolchains

- MSRV: Rust 1.88
- internal build/test lane: Rust 1.96

## Local Checks

```bash
make test
make test-canisters
make build-test-canisters
make release-check
```

## Releases

Patch and minor releases use the same guarded local flow as `ic-query`. Commit
the changelog entry for the target version and start from a clean worktree,
then run one of:

```bash
make release-patch
make release-minor
```

This runs CI, bumps the workspace package version, stages the version files,
commits and tags the release, re-runs CI, and pushes the commit and tag. After
the tag CI succeeds, publish the tagged commit with:

```bash
make publish
```

Publication requires a clean worktree and a matching `v<version>` tag at
`HEAD`. Re-running it is safe when that crate version is already on crates.io.
