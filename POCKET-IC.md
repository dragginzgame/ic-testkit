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

`ic-testkit` currently resolves and validates the PocketIC server binary before
constructing a `PocketIc` instance. It supports an explicit `POCKET_IC_BIN`,
versioned cache paths, opt-in downloads, executable checks, and optional
SHA-256 verification.

It would be useful if upstream provided a first-class, non-panicking server
binary resolver with:

- explicit binary path configuration;
- predictable versioned cache locations;
- opt-in download policy;
- checksum verification hooks;
- typed errors with setup guidance;
- support for offline CI environments.

That would let downstream test harnesses share one trusted startup path instead
of each wrapper handling binary availability and download behavior differently.

### Typed Startup Errors

Some PocketIC startup failures currently surface as panics or stringly-typed
messages. `ic-testkit` catches and classifies a small set of those failures so
test harnesses can distinguish missing binaries, invalid binaries, download
failures, startup timeouts, and unreachable server transports.

Upstream typed errors would make this cleaner and more reliable. In particular,
`PocketIcBuilder::build` could have a non-panicking counterpart that returns a
structured startup error.

### Parallel Test Isolation

`ic-testkit` serializes PocketIC usage across processes with a filesystem lock
because concurrent local instances can interfere with each other in practice,
including wasm chunk store exhaustion and shared server/runtime resource
contention.

Upstream improvements that would reduce or remove this need:

- documented concurrency guarantees for multiple local PocketIC instances;
- per-instance isolation for runtime directories, chunk stores, and server
  state;
- configurable resource roots for independent test workers;
- clear typed failures when host-level resources are exhausted.

### Install-Code Rate Limiting

`ic-testkit` contains helpers that advance PocketIC time and retry operations
when install-code rate limiting is hit. This is useful, but it requires callers
to recognize the rate-limit message and encode retry policy themselves.

Useful upstream behavior would include:

- a typed reject/error variant for install-code rate limiting;
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

`ic-testkit` wraps PocketIC calls with Candid encode/decode helpers and
contextual errors. This is a convenience layer, but the underlying need is
common for Rust canister tests.

Potential upstream additions:

- typed `update_call` and `query_call` helpers that encode arguments and decode
  replies;
- caller-aware variants;
- error types that distinguish transport/reject failures from Candid encoding
  and decoding failures;
- optional panic-on-transport helpers for tests where application-level
  `Result<T, E>` values should remain explicit.

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

Upstream could expose stable APIs for:

- PocketIC server version;
- server binary path;
- server process or endpoint metadata;
- effective runtime directories;
- feature flags or subnet layout configured for an instance.

## Current `ic-testkit` Wrapper Areas To Revisit

These modules should be checked against upstream capabilities when updating
`pocket-ic`:

- `crates/ic-testkit/src/pic/runtime.rs`
- `crates/ic-testkit/src/pic/startup.rs`
- `crates/ic-testkit/src/pic/process_lock.rs`
- `crates/ic-testkit/src/pic/lifecycle.rs`
- `crates/ic-testkit/src/pic/calls.rs`
- `crates/ic-testkit/src/pic/baseline.rs`
- `crates/ic-testkit/src/pic/diagnostics.rs`
