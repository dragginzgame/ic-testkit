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

`ic-testkit` is test infrastructure for Internet Computer applications that
have outgrown one-off PocketIC scripts. It keeps
[`pocket-ic`](https://crates.io/crates/pocket-ic) visible while adding reusable
building blocks for typed calls, multi-canister fixtures, safe baseline reuse,
reproducible Wasm pipelines, diagnostics, and performance reports.

It does not wrap the simulator, mirror PocketIC's API, manage a second server
binary cache, or serialize independent PocketIC instances.

## Install

Host-side test crates normally add:

```toml
[dev-dependencies]
ic-testkit = "0.6.1"
```

Canister crates that emit benchmark markers can add the same version under
`[dependencies]` and use `ic_testkit::performance`.

The crate supports Rust 1.88 and uses PocketIC 15.

## Features for application-scale test suites

Complex applications tend to make test infrastructure expensive in several
dimensions at once: many Wasm variants must be built, a topology must be
installed and seeded, state must be reset honestly between tests, and failures
must retain enough context to diagnose in CI. The main features are designed to
compose across that complete loop:

| Test-suite need | What ic-testkit provides | Why it matters |
| --- | --- | --- |
| Exercise the real topology | Direct `PocketIc` access, generic installation, caller-owned multi-canister recipes | Tests can model their actual canister graph without fitting it into a framework-owned abstraction |
| Reuse expensive setup safely | Transactional snapshots, bounded fixture pools, typed reset requirements, readiness and invariant receipts | Warm tests are fast, while every reusable state domain has an explicit application-owned policy |
| Build reproducible inputs | Content-addressed Wasm builds, transactional external artifact sets, exact freshness checks, batching and retention | Concurrent processes reuse complete artifacts and reject source, toolchain, or publication races |
| Make failures actionable | Typed startup, Candid, install, snapshot, pool, and artifact errors with preserved causes and partial timings | A rejection stays distinct from transport failure, and recovery failures keep both the original and rebuild context |
| Observe long and parallel suites | Structured build progress, cache/pool outcomes, phase timings, and best-effort canister diagnostics | CI logs can show whether time was spent waiting, building, restoring, validating, or recovering |
| Catch performance regressions | Canister-side instruction and memory markers plus host-side parsing, aggregation, comparison, and report writing | Functional and performance coverage can exercise the same application workflows |

### A typical complex-suite workflow

1. Describe each Wasm variant with `WasmBuildSpec`; exact hits reuse immutable
   outputs, while independent batch entries keep Cargo feature resolution
   separate.
2. Install and seed the application topology in a `PocketIcBaselineRecipe`,
   including every canister and external resource relevant to the baseline.
3. Acquire a bounded pool lease. A cold slot builds once; a warm slot restores
   snapshots, resets non-snapshot state, drives the topology to readiness, and
   validates final invariants before the test receives it.
4. Drive the application through typed Candid calls. Emit structured outcomes
   and timings, dump canister diagnostics on failure, and invalidate the lease
   after any mutation outside the recipe's reset contract.
5. Parse performance markers into spans, aggregates, comparisons, and reports
   when the same scenario also has instruction or memory budgets.

See the compile-checked
[`multi_canister_baseline_pool`](crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
and
[`transactional_artifact_cache`](crates/ic-testkit/examples/transactional_artifact_cache.rs)
examples for complete reusable recipes.

### Choose the right isolation level

Not every test should use a cache. Choose the narrowest fixture that preserves
the behavior under test:

| Fixture | Best fit |
| --- | --- |
| Fresh caller-owned `PocketIc` | Installation, upgrade, topology, teardown, time, cycle-accounting, and snapshot behavior |
| `StandaloneCanisterFixture` | A single installed canister with concise typed calls, but no cross-test reuse |
| `CachedStandaloneCanisterFixturePool<N>` | Repeated single-canister scenarios whose relevant state is captured by one baseline snapshot |
| `CachedPocketIcBaselinePool<R>` | Expensive multi-canister topologies with an explicit recipe for snapshots, non-snapshot state, readiness, and invariants |

Pools bound resource use; they do not turn incomplete reset coverage into safe
reuse. Keep a test on a fresh instance when its mutations cannot be completely
restored or validated.

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
| Artifacts | `ArtifactCacheSpec`, `WasmBuildSpec`, `WatchedInputSnapshot` | Transactional external artifact sets, content-addressed Wasm builds, bounded retention, and exact freshness stamps |
| Benchmarks | `benchmark`, `performance` | Marker emission, parsing, aggregation, comparison, and reports |
| Test identities | `Fake` | Stable deterministic principals |

`ic_testkit::pic::prelude::*` imports the seven extension traits only. Data types
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

Mixed-controller topologies can avoid expected rejected fallback calls by
selecting the exact sender for each canister:

```rust,no_run
use ic_testkit::pic::{
    CanisterSnapshotTarget, PocketIcCapturedSnapshotExt, PocketIcSnapshotExt,
};

let snapshots = pocket_ic.capture_snapshots_with_senders([
    CanisterSnapshotTarget::new(root_canister, Some(root_controller)),
    CanisterSnapshotTarget::new(child_canister, None),
])?;
pocket_ic.restore_snapshots_with_captured_senders(&snapshots)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The existing controller-based method remains useful when its ordered sender
fallback is desired. `CachedPocketIcBaseline::capture_with_senders` exposes the
same exact-sender policy for pooled baselines, and
`restore_with_captured_senders` restores it without requiring or attempting a
fallback controller.

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
Cache and pool outcomes and timing records implement compact single-line
`Display`, so consumers can emit useful diagnostics without reformatting every
phase themselves.

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

High-frequency suites can use `with_prune_policy_at_most_every` to retain the
same limits while replacing repeated hit-path directory scans with a small
namespace marker check. The original `with_prune_policy` continues to run on
every successful acquisition.

`WasmBuildTimings::input_resolution_detail` separates Cargo/rustc identity,
Cargo metadata, input discovery, and content hashing. The existing
`input_resolution` accessor remains the aggregate for compatibility.

Source-edit-heavy suites can opt into a caller-owned shared Cargo target while
retaining exact immutable final Wasm entries:

```rust,no_run
use ic_testkit::artifacts::{
    SharedIncrementalTargetPrunePolicy, WasmBuildSpec, build_wasm_canisters_cached,
    inspect_shared_incremental_target, maintain_shared_incremental_target,
    resolve_cargo_build_inputs,
};

# let workspace = std::path::PathBuf::from(".");
let exact_cache = workspace.join("target/pic-wasm");
let shared_incremental = workspace.join("target/pic-wasm-incremental");
let spec = WasmBuildSpec::new(
    &workspace,
    &exact_cache,
    &["counter_canister"],
    "debug",
)
.with_shared_incremental_target(&shared_incremental);

let inputs = resolve_cargo_build_inputs(&spec)?;
let outcome = build_wasm_canisters_cached(&spec)?;
let shared_usage = inspect_shared_incremental_target(&spec)?
    .expect("the shared target exists after a cache miss");
let maintenance = maintain_shared_incremental_target(
    &spec,
    SharedIncrementalTargetPrunePolicy::new()
        .with_max_age(std::time::Duration::from_secs(7 * 24 * 60 * 60))
        .with_max_size_bytes(4 * 1024 * 1024 * 1024),
)?;
eprintln!(
    "{} inputs={} shared_bytes={} maintenance={maintenance:?} {}",
    inputs.fingerprint(),
    inputs.inputs().len(),
    shared_usage.logical_size_bytes(),
    outcome,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

An exact hit never acquires or mutates the shared target. On a miss, callers
using that target serialize across processes; Cargo builds incrementally, then
`ic-testkit` re-resolves every exact input and publishes only the final Wasm
files. A source race rejects publication, while a failed build preserves the
caller-owned incremental state. Retention owns only immutable output entries,
not the shared Cargo target. `inspect_shared_incremental_target` takes the same
cross-process lock and reports its canonical path, logical size, last recorded
build use, and lock wait without removing caller-owned Cargo state.
`maintain_shared_incremental_target` is an explicit whole-target operation: if
an age or size threshold is exceeded it removes every other child while
retaining the root, cache tag, and live coordination lock. Callers must not
colocate unrelated data that needs to survive a clear. Exact Wasm acquisitions
never invoke that maintenance automatically. Before clearing, the exact Cargo
resolver rejects a target that overlaps source, configuration, or additional
inputs.

Long cold Cargo builds can expose synchronous structured progress without
changing the existing silent API:

```rust,no_run
use ic_testkit::artifacts::{
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildSpec,
    build_wasm_canisters_cached_with_progress,
};
use std::time::Duration;

# let workspace = std::path::PathBuf::from(".");
let spec = WasmBuildSpec::new(
    &workspace,
    &workspace.join("target/pic-wasm"),
    &["counter_canister"],
    "debug",
);
let outcome = build_wasm_canisters_cached_with_progress(
    &spec,
    WasmBuildProgressConfig::new()
        .with_heartbeat_interval(Duration::from_secs(15))
        .with_cargo_output(false),
    |event| {
        if let WasmBuildProgressEvent::CargoHeartbeat { elapsed } = event {
            eprintln!("Cargo is still running after {elapsed:?}");
        }
    },
)?;
eprintln!("{outcome}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Observers run synchronously on the build thread and should return promptly.
Raw output events preserve non-UTF-8 bytes; output is still captured in
`WasmBuildError` when forwarding is disabled. An observer panic terminates and
waits for the Cargo child before normal lock and incomplete-entry cleanup.

Suites building several variants can batch independent specs without changing
Cargo feature semantics:

```rust,no_run
use ic_testkit::artifacts::{WasmBuildSpec, build_wasm_canisters_cached_batch};

# let workspace = std::path::PathBuf::from(".");
# let target = workspace.join("target/pic-wasm");
let specs = [
    WasmBuildSpec::new(&workspace, &target, &["role_a"], "debug"),
    WasmBuildSpec::new(&workspace, &target, &["role_b"], "debug"),
];
let batch = build_wasm_canisters_cached_batch(&specs)?;
eprintln!("{batch}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

The batch is sequential and fail-fast. Every entry uses the ordinary
single-spec implementation and its own exact identity, locks, policy, and
Cargo command; packages are never collapsed into one invocation that could
unify shared dependency features. A failure exposes the already-completed
prefix, whose artifacts remain valid.

`ResolvedCargoBuildInputs::is_current` provides the same before/after identity
check for external Cargo-derived workflows. OS-native iterator builders ending
in `_os` preserve dynamic and non-UTF-8 argument and environment bytes. Use
`resolve_executable` to turn a bare `PATH` program into a canonical executable
file before declaring it through `ArtifactCacheSpec::with_tool`.

External transforms derived from a Cargo package closure can pass both the
build spec and its resolved snapshot to
`ArtifactCacheSpec::with_cargo_build_inputs`. The complete Cargo fingerprint
becomes transactional identity, and the exact Cargo-aware input paths and
generated-state exclusions are revalidated during preparation, hit
materialization, and commit. This avoids duplicating Cargo discovery as broad
workspace scans while still rejecting source or toolchain races.

External deterministic tools use the transactional artifact-set cache. The
caller declares exact inputs, tool bytes, arguments, relevant environment, and
the complete output schema; `ic-testkit` owns cross-process locking, staging,
before/after input verification, validation, publication, materialization, and
retention:

```rust,no_run
use ic_testkit::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCachePrunePolicy,
    ArtifactCacheSpec, prepare_artifact_cache,
};
use std::{io, process::Command, time::Duration};

# let workspace = std::path::PathBuf::from(".");
let input = workspace.join("target/wasm/counter.wasm");
let optimizer = workspace.join("tools/wasm-optimizer");
let destination = workspace.join("target/optimized/counter.wasm");
let spec = ArtifactCacheSpec::new(
    &workspace.join("target/external-artifact-cache"),
    "counter-optimizer",
    "counter/optimizer/v1",
)
.with_input("counter.wasm", &input)
.with_tool("optimizer", &optimizer)
.with_arguments(&["--optimize-for-size"])
.with_environment(&[("OPTIMIZER_MODE", "deterministic")])
.with_output("optimized.wasm", &destination)
.with_prune_policy_at_most_every(
    ArtifactCachePrunePolicy::new()
        .with_max_age(Duration::from_secs(14 * 24 * 60 * 60))
        .with_max_size_bytes(2 * 1024 * 1024 * 1024),
    Duration::from_secs(60 * 60),
);

let outcome = match prepare_artifact_cache(&spec)? {
    ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
    ArtifactCachePreparation::Build(transaction) => {
        let staged = transaction.output_path("optimized.wasm")?;
        let status = Command::new(&optimizer)
            .args(["--optimize-for-size", "-o"])
            .arg(staged)
            .arg(&input)
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!("optimizer failed with {status}")).into());
        }
        transaction.commit()?
    }
};
eprintln!("artifact key: {}", outcome.record().key());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Multiple independent external recipes can use
`build_artifact_caches_batch`. Its callback receives only cache misses and only
one live transaction at a time, avoiding self-deadlock when specs share a
coordination scope. Callback failures synchronously abort the current staging
directory. The operation is deliberately not atomic across specs: use one
`ArtifactCacheSpec` with several outputs when all artifacts must publish as one
transaction.

A miss transaction exposes checked staging paths for redirectable tools and an
`import_output` helper for commands that write to fixed locations. `commit`
accepts only the complete declared output set and publishes its manifest last;
dropped, failed, or panicked transactions remove staging without creating a
cache hit. Multi-output recipes declare several logical outputs and commit them
as one unit. Recipe identity is caller-owned and must change when undeclared
pipeline semantics change. Namespace and recipe identifiers are hashed before
being used on disk, environment values are never rendered in `Debug` or cache
manifests, and public destination paths do not affect content identity.
Prefer declaring pipeline implementation files, configuration, and wrappers as
ordinary exact inputs. Reserve manual `recipe_id` bumps for semantic changes
that cannot be represented by stable input bytes.

Declared input and tool roots may contain the cache directory, which is
excluded while recursively hashing that broader tree, but they must not be
located inside the cache themselves. Public output destinations must also stay
outside the cache, resolve to distinct paths, and not overlap a declared input
or tool. These checks prevent silently unkeyed inputs, self-invalidating
recipes, and two logical outputs overwriting the same file.

The default coordination scope is the namespace. Recipes that use the same
mutable external work tree can select a shared scope even when their content
keys differ. Independent recipes can use distinct scopes. Exact content-key
locking still ensures that overlapping callers build one result only.

Artifact contents are hashed, imported, and materialized with bounded-memory
streaming. Configured or explicit retention also removes staging abandoned by
terminated processes when the corresponding content-key lock proves that no
transaction is active. `ArtifactCachePruneReport` reports those uncommitted
directories separately from committed entry age/size eviction.

`ArtifactCacheSpec::with_prune_policy_at_most_every` applies the same explicit
limits but scans no more frequently than the selected interval. A skipped pass
has no maintenance outcome; its inexpensive marker-check duration remains in
the timing record.

No retention limit is automatic. As a starting point for developer caches,
the examples use a 14-day age bound with a size appropriate to the entry type
(larger for isolated Cargo targets, smaller for final artifact sets). CI should
choose an explicit limit that fits its cache quota and expected fingerprint
count rather than inheriting a machine-wide default.

`WatchedInputSnapshot` likewise hashes file paths and contents instead of
modification times. Generated artifacts become reusable only after
`mark_artifact_fresh` atomically records the snapshot beside the successfully
built artifact; an unstamped artifact is conservatively stale.

The cache intentionally contains no Binaryen, `icp build`, package-name, or
PocketIC policy. Its complete contract is recorded in the
[`0.5` transactional artifact-set design](docs/design/0.5-artifact-transactions/0.5-design.md).
Shared Cargo-target ownership and exact-output guarantees are recorded in the
[`0.6` shared-incremental Wasm design](docs/design/0.6-shared-incremental-wasm/0.6-design.md).
Independent orchestration, progress, and explicit mutable-target maintenance
are recorded in the
[`0.7` artifact orchestration design](docs/design/0.7-artifact-orchestration/0.7-design.md).

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
