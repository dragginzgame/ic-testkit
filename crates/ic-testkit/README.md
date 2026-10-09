<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

```toml
[dev-dependencies]
ic-testkit = "0.26"
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
- explicit authenticated `ic-testkit-server setup`, offline `check`, and a shared
  environment startup contract with `ic-testkit-server run -- COMMAND`
  for suites spanning several test processes
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
instances. Its published CLI owns explicit server provisioning and offline
verification; tests never install through that CLI implicitly. Tests normally create one
fresh `PocketIc` each and use its inherent methods for simulator operations;
focused extension traits provide reusable harness behavior. Less common
upstream types remain available through the complete `ic_testkit::pocket_ic`
re-export instead of an expanding mirrored list.

```bash
cargo install --locked ic-testkit
ic-testkit-server setup
ic-testkit-server check
ic-testkit-server run -- cargo test --locked
```

Setup selects Testkit's reviewed PocketIC 16.1.0 archive for Linux x86-64,
macOS Intel or macOS Apple Silicon. It authenticates the archive's pinned digest,
bounds decompression and admits executable bytes/version before publication.
Setup/check print the absolute executable path; diagnostics go to stderr.
`--directory DIRECTORY` selects a physical root (default
`.tools/ic-testkit-server` in the working directory). Checks and run never
download, update or repair a server. Failed attempts and previous bundles remain
retained; a changed installation requires a fresh selected root.

Run uses that prepared selection when neither `POCKET_IC_BIN` nor
`IC_TESTKIT_POCKET_IC_URL` is selected, and exports the executable and managed
URL to the command. Explicit binary overrides admit stable 16.x servers,
independently of the Rust client's latest download constant. URL mode borrows an
existing server. For long pre-client work, select `run --idle-ttl SECONDS -- COMMAND`
or `PocketIcStartupConfig::with_server_idle_ttl`. This selects operation-idle
lifetime independently of `--ttl`'s hard limit; omitted idle selection retains
PocketIC's default. Both options require owned-server mode and positive seconds.

Most users should read the
[repository README](https://github.com/dragginzgame/ic-testkit#readme) for
setup, examples, local checks, and release notes.
The [documentation index](https://github.com/dragginzgame/ic-testkit/blob/main/docs/README.md)
links current usage guidance and historical design records.

The pending 0.27 startup-error hard cut uses `PocketIcStartupError::failure()`
and `PocketIcStartupFailure` for cause matching. Read bounded server excerpts
through `output()`, and inspect typed secondary failures through
`command_cleanup()` and `server_cleanup()`. The original cause remains primary;
raw output files remain caller-owned. Replace matches on the old error enum
with matches on `error.failure()`; no compatibility entry points are retained.

Host-only shared APIs are available through the complete `ic_host_artifacts`,
`ic_host_fs`, `ic_host_process` and `ic_host_tools` re-exports under `ic_testkit`.
The [0.24 migration guide](CHANGELOG.md#0240) covers IC Host 0.7's execution
failure categories and replacing `artifacts::read_wasm` with
`ic_host_fs::read::read_file` on an `artifacts::wasm_path`. The
[0.20 migration guide](CHANGELOG.md#0200) maps the split owners.
The [0.19 guide](CHANGELOG.md#0190) covers
explicit tool resolution and verified external-transform arguments.

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
label; aggregate rows use `average()` for approximate `f64` averages derived from
totals and run count. Distinct integers above 2^53 can round to the same average
and report zero percentage change; use exact totals and run counts for exact
comparisons. Text reports also round displayed averages and percentages to whole
numbers. Baseline recipes construct reset requirements with
`ResetRequirements::try_new(cycle_policy, non_snapshot_requirements)`. The pool
verifies snapshots and cycle policy through `CanisterRestoreReceipt`, while
`ResetReceipt` covers non-snapshot guarantees. These are source API hard cuts.
