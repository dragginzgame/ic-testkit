# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

This crate is the published Rust package in the `ic-testkit` workspace. It
provides:

- direct re-exports of `PocketIc` and `PocketIcBuilder`
- typed Candid query/update helpers with contextual, structured errors
- canister install and retry helpers
- cached PocketIC baseline helpers
- deterministic fake principals and account-like values
- content-addressed wasm artifact helpers with bounded cache retention
- compact benchmark marker parsing, aggregation, comparison, and report writing
- canister-side `Performance::measure` marker emission

`ic-testkit` does not wrap the PocketIC simulator API, serialize independent
instances, or own PocketIC's server-binary cache. Tests normally create one
fresh `PocketIc` each and use its inherent methods for simulator operations;
focused extension traits provide reusable harness behavior.

Most users should read the
[repository README](https://github.com/dragginzgame/ic-testkit#readme) for
setup, examples, local checks, and release notes.
