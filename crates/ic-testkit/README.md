<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

```toml
[dev-dependencies]
ic-testkit = "0.18"
```

The published MSRV is Rust 1.88, and the selected PocketIC line is 16.
Canisters using benchmark markers can add the crate under `[dependencies]`.
`pocket_ic`, `pic`, and `artifacts` are host-only; `benchmark`, `performance`,
and `Fake` are available on `wasm32` as well.

This crate is the published Rust package in the `ic-testkit` workspace. It
provides:

- direct re-exports of `PocketIc` and `PocketIcBuilder`
- the complete host-only upstream crate at `ic_testkit::pocket_ic`
- typed Candid query/update helpers with contextual, structured errors
- canister install and retry helpers
- explicit bounded PocketIC startup, caller-owned managed-server handles, and
  structured child/readiness failures
- cached single- and multi-canister PocketIC baseline pools
- deterministic fake principals
- transactional external artifact sets and content-addressed Wasm builds with
  retained read-only outputs, caller-labeled sequential batches, explicit
  immutable-source sessions and concurrent-reader snapshots, and bounded cache
  retention
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
The [documentation index](https://github.com/dragginzgame/ic-testkit/blob/main/docs/README.md)
links current usage guidance and historical design records.

The published archive includes [`CHANGELOG.md`](CHANGELOG.md), which contains
the [0.14.0 standalone pool migration](CHANGELOG.md#0140),
the [0.13.0 benchmark and reset-requirement migration](CHANGELOG.md#0130),
the `0.11.0` API and report-schema cuts, the `0.10` retained-artifact migration,
the `0.9` managed-server lifetime migration, and earlier hard-cut tables.

The repository also includes a complete
[multi-canister baseline recipe](https://github.com/dragginzgame/ic-testkit/blob/main/crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
and a
[transactional external-artifact example](https://github.com/dragginzgame/ic-testkit/blob/main/crates/ic-testkit/examples/transactional_artifact_cache.rs)
that are compiled by the crate's normal all-target checks.

Benchmark aggregate/comparison rows and aggregate errors use `suite()` for their
label; aggregate rows use `average()` for averages derived from totals and run
count. Baseline recipes construct reset requirements with
`ResetRequirements::try_new(cycle_policy, non_snapshot_requirements)`. The pool
verifies snapshots and cycle policy through `CanisterRestoreReceipt`, while
`ResetReceipt` covers non-snapshot guarantees. These are source API hard cuts.
