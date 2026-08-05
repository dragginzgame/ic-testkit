# ic-testkit

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

`ic-testkit` is a focused test-harness layer around
[`pocket-ic`](https://crates.io/crates/pocket-ic). It re-exports PocketIC's
primary native types and adds reusable behavior where a harness benefits from
typed errors, deterministic policy, or shared test infrastructure.

It does not wrap the simulator, mirror PocketIC's API, manage a second server
binary cache, or serialize independent PocketIC instances.

## Install

Host-side test crates normally add:

```toml
[dev-dependencies]
ic-testkit = "0.3.1"
```

Canister crates that emit benchmark markers can add the same version under
`[dependencies]` and use `ic_testkit::performance`.

The crate supports Rust 1.88 and uses PocketIC 15.

## API at a glance

| Area | Main surface | Value added by ic-testkit |
| --- | --- | --- |
| PocketIC runtime | `PocketIc`, `PocketIcBuilder` | Direct upstream re-exports; no wrapper |
| Startup | `PocketIcBuilderExt` | Converts the currently panicking builder boundary into a typed result |
| Calls | `CandidCallExt` | Candid encoding/decoding, contextual errors, preserved rejections |
| Installation | `CanisterInstallExt`, `InstallSpec` | Generic install policy, diagnostics, structured rate-limit retry |
| Standalone fixtures | `StandaloneCanisterFixture` | Owns one caller-built instance and one installed canister id |
| Snapshots | `PocketIcSnapshotExt`, `CachedPocketIcBaseline` | Ordered transactional capture, explicit restore funding, scoped caching |
| Fixture pools | `CachedStandaloneCanisterFixturePool`, `CachedPocketIcBaselinePool` | Bounded standalone or recipe-driven multi-canister baseline reuse |
| Diagnostics | `PocketIcDiagnosticsExt` | Best-effort status and log reporting |
| Time | `PocketIcTimeExt` | Nanoseconds-since-epoch conversion only |
| Artifacts | `WasmBuildSpec`, `WasmBuildOutcome`, `WasmBuildCachePrunePolicy`, `WatchedInputSnapshot` | Content-addressed Wasm builds, bounded cache retention, exact freshness stamps, and dedicated target directories |
| Benchmarks | `benchmark`, `performance` | Marker emission, parsing, aggregation, comparison, and reports |
| Test identities | `Fake` | Stable deterministic principals |

`ic_testkit::pic::prelude::*` imports the six extension traits only. Data types
and PocketIC types remain explicit imports.

## PocketIC ownership and concurrency

Each test normally constructs and directly owns one fresh `PocketIc`:

```rust,no_run
use ic_testkit::pic::{PocketIc, prelude::*};

let pocket_ic = PocketIc::new();
let canister_id = install_counter(&pocket_ic);

let value: u64 = pocket_ic.query_candid(canister_id, "get", ())?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use PocketIC's inherent methods for topology, controllers, raw ingress, time,
rounds, cycles, and lifecycle operations. ic-testkit does not implement
`Deref`, retain a process-wide runtime guard, or reconnect to an instance by
port.

Independent instances may run concurrently. If a heavy E2E target exceeds CI
capacity, tune that target through libtest or the CI scheduler:

```bash
cargo test --test pocket_ic_e2e -- --test-threads=1
```

Keep ordinary unit tests parallel. The thread count is downstream capacity
tuning, not an ic-testkit correctness requirement.

## Typed Candid calls

`CandidCallExt` supplies anonymous and caller-aware query/update variants:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{CandidCallExt, PocketIc};

let pocket_ic = PocketIc::new();
let canister_id = install_counter(&pocket_ic);

let _: () = pocket_ic.update_candid(canister_id, "increment", ())?;
let value: u64 = pocket_ic.query_candid_as(
    canister_id,
    Principal::anonymous(),
    "get",
    (),
)?;
assert_eq!(value, 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The corresponding `_or_panic` methods unwrap only `CandidCallError`.
Application values such as `Result<T, E>` remain unchanged.

`CandidCallErrorKind` distinguishes:

- `Encode` and `Decode` failures with call context;
- `CanisterReject`, retaining the complete upstream `RejectResponse`;
- `Transport` when the PocketIC instance is unreachable;
- `Other` for uncategorized harness failures.

A canister rejection is not a transport failure. Inspect it through
`CandidCallError::reject_response()`.

## Startup and runtime provenance

Configure topology and binary selection on the upstream builder:

```rust,no_run
use ic_testkit::pic::{PocketIc, PocketIcBuilder, PocketIcBuilderExt};

fn build_test_ic() -> Result<PocketIc, ic_testkit::pic::PocketIcStartupError> {
    PocketIcBuilder::new()
        .with_application_subnet()
        .with_ii_subnet()
        .try_build()
}
```

`try_build()` catches PocketIC's current builder panic and returns
`PocketIcStartupError`. It deliberately preserves the message without parsing
it into guessed categories. The upstream panic hook may still print before the
panic is caught. Recreate the consumed builder for each bounded retry attempt.

PocketIC remains responsible for `POCKET_IC_BIN`, downloads, validation, and
its cache. ic-testkit never mutates the process environment and does not
maintain a parallel binary resolver.

For reproducible benchmarks, require an explicit `POCKET_IC_BIN` or pass an
explicit path to `PocketIcBuilder::with_server_binary`. Resolve and hash that
caller-owned file before construction and record it with the report.
`LATEST_SERVER_VERSION` exposes the version expected by the client, and
`PocketIc::get_server_url()` exposes the active endpoint. PocketIC 15 does not
expose the resolved binary path or digest from a built instance.

## Canister installation and fixtures

`InstallSpec` keeps generic install inputs and diagnostics together:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{
    InstallSpec, PocketIcBuilder, StandaloneCanisterFixture,
};

let pocket_ic = PocketIcBuilder::new()
    .with_application_subnet()
    .build();
let fixture = StandaloneCanisterFixture::try_install(
    pocket_ic,
    InstallSpec::new(wasm, init_bytes, 1_000_000_000_000)
        .install_sender(Principal::anonymous())
        .label("counter"),
)?;

let value: u64 = fixture.query_candid("get", ())?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The fixture owns exactly one `PocketIc`. `pocket_ic()` borrows it,
`canister_id()` identifies the installed canister, and `into_parts()` returns
both without changing instance ownership. Failed `try_install` returns a
`StandaloneCanisterInstallError` containing both the caller's instance and the
`CanisterInstallError`.

For installation into an existing instance, import `CanisterInstallExt` and
use `try_create_and_install` or `try_create_and_install_many`. Batch installs
run in iterator order. If one fails, earlier installs remain, the failed
canister may already exist, and later installs are not attempted.

### Install-code rate limiting

`retry_install_code` retries only a structured
`ErrorCode::CanisterInstallCodeRateLimited` rejection:

```rust,no_run
use std::time::Duration;
use ic_testkit::pic::{CanisterInstallExt, RejectResponse, RetryPolicy};

let policy = RetryPolicy::try_new(3, Duration::from_secs(60))?;
let result: Result<(), RejectResponse> =
    pocket_ic.retry_install_code(policy, || install_again());
# Ok::<(), Box<dyn std::error::Error>>(())
```

`max_attempts` includes the first call. Between retryable attempts, the helper
advances simulated time by the configured cooldown and executes two ticks. It
returns all non-rate-limit rejections unchanged.

## Snapshots and cached baselines

Snapshot capture validates duplicate ids before doing work, stores entries in
deterministic principal order, and cleans up earlier snapshots if a later
capture fails:

```rust,no_run
use ic_testkit::pic::{PocketIcSnapshotExt, SnapshotRestoreFunding};

let snapshots = pocket_ic.capture_controller_snapshots(
    controller_id,
    [first_canister, second_canister],
)?;

// Default: do not add cycles.
pocket_ic.restore_controller_snapshots(controller_id, &snapshots)?;

// Optional explicit fixture policy.
pocket_ic.restore_controller_snapshots_with_funding(
    controller_id,
    &snapshots,
    SnapshotRestoreFunding::TopUpTo {
        minimum_cycles: 200_000_000_000_000,
    },
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ControllerSnapshotError` preserves rejected sender attempts, panic context,
and any cleanup failures. Restore never adds cycles unless `TopUpTo` is
explicitly selected.

`CachedPocketIcBaseline<T>` stores one owned instance, its snapshots, and
caller metadata. `restore_or_rebuild_cached_pocket_ic_baseline` synchronizes
only the supplied `Mutex` slot. On a cache hit it invokes the caller's restore
closure; it rebuilds only when the owned PocketIC transport is dead and resumes
unrelated panics.

```rust,no_run
use std::sync::Mutex;
use ic_testkit::pic::{
    CachedPocketIcBaseline, restore_or_rebuild_cached_pocket_ic_baseline,
};

static BASELINE: Mutex<Option<CachedPocketIcBaseline<Metadata>>> = Mutex::new(None);

let (baseline, cache_hit) = restore_or_rebuild_cached_pocket_ic_baseline(
    &BASELINE,
    build_baseline,
    |baseline| baseline.restore(baseline.metadata().controller_id).unwrap(),
);

if cache_hit {
    baseline.pocket_ic().tick();
}
```

The returned guard retains exclusive access to that slot until dropped. It
does not block fresh PocketIC instances or baselines stored in other slots.

Heavy suites that need bounded parallelism can own a fixed-capacity pool of
independent standalone fixtures. Every populated slot has its own PocketIC
instance and baseline snapshot; a lease restores only that slot and dereferences
to the ordinary `StandaloneCanisterFixture` API:

```rust,no_run
use ic_testkit::pic::{
    CachedStandaloneCanisterFixturePool, StandaloneFixturePoolOutcome,
};

static POOL: CachedStandaloneCanisterFixturePool<8> =
    CachedStandaloneCanisterFixturePool::new();

let (fixture, outcome) = POOL.acquire_with_outcome(build_fixture)?;
let value: u64 = fixture.query_candid("get", ())?;
assert!(matches!(
    outcome,
    StandaloneFixturePoolOutcome::Built { .. }
        | StandaloneFixturePoolOutcome::Restored { .. }
        | StandaloneFixturePoolOutcome::Rebuilt { .. }
));
# let _ = value;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`acquire_with_outcome` reports queue wait, fixture build and snapshot capture,
restore, stale teardown, and total time. Snapshot failures retain their partial
timings, while dead-transport recovery preserves both the restore and rebuild
failure when replacement capture also fails. Existing callers can continue to
use `acquire`; its boolean remains `true` only for a successfully restored slot
and `false` for both builds and rebuilds.

One pool must represent one fixture recipe. Pass a builder with the same Wasm,
init arguments, topology, and seeded state on every acquisition; use a separate
pool for a different recipe. Setup performed by the builder is included in the
captured baseline.

The lease's `Drop` implementation releases a capacity slot, so Clippy can
suggest tightening its scope. Drop the lease explicitly when the test no longer
needs it. When a suite intentionally retains the lease for the complete test,
use the narrowest practical allowance and record the reason, for example at the
test or dedicated test-file boundary:

```rust
#![allow(
    clippy::significant_drop_tightening,
    reason = "each test intentionally retains its pooled fixture lease for its full scope"
)]
```

The caller still chooses capacity and which tests may reuse the baseline. A
snapshot restores the installed canister, not the complete PocketIC instance:
instance time, other canisters, and cycle changes outside the selected restore
funding policy may persist. Keep installation, upgrade, topology, teardown,
time-sensitive, cycle-accounting, and snapshot tests on fresh directly owned
fixtures.

For a topology with multiple captured canisters, use
`CachedPocketIcBaselinePool<R>`. Its runtime capacity can be tuned per host and
one pool structurally owns one `PocketIcBaselineRecipe` for its entire
lifetime:

```rust,ignore
use std::num::NonZeroUsize;
use ic_testkit::pic::{BaselinePoolOutcome, CachedPocketIcBaselinePool};

let pool = CachedPocketIcBaselinePool::new(
    NonZeroUsize::new(1).unwrap(),
    root_topology_recipe(),
);

let (baseline, outcome) = pool.acquire()?;
assert!(matches!(
    outcome,
    BaselinePoolOutcome::Built { .. }
        | BaselinePoolOutcome::Restored { .. }
        | BaselinePoolOutcome::Rebuilt { .. }
));

run_test(baseline.pocket_ic(), baseline.metadata());
# Ok::<(), Box<dyn std::error::Error>>(())
```

A complete, compile-checked
[`PocketIcBaselineRecipe`](crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
shows a two-canister build, exact restore receipt, readiness boundary, invariant
validation, dead-transport classification, and cold/warm acquisition.

The recipe declares typed reset requirements and implements the exact reuse
sequence: restore every captured canister, reset non-snapshot state, drive the
topology to readiness, then validate final invariants. Snapshot and cycle
domains are mandatory. The pool checks that the restore receipt names the
complete captured set and that every required reset policy has an exact
matching achievement before returning a warm lease. Built and restored slots
run the same validation hook. After a successful baseline restore, prefer
`CanisterRestoreReceipt::try_from_baseline` so receipt membership is derived
from the captured set rather than duplicated in recipe metadata.

A recoverable preparation failure discards the slot and rebuilds once; if that
also fails, `BaselinePoolError::RecoveryFailed` retains both errors. Recipe or
test panics keep unwinding and mark the slot invalid for a later rebuild.
Callers can explicitly invalidate a lease after an operation outside the
recipe's reset contract. `BaselinePoolOutcome` and `BaselinePoolTimings` expose
whether the lease was built, restored, or rebuilt and where acquisition time
was spent. Failed acquisitions retain the partial or combined phase timings on
`BaselinePoolError::timings`, including queue wait, failed preparation, stale
teardown, and a failed rebuild attempt when applicable.

Recipes that wrap PocketIC's currently unstructured transport failures can use
`is_dead_pocket_ic_transport_error` in `classify_failure`, returning
`RebuildReason::DeadPocketIcTransport` when it matches and
`stage.default_rebuild_reason()` otherwise. The classifier searches the error
source chain; it remains a conservative message boundary until PocketIC
provides a structured transport error.

This is still baseline reuse, not complete simulator rollback. Recipes must
honestly account for time, extra canisters, pending messages, subnet state,
cycles, and external resources, or keep affected tests on fresh instances.
These domains can interact: advancing simulator time may also affect cycle
accounting, for example. Add and validate one reset guarantee at a time rather
than treating independent receipt entries as proof that the underlying state
is independent.
Capacity greater than one permits overlapping leases but does not make a
serial test runner parallel. See the
[`0.4` bounded baseline-pool design](docs/design/0.4-baseline-pooling/0.4-design.md)
for the contract, consumer eligibility, and benchmark plan.

## Diagnostics and time

`PocketIcDiagnosticsExt::dump_canister_debug` prints best-effort status and
canister logs without allowing a secondary diagnostics failure to replace the
original operation error.

`PocketIcTimeExt` intentionally contains one convenience:

```rust,no_run
use ic_testkit::pic::{PocketIc, PocketIcTimeExt};

let pocket_ic = PocketIc::new();
let now_ns = pocket_ic.current_time_nanos();
```

Use PocketIC's inherent `get_time`, `set_time`, `set_certified_time`,
`advance_time`, and `tick` methods for everything else.

## Wasm artifact helpers

The host-only `artifacts` module provides workspace-relative paths, dedicated
test target directories, content-addressed build coordination, Wasm loading,
and exact generated-artifact freshness checks:

```rust,no_run
use ic_testkit::artifacts::{
    WasmBuildCachePrunePolicy, WasmBuildOutcome, WasmBuildSpec,
    build_wasm_canisters_cached, read_wasm, test_target_dir, workspace_root_for,
};
use std::time::Duration;

let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
let target = test_target_dir(&workspace, "pic-wasm");
let spec = WasmBuildSpec::new(
    &workspace,
    &target,
    &["counter_canister"],
    "release",
)
.with_cargo_profile_args(&["--release", "--locked"])
.with_inherited_env(&["COUNTER_SCHEMA_MODE"])
.with_additional_inputs(&["config/counter-schema.json"])
.with_prune_policy(
    WasmBuildCachePrunePolicy::new()
        .with_max_age(Duration::from_secs(14 * 24 * 60 * 60))
        .with_max_size_bytes(10 * 1024 * 1024 * 1024),
);

let outcome = build_wasm_canisters_cached(&spec)?;
match &outcome {
    WasmBuildOutcome::Built(record) => {
        eprintln!("built {} in {:?}", record.fingerprint(), record.timings().total());
    }
    WasmBuildOutcome::Reused(record) => {
        eprintln!("reused {}", record.fingerprint());
    }
}

let wasm = read_wasm(&target, "counter_canister", "release");

if let Some(pruned) = outcome
    .record()
    .maintenance()
    .and_then(|maintenance| maintenance.prune_report())
{
    eprintln!(
        "removed {} builds ({} bytes)",
        pruned.entries_removed(),
        pruned.bytes_removed(),
    );
}
# let _ = wasm;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The fingerprint covers the selected package set and local dependency sources,
workspace manifest, lockfile, Cargo configuration, Rust toolchain files,
target and profile arguments, Cargo and rustc identities, explicit child
environment, selected inherited environment, and caller-declared additional
inputs. Cargo configuration discovery follows the configuration files Cargo can
read from the invocation directory and its ancestors plus the effective Cargo
home. The extensionless `.cargo/config` takes precedence when both supported
names exist, and recursive required or optional `include` entries are part of
the fingerprint. Build scripts that read application-specific environment
variables or files outside Cargo's package graph must still declare them on the
spec. Package inputs are content-hashed exactly but conservatively; exact
hashing does not promise that every file in the package closure is semantically
relevant to a build.

Calls sharing a target directory coordinate through one process lock. A cache
hit requires every expected nonempty Wasm artifact to carry the exact atomic
stamp for both the current fingerprint and artifact content. Exact builds are
retained under fingerprint-specific Cargo target directories, so a prior spec
can be materialized again after another spec used the caller-facing output.
`build_wasm_canisters` remains as the panicking convenience API and uses the
same cache automatically.

Failed builds remove their incomplete fingerprint directory before returning;
if cleanup also fails, `WasmBuildError::FailedBuildCleanup` preserves both
failures. Every used target root receives a standards-compliant
`CACHEDIR.TAG`. Successful builds and cache hits atomically update a per-entry
last-use marker. `prune_wasm_build_cache` takes the build lock, removes entries
over the optional age limit, then removes least-recently-used entries until the
optional logical-byte limit is met. It only prunes direct fingerprint-named
children below `.ic-testkit/wasm-targets`; public artifacts and unrelated Cargo
target contents remain caller-owned. `WasmBuildSpec::with_prune_policy` performs
the same maintenance without reacquiring the lock, protects the active
fingerprint even when it exceeds the configured bound, and reports a nonfatal
`WasmBuildCacheMaintenance` result plus its duration on the successful build
record. The standalone pruning function remains available when maintenance
failure should be returned directly.

`WasmBuildTimings::input_resolution_detail` separates Cargo/rustc identity,
Cargo metadata, input discovery, and content hashing. The existing
`input_resolution` accessor remains the aggregate for compatibility.

`WatchedInputSnapshot` likewise hashes file paths and contents instead of
modification times. Generated artifacts become reusable only after
`mark_artifact_fresh` atomically records the snapshot beside the successfully
built artifact; an unstamped artifact is conservatively stale.

The proposed follow-up unifies external multi-output builds and post-link
transforms behind one transactional cache rather than adding tool-specific
APIs. See the
[consolidated artifact and fixture cache design](docs/design/0.3-artifact-cache/follow-up-design.md).

## Benchmark markers and reports

Canister code emits compact markers around the measured region:

```rust,no_run
use ic_testkit::performance::Performance;

Performance::measure("storage/write:start");
// measured work
Performance::measure("storage/write:end");
```

The default line format is:

```text
ICTK|<label>:<start-or-end>|<instructions>|<heap_bytes>|<memory_bytes>|<total_allocation>
```

Host code parses, pairs, and aggregates captured markers:

```rust
use ic_testkit::benchmark::{
    BenchmarkParserConfig, aggregate_benchmark_spans,
    pair_benchmark_spans, parse_benchmark_events,
};

let input = "\
ICTK|storage/write:start|100|200|300|400
ICTK|storage/write:end|150|260|390|430
";
let parsed = parse_benchmark_events(input, &BenchmarkParserConfig::default());
let spans = pair_benchmark_spans(&parsed.events);
let aggregates = aggregate_benchmark_spans(&spans.spans);

assert_eq!(aggregates.rows[0].span_label, "storage/write");
```

The report writer emits raw events, spans, aggregates, comparisons, malformed
and unpaired markers, `bench-summary.md`, and `metadata.json`. Run-directory
helpers discover compatible previous runs but do not reserve paths. Concurrent
writers must use unique destinations or synchronize only the shared output
path.

An authored suite named `ALL` remains distinct from the internal cross-suite
aggregate. Use `BenchmarkAggregateRow::is_all_suites()` to distinguish them.

## Deterministic principals

```rust
use ic_testkit::Fake;

let alice = Fake::principal(1);
let bob = Fake::principal(2);

assert_ne!(alice, bob);
assert_eq!(alice, Fake::principal(1));
```

## Scope boundaries

ic-testkit remains generic. It does not define application init payloads,
endpoint names, role models, readiness polling, canister graph topology,
benchmark labels, regression thresholds, CI failure policy, or broad self-test
orchestration.

Mutable host resources must either be unique to one test invocation or
synchronized by a guard scoped only to that resource. The crate treats process
environment variables as caller-owned, read-only configuration.

See the maintained [PocketIC upstream boundary](https://github.com/dragginzgame/ic-testkit/blob/main/POCKET-IC.md)
for limitations that should ultimately be solved in PocketIC. The documents
under [`docs/design`](https://github.com/dragginzgame/ic-testkit/tree/main/docs/design)
are historical decision records; current behavior is documented here and in
rustdoc.

## Toolchains and checks

- Published MSRV: Rust 1.88
- Repository toolchain: Rust 1.96
- PocketIC client/server line: 15

Run the ordinary checks with:

```bash
make test
make test-canisters
```

Before release-oriented changes, run the complete gate:

```bash
make release-check
```

The release gate includes formatting, native and Wasm checks, warnings-denied
Clippy, rustdoc, unit and live PocketIC tests, canister fixture builds, package
verification, publish dry-run, and the Rust 1.88 MSRV check.

## Releases

Commit the changelog entry for the target version and start from a clean
worktree, then run:

```bash
make release-patch
# or: make release-minor
```

The guarded flow runs CI, bumps and stages the version files, creates the
release commit and tag, reruns CI, and pushes the commit and tag. After tag CI
succeeds:

```bash
make publish
```

Publication requires a clean worktree and a matching `v<version>` tag at
`HEAD`. Re-running `make publish` is safe when that version already exists on
crates.io.
