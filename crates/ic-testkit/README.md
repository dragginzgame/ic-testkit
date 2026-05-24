# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

This crate is the published Rust package in the `ic-testkit` workspace. It
provides:

- a narrow `pocket-ic` wrapper for common test operations
- Candid query/update helpers with contextual errors
- canister install and retry helpers
- cached PocketIC baseline helpers
- deterministic fake principals and account-like values
- wasm artifact helpers for test harnesses
- compact benchmark marker parsing, aggregation, comparison, and report writing
- canister-side `Performance::measure` marker emission

Most users should read the repository README at `../../README.md` for setup,
examples, local checks, and release notes.
