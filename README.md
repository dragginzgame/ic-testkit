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
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/PocketIC-16.1-green.svg" alt="PocketIC"></a>
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
ic-testkit = "0.42"
```

Canister crates that emit benchmark markers can add the same version under
`[dependencies]` and use `ic_testkit::performance`.

The crate supports Rust 1.88 and uses PocketIC 16. The `pocket_ic` and `pic` modules are host-only. `benchmark`, `performance`, and `Fake` are
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
[IC Build transactional transform](https://github.com/dragginzgame/ic-build/blob/main/crates/ic-build/examples/transactional_artifact_cache.rs)
examples for complete developer-facing recipes.

> The rest of this README is a developer reference. If you only needed the
> product-level explanation, the sections above describe the purpose and value
> of ic-testkit.

### Choose the right isolation level

Not every test should use a cache. Choose the narrowest fixture that preserves
the behavior under test:



| Fixture | Best fit |
| --- | --- |
| Fresh caller-owned `PocketIc` | Installation, upgrade, topology, teardown, time, cycle-accounting, and snapshot behavior |
| `StandaloneCanisterFixture` | Owns a single installed canister; use explicit-ID calls, without cross-test reuse |
| `CachedPocketIcBaselinePool<StandaloneCanisterRecipe<_>>` | Repeated single-canister scenarios whose relevant state is captured by one baseline snapshot |
| `CachedPocketIcBaselinePool<R>` | Expensive multi-canister topologies with an explicit recipe for snapshots, non-snapshot state, readiness, and invariants |

Pools bound resource use; they do not turn incomplete reset coverage into safe
reuse. Keep a test on a fresh instance when its mutations cannot be completely
restored or validated.

## API at a glance

| Area | Main surface | Value added by ic-testkit |
| --- | --- | --- |
| PocketIC runtime | `pocket_ic`; `pic::{PocketIc, PocketIcBuilder}` | Complete upstream crate escape hatch plus focused convenience re-exports; no wrapper |
| Startup | `PocketIcStartupConfig`, `PocketIcManagedServer` | Explicit bounded spawn/connect policy, owned shared-server lifecycle, and structured failures |
| Calls | `CandidCallExt` | Candid encoding/decoding, contextual errors, preserved rejections |
| Installation | `CanisterInstallExt`, `InstallSpec` | Generic install policy, diagnostics, structured rate-limit retry |
| Standalone fixtures | `StandaloneCanisterFixture` | Owns one caller-built instance and one installed canister id |
| Snapshots | `PocketIcSnapshotExt`, `CachedPocketIcBaseline` | Ordered transactional capture, explicit restore funding, scoped caching |
| Fixture pools | `CachedPocketIcBaselinePool`, `StandaloneCanisterRecipe` | One bounded lifecycle for single- and multi-canister recipes |
| Diagnostics | `PocketIcDiagnosticsExt` | Controller-aware structured status and bounded log reporting |
| Time | `PocketIcTimeExt` | Nanoseconds-since-epoch conversion only |
| Readiness progression | `tick_until` | Bounded advance/tick rounds with a caller-owned predicate |
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
    PocketIc, PocketIcBuilder, PocketIcInstance, PocketIcStartupConfig,
};
use std::{path::Path, time::Duration};

fn build_test_ic(server_binary: &Path) -> Result<PocketIcInstance<PocketIc>, ic_testkit::pic::PocketIcStartupError> {
    PocketIcStartupConfig::spawn(server_binary, Duration::from_secs(30))
        .build_instance(PocketIcBuilder::new().with_application_subnet().with_ii_subnet())
}
```

`PocketIcStartupConfig::spawn` launches one exact, caller-resolved executable,
monitors it while waiting for its port file and while PocketIC creates the
instance, and terminates it if the complete deadline expires. A child exit,
readiness timeout, invalid port, spawn failure, builder panic, and instance
creation timeout remain distinct `PocketIcStartupFailure` cases, accessed through
`PocketIcStartupError::failure()`. Captured
server output retains at most the first 16 KiB per stream as lossy UTF-8 and
appends an omitted-byte marker when truncated. Read it through `error.output()`;
`error.command_cleanup()` and `error.server_cleanup()` expose separate typed
Host cleanup reports. Cleanup does not replace the original cause or native
error source. Consumers must replace matches on the former error enum with
matches on `error.failure()`. The CLI preserves a failed command's status when
server teardown also fails and reports failure if a successful command's server
cannot be cleaned up. No absolute hard TTL is applied
by default; PocketIC's activity-based soft TTL still bounds orphaned servers.
Call `with_server_hard_ttl` to opt into an absolute lifetime.

For a fresh server whose lifetime ends with the fixture, use
`config.build_instance(builder)`. It returns `PocketIcInstance<PocketIc>` with
public `instance` and `server` fields. Its field order deletes the synchronous
instance before terminating and reaping the server. This replaces repeated
start-server/connect/build composition while preserving the caller's topology,
state paths, output files and lifetime policy. Pass the whole ownership record to
standalone installation or baseline capture; their retained owner keeps the server
alive. Destructuring transfers responsibility for teardown ordering to the caller.

Async consumers use `config.build_instance_async(builder).await` on a Tokio
runtime with time enabled. The result exposes the upstream nonblocking instance
and retains the same managed server handle. Readiness and HTTP waits yield;
process spawning, small file reads and exact child cleanup remain synchronous.
The deadline cannot preempt synchronous work within an upstream future poll,
including read-only state copying. Drop the construction future to cancel local
work and reap its owned server; select output files to retain cancellation
logs. Cancellation of a borrowed connection leaves server ownership with the
caller and cannot reconcile an instance created before timeout or a lost reply.

The async instance requires explicit deletion while its server is retained:

```rust,no_run
# async fn example(binary: &std::path::Path) -> Result<(), ic_testkit::pic::PocketIcStartupError> {
use ic_testkit::pic::{PocketIcBuilder, PocketIcStartupConfig};
use std::time::Duration;
let owned = PocketIcStartupConfig::spawn(binary, Duration::from_secs(60))
    .build_instance_async(PocketIcBuilder::new().with_application_subnet()).await?;
let canister = owned.instance.create_canister().await;
assert!(owned.instance.canister_exists(canister).await);
owned.instance.drop().await;
drop(owned.server);
# Ok(())
# }
```

Neither construction method adds an instance-operation or deletion deadline.
The synchronous upstream drop limitation below still applies.

Use `PocketIcStartupConfig::connect(url, timeout)` for a caller-owned existing
server. Both modes set the server URL on the builder, preventing its implicit,
unbounded child-startup path. There is no implicit binary fallback or hidden retry.

The deadline covers construction. After it returns, operations and instance
teardown follow upstream PocketIC behavior. In PocketIC 16, dropping an
instance waits for HTTP DELETE without a request deadline; the startup timeout
and operation request budget do not bound that wait. See the
[teardown reproduction](POCKET-IC.md#bounded-instance-teardown).

A serial suite can retain one testkit-owned server explicitly and construct
several bounded instances against it:

```rust,no_run
use ic_testkit::pic::{
    PocketIcBuilder, PocketIcStartupConfig,
};
use std::{path::Path, time::Duration};

fn run_serial_suite(server_binary: &Path) -> Result<(), ic_testkit::pic::PocketIcStartupError> {
    let server = PocketIcStartupConfig::spawn(server_binary, Duration::from_secs(30))
        .start_managed_server()?;
    eprintln!("managed server PID: {}", server.process_id());
    let first = PocketIcStartupConfig::connect(server.url(), Duration::from_secs(30))
        .build_instance(PocketIcBuilder::new().with_application_subnet())?;
    let second = PocketIcStartupConfig::connect(server.url(), Duration::from_secs(30))
        .build_instance(PocketIcBuilder::new().with_application_subnet())?;
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
bounded lossy UTF-8 for the handle lifetime. Startup readers require regular
files and use nonblocking opens on Unix so a replaced FIFO cannot stall startup
or output capture. Non-regular port files return `PocketIcStartupFailure::Io` with
`InvalidData`; unreadable output streams are omitted.
Keep the handle alive until every instance connected through its URL has been
dropped. For suites spanning several Cargo or test-runner processes, keep one
runner-owned server and use the shared environment contract in each process:

```rust,no_run
use ic_testkit::pic::{PocketIcBuilder, PocketIcStartupConfig};
use std::time::Duration;

let config = PocketIcStartupConfig::from_env(Duration::from_secs(30))?;
let owned = config.build_instance(PocketIcBuilder::new().with_application_subnet())?;
# Ok::<(), ic_testkit::pic::PocketIcStartupError>(())
```

`from_env` prefers `IC_TESTKIT_POCKET_IC_URL`, then `POCKET_IC_BIN`. An empty or
invalid selected value fails; it never falls back to another variable or searches
a cache. URL mode connects without taking server ownership. Binary mode requires
a successful, bounded `--version` check satisfying Testkit's stable 16.x protocol
policy (`>=16.0.0,<17.0.0`), independently of `LATEST_SERVER_VERSION`; the
probe and subsequent startup each have the supplied timeout budget. The probe
owns a process group and stops same-group wrapper descendants on completion or
failure; it is not a background-launch contract.

Install the command with `cargo install --locked ic-testkit`, or run it from this
checkout with `cargo run --locked -p ic-testkit --bin ic-testkit-server --`:

```bash
ic-testkit-server setup
ic-testkit-server check
ic-testkit-server run --ttl 900 -- cargo test --locked -p my-integration-tests
```

`setup` explicitly downloads Testkit's reviewed PocketIC 16.1.0 asset over HTTPS,
verifies its pinned SHA-256 before bounded decoding, and admits the executable's
exact bytes and version before publishing. `check` verifies the retained archive
and installed executable offline. Both print only the absolute executable path
to stdout; diagnostics and retained setup-evidence locations go to stderr.
Use `--directory DIRECTORY` on setup/check/run to select another physical root;
the default is `.tools/ic-testkit-server` relative to the working directory.
Only Linux x86-64, macOS Intel and macOS Apple Silicon have reviewed assets.
Provisioning requires `curl`; offline checking does not.

`run` without a server environment selection checks that prepared bundle and
exports both its `POCKET_IC_BIN` and the governed server URL to the command.
It never installs or updates a server. An explicit `--directory` conflicts with
an explicit binary/URL environment selection. Low-level binary overrides still
require protocol admission; borrowed URLs retain their existing topology contract.
Client dependency updates do not change Testkit's server selection automatically.

Setup serializes cooperating installers, rechecks an existing bundle before any
download and refuses a changed or redirected installation. Failed/interrupted
attempts and previous bundles are preserved. During a download, SIGINT, SIGTERM
and SIGHUP cancel the owned downloader group, retain the partial archive and
captured failure evidence, and return `128 + signal`. Select a fresh root to replace an
invalid retained installation; no automatic repair, removal or format migration
occurs. The caller must control the physical root and exclude untrusted namespace
writers. Parent traversal and symlinked directory components are refused. Checks
create no installation files; the version probe executes authenticated bytes.

Shared Tooling 0.2.0 prepares five generic IC tools; PocketIC is provisioned
only by Testkit. Run `make install-tools` during explicit setup
and `make tools-check` for offline verification; these aggregates include the
owning PocketIC targets after the complete common toolset. Existing shared
bundles, receipts and failed attempts remain intact; rerun `make install-ic-tools`
to activate the new five-tool selection. The ownership handoff is tracked in
[#38](https://github.com/dragginzgame/ic-testkit/issues/38).

The runner exports `IC_TESTKIT_POCKET_IC_URL` to the command, retains the managed
server until completion, inherits terminal IO and preserves the command's exit
status. If the owned server exits while the command is pending, the runner
terminates the command group and reports the server failure, including TTL expiry.
SIGINT, SIGTERM and SIGHUP clean up the owned command/server groups and
return `128 + signal`. Interruption during startup is observed after the bounded
startup phase. Descendants that deliberately leave an owned group remain
caller-owned; there is no command deadline or protection against SIGKILL.
`--startup-timeout SECONDS` defaults to 30. Hard TTL is disabled unless `--ttl`
is supplied; supplying it with an external URL is rejected. External URL mode
borrows the existing server and leaves it running. Library callers can use
`PocketIcStartupConfig::run_command` with their own cancellation callback;
library code installs no process-wide signal handlers.

`--idle-ttl SECONDS` separately selects the owned server's operation-idle
lifetime, allowing compilation or other work before the command's first client
request. Without this option, PocketIC keeps its own idle default (60 seconds
in the prepared 16.1.0 server). For example, `run --idle-ttl 10800 -- COMMAND`
allows up to three idle hours without imposing a hard lifetime. Both TTL options
require positive whole seconds and owned-server mode. Library callers use
`with_server_idle_ttl`; `--ttl` continues to select the independent hard lifetime.

For complete raw diagnostics after startup failure, command failure or interruption,
select two new files with `PocketIcStartupConfig::with_server_output_files(stdout,
stderr)`, or use the paired CLI flags:

```bash
attempt="$(mktemp -d "${TMPDIR:-/tmp}/pocket-ic-attempt.XXXXXX")"
printf 'PocketIC diagnostics: %s\n' "$attempt"
ic-testkit-server run --server-stdout "$attempt/stdout" \
  --server-stderr "$attempt/stderr" -- cargo test --locked -p my-integration-tests
```

The parent directories must already exist and remain under caller control.
Existing files, symlinks and special files are refused without truncation.
Created files have private Unix permissions and survive successful runs and
all cleanup paths, including a partially prepared pair. Callers select their
retention, disk budget and presentation; streams have no automatic size limit.
Port/readiness files remain private temporary state and are removed normally.
This selection requires an owned server; external URL mode rejects it.
Public `output()` and error excerpts still read only the first 16 KiB per stream.

ic-testkit does not discover, download or cache server binaries. Explicit `spawn`
continues to assume caller admission. Version checks do not authenticate executable
bytes: use the shared `make tools-check` receipt verification for a prepared
bundle, or admit an external binary yourself. `PocketIc::get_server_url()` exposes
the active endpoint.

## Canister installation and fixtures

`InstallSpec` keeps generic install inputs and diagnostics together:

```rust,no_run
use candid::Principal;
use ic_testkit::pic::{
    CandidCallExt, InstallSpec, PocketIcBuilder, PocketIcStartupConfig, StandaloneCanisterFixture,
};

fn install_counter(wasm: Vec<u8>, init_bytes: Vec<u8>)
    -> Result<StandaloneCanisterFixture, Box<dyn std::error::Error>>
{
    let owned = PocketIcStartupConfig::from_env(std::time::Duration::from_secs(30))?
        .build_instance(PocketIcBuilder::new().with_application_subnet())?;
    let fixture = StandaloneCanisterFixture::try_install(
        owned,
        InstallSpec::new(wasm, init_bytes, 1_000_000_000_000)
            .install_sender(Principal::anonymous())
            .label("counter"),
    )?;

    let _value: u64 = fixture.pocket_ic().query_candid(fixture.canister_id(), "get", ())?;
    Ok(fixture)
}
```

The fixture owns one `PocketIcInstance<PocketIc>` record. `pocket_ic()` borrows
its instance, `canister_id()` identifies the installed canister, and `into_parts()`
returns the ownership record and ID. Failed `try_install` returns a
`StandaloneCanisterInstallError` containing the same instance/server record and
the `CanisterInstallError`. A caller-built instance on an externally owned server
can be passed explicitly as `PocketIcInstance { instance, server: None }`.
Creation, cycle funding, and installation panics are captured as structured failures. `phase()` identifies the failed operation;
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

Single-canister suites use the same pool with `StandaloneCanisterRecipe`. Each
slot owns an independent instance and captured snapshot. The recipe requires an
explicit identity and restore-funding policy, and the lease's metadata carries
its installed canister ID:

```rust,no_run
use std::{num::NonZeroUsize, sync::LazyLock};
use ic_testkit::pic::{
    CachedPocketIcBaselinePool, CandidCallExt, FixtureRecipeId, SnapshotRestoreFunding,
    StandaloneCanisterFixture, StandaloneCanisterRecipe,
};

static POOL: LazyLock<
    CachedPocketIcBaselinePool<StandaloneCanisterRecipe<fn() -> StandaloneCanisterFixture>>,
> = LazyLock::new(|| {
    CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(8).unwrap(),
        StandaloneCanisterRecipe::new(
            FixtureRecipeId::try_new("suite/counter").unwrap(),
            SnapshotRestoreFunding::Preserve,
            build_fixture as fn() -> _,
        ),
    )
});

# fn build_fixture() -> StandaloneCanisterFixture { unimplemented!() }
fn read_pooled_counter() -> Result<u64, Box<dyn std::error::Error>> {
    let (baseline, _outcome) = POOL.acquire()?;
    Ok(baseline.pocket_ic().query_candid(*baseline.metadata(), "get", ())?)
}
```

`acquire` returns `BaselinePoolOutcome` and records queue wait, build/capture,
restore, reset/readiness/validation, stale teardown and total time. Failures
retain completed timings and typed recipe causes; dead-transport recovery retains
both restore and replacement failure. A non-transport restore failure drops the
slot immediately; its next acquisition is a fresh build.

One pool owns one recipe. The builder must produce the same Wasm, init arguments,
topology and seeded state whenever invoked. Builders may own configuration in a
`Send + Sync + 'static` closure. Use a fresh instance or a custom recipe for state
outside the captured canister. See the [0.38 migration](crates/ic-testkit/CHANGELOG.md#0380)
for the removed standalone-pool interface.

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
URL and a recognized transport source, including OS connection resets during
HTTP requests. Such resets can select the recipe's dead-transport rebuild path.
Generic `channel closed` or
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
`advance_time`, and `tick` methods for direct simulator operations. For a
caller-owned readiness condition, `tick_until` bounds simulated progression:

```rust,no_run
use ic_testkit::pic::{PocketIc, TickUntilError, tick_until};
use std::{task::Poll, time::Duration};

fn await_condition(
    pic: &PocketIc,
    mut ready: impl FnMut(&PocketIc) -> Result<bool, String>,
) -> Result<(), TickUntilError<String>> {
    tick_until(pic, 20, Duration::from_secs(1), |pic| match ready(pic) {
        Ok(true) => Poll::Ready(Ok(())),
        Ok(false) => Poll::Pending,
        Err(error) => Poll::Ready(Err(error)),
    })
}
```

The initial check does not mutate time. Each pending check permits one time
advance and tick, followed by another check, up to the round budget. Completion
on the last round succeeds. Zero rounds still permit the initial check; zero
duration allows ticking without advancing time. This is a simulated-round
budget, not a wall-clock timeout for PocketIC calls or the predicate.

## Building test artifacts

Use [IC Build](https://github.com/dragginzgame/ic-build) directly for Cargo/Wasm
builds, exact input resolution, transactional artifact caches, retained records
and explicit maintenance. Testkit owns PocketIC provisioning, runtime lifecycle
and test/benchmark policy. Generic build APIs are not re-exported here.

The [0.35 migration guide](crates/ic-testkit/CHANGELOG.md#0350) also covers
the retired `ic_testkit::ic_host_tools` convenience path and direct Host 0.15 imports.

The [0.35 migration guide](crates/ic-testkit/CHANGELOG.md#0350) also covers
the retired `ic_testkit::ic_host_tools` convenience path and direct Host 0.15 imports.

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

For benchmarks, fetch the measured canister's logs and select
`BenchmarkEventSource::FetchedLog`. Confirm that the capture contains the complete
sample before accepting results. Managed server output/error excerpts retain only their
first 16 KiB per stream and can omit later markers. Caller-selected raw output
files contain the complete streams, but do not by themselves establish a
complete benchmark sample.
Host code parses, pairs, and aggregates captured canister markers:

```rust
use ic_testkit::benchmark::{
    BenchmarkEventSource, BenchmarkParserConfig, aggregate_benchmark_spans,
    pair_benchmark_spans, parse_benchmark_events_from_source,
};

let input = "\
ICTK|storage/write:start|100|200|300|400
ICTK|storage/write:end|150|260|390|430
";
let parsed = parse_benchmark_events_from_source(
    input, &BenchmarkParserConfig::default(), BenchmarkEventSource::FetchedLog,
);
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
label through `suite()`. Use `average(scale)` for checked, floor-rounded `u128`
means derived from exact totals and run counts through IC Metrics. For example,
scale ten projects totals 3 and 4 across two runs to 35 (mean 3.5). Projection
returns a typed error for zero runs or an unrepresentable scaled result.
`compare_benchmark_aggregates(current, previous, scale)` returns a fallible report
with current/previous means and run counts; missing rows remain distinct from
measured zero. Percentage-change fields are removed in the 0.41 hard cut.
CSV/text aggregates report integer means at scale one in original counter units;
comparison CSV carries both sides' scales and values. Exact totals remain available
when floor rounding would hide a fraction. Marker framing, metadata discovery and
retained prior report files are unchanged; new reports must use unique destinations
rather than reinterpret or overwrite an older report under the changed columns.

Native synthetic peers can use `testing::capture_http_request` with an explicit
read budget, header/body byte bounds and Content-Length rule. It selects blocking
reads on accepted sockets, rejects ambiguous framing and retains partial bytes
with a typed failure when a request is incomplete. Callers own responses,
evidence files and transport-failure injection.

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
endpoint names, role models, application readiness conditions, canister graph topology,
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

Developer setup and updates prepare `rustfmt` for the selected toolchain,
the pinned checkout-local Rust tool bundle, and the reviewed repository-local hook:

```bash
rustup component add rustfmt
make install-tools
make install-hooks
make tools-check dependency-pins-check
```

The reviewed Shared Tooling snapshot owns the default versions and digests in
`ci/tool-versions.env` and `ci/ic-tools.tsv`, also used by CI. Host, Rust and IC
exceptions are independent; see [catalog selection](docs/local-setup.md#catalog-selection).
`make install-tools` prepares pinned jq/yq, ripgrep with
PCRE2, cloc, common IC executables and the Cargo toolset in order under
`.tools/host/bin`, `.tools/ic/bin` and `.tools/rust/common/bin`, then builds the owning
CLI and prepares PocketIC. The Cargo set includes `cargo-sort`,
`cargo-sort-derives` and `candid-extractor`. `make tools-check` verifies all
selections and the prepared server offline, without compiling or installing
the owner CLI. Prepare the declared Rust toolchain before explicit setup.
Host and IC paths select verified bundles in Shared Tooling's per-user store.
After adopting that storage layout, rerun `make install-host-tools`, then
`make install-ic-tools`, followed by their offline checks. See
[store configuration and retention](docs/local-setup.md#shared-executable-storage).
After adopting Shared Tooling 0.6.0's shared Cargo storage, rerun
`make install-rust-tools`, then `make rust-tools-check`. Exact compiler, native
environment and packaged source/lock identities control reuse. Tool builds use
the shared standard environment; application flags and wrappers stay local.
Previous installations and failed build evidence remain retained.
Make and the formatting hook select these local tool directories; direct Cargo
commands should export them explicitly:

```bash
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/common/bin:$PATH"
server="$(target/debug/ic-testkit-server check)" || exit
export POCKET_IC_BIN="$server"
```

See [bootstrap prerequisites](docs/local-setup.md#bootstrap-prerequisites) and
[IC tool identities](docs/ic-tools.md). Offline checks verify the prepared
tools and dependency declarations without downloading or upgrading anything.
Setup installs tools explicitly; formatting never installs or fetches them.
For an optional online Cargo tool freshness report, run:

```bash
perl scripts/dev/check-cargo-tool-versions.pl --consumer "$PWD"
```

It compares pins and installation receipts with stable crates.io releases,
retains registry evidence, and distinguishes missing tools from unavailable
observations. Add `--json` for structured output; see the
[freshness task](tasks/cargo-tool-versions.md) for prerequisites and statuses.
The reviewed [consumer Make interface](docs/make-interface.md) owns common setup,
offline checks, formatting, hook installation, snapshot checks and release
routing through `make/consumer.mk`. PocketIC setup and product validation stay
local. Run fleet tooling reports from Shared Tooling. Existing
checkouts must rerun explicit setup to prepare the newly selected tools.
Run `make cloc` for this workspace's Rust LOC and test counts, or
`bash scripts/dev/cloc.sh --json` for per-crate rows and numeric totals.
Hook activation and formatting checks reject missing formatter prerequisites.
`make fmt` sorts every workspace Cargo manifest before formatting all Rust code.
`make fmt-check` performs the matching checks without changing files.
The pre-commit hook formats an isolated export of the selected index, refreshes
only selected files, and rejects partial staging. Unrelated working edits stay
untouched. `make git-hooks-check` exercises preservation and installation with
isolated fixtures; hook activation is separate from CI formatting evidence.
See the [hook rules](rules/git-hooks.md) and [host prerequisites](docs/hosts.md).

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

Failed native `checks` and `portable-hosts` jobs upload scoped diagnostic artifacts
for 14 days. Download `checks-failure-<runner>` or `portable-failure-<runner>`
from the Actions run to inspect validation logs, retained tool-installation
candidates and portable tooling fixtures. The general Cargo build directory
is excluded.

To exercise bounded managed startup against one exact caller-provided PocketIC
server binary without invoking any downloader or resolver:

```bash
POCKET_IC_BIN=/path/to/pocket-ic \
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

Maintain one numbered, undated pending release, `## [X.Y.Z]`, in both changelog
views. Derive it from the latest finalized release and the complete pending
batch: compatible changes select the next patch; breaking changes before 1.0
select the next minor. Reuse that entry until released and keep its links aligned.
This chooses release notes without changing package metadata. Select the matching
release command; a conflicting pending version is rejected during preparation.
Commit implementation changes before releasing; only the declared release
metadata may remain dirty.
Prepare both supported Rust toolchains, then select exactly one maintainer command:

```bash
make release-patch
# or: make release-minor
# or: make release-major
```

This repository selects direct atomic branch/tag delivery; adopting the shared
PR flow is a separate workflow change. All three commands use the reviewed
[common release workflow](docs/releases.md) and the same complete
`make release-check` gate, including MSRV, before changing metadata. They finalize both changelog views with the selected version and UTC
date, update both installation examples, preserve dependency selections, stage
the explicit release files, and create the maintainer-owned commit and annotated
tag. They atomically push only the selected branch and exact tag. Defaults are
`RELEASE_REMOTE=origin` and `RELEASE_BRANCH=main`; override these explicitly when
using another destination. GitHub Actions runs CI for the pushed tag. After tag CI
succeeds:

```bash
make publish
```

Preflight prepares locked dependencies using Cargo's configured network policy.
Metadata preparation stays offline and preserves dependency selections. The
complete gate includes `cargo publish --dry-run`, which requires registry access
without uploading a package. Explicit offline requests fail on missing cache
entries or required registry access; no step retries online automatically.
Validation retains each attempt's identity and full log in a unique directory
under `.git/release-state/`. A failed attempt cannot reuse
an earlier successful validation receipt. Failed reads of retained validation
or readiness records stop admission even if they return apparently complete
bytes, preserving metadata and saved evidence for retry. It preserves Cargo
artifacts. Version reads and next-version derivation must complete successfully
before their output can admit a release. Validation does not remove temporary
files around upstream background servers.
Managed PocketIC handles retain
ownership of their child processes. See
[temporary-directory ownership](POCKET-IC.md#temporary-directory-ownership)
and the [host matrix](docs/hosts.md) for portable checks and qualification gaps.
The release file set is Cargo.toml, Cargo.lock, both changelogs and both READMEs.
If preflight or validation fails, correct the inputs and rerun the normal release
target; both gates run afresh against the current source. Once preparation may
begin, an interruption retains its exact plan and prepared old/new metadata
under the repository's Git directory. Rerun a normal release target to
automatically recover that release at its saved version. The runner selects
unfinished intent before computing another increment, even if metadata has
already been bumped. For explicit selection of the saved release:

```bash
make release-resume VERSION=X.Y.Z
```

Both recovery paths verify the saved source, version, payload, commit, branch
and destination and do not recreate a matching release commit or tag. Automatic
recovery rejects competing unfinished plans. Uncommitted preparation remains
bound to its saved source and kind. After the release commit exists, a normal
target can reconcile it despite newer committed fixes, then perform fresh
preflight and complete validation for the requested increment from the actual
local version. An unchanged same-kind retry finishes only its saved release;
explicit resume never starts a next increment. Late consumer checks verify
metadata from the selected release commit rather than newer HEAD files.
Matching remote branch and tag identities complete an interrupted push without
another push; an unavailable remote query stops recovery.
Preparation and recovery reject symlinks in the retained metadata tree before
changing release files or linked targets. Keep rejected state for inspection;
the adapter does not discard its evidence to retry.
If publication stopped between writing Cargo.lock and Cargo.toml, recovery
checks offline caches against the verified saved workspace. It accepts only
the saved old/new metadata bytes and unchanged member manifests, and leaves
the live manifest and lockfile untouched until publication resumes.
Leftover `.release-metadata.*` staging files in the root and `crates/ic-testkit`
are helper-owned scratch, excluded from untracked-source checks and preserved
during recovery. Other untracked files still block the release.
An occupied release lock requires inspection of its recorded owner before
manual removal. Ordinary repair and continuation requests authorize local work.
Commit, PR and release effects require the corresponding explicit request under
the [contribution rules](rules/contributions.md), subject to this repository's
manifest-version ownership and validation overlay in [AGENTS.md](AGENTS.md).

`make installation-check` verifies the root and packaged README dependency
examples against the current workspace version. Release preparation uses the
same TOML example locator to update only those requirements, then checks its
exact prepared installation line before staging. Historical migration notes,
other code examples and surrounding formatting are preserved.

Publication requires a clean worktree and an annotated `v<version>` tag bound
exactly to `HEAD`. Failed Git inspections stop admission. The exact crates.io
package/version lookup uses `curl`: HTTP 200 skips an already published version,
and a successful HTTP 404 permits Cargo publication. Transport failures and
unexpected responses stop the command. Explicit `CARGO_NET_OFFLINE=true` or `1`
stops before the lookup; publication requires registry HTTP access.
