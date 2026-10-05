<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->

<p align="center">
  <a href="https://crates.io/crates/ic-testkit"><img src="https://img.shields.io/crates/v/ic-testkit.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/ic-testkit"><img src="https://docs.rs/ic-testkit/badge.svg" alt="Docs.rs"></a>
  <a href="https://crates.io/crates/ic-testkit"><img src="https://img.shields.io/crates/d/ic-testkit.svg" alt="Downloads"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/MSRV-1.88.0-blue.svg" alt="MSRV"></a>
  <a href="README.md#toolchains-and-checks"><img src="https://img.shields.io/badge/internal%20rust-1.99.0-orange.svg" alt="Internal Rust"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/edition-2024-purple.svg" alt="Rust edition"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/PocketIC-16.0-green.svg" alt="PocketIC"></a>
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://img.shields.io/badge/GitHub-dragginzgame%2Fic--testkit-black.svg" alt="Repository"></a>
</p>

`ic-testkit` helps teams check that an Internet Computer application works as a
complete system before they deploy it. It runs the application in PocketIC, a
local simulation of the Internet Computer, and adds a repeatable way to build
the application, prepare test data, run scenarios, reset state, diagnose
failures, and track performance.

In short: **PocketIC provides the simulated network; ic-testkit helps turn it
into a reliable test environment for a real application.**

## What does that mean?

Internet Computer applications are made from **canisters**: independently
deployed services that contain code and data. Their compiled code is stored in
**Wasm** files. A production application may use several canisters for jobs
such as identity, payments, game state, governance, or asset storage.

Testing that whole system repeatedly is harder than testing one function:

- every canister must be built and installed with the right configuration;
- the application often needs accounts, permissions, balances, or other
  starting data before a test can begin;
- one test must not accidentally inherit changes made by another;
- a failure needs enough context to explain what happened in the automated
  build-and-test system (CI); and
- slow setup can make a large test suite impractical to run often.

ic-testkit provides reusable pieces for that work. It can save a known-good
starting point, restore it between eligible tests, rebuild it when it is no
longer safe to reuse, and report where time was spent. The application still
owns its test scenario and decides what “clean,” “ready,” and “correct” mean.

### A concrete example

Imagine a game with separate canisters for player accounts, inventories, and
payments. A test for buying an item needs all three canisters installed, a
player with funds, and an item for sale. After the purchase, balances and
inventory have changed.

Without a reusable fixture, every test may rebuild and prepare the entire game.
With ic-testkit, the team can describe the prepared game once, capture that
known-good state, and safely restore it for the next compatible test. Tests
that change something outside that reset contract can use a fresh environment
instead. If a run fails, structured diagnostics help distinguish an application
rejection from a broken simulator connection or setup failure.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-game-topology.svg" alt="A purchase test exercises accounts, inventory, and payment canisters inside PocketIC while ic-testkit manages the shared baseline around the system." width="900">
</p>

The core lifecycle is:

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-core-lifecycle.svg" alt="Run a test, validate whether its environment is safe to reuse, and then restore or rebuild before the next test." width="900">
</p>

This is most useful for applications with multiple canisters, expensive setup,
many integration tests, intermittent CI failures, or performance budgets. A
small project with a few independent tests may only need PocketIC directly.

ic-testkit builds on PocketIC rather than replacing or hiding it. Developers
can use the complete upstream API through `ic_testkit::pocket_ic` and add only
the testkit helpers they need.

## Install

Host-side test crates normally add:

```toml
[dev-dependencies]
ic-testkit = "0.14"
```

Canister crates that emit benchmark markers can add the same version under
`[dependencies]` and use `ic_testkit::performance`.

The crate supports Rust 1.88 and uses PocketIC 16. The `pocket_ic`, `pic`, and
`artifacts` modules are host-only. `benchmark`, `performance`, and `Fake` are
also available when compiling for `wasm32`.

## What ic-testkit provides

The pieces can be used separately or combined into a complete test workflow:

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-capabilities.svg" alt="Six ic-testkit capabilities: system testing, safe reuse, reproducible builds, diagnostics, timings, and performance." width="900">
</p>

| Test-suite need | What ic-testkit does | Practical benefit |
| --- | --- | --- |
| Test the real application shape | Runs the caller's actual set of canisters in PocketIC | A test can exercise interactions between services, not just isolated functions |
| Reuse slow setup safely | Captures prepared state and restores it only under an explicit reset policy | Compatible tests start quickly without silently sharing dirty state |
| Build the right code | Reuses Wasm only when its source, settings, tools, and other declared inputs still match | Local and CI runs test the intended binaries and avoid unnecessary rebuilds |
| Understand failures | Preserves setup, call, installation, snapshot, and simulator errors with useful context | A product rejection is not confused with a disconnected test environment |
| See where time goes | Reports build, queue, restore, validation, and recovery timings | Teams can improve the slow part instead of guessing |
| Watch performance over time | Records instruction and memory measurements and writes comparison reports | Functional tests can also reveal performance regressions |

### A typical complex-suite workflow

1. Describe which application binaries and build inputs the test needs.
2. Start PocketIC, install the canisters, and prepare the application's starting
   data.
3. Record what must be restored or checked before that environment can be used
   again.
4. Acquire a prepared environment, run the scenario, and collect structured
   results and diagnostics.
5. Restore and validate the environment for another compatible test, or discard
   and rebuild it when safe reuse cannot be proven.
6. Optionally turn instruction and memory markers into performance reports.

See the compile-checked
[`multi_canister_baseline_pool`](crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
and
[`transactional_artifact_cache`](crates/ic-testkit/examples/transactional_artifact_cache.rs)
examples for complete developer-facing recipes.

> The rest of this README is a developer reference. If you only needed the
> product-level explanation, the sections above describe the purpose and value
> of ic-testkit.

### Choose the right isolation level

Not every test should use a cache. Choose the narrowest fixture that preserves
the behavior under test:

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-isolation-decision-tree.svg" alt="Decision tree for choosing a fresh PocketIC, standalone fixture, standalone fixture pool, or multi-canister baseline pool." width="900">
</p>

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
| PocketIC runtime | `pocket_ic`; `pic::{PocketIc, PocketIcBuilder}` | Complete upstream crate escape hatch plus focused convenience re-exports; no wrapper |
| Startup | `PocketIcBuilderExt`, `PocketIcStartupConfig`, `PocketIcManagedServer` | Explicit bounded spawn/connect policy, owned shared-server lifecycle, and structured failures |
| Calls | `CandidCallExt` | Candid encoding/decoding, contextual errors, preserved rejections |
| Installation | `CanisterInstallExt`, `InstallSpec` | Generic install policy, diagnostics, structured rate-limit retry |
| Standalone fixtures | `StandaloneCanisterFixture` | Owns one caller-built instance and one installed canister id |
| Snapshots | `PocketIcSnapshotExt`, `CachedPocketIcBaseline` | Ordered transactional capture, explicit restore funding, scoped caching |
| Fixture pools | `CachedStandaloneCanisterFixturePool`, `CachedPocketIcBaselinePool` | Bounded standalone or recipe-driven multi-canister baseline reuse |
| Diagnostics | `PocketIcDiagnosticsExt` | Controller-aware structured status and bounded log reporting |
| Time | `PocketIcTimeExt` | Nanoseconds-since-epoch conversion only |
| Artifacts | `ArtifactCacheSpec`, `WasmBuildSpec`, `WatchedInputSnapshot` | Transactional external artifact sets, content-addressed Wasm builds, bounded retention, and exact freshness stamps |
| Benchmarks | `benchmark`, `performance` | Marker emission, parsing, aggregation, comparison, and reports |
| Test identities | `Fake` | Stable deterministic principals |

`ic_testkit::pic::prelude::*` imports the six extension traits only. Data types
and PocketIC types remain explicit imports.

## PocketIC ownership and concurrency

Each test normally constructs and directly owns one fresh `PocketIc`:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{PocketIc, prelude::*};

fn read_counter(pocket_ic: &PocketIc, canister_id: Principal)
    -> Result<u64, ic_testkit::pic::CandidCallError>
{
    pocket_ic.query_candid(canister_id, "get", ())
}
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

fn increment_counter(pocket_ic: &PocketIc, canister_id: Principal)
    -> Result<(), ic_testkit::pic::CandidCallError>
{
    let _: () = pocket_ic.update_candid(canister_id, "increment", ())?;
    let value: u64 = pocket_ic.query_candid_as(
        canister_id,
        Principal::anonymous(),
        "get",
        (),
    )?;
    assert_eq!(value, 1); // This example assumes a counter initially at zero.
    Ok(())
}
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

Configure topology on the upstream builder and make the startup source and
deadline explicit:

```rust,no_run
use ic_testkit::pic::{
    PocketIc, PocketIcBuilder, PocketIcBuilderExt, PocketIcStartupConfig,
};
use std::{path::Path, time::Duration};

fn build_test_ic(server_binary: &Path) -> Result<PocketIc, ic_testkit::pic::PocketIcStartupError> {
    PocketIcBuilder::new()
        .with_application_subnet()
        .with_ii_subnet()
        .try_build(PocketIcStartupConfig::spawn(
            server_binary,
            Duration::from_secs(30),
        ))
}
```

`PocketIcStartupConfig::spawn` launches one exact, caller-resolved executable,
monitors it while waiting for its port file and while PocketIC creates the
instance, and terminates it if the complete deadline expires. A child exit,
readiness timeout, invalid port, spawn failure, builder panic, and instance
creation timeout remain distinct `PocketIcStartupError` variants. Captured
server output retains at most the first 16 KiB per stream as lossy UTF-8 and
appends an omitted-byte marker when truncated. No absolute hard TTL is applied
by default; PocketIC's activity-based soft TTL still bounds orphaned servers.
Call `with_server_hard_ttl` to opt into an absolute lifetime. After successful
one-shot construction, an internal reaper owns the child until it exits.

Use `PocketIcStartupConfig::connect(url, timeout)` for a caller-owned existing
server. Both modes set the server URL on the builder, preventing its implicit,
unbounded child-startup path. There is no zero-argument `try_build`, implicit
binary fallback, or hidden retry.

The deadline covers construction. After it returns, operations and instance
teardown follow upstream PocketIC behavior. In PocketIC 16, dropping an
instance waits for HTTP DELETE without a request deadline; the startup timeout
and operation request budget do not bound that wait. See the
[teardown reproduction](POCKET-IC.md#bounded-instance-teardown).

A serial suite can retain one testkit-owned server explicitly and construct
several bounded instances against it:

```rust,no_run
use ic_testkit::pic::{
    PocketIcBuilder, PocketIcBuilderExt, PocketIcStartupConfig,
};
use std::{path::Path, time::Duration};

fn run_serial_suite(server_binary: &Path) -> Result<(), ic_testkit::pic::PocketIcStartupError> {
    let server = PocketIcStartupConfig::spawn(server_binary, Duration::from_secs(30))
        .start_managed_server()?;
    eprintln!("managed server PID: {}", server.process_id());
    let first = PocketIcBuilder::new().with_application_subnet().try_build(
        PocketIcStartupConfig::connect(server.url(), Duration::from_secs(30)),
    )?;
    let second = PocketIcBuilder::new().with_application_subnet().try_build(
        PocketIcStartupConfig::connect(server.url(), Duration::from_secs(30)),
    )?;
    eprintln!("managed server stdout: {}", server.output().stdout());
    drop((first, second));
    Ok(())
}
```

`PocketIcManagedServer` owns the child and terminates and waits for it on drop.
On Unix, teardown also terminates descendants remaining in the server's owned
process group, including after startup failure or a natural server exit.
Its `process_id()` accessor identifies that child for caller-managed resource
monitoring. It does not include descendants or establish liveness; the OS may
reuse the PID after the child exits and is reaped. A copied PID retains no
ownership.
Managed startup creates a unique private temporary directory but leaves the
actual `--port-file` path absent for PocketIC to create. Output is retained as
bounded lossy UTF-8 for the handle lifetime. Keep the handle alive until every
instance connected through its URL has been dropped. The handle is
process-local: a CI topology spanning several Cargo or test-runner processes
should keep one runner-owned external server and give each process its URL via
bounded `PocketIcStartupConfig::connect` instead.

ic-testkit does not discover, download, cache, or validate server binaries.
Resolve the exact compatible executable before `spawn`, hash it when runtime
provenance matters, and record it with the report. `LATEST_SERVER_VERSION`
exposes the version expected by the client, and `PocketIc::get_server_url()`
exposes the active endpoint.

## Canister installation and fixtures

`InstallSpec` keeps generic install inputs and diagnostics together:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{
    InstallSpec, PocketIcBuilder, StandaloneCanisterFixture,
};

fn install_counter(wasm: Vec<u8>, init_bytes: Vec<u8>)
    -> Result<StandaloneCanisterFixture, Box<dyn std::error::Error>>
{
    let pocket_ic = PocketIcBuilder::new()
        .with_application_subnet()
        .build();
    let fixture = StandaloneCanisterFixture::try_install(
        pocket_ic,
        InstallSpec::new(wasm, init_bytes, 1_000_000_000_000)
            .install_sender(Principal::anonymous())
            .label("counter"),
    )?;

    let _value: u64 = fixture.query_candid("get", ())?;
    Ok(fixture)
}
```

The fixture owns exactly one `PocketIc`. `pocket_ic()` borrows it,
`canister_id()` identifies the installed canister, and `into_parts()` returns
both without changing instance ownership. Failed `try_install` returns a
`StandaloneCanisterInstallError` containing both the caller's instance and the
`CanisterInstallError`. Creation, cycle funding, and installation panics are
captured as structured failures. `phase()` identifies the failed operation;
`canister_id()` returns `None` if creation failed and `Some(id)` afterward.
The original operation error remains available through the error source chain,
including its transport classification.

For installation into an existing instance, import `CanisterInstallExt` and
use `try_create_and_install` or `try_create_and_install_many`. Batch installs
run in iterator order. If one fails, earlier installs remain, the failed
canister may already exist, and later installs are not attempted.

### Install-code rate limiting

`retry_install_code` retries only a structured
`ErrorCode::CanisterInstallCodeRateLimited` rejection:

```rust,no_run
use std::time::Duration;
use ic_testkit::pic::{CanisterInstallExt, PocketIc, RejectResponse, RetryPolicy};

fn retry_install(pocket_ic: &PocketIc, install_again: impl FnMut() -> Result<(), RejectResponse>)
    -> Result<(), RejectResponse>
{
    let policy = RetryPolicy::try_new(3, Duration::from_secs(60))
        .expect("nonzero attempts and cooldown");
    pocket_ic.retry_install_code(policy, install_again)
}
```

`max_attempts` includes the first call. Between retryable attempts, the helper
advances simulated time by the configured cooldown and executes two ticks. It
returns all non-rate-limit rejections unchanged.

## Snapshots and cached baselines

Snapshot capture validates duplicate ids before doing work, stores entries in
deterministic principal order, and cleans up earlier snapshots if a later
capture fails:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{PocketIc, PocketIcSnapshotExt, SnapshotRestoreFunding};

fn restore_pair(pocket_ic: &PocketIc, controller_id: Principal, canisters: [Principal; 2])
    -> Result<(), ic_testkit::pic::ControllerSnapshotError>
{
    let snapshots = pocket_ic.capture_controller_snapshots(controller_id, canisters)?;

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
    Ok(())
}
```

`ControllerSnapshotError` preserves rejected sender attempts, panic context,
and any cleanup failures. Restore never adds cycles unless `TopUpTo` is
explicitly selected. Callers must satisfy upstream snapshot preconditions;
these helpers do not stop or restart canisters.

Mixed-controller topologies can avoid expected rejected fallback calls by
selecting the exact sender for each canister:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{
    CanisterSnapshotTarget, PocketIc, PocketIcSnapshotExt,
};

fn restore_with_exact_senders(
    pocket_ic: &PocketIc, root_canister: Principal, root_controller: Principal,
    child_canister: Principal,
) -> Result<(), ic_testkit::pic::ControllerSnapshotError> {
    let snapshots = pocket_ic.capture_snapshots_with_senders([
        CanisterSnapshotTarget::new(root_canister, Some(root_controller)),
        CanisterSnapshotTarget::new(child_canister, None),
    ])?;
    pocket_ic.restore_snapshots_with_captured_senders(&snapshots)
}
```

The existing controller-based method remains useful when its ordered sender
fallback is desired. `CachedPocketIcBaseline::capture_with_senders` exposes the
same exact-sender policy for pooled baselines, and
`restore_with_captured_senders` restores it without requiring or attempting a
fallback controller.

`CachedPocketIcBaseline<T>` stores one owned instance, its snapshots, and
caller metadata. Use it as the fixture owned by a `PocketIcBaselineRecipe` in
`CachedPocketIcBaselinePool<Recipe>` with capacity one for exclusive sequential reuse, or
choose a larger capacity for independent parallel leases. The recipe declares
its reset requirements and recovery policy, and acquisition returns a typed
outcome rather than a cache-hit boolean. See the [complete recipe example](crates/ic-testkit/examples/multi_canister_baseline_pool.rs).

Heavy suites that need bounded parallelism can own a fixed-capacity pool of
independent standalone fixtures. Every populated slot has its own PocketIC
instance and baseline snapshot; a lease restores only that slot and dereferences
to the ordinary `StandaloneCanisterFixture` API:

```rust,no_run
use ic_testkit::pic::{
    CachedStandaloneCanisterFixturePool, StandaloneCanisterFixture,
    StandaloneFixturePoolOutcome,
};

static POOL: CachedStandaloneCanisterFixturePool<8> =
    CachedStandaloneCanisterFixturePool::new(build_fixture);

# fn build_fixture() -> StandaloneCanisterFixture { unimplemented!() }
fn read_pooled_counter() -> Result<u64, Box<dyn std::error::Error>>
{
    let (fixture, outcome) = POOL.acquire()?;
    let value: u64 = fixture.query_candid("get", ())?;
    assert!(matches!(
        outcome,
        StandaloneFixturePoolOutcome::Built { .. }
            | StandaloneFixturePoolOutcome::Restored { .. }
            | StandaloneFixturePoolOutcome::Rebuilt { .. }
    ));
    Ok(value)
}
```

`acquire` reports queue wait, fixture build and snapshot capture, restore,
stale teardown, and total time. Snapshot failures retain their partial timings,
while dead-transport recovery preserves both the restore and rebuild failure
when replacement capture also fails.

One pool owns one fixture builder. It must produce the same Wasm, init arguments,
topology, and seeded state whenever it runs; use a separate pool for a different
recipe. Setup performed by the builder is included in the captured baseline.
For builders that capture configuration, use
`CachedStandaloneCanisterFixturePool::<8, _>::new(move || build_fixture(&config))`.

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

```rust,no_run
use std::num::NonZeroUsize;
use ic_testkit::pic::{
    BaselinePoolOutcome, CachedPocketIcBaselinePool, PocketIcBaselineRecipe,
};

fn use_recipe<R: PocketIcBaselineRecipe>(recipe: R)
    -> Result<(), ic_testkit::pic::BaselinePoolError<R::Error>>
{
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        recipe,
    );

    let (baseline, outcome) = pool.acquire()?;
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Built { .. }
            | BaselinePoolOutcome::Restored { .. }
            | BaselinePoolOutcome::Rebuilt { .. }
    ));

    // Drive this recipe's topology through baseline.pocket_ic() and metadata().
    drop(baseline);
    Ok(())
}
```

A complete, compile-checked
[`PocketIcBaselineRecipe`](crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
shows a two-canister build, exact restore receipt, readiness boundary, invariant
validation, dead-transport classification, and cold/warm acquisition.

The recipe declares typed reset requirements and implements the exact reuse
sequence: restore every captured canister, reset non-snapshot state, drive the
topology to readiness, then validate final invariants. Construct requirements
with `ResetRequirements::try_new(cycle_policy, non_snapshot_requirements)`.
The pool checks the complete captured set and cycle policy directly against
`CanisterRestoreReceipt`; `ResetReceipt` reports only non-snapshot guarantees.
Every required policy must have an exact matching achievement before a warm
lease is returned. Built and restored slots
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

For a long suite, log `eprintln!("baseline={outcome}")` after each acquisition
and group the results by recipe. Compare cold `build` time with warm `restore`,
`reset`, `readiness`, and `validation` time. Repeated `rebuilt` outcomes identify
recovery costs; inspect their reasons before changing the recipe. Queue `wait`
time shows time spent waiting for a pool slot. Measure total suite time when
tuning capacity, since more concurrent instances also consume more host resources.

Recipes that wrap PocketIC's currently unstructured transport failures can use
`is_dead_pocket_ic_transport_error` in `classify_failure`, returning
`RebuildReason::DeadPocketIcTransport` when it matches and
`stage.default_rebuild_reason()` otherwise. The classifier searches the error
source chain and recognizes testkit transport error kinds directly. For
unstructured errors it requires a maintained reqwest error shape, an instance
URL and a recognized transport source. Generic `channel closed` or
`ConnectionRefused` text and bare I/O errors do not qualify. Use it only for
errors originating in PocketIC operations; it is a heuristic, not proof that
an instance or server has died.

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

`PocketIcDiagnosticsExt::collect_canister_diagnostics` accepts exact,
independent principals for canister status and log access. It attempts both
operations even if one fails and returns their results independently:

```rust,no_run
use ic_testkit::pic::{CanisterDiagnosticsRequest, PocketIc, PocketIcDiagnosticsExt};

fn diagnose(pocket_ic: &PocketIc, request: CanisterDiagnosticsRequest) {
    let report = pocket_ic.collect_canister_diagnostics(request);
    if report.status().is_err() || report.logs().is_err() {
        eprintln!("{}", report.render_compact());
    }
}
```

For several controllers or roles, wrap each exact request in a stable label and
collect them sequentially without fail-fast behavior:

```rust,no_run
use ic_testkit::pic::{
    CanisterDiagnosticsRequest, LabeledCanisterDiagnosticsRequest, PocketIc,
    PocketIcDiagnosticsExt,
};

fn diagnose_pair(
    pocket_ic: &PocketIc, root: CanisterDiagnosticsRequest, worker: CanisterDiagnosticsRequest,
) -> Result<(), ic_testkit::pic::CanisterDiagnosticsBatchContractError> {
    let requests = [
        LabeledCanisterDiagnosticsRequest::new("root", root),
        LabeledCanisterDiagnosticsRequest::new("worker", worker),
    ];
    let batch = pocket_ic.collect_canister_diagnostics_batch(&requests)?;
    if !batch.is_success() {
        eprintln!("{}", batch.render_compact());
    }
    Ok(())
}
```

The ordered batch retains every label and structured single-canister report.
Each entry retains its collection wall time and the report retains total
sequential batch time. An earlier rejection, transport failure, or panic does
not prevent later requests from being attempted, and no operation retries
anonymously. Empty or duplicate labels return
`CanisterDiagnosticsBatchContractError` before any diagnostic call starts.

Fetched log content is retained as bounded lossy UTF-8 rather than raw byte
arrays. `CanisterLogRenderLimits` controls the record and aggregate byte bounds;
the structured log result records omitted records and bytes. PocketIC transport
panics are captured per operation, and install-failure diagnostics remain
wrapped so a dead instance or failed renderer cannot replace the original
install error.

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
    ArtifactCachePrunePolicy, WasmBuildOutcome, WasmBuildSpec,
    build_wasm_canisters_cached, test_target_dir, workspace_root_for,
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
.with_cargo_profile_args(["--release", "--locked"])
.with_inherited_env(["COUNTER_SCHEMA_MODE"])
.with_additional_inputs(["config/counter-schema.json"])
.with_prune_policy(
    ArtifactCachePrunePolicy::new()
        .with_max_age(Duration::from_secs(14 * 24 * 60 * 60))
        .with_max_size_bytes(10 * 1024 * 1024 * 1024),
);

let outcome = build_wasm_canisters_cached(&spec)?;
eprintln!("immutable cache: {}", outcome.record().exact_cache_path().display());
match &outcome {
    WasmBuildOutcome::Built(record) => {
        eprintln!("built {} in {:?}", record.fingerprint(), record.timings().total());
    }
    WasmBuildOutcome::Reused(record) => {
        eprintln!("reused {}", record.fingerprint());
    }
}

// Keep the outcome alive while reading its retained, read-only artifact.
let wasm = std::fs::read(&outcome.record().artifacts()[0])?;

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

The fingerprint covers the selected package set and resolved dependency nodes,
enabled features, exact external source/checksum/revision identities, local
dependency sources, relevant inherited package fields, workspace profiles,
resolver and lint settings, Cargo configuration, Rust toolchain files, target
and profile arguments, Cargo and rustc identities, explicit child environment,
selected inherited environment, and caller-declared additional inputs. This
validated semantic workspace projection lets an unrelated host-only workspace
dependency or lockfile change retain the same exact Wasm key.

The complete workspace manifest and lockfile remain part of a separate,
conservative validation digest. They are rehashed around builds and attached
external artifact transactions, so any raw workspace mutation during an
acquisition is rejected even when its semantic projection is unchanged.
`ResolvedCargoBuildInputs` exposes both `input_digest` (semantic cache identity)
and `validation_digest` (mutation guard). Projection falls back to the complete
manifest/lockfile identity when a selected local package is rooted at or cannot
be normalized beneath the workspace root. Cargo configuration discovery follows
the files Cargo can read from the invocation directory and its ancestors plus
the effective Cargo home. The extensionless `.cargo/config` takes precedence
when both supported names exist, and recursive required or optional `include`
entries are exact inputs. Build scripts that read application-specific
environment variables or files outside Cargo's package graph must still declare
them on the spec. Package inputs are content-hashed exactly but conservatively;
exact hashing does not promise that every file in the package closure is
semantically relevant to a build.

Calls sharing a target directory coordinate through one process lock. A cache
hit requires every expected nonempty Wasm artifact to carry the exact atomic
stamp for both the current fingerprint and artifact content. Exact builds are
retained under fingerprint-specific Cargo target directories, so a prior spec
can be materialized again after another spec used the caller-facing output.
`WasmBuildRecord::exact_cache_path` exposes the selected immutable directory;
CI cache collection does not need to reconstruct its private on-disk layout.
Both `WasmBuildRecord::artifacts()` and `ArtifactCacheRecord::artifacts()` return
read-only exact-cache paths retained by the record. Keep the outcome, batch
report, or a cloned record alive through post-link commit and final reading or
staging. A copied path or artifact descriptor alone does not retain ownership.
Configured compiler and post-link destinations remain mutable. Never transform
an exact artifact in place; write into the next transaction's staging outputs.
See the `artifacts` module documentation for a complete handoff example.

Pruning skips all retained entries, including those owned by another process.
The last record clone dropping or its process exiting releases retention; the
next maintenance pass can reclaim the entry. Cache bounds can be temporarily
exceeded while consumers hold records. Batch successes remain independently
retained when later entries fail, so keep their records instead of reducing a
report to indexes. Manual deletion or mutation of exact-cache files is outside
this contract.

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
`ArtifactCacheMaintenance` result plus its duration on the successful build
record. The standalone pruning function remains available when maintenance
failure should be returned directly.

High-frequency suites can use `with_prune_policy_at_most_every` to retain the
same limits while replacing repeated hit-path directory scans with a small
namespace marker check. The original `with_prune_policy` continues to run on
every successful acquisition.

`WasmBuildTimings::input_resolution` returns the structured Cargo/rustc
identity, Cargo metadata, input discovery, content hashing, and total timing.

Relative exact target paths use the caller's working directory, matching cache
filesystem operations. Relative shared targets use the workspace root. Select
these paths through `WasmBuildSpec`; overriding `CARGO_TARGET_DIR` through
`with_extra_env` or passing `--target-dir` through `with_cargo_profile_args`
returns `InvalidSpec` before acquisition.

The profile output directory must match the Cargo arguments: default builds and
`--profile dev` or `--profile test` use `debug`; `--release`, `-r`, and
`--profile bench` use `release`; custom profiles use their own name. A mismatch
returns `InvalidSpec` before input resolution or cache work. Setting the output
directory alone does not select a Cargo profile.

Wasm acquisition always builds package libraries with Cargo's `--lib` flag.
Binary, example, test, bench, and `--all-targets` selectors return `InvalidSpec`
before input resolution or cache work. Run other targets separately; they can
share the library's output filename and overwrite the canister Wasm. An explicit
`--lib` in `with_cargo_profile_args` remains accepted.

Acquired packages must declare a `cdylib` library with the same name as the
package, matching the expected `<package>.wasm` output. Acquisition validates
this declaration from Cargo metadata before reuse, compilation, or scheduled
shared-target maintenance. Libraries that only produce an `rlib`, renamed
libraries, and binary-only packages return `InvalidSpec`. Input-only snapshots
from `resolve_cargo_build_inputs` can still describe arbitrary Cargo targets
for transactional artifact recipes.

Source-edit-heavy suites can opt into a caller-owned shared Cargo target while
retaining exact immutable final Wasm entries:

```rust,no_run
use ic_testkit::artifacts::{
    SharedIncrementalTargetPrunePolicy, WasmBuildSpec, build_wasm_canisters_cached,
    inspect_shared_incremental_target,
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
.with_shared_incremental_target(&shared_incremental)
.with_shared_incremental_target_maintenance_at_most_every(
    SharedIncrementalTargetPrunePolicy::new()
        .with_max_age(std::time::Duration::from_secs(7 * 24 * 60 * 60))
        .with_max_size_bytes(4 * 1024 * 1024 * 1024),
    std::time::Duration::from_secs(24 * 60 * 60),
);
let outcome = build_wasm_canisters_cached(&spec)?;
let shared_usage = inspect_shared_incremental_target(&spec)?
    .expect("configured acquisition creates the shared target");
let maintenance = outcome
    .record()
    .shared_incremental_maintenance()
    .expect("shared maintenance was configured");
eprintln!(
    "shared_bytes={} maintenance={} {}",
    shared_usage.logical_size_bytes(),
    maintenance,
    outcome,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Without integrated shared-target maintenance, an exact hit never acquires or
mutates the shared target. On a miss, callers using that target serialize
across processes; Cargo builds incrementally, then `ic-testkit` re-resolves
every exact input and publishes only the final Wasm files. A source race
rejects publication, while a failed build preserves the caller-owned
incremental state. Retention owns only immutable output entries, not the
shared Cargo target. `inspect_shared_incremental_target` takes the same
cross-process lock and reports its canonical path, logical size, last recorded
build use, and lock wait without removing caller-owned Cargo state.
`maintain_shared_incremental_target` is an explicit whole-target operation: if
an age or size threshold is exceeded it removes every other child while
retaining the root, cache tag, and live coordination lock. Callers must not
colocate unrelated data that needs to survive a clear. Before clearing, the
exact Cargo resolver rejects a target that overlaps source, configuration, or
additional inputs, or whose clearing could reach exact cached artifacts.
The exact and shared target may use the same container, whose `.ic-testkit`
metadata remains preserved. Independent high-frequency callers can use
`maintain_shared_incremental_target_at_most_every`; matching recent passes
coordinate through a small cross-process marker and skip both Cargo input
resolution and whole-target traversal. Missing targets remain uncreated,
policy changes are immediately due, and a zero interval always runs. When
retention always accompanies a Wasm acquisition, configure
`with_shared_incremental_target_maintenance_at_most_every` instead. It creates
and coordinates the target even on exact hits, reuses the acquisition's input
resolution for due maintenance, and reports the result through the build
record and structured progress events. Integrated maintenance is strict by
default. Consumers that prefer a usable Wasm acquisition over successful
retention can supply an explicit best-effort configuration:

```rust,no_run
use ic_testkit::artifacts::{
    SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode,
    SharedIncrementalTargetPrunePolicy, WasmBuildSpec,
};
use std::time::Duration;

# let workspace = std::path::PathBuf::from(".");
# let target = workspace.join("target/pic-wasm");
let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
    SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(4 * 1024 * 1024 * 1024),
    Duration::from_secs(24 * 60 * 60),
)
.with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
let spec = WasmBuildSpec::new(
    &workspace,
    &target,
    &["counter_canister"],
    "debug",
)
.with_shared_incremental_target(target.join("shared"))
.with_shared_incremental_target_maintenance(maintenance);
# let _ = spec;
```

A best-effort failure is retained as
`SharedIncrementalTargetMaintenanceOutcome::Failed` and does not record a
successful schedule marker. Spec validation, Cargo input resolution, builds,
source-race detection, and artifact publication remain strict. Configuration
getters support policy assertions without performing work.

Long input resolution, lock waits, maintenance, Cargo builds, and artifact
publication can expose synchronous structured progress without changing the
existing silent API:

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
        if let WasmBuildProgressEvent::Heartbeat { phase, elapsed } = event {
            eprintln!("{phase} is still active after {elapsed:?}");
        }
    },
)?;
eprintln!("{outcome}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Observers run synchronously on the build thread and should return promptly.
Raw output events preserve non-UTF-8 bytes; output is still captured in
`WasmBuildError` when forwarding is disabled. An observer panic joins active
phase work, terminates and waits for a running Cargo build, and then performs
normal lock and incomplete-entry cleanup.

Suites that require standalone feature resolution for several variants can
batch independent specs without changing Cargo feature semantics:

```rust,no_run
use ic_testkit::artifacts::{
    LabeledWasmBuildSpec, WasmBuildSpec, build_wasm_canisters_cached_batch,
};

# let workspace = std::path::PathBuf::from(".");
# let target = workspace.join("target/pic-wasm");
let specs = [
    LabeledWasmBuildSpec::new(
        "role-a",
        WasmBuildSpec::new(&workspace, &target, &["role_a"], "debug"),
    ),
    LabeledWasmBuildSpec::new(
        "role-b",
        WasmBuildSpec::new(&workspace, &target, &["role_b"], "debug"),
    ),
];
let batch = build_wasm_canisters_cached_batch(&specs)?;
eprintln!("{batch}");
for failure in batch.failures() {
    eprintln!(
        "build {} ({}) failed in {:?} after {:?}: {}",
        failure.label(),
        failure.index(),
        failure.phase(),
        failure.entry_elapsed(),
        failure.error(),
    );
    eprintln!("partial timings: {:?}", failure.timings());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

The batch is sequential and collect-all: every independent failure is returned
in one report, but entries are not run simultaneously. Every
`LabeledWasmBuildSpec` label
must be nonempty and unique; invalid structure returns
`WasmBuildBatchContractError` before resolution, progress, or build work. Its
`WasmBuildBatchReport` retains canonical ordered entries and provides
structured labeled `outcomes` and `failures` iterators. Labels are reporting
identity only and never affect exact cache keys. Progress and integrated
maintenance entries carry the same label. Every entry uses the ordinary
single-spec implementation and its own exact identity, locks, policy, and
Cargo command; packages are never collapsed into one invocation that could
unify shared dependency features.
When several packages are intentionally built together with Cargo's normal
shared feature resolution, keep them in one multi-package `WasmBuildSpec`;
splitting that build into per-package batch entries would repeat Cargo work and
may change feature resolution.

Resolution-compatible entries share one workspace/toolchain snapshot: Cargo
and rustc identity, Cargo metadata, input discovery, and memoized per-root
content digests are reused while each spec retains its standalone fingerprint
and Cargo invocation. A different workspace, tool environment, or
metadata-affecting feature argument starts a separate snapshot so independent
feature semantics remain intact.

`WasmBuildBatchReport::metrics` aggregates built, exact-cache reused, and failed
counts; input-resolution runs and compatible-snapshot reuses; and summed
successful acquisition timings. The structured input-resolution total makes
distinct feature-resolution costs visible while reused snapshots remain
counted explicitly, including reuse from an explicit session or prepared
concurrent snapshot. `entry_elapsed` retains wall time for every ordered entry,
including failures; `failures` returns structured entries that bundle the
label, index, error, primary failed phase, partial phase timings, and elapsed
time. Successful entries continue to expose detailed phase timings through
their build records.

Repeated batches can reuse exact input snapshots when the caller owns a real
write-exclusion boundary for every build input:

```rust,no_run
use ic_testkit::artifacts::{
    LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildSession,
};
use std::sync::Mutex;

# let specs: Vec<LabeledWasmBuildSpec> = Vec::new();
// Every source/config/tool/environment writer must coordinate on this lock.
let source_write_exclusion = Mutex::new(());
let source_guard = source_write_exclusion.lock().expect("acquire source lease");
let mut session = WasmBuildSession::assume_sources_immutable(&source_guard);
let first = session.build_batch(&specs, WasmBuildBatchConfig::new())?;
let second = session.build_batch(&specs, WasmBuildBatchConfig::new())?;
eprintln!("first={first}; second={second}; session={:?}", session.metrics());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The `assume_sources_immutable` name is an explicit caller assertion. The
borrowed guard is only a lifetime boundary, not filesystem locking or guard
provenance validation performed by `ic-testkit`; supplying an unrelated value
violates the contract and can reuse stale inputs. The lease covers Cargo and
rustc executables, manifests, Cargo configuration, discovered sources,
declared additional inputs, and relevant environment values. A detected input
race permanently invalidates the session, clears pending snapshots, and makes
later calls return
`SourceLeaseInvalidated`. Ordinary batch functions still resolve current inputs
per call, and there is no silent process-global cache. The complete contract is
recorded in the
[`0.7` orchestration design](docs/design/0.7-artifact-orchestration/0.7-design.md#explicit-cross-call-immutable-build-session).

When independent callers need the same resolution concurrently, prepare their
complete exact specification set before starting readers:

```rust,no_run
use ic_testkit::artifacts::{
    LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildInputSnapshot,
    WasmBuildSpec,
};
use std::sync::Mutex;

# let local_test: Vec<LabeledWasmBuildSpec> = Vec::new();
# let production: Vec<LabeledWasmBuildSpec> = Vec::new();
let prepared_specs = local_test
    .iter()
    .chain(&production)
    .map(|entry| entry.spec().clone())
    .collect::<Vec<WasmBuildSpec>>();
// Every source/config/tool/environment writer must coordinate on this lock.
let source_write_exclusion = Mutex::new(());
let source_guard = source_write_exclusion.lock().expect("acquire source lease");
let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
    &source_guard,
    &prepared_specs,
)?;
let (local_report, production_report) = std::thread::scope(|scope| {
    let local = scope.spawn(|| {
        snapshot.build_batch(&local_test, WasmBuildBatchConfig::new())
    });
    let production = scope.spawn(|| {
        snapshot.build_batch(&production, WasmBuildBatchConfig::new())
    });
    (
        local.join().expect("join LocalTest reader"),
        production.join().expect("join Production reader"),
    )
});
local_report?;
production_report?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Preparation is closed: a reader requesting an exact spec absent from
`prepared_specs` receives `SpecificationNotPrepared` before progress or build
work; it never extends the snapshot or hashes a new path. A detected post-build
source race invalidates the snapshot for every later reader. Publication takes
a shared read boundary against invalidation, so a publication already at that
boundary completes before invalidation while every later publication is
rejected. `WasmBuildInputSnapshot::metrics` exposes preparation work,
cumulative reader reuse, and invalidation; batch metrics count
`input_resolution_prepared_reuses` separately.

Each batch remains sequential. Separate snapshot readers may overlap, but
shared incremental Cargo targets still participate in their existing locks;
use isolated targets to retain useful Cargo parallelism. There is no built-in
parallel scheduler, ambient cache, or mutation-owning workspace abstraction.
Callers without a genuine source write-exclusion guard must keep ordinary
per-call resolution.

When independent specs share incremental targets, batch-owned maintenance
removes the caller convention of modifying the first spec:

```rust,no_run
use ic_testkit::artifacts::{
    LabeledWasmBuildSpec,
    SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode,
    SharedIncrementalTargetPrunePolicy, WasmBuildBatchConfig,
    build_wasm_canisters_cached_batch_with_config,
};
use std::time::Duration;

# let specs: Vec<LabeledWasmBuildSpec> = Vec::new();
let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
    SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(4 * 1024 * 1024 * 1024),
    Duration::from_secs(24 * 60 * 60),
)
.with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
let batch = build_wasm_canisters_cached_batch_with_config(
    &specs,
    WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance),
)?;
for entry in batch.shared_incremental_maintenance_outcomes() {
    eprintln!("build {} ({}): {}", entry.label(), entry.index(), entry.outcome());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Maintenance is attached to the first specification for each distinct resolved
shared-target directory. Relative paths are workspace-relative, and filesystem
aliases of the same directory share one maintenance attempt. Isolated specs
are unaffected. A specification
that also configures per-spec integrated maintenance receives a labeled,
indexed error, and later entries still run.

`ResolvedCargoBuildInputs::is_current` resolves the semantic fingerprint again,
while `is_content_current` cheaply rehashes the conservative validation paths.
The generic builders preserve dynamic and non-UTF-8 argument and environment
bytes. Use
`resolve_executable` to turn a bare `PATH` program into a canonical executable
file before declaring it through `ArtifactCacheSpec::with_tool`.

External transforms derived from a Cargo package closure can pass both the
build spec and its resolved snapshot to
`ArtifactCacheSpec::with_cargo_build_inputs`. The semantic Cargo fingerprint
becomes transactional identity, while the conservative Cargo-aware input paths
and generated-state exclusions are revalidated during preparation, hit
materialization, and commit. This avoids duplicating Cargo discovery as broad
workspace scans while still rejecting source, workspace, or toolchain races.

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
.with_arguments(["--optimize-for-size"])
.with_environment([("OPTIMIZER_MODE", "deterministic")])
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
            .env("OPTIMIZER_MODE", "deterministic")
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!("optimizer failed with {status}")).into());
        }
        transaction.commit()?
    }
};
eprintln!("artifact key: {}", outcome.record().key());
// Retain the outcome until reading or staging the exact output is complete.
let optimized_wasm = std::fs::read(outcome.record().artifacts()[0].path())?;
# let _ = optimized_wasm;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Multiple independent external recipes can use
`build_artifact_caches_batch`. Every `ArtifactCacheSpec` is wrapped in a
`LabeledArtifactCacheSpec`; labels must be nonempty and unique and are retained
in callbacks and ordered report entries. Labels are composition metadata, not
cache identity. A structural label error rejects the batch before work starts.

The spec describes command identity; it does not execute a tool or apply its
declared arguments and environment to `Command`. The caller's build callback
must run the declared recipe, as in the example above.

The callback receives only cache misses and only one live transaction at a
time, avoiding self-deadlock when specs share a coordination scope. Callback
failures synchronously abort the current staging directory, retain their label,
and do not stop later independent entries. `ArtifactCacheBatchReport::metrics`
aggregates built, reused, and failed counts plus all successful acquisition
timings. Every entry retains wall time; failed entries additionally distinguish
preparation, callback, explicit abort cleanup, and commit time. Commit timing
includes commit-owned failure cleanup. The operation is deliberately not atomic
across specs: use one `ArtifactCacheSpec` with several outputs when all
artifacts must publish as one transaction.

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
outside the cache, resolve to distinct non-nested paths, and not overlap a
declared input
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
built artifact; an unstamped artifact is conservatively stale. This lightweight
stamp records input identity only: it does not hash the output bytes, acquire
producer locks, or retain artifact paths against replacement. The caller must
coordinate the build and verify inputs stayed unchanged before stamping. Use
`ArtifactCacheSpec` when a pipeline needs transactional publication, output
content validation, and retained consumption.

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
let aggregates = aggregate_benchmark_spans(&spans.spans).expect("aggregate benchmark spans");

assert_eq!(aggregates.rows[0].span_label, "storage/write");
```

`comparison.csv` includes a `scope` column: `suite` identifies a named suite,
and `all` identifies the cross-suite aggregate. A suite named `ALL` remains
distinct from the global aggregate. Run indices have a minimum width of four
digits, continue beyond `9999`, and fail explicitly if the numeric range is
exhausted; previous-run discovery orders the indices numerically.

`aggregate_benchmark_spans` returns `Result<BenchmarkAggregateReport,
BenchmarkAggregateError>`. Counter totals and run counts use checked arithmetic;
overflow rejects the complete aggregation rather than returning partial or
wrapped totals. The error identifies the suite, span label and counter, and
`is_all_suites()` distinguishes a cross-suite failure from a named suite `ALL`.

The report writer emits raw events, spans, aggregates, comparisons, malformed
and unpaired markers, `bench-summary.md`, and `metadata.json`. Run-directory
helpers discover compatible previous runs but do not reserve paths. Concurrent
writers must use unique destinations or synchronize only the shared output
path.

An authored suite named `ALL` remains distinct from the internal cross-suite
aggregate. Use `BenchmarkAggregateRow::is_all_suites()` to distinguish them.
Aggregate and comparison rows, and aggregate errors, expose their authoritative
label through `suite()`. Read aggregate averages with `average()`; they are
derived from the row's totals and run count rather than stored separately.
These accessors replace the former `suite` and `average` fields.

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
under [`docs/design`](docs/README.md#historical-design-records)
are historical decision records; current behavior is documented here and in
rustdoc.

## Toolchains and checks

- Published MSRV: Rust 1.88
- Repository toolchain: Rust 1.99
- PocketIC client/server line: 16

For a documentation change, build rustdoc and run its examples:

```bash
make docs-check
cargo test -p ic-testkit --locked --doc
```

For code changes, run tests for the affected module or integration target, for
example:

```bash
cargo test -p ic-testkit --locked --lib artifacts::icp::tests
cargo test -p ic-testkit --locked --test benchmark
```

Live PocketIC tests need a compatible server. Upstream `PocketIc::new()` and
`PocketIcBuilder::build()` may download it when `POCKET_IC_BIN` is unset;
set `POCKET_IC_BIN=/path/to/pocket-ic` to use a local binary. Wasm build tests
also require the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
```

The maintainer runs the complete pre-push and release gates:

```bash
make ci
make release-check
```

The release gate includes formatting, native and Wasm checks, warnings-denied
Clippy, rustdoc, unit and live PocketIC tests, package verification, publish
dry-run, and the Rust 1.88 MSRV check. The test suite builds its own canister
artifacts; CI runs the canister integration target once through `make test`.

CI, release, and publish flows preserve Cargo build artifacts for incremental
reuse. Only the standalone, manually invoked `make clean` target runs
`cargo clean`; it is not part of `CI_TARGETS` or any release recipe.

To exercise bounded managed startup against one exact caller-provided PocketIC
server binary without invoking any downloader or resolver:

```bash
IC_TESTKIT_POCKET_IC_SERVER=/path/to/pocket-ic \
  cargo test -p ic-testkit --lib \
  pic::startup::tests::caller_provided_server_publishes_port_constructs_instance_and_cleans_up \
  -- --ignored --exact
```

The repository adopts a [reviewed Shared Tooling snapshot](.shared-tooling.snapshot)
through [AGENTS.md](AGENTS.md). `make shared-tooling-check` verifies its contents.
Host prerequisites and native qualification commands are in [docs/hosts.md](docs/hosts.md).

## Releases

### Upgrading from an earlier release

The `0.16.0` release requires acquired packages to declare a same-name `cdylib`
library. Its [migration guide](crates/ic-testkit/CHANGELOG.md#0160) describes the
library-only build contract and rejected target selectors.

The `0.14.0` release makes standalone fixture pools own their fixture builder.
See the packaged [0.14.0 migration guide](crates/ic-testkit/CHANGELOG.md#0140)
for the constructor and acquisition API changes.

The `0.13.0` release simplifies benchmark rows and baseline reset requirements;
its [migration guide](crates/ic-testkit/CHANGELOG.md#0130) records those source
API changes. The packaged changelog also covers the
[0.11.0 API and report-schema changes](crates/ic-testkit/CHANGELOG.md#0110) and
the [0.10 retained-artifact migration](crates/ic-testkit/CHANGELOG.md#0100),
including earlier pre-1.0 hard cuts.

### Publishing a release

Commit the changelog entry for the target version and start from a clean
worktree, then run:

```bash
make release-patch
# or: make release-minor
```

The guarded flow runs the local `make ci` gate before bumping the version,
then bumps and stages the version files, creates the release commit and tag,
and pushes both. GitHub Actions runs CI for the pushed tag. After tag CI
succeeds:

```bash
make publish
```

Release CI invokes the ordinary `make ci` gate with the caller's environment;
it preserves Cargo artifacts and does not create or remove a private temporary
directory around upstream background servers. Managed PocketIC handles retain
ownership of their child processes. See
[temporary-directory ownership](POCKET-IC.md#temporary-directory-ownership)
and the [host matrix](docs/hosts.md) for portable checks and qualification gaps.
The version helper checks offline dependency availability and updates workspace
lockfile versions without regenerating external dependency selections.

Publication requires a clean worktree and a matching `v<version>` tag at
`HEAD`. Re-running `make publish` is safe when that version already exists on
crates.io.
