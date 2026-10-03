# PocketIC Upstream Boundary

> Status: maintained against `pocket-ic` 16 and the current ic-testkit API.
> Revalidate these claims whenever the client or server version changes.

This document tracks upstream limitations that currently justify ic-testkit
harness code. It is not a roadmap for wrapping more of PocketIC.

`ic-testkit` is not intended to replace `pocket-ic`. The goal is to keep this
crate small, generic, and mostly focused on reusable test-harness ergonomics.
When a need is broadly useful to PocketIC users, the preferred long-term home is
upstream.

The complete host-only upstream crate is re-exported at
`ic_testkit::pocket_ic`. This is a version- and type-preserving access path, not
a mirrored API; the focused `ic_testkit::pic` exports remain conveniences for
the testkit extension traits.

## Maintenance

- Review this file whenever bumping the `pocket-ic` dependency.
- Remove items once upstream exposes a stable equivalent and `ic-testkit` no
  longer needs the workaround.
- Add concrete links to upstream issues or pull requests when they exist.
- Keep entries generic. Application-specific test conventions belong outside
  this repository.

The upstream discussion is tracked in
[PocketIC issue #67](https://github.com/dfinity/pocketic/issues/67), which links
this document. As of 2026-10-03 it has no replies; the latest published server
release is [16.0.0](https://github.com/dfinity/pocketic/releases/tag/16.0.0).

## High-Value Upstream Improvements

### Server Binary Resolution

PocketIC owns its default server-binary discovery, downloading, validation,
and caching. The bounded ic-testkit startup path deliberately bypasses that
implicit path: the caller supplies one already-resolved compatible executable,
and ic-testkit owns that child's lifecycle and, on Unix, its process group.
Teardown terminates descendants in that group before reaping the child, keeping
the group identity reserved through cleanup. One-shot managed
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

PocketIC 15 and later require the `--port-file` path not to exist before spawn.
If an empty file is pre-created, the server exits successfully and silently
rather than binding and publishing its port. The local launcher therefore
creates a unique private directory and output files but deliberately leaves the
port path absent; `NotFound` remains pending until PocketIC publishes a
newline-terminated port. Synthetic startup tests verify both the default
argument shape and explicit hard-TTL forwarding. An ignored live regression
test accepts the exact binary through `IC_TESTKIT_POCKET_IC_SERVER` and verifies
real port publication, bounded instance construction, owned shutdown, and
temporary-directory cleanup.

`PocketIcStartupConfig::start_managed_server` exposes the same bounded launcher
without constructing an instance. The returned `PocketIcManagedServer` retains
the URL and bounded lossy output and terminates and waits for the child on drop.
Matching PocketIC 16, the launcher relies on the server's activity-based soft
TTL by default and passes no absolute `--hard-ttl`; callers can opt into one
explicitly. Serial runners can keep that handle alive and use bounded
connect-mode builders without reimplementing process ownership. This remains
explicit caller scope, not a process-global singleton. The handle cannot
transfer ownership across Cargo or test-runner processes; multi-process CI
should retain a runner-owned external server and pass its URL to bounded connect
mode in each process.

Upstream typed errors would make this cleaner and more reliable. In particular,
`PocketIcBuilder::build` could have a non-panicking counterpart that returns a
structured startup error, while `start_server` could accept a readiness
deadline, poll `Child::try_wait`, retain bounded output, and terminate/reap on
failure. An upstream owned-server handle would additionally remove the need for
the local serial-suite lifecycle type. Once those cover the same lifecycle,
ic-testkit should delegate or remove its process-owning extension.

[Upstream PR #10751](https://github.com/dfinity/ic/pull/10751) proposes a Unix
process-exit registry that kills server process groups and reaps direct
children. It remains open as of 2026-10-03, and does not provide bounded
instance deletion. Separately,
[PocketIC issue #62](https://github.com/dfinity/pocketic/issues/62) reports
child-process leaks when instance creation panics after partial setup; it also
remains open. These are upstream lifecycle concerns, not reasons to add another
local server registry.

[Upstream PR #11167](https://github.com/dfinity/ic/pull/11167) merged the removal
of the default hard TTL. The local launcher already follows that behavior.

### Fallible Lifecycle and Transport APIs

Some PocketIC lifecycle and observation operations still panic on failure.
ic-testkit consequently catches panics around canister installation, calls,
snapshots, cached-baseline restoration, and best-effort diagnostics. It also
recognizes maintained reqwest request-error shapes with an instance URL and
specific transport sources so a stale cached baseline can be rebuilt and
contextual call errors can preserve their transport classification. The public
classifier recognizes testkit call transport kinds directly. Bare I/O errors
and generic application or quoted transport text do not qualify. Message
matching remains a heuristic for PocketIC-originating errors, not a liveness
proof.

This is a temporary upstream limitation, not an error model ic-testkit wants to
own. Result-returning upstream lifecycle, call, snapshot, status, and log APIs
should expose structured transport and dead-instance variants. Once those
variants cover the operations ic-testkit uses, remove `pic/transport.rs`, the
corresponding `catch_unwind` adapters, and all transport-message matching.

### Bounded Instance Teardown

PocketIC 16's synchronous `PocketIc::drop` waits for its asynchronous
`do_drop`, which sends HTTP DELETE to the instance URL with a reqwest client
that has no request timeout. It does not check the response status. The
operation-level `max_request_time_ms` retry budget is not applied to this
deletion, and the ic-testkit construction deadline ends when startup returns.
Catching a panic around drop does not bound a blocked HTTP request.

`tests/pocket_ic_teardown.rs` provides a controlled reproduction against a
synthetic loopback HTTP peer. An isolated child constructs an instance through
bounded startup with a one-millisecond operation budget, then drops it. The
parent receives DELETE, observes that the child is still pending, acknowledges
the request, and verifies completion. A parent-owned kill/reap guard and bounded
socket/process waits contain fixture failures. The ordering assertion has no
elapsed-time threshold. It illustrates the deletion wait; it does not reproduce
the cause of a real server's earlier Busy/tick timeout.

Run it without a PocketIC binary or download:

```bash
cargo test -p ic-testkit --locked --test pocket_ic_teardown \
  instance_drop_waits_for_the_deletion_response -- --exact --nocapture
```

Instance teardown improvements are deferred until an upstream PocketIC release
provides them. ic-testkit uses the unmodified registry dependency and returns
the upstream instance directly. The isolated patch experiment introduced in
0.10.3 was removed in 0.10.4. The reproduction above remains evidence of the
current limitation.

### Release Temporary-Directory Cleanup

PocketIC 16's server HTTP adapter destructor calls
`remove_file(self.uds_path.clone()).unwrap()` at
[server source line 457](https://github.com/dfinity/ic/blob/fc21803c3c3a8dd452b3b58b959751c41fecb89c/rs/pocket_ic_server/src/pocket_ic.rs#L454-L458).
Removing the server's temporary directory while it is alive deletes that Unix
socket first, so later adapter teardown panics with `NotFound`. This is separate
from the client HTTP DELETE wait above.

A focused before/after probe against the released PocketIC 16.0.0 binary
reproduced that exact panic with directory deletion first. Stopping the owned
server first completed without it; no canister Wasm build was required.

The release CI runner stops servers with port files inside its private scratch
directory before removing that directory. Cleanup uses Linux `/proc` to check
the complete argument list and match the running executable's device/inode to
the selected server binary. It accepts `POCKET_IC_BIN` and
`IC_TESTKIT_POCKET_IC_SERVER`, including renamed binaries, relative paths and
bare names resolved through `PATH`. Without `POCKET_IC_BIN`, it recognises
PocketIC 16's exact default download path under scratch,
`pocket-ic-server-16.0.0/pocket-ic`; keep that path aligned with upstream's
`LATEST_SERVER_VERSION` when updating the dependency. A caller-selected binary
outside these paths is unknown to the runner and retains scratch if still alive.
Missing, replaced or inaccessible selected binaries also retain scratch when a
live process uses a private port file; an executable name alone cannot establish
ownership.

Cleanup uses pidfds to signal and await the exact processes;
servers with external port files are left alone. It requests termination, waits
up to five seconds, then uses forced termination with another five-second bound.
If cleanup fails or safe process ownership is unavailable, it retains scratch
instead of removing files under a potentially live server. The targeted
`scripts/ci/test-release-pocketic-cleanup.py` regressions run as part of the
release guard checks.

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

PocketIC 16 already provides typed `query_candid`, `update_candid`, and
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

PocketIC 16 exposes the expected server version through
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

PocketIC 16 supports many independent IC instances, and the official testing
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
