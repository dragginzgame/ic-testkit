# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

This crate is the published Rust package in the `ic-testkit` workspace. It
provides:

- direct re-exports of `PocketIc` and `PocketIcBuilder`
- the complete host-only upstream crate at `ic_testkit::pocket_ic`
- typed Candid query/update helpers with contextual, structured errors
- canister install and retry helpers
- explicit bounded PocketIC startup, caller-owned managed-server handles, and
  structured child/readiness failures
- cached single- and multi-canister PocketIC baseline pools
- deterministic fake principals and account-like values
- transactional external artifact sets and content-addressed Wasm builds with
  caller-labeled sequential batches, explicit immutable-source sessions, and
  bounded cache retention
- controller-aware, caller-labeled collect-all diagnostics
- compact benchmark marker parsing, aggregation, comparison, and report writing
- canister-side `Performance::measure` marker emission

`ic-testkit` does not wrap the PocketIC simulator API, serialize independent
instances, or own PocketIC's server-binary cache. Tests normally create one
fresh `PocketIc` each and use its inherent methods for simulator operations;
focused extension traits provide reusable harness behavior. Less common
upstream types remain available through the complete `ic_testkit::pocket_ic`
re-export instead of an expanding mirrored list.

Most users should read the
[repository README](https://github.com/dragginzgame/ic-testkit#readme) for
setup, examples, local checks, and release notes.

The published archive includes [`CHANGELOG.md`](CHANGELOG.md), which contains
the `0.9` managed-server lifetime migration and earlier hard-cut tables.

The repository also includes a complete
[multi-canister baseline recipe](https://github.com/dragginzgame/ic-testkit/blob/main/crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
and a
[transactional external-artifact example](https://github.com/dragginzgame/ic-testkit/blob/main/crates/ic-testkit/examples/transactional_artifact_cache.rs)
that are compiled by the crate's normal all-target checks.
