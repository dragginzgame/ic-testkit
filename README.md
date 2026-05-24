# ic-testkit

Reusable PocketIC-oriented test utilities for IC canister tests.

Use this crate when you want generic host-side test infrastructure that can be
shared across IC canister projects.

Toolchains:
- MSRV: Rust 1.88
- internal build/test lane: Rust 1.95

What it adds over raw `pocket-ic`:
- a narrower `Pic` wrapper that exposes the PocketIC operations this crate
  supports as a stable test surface
- typed startup errors for common PocketIC launch failures, including missing
  binaries, invalid binaries, failed downloads, server startup failures, and
  startup timeouts
- a cross-process `PicSerialGuard` for test suites that need to serialize
  PocketIC usage and avoid shared server/resource exhaustion
- Candid `update_call` and `query_call` helpers that encode arguments, decode
  results, and preserve useful canister/method context on failures
- generic create/install helpers for caller-provided wasm modules and init bytes
- install-code rate-limit retry helpers that advance PocketIC time between
  attempts
- canister diagnostics that dump status and logs for failed test paths
- standalone prebuilt-wasm fixtures that own their `Pic`, canister id, and
  serialization guard together
- cached baseline primitives that snapshot canisters once and restore them
  between tests, rebuilding automatically when a cached PocketIC instance dies
- controller snapshot helpers for capture/restore flows that need sender
  fallbacks
- deterministic fake principals and ledger-style accounts for reproducible tests
- wasm artifact path, readiness, build, and read helpers for host-side test
  harnesses
- watched-input freshness checks for generated `.icp` artifacts
- workspace target-directory helpers for crates living at a workspace root or
  under `crates/`

Current API shape:
- `Pic` is the intentional host-side wrapper surface for PocketIC calls used by this crate
- cached baseline guards expose explicit accessors instead of transparently derefing into raw `PocketIc`
- tests should prefer the wrapper methods and fixture helpers here instead of reaching through to the underlying PocketIC client directly

What it intentionally does not own:
- application init payloads, role names, or endpoint method constants
- application-specific readiness polling
- product-specific canister fixture graphs
- attestation-specific fixture policy
- repo-only audit probes
- broad self-test orchestration

If you are writing downstream PocketIC tests, start here.
If you are editing application-specific integration harnesses, keep that code
in the owning application repo instead of widening this generic surface.
