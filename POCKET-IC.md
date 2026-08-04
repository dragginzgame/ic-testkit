# PocketIC Upstream Wishlist

> Warning: this document was LLM-generated. Treat it as a working draft and
> take it with a pinch of salt until each item has been checked against current
> upstream `pocket-ic` behavior.

This document tracks what `ic-testkit` would like to see improve in the
upstream `pocket-ic` crate and server. Keep it current as `ic-testkit` adds or
removes wrapper behavior.

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

PocketIC already owns server-binary discovery and downloading. `ic-testkit`
0.2 removed its duplicate resolver, downloader, and cache rather than growing
a second runtime manager.

It would be useful if upstream provided a first-class, non-panicking server
binary resolver with:

- explicit binary path configuration;
- predictable versioned cache locations;
- opt-in download policy;
- checksum verification hooks;
- typed errors with setup guidance;
- support for offline CI environments.

That would let downstream test harnesses use one trusted, upstream-owned
startup path without recreating binary management.

### Typed Startup Errors

Some PocketIC startup failures currently surface as panics or stringly-typed
messages. ic-testkit previously caught and classified selected panic text, but
0.2.2 removes that brittle parallel startup API. Callers now construct
`PocketIc` directly before passing it to a fixture.

Upstream typed errors would make this cleaner and more reliable. In particular,
`PocketIcBuilder::build` could have a non-panicking counterpart that returns a
structured startup error.

### Install-Code Rate Limiting

PocketIC exposes `RejectResponse::error_code` and the structured
`ErrorCode::CanisterInstallCodeRateLimited` variant. ic-testkit 0.2.2 uses that
field directly when applying install retry policy and never classifies the
display text.

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

`ic-testkit` 0.2 therefore removes its host-wide PocketIC serialization guard
rather than request more upstream locking or runtime isolation. It re-exports
`PocketIc` directly instead of retaining a forwarding `Pic` wrapper,
while downstream suites tune heavy test capacity through
`cargo test -- --test-threads=N` or their CI scheduler.

The previous wasm chunk-store rationale was incorrect: the management-canister
interface defines chunk storage per canister, so it does not justify a lock
across unrelated PocketIC instances.

Locks remain local to genuinely shared resources owned by `ic-testkit`, such
as one benchmark output path or one explicitly shared cached baseline. The
PocketIC server cache and any synchronization around it remain upstream-owned.

See [`docs/design/0.2-concurrency/0.2-design.md`](docs/design/0.2-concurrency/0.2-design.md)
for the accepted 0.2 direction.

## Current `ic-testkit` Helper Areas To Revisit

These modules should be checked against upstream capabilities when updating
`pocket-ic`:

- `crates/ic-testkit/src/pic/transport.rs`
- `crates/ic-testkit/src/pic/lifecycle.rs`
- `crates/ic-testkit/src/pic/calls.rs`
- `crates/ic-testkit/src/pic/baseline.rs`
- `crates/ic-testkit/src/pic/diagnostics.rs`
