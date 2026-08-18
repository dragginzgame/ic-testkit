# PocketIC Upstream Boundary

> Status: maintained against `pocket-ic` 15 and the current ic-testkit API.
> Revalidate these claims whenever the client or server version changes.

This document tracks upstream limitations that currently justify ic-testkit
harness code. It is not a roadmap for wrapping more of PocketIC.

`ic-testkit` is not intended to replace `pocket-ic`. The goal is to keep this
crate small, generic, and mostly focused on reusable test-harness ergonomics.
When a need is broadly useful to PocketIC users, the preferred long-term home is
upstream.

## Maintenance

- Review this file whenever bumping the `pocket-ic` dependency.
- Remove items once upstream exposes a stable equivalent and `ic-testkit` no
  longer needs the workaround.
- Add concrete links to upstream issues or pull requests when they exist.
- Keep entries generic. Application-specific test conventions belong outside
  this repository.

## High-Value Upstream Improvements

### Server Binary Resolution

PocketIC owns its default server-binary discovery, downloading, validation,
and caching. The bounded ic-testkit startup path deliberately bypasses that
implicit path: the caller supplies one already-resolved compatible executable,
and ic-testkit owns only that exact child lifecycle. One-shot managed
`try_build` keeps a reaper until the child exits; a caller can instead retain
`PocketIcManagedServer` explicitly and construct several instances through its
URL. Neither path adds a downloader, resolver, binary cache, or compatibility
guess.

It would be useful if upstream provided a first-class, non-panicking server
binary resolver with:

- explicit binary path configuration;
- predictable versioned cache locations;
- opt-in download policy;
- checksum verification hooks;
- typed errors with setup guidance;
- support for offline CI environments.

That would let downstream test harnesses combine one trusted upstream-owned
resolution path with bounded process startup instead of resolving the exact
binary before calling `PocketIcStartupConfig::spawn`.

### Typed Startup Errors

Some PocketIC startup failures currently surface as panics or stringly typed
messages. More importantly, the upstream implicit server path waits for a
port-file newline without inspecting whether the spawned child has already
exited and without a readiness deadline. A server that fails before binding
can therefore leave construction blocked indefinitely.

ic-testkit provides a narrow `PocketIcBuilderExt::try_build` boundary requiring
an explicit `PocketIcStartupConfig`. Managed mode spawns the exact
caller-resolved binary itself, observes child exit while awaiting both the port
file and instance construction, captures bounded stdout/stderr, and terminates
the child when the complete deadline expires. Connect mode bounds construction
against an existing caller-owned server. Upstream panics remain unclassified
structured errors.

PocketIC 15 also requires the `--port-file` path not to exist before spawn. If
an empty file is pre-created, the server exits successfully and silently rather
than binding and publishing its port. The local launcher therefore creates a
unique private directory and output files but deliberately leaves the port path
absent; `NotFound` remains pending until PocketIC publishes a newline-terminated
port. Synthetic startup tests reject a pre-existing fourth argument explicitly.
An ignored live regression test accepts the exact binary through
`IC_TESTKIT_POCKET_IC_SERVER` and verifies real port publication, bounded
instance construction, owned shutdown, and temporary-directory cleanup.

`PocketIcStartupConfig::start_managed_server` exposes the same bounded launcher
without constructing an instance. The returned `PocketIcManagedServer` retains
the URL and bounded lossy output and terminates and waits for the child on drop.
Serial runners can keep that handle alive and use bounded connect-mode builders
without reimplementing process ownership. This remains explicit caller scope,
not a process-global singleton.

Upstream typed errors would make this cleaner and more reliable. In particular,
`PocketIcBuilder::build` could have a non-panicking counterpart that returns a
structured startup error, while `start_server` could accept a readiness
deadline, poll `Child::try_wait`, retain bounded output, and terminate/reap on
failure. An upstream owned-server handle would additionally remove the need for
the local serial-suite lifecycle type. Once those cover the same lifecycle,
ic-testkit should delegate or remove its process-owning extension.

### Fallible Lifecycle and Transport APIs

Some PocketIC lifecycle and observation operations still panic on failure.
ic-testkit consequently catches panics around canister installation, calls,
snapshots, cached-baseline restoration, and best-effort diagnostics. It also
recognizes a small set of dead-instance transport message fragments so a stale
cached baseline can be rebuilt and contextual call errors can preserve their
transport classification.

This is a temporary upstream limitation, not an error model ic-testkit wants to
own. Result-returning upstream lifecycle, call, snapshot, status, and log APIs
should expose structured transport and dead-instance variants. Once those
variants cover the operations ic-testkit uses, remove `pic/transport.rs`, the
corresponding `catch_unwind` adapters, and all transport-message matching.

### Install-Code Rate Limiting

PocketIC exposes `RejectResponse::error_code` and the structured
`ErrorCode::CanisterInstallCodeRateLimited` variant. ic-testkit uses that field
directly when applying install retry policy and never classifies display text.

Useful upstream behavior would include:

- an accessor for the required cooldown, when available;
- a helper that advances simulated time enough for a retry in deterministic
  tests;
- documentation for when the rate limit applies inside PocketIC.

### Canister Install Diagnostics

When `install_canister` panics or rejects, `ic-testkit` tries to print canister
status and logs to make the failure actionable. This is generic harness behavior
that many PocketIC users would benefit from.

Upstream could expose richer install errors that include:

- canister id;
- reject code and message;
- canister status, when available;
- recent canister logs, when available;
- whether the canister was created before install failed.

### Candid-Aware Call Helpers

PocketIC 15 already provides typed `query_candid`, `update_candid`, and
caller-aware variants. They panic on Candid encoding and decoding failures and
return `RejectResponse` for canister rejection. `ic-testkit` therefore does not
claim typed calls themselves as missing upstream functionality.

The remaining upstream opportunity is a structured error model that can
distinguish Candid encoding, Candid decoding, transport failure, and canister
rejection while preserving call context. If upstream gains that behavior,
`CandidCallExt` should be reduced or removed rather than maintained as a
parallel call API.

### Snapshot Baselines

`ic-testkit` uses PocketIC snapshots to cache expensive setup and restore
canisters between tests. It also rebuilds the baseline if the underlying
PocketIC instance becomes unreachable.

Useful upstream support would include:

- documented snapshot lifecycle guarantees;
- structured errors for restore failures and dead transports;
- examples for baseline-style test reuse;
- APIs that make it clear which parts of instance state are captured or omitted
  by canister snapshots.

### Log Access and Benchmarking

`ic-testkit` parses canister log markers for benchmark reports. Direct log
fetching is useful for diagnostics, but log buffering and trimming behavior need
to be clear for high-volume benchmark output.

Upstream improvements that would help:

- documented log retention limits;
- streaming or incremental log access for tests;
- stable ordering and source metadata for fetched log records;
- guidance on stdout/stderr behavior when canister logs are emitted during
  PocketIC calls.

### Runtime Introspection

Test harnesses often need to know which runtime they used when writing reports
or debugging CI failures.

PocketIC 15 exposes the expected server version through
`LATEST_SERVER_VERSION` and the active endpoint through
`PocketIc::get_server_url()`. ic-testkit re-exports the version constant. A
built instance does not expose the resolved binary path or its digest, so
ic-testkit cannot truthfully forward those values.

Until upstream exposes that provenance, reproducible benchmark suites should
resolve and hash a compatible binary themselves, pass that exact path to
`PocketIcStartupConfig::spawn` for either one-shot construction or an explicit
managed handle, and record those values. ic-testkit should not recreate
PocketIC's binary resolver to infer them.

Useful additional upstream APIs are:

- resolved PocketIC server version;
- resolved server binary path and digest;
- server process or endpoint metadata;
- effective runtime directories;
- feature flags or subnet layout configured for an instance.

## Reviewed Upstream Capabilities And Local Decisions

### Independent Test Instances

PocketIC 15 supports many independent IC instances, and the official testing
guidance describes parallel execution as one fresh `PocketIc` instance per
test. The Rust documentation also says sharing one instance among test cases is
generally not recommended.

ic-testkit has no host-wide PocketIC serialization guard. It re-exports
`PocketIc` directly instead of retaining a forwarding simulator wrapper, while
downstream suites tune heavy test capacity through
`cargo test -- --test-threads=N` or their CI scheduler.

The previous wasm chunk-store rationale was incorrect: the management-canister
interface defines chunk storage per canister, so it does not justify a lock
across unrelated PocketIC instances.

Locks remain local to genuinely shared resources owned by `ic-testkit`, such
as one benchmark output path or one explicitly shared cached baseline. The
PocketIC server cache and any synchronization around it remain upstream-owned.

An optional `PocketIcManagedServer` is likewise caller-scoped ownership of one
server process, not a host-wide instance lock. Several `PocketIc` instances may
use its endpoint while retaining independent upstream instance state and
lifetimes; the caller decides whether that serial-suite topology is appropriate.

See [`docs/design/0.2-concurrency/0.2-design.md`](docs/design/0.2-concurrency/0.2-design.md)
for the historical design record behind this decision.

## Current `ic-testkit` Helper Areas To Revisit

These modules should be checked against upstream capabilities when updating
`pocket-ic`:

- `crates/ic-testkit/src/pic/transport.rs`
- `crates/ic-testkit/src/pic/startup.rs`
- `crates/ic-testkit/src/pic/lifecycle.rs`
- `crates/ic-testkit/src/pic/calls.rs`
- `crates/ic-testkit/src/pic/snapshot.rs`
- `crates/ic-testkit/src/pic/baseline.rs`
- `crates/ic-testkit/src/pic/diagnostics.rs`
