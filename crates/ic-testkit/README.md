<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit

PocketIC-oriented test utilities for Internet Computer canister tests.

```toml
[dev-dependencies]
ic-testkit = "0.46"
```

The published MSRV is Rust 1.88, and the selected PocketIC line is 16.
Canisters using benchmark markers can add the crate under `[dependencies]`.
`pocket_ic` and `pic` are host-only. `performance` is Wasm-only;
`benchmark` counters, parsing and aggregation, and `Fake` are available on both
targets. Native directory, metadata and file-output APIs live in
`benchmark::report`. Candid calls return contextual errors; choose caller-side
`expect` when a fixture must panic. Standalone recipe builders return
`std::io::Result<StandaloneCanisterFixture>`.

This is the PocketIC package in the `ic-testkit` workspace. It provides:

- direct re-exports of `PocketIc` and `PocketIcBuilder`
- the complete host-only upstream crate at `ic_testkit::pocket_ic`
- typed Candid query/update helpers with contextual, structured errors
- canister install and retry helpers
- explicit PocketIC startup, retained server/instance ownership, fallible async construction, and
  structured child/readiness failures
- explicit authenticated `ic-testkit-server setup`, offline `check`, and a shared
  environment startup contract with `ic-testkit-server run -- COMMAND`
  for suites spanning several test processes
- one recipe baseline pool for single- and multi-canister PocketIC reuse
- deterministic fake principals
- controller-aware, caller-labeled collect-all diagnostics
- compact benchmark marker parsing, aggregation, comparison, and report writing
- canister-side `Performance::measure` marker emission

`ic-testkit` does not wrap the PocketIC simulator API, serialize independent
instances. Its published CLI owns explicit server provisioning and offline
verification; tests never install through that CLI implicitly. Tests normally retain one
`PocketIcInstance<PocketIc>` and use `owned.instance` for simulator operations;
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

The 0.27 startup-error contract uses `PocketIcStartupError::failure()`
and `PocketIcStartupFailure` for cause matching. Read bounded server excerpts
through `output()`, and inspect typed secondary failures through
`command_cleanup()` and `server_cleanup()`. The original cause remains primary;
raw output files remain caller-owned. Replace matches on the old error enum
with matches on `error.failure()`; no compatibility entry points are retained.

Import generic Host APIs directly from `ic-host-artifacts`, `ic-host-fs` and
`ic-host-process` dependencies. Testkit no longer re-exports those crates.
Add `ic-host-tools` directly for Candid and IC helpers;
the [0.35 migration guide](CHANGELOG.md#0350) covers the removed convenience
re-export and current import paths. The [Host 0.14 migration guide](CHANGELOG.md#0340)
covers dependency alignment, completed results for nonzero exits, explicit
`require_success()` before producer publication and fallible output observers.
The [Host 0.13 upgrade guidance](CHANGELOG.md#0334) covers descriptor-based locking
after removal of the pathname progress-lock helper. The [0.28 migration guide](CHANGELOG.md#0280)
covers the earlier consolidated durable writer and retained publication-error
evidence.
The [0.24 migration guide](CHANGELOG.md#0240) covers IC Host 0.7's execution
failure categories and replacing `artifacts::read_wasm` with
`ic_host_fs::read::read_file` on an `artifacts::wasm_path`. The
[0.20 migration guide](CHANGELOG.md#0200) maps the split owners.
The [0.19 guide](CHANGELOG.md#0190) covers
explicit tool resolution and verified external-transform arguments.

The published archive includes [`CHANGELOG.md`](CHANGELOG.md), which contains
the [0.41.0 integer benchmark projection migration](CHANGELOG.md#0410),
the [0.40.0 instance/server ownership migration](CHANGELOG.md#0400),
the [0.38.0 single-pool migration](CHANGELOG.md#0380),
the [0.13.0 benchmark and reset-requirement migration](CHANGELOG.md#0130),
the `0.11.0` API and report-schema cuts, the `0.10` retained-artifact migration,
the `0.9` managed-server lifetime migration, and earlier hard-cut tables.

The repository also includes a complete
[multi-canister baseline recipe](https://github.com/dragginzgame/ic-testkit/blob/main/crates/ic-testkit/examples/multi_canister_baseline_pool.rs)
compiled by the crate's normal all-target checks. The
[transactional external-artifact example](https://github.com/dragginzgame/ic-build/blob/main/crates/ic-build/examples/transactional_artifact_cache.rs)
is compiled and maintained by IC Build.

Benchmark aggregate/comparison rows and aggregate errors expose
`scope: BenchmarkScope::{Suite, All}` for their identity; `average(scale)` returns
checked floor-rounded `u128` means through
IC Metrics. `compare_benchmark_aggregates(current, previous, scale)` returns
current/previous means and run counts, preserving missing rows and measured zero.
Approximate percentage fields are removed. Aggregate CSV/text uses scale one in
original counter units; comparison CSV includes both scales. Exact totals remain
available, and zero runs/scaled overflow return typed errors. Write the changed
columns to new report directories; retained reports are not reinterpreted.
Baseline recipes construct reset requirements with
`ResetRequirements::try_new(cycle_policy, non_snapshot_requirements)`. The pool
verifies snapshots and cycle policy through `CanisterRestoreReceipt`, while
`ResetReceipt` covers non-snapshot guarantees. These are source API hard cuts.

Generic Cargo/Wasm builds, retained artifact caches and freshness APIs belong to
[IC Build](https://github.com/dragginzgame/ic-build). Depend on it directly and use
`ic_build::artifacts`; Testkit keeps PocketIC lifecycle and benchmark policy.

For 0.39, invoke Candid extensions on `fixture.pocket_ic()` with
`fixture.canister_id()` explicitly; the standalone fixture owns installation
and lifetime, while `CandidCallExt` owns encoding, invocation and decoding.
See the [packaged migration notes](CHANGELOG.md#0390).

### Small HTTP transport fixtures

`testing::http::HttpFixture` serves one HTTP 200 response on an explicitly
returned IPv4 loopback endpoint. Supply response bytes, a Content-Type and
`HttpFixtureLimits` with the total deadline, header budget and body budget.
The deadline starts at fixture construction and covers waiting for a client,
request admission and writing the complete response. `finish()` joins the worker
and returns the unmodified raw request, including headers. Dropping an unfinished
fixture shuts down an active socket and joins its worker. The caller owns client
timeouts, Candid/CBOR encoding and assertions; no PocketIC server is provisioned.
Use a caller-owned server for multi-request sequencing or malformed HTTP responses.
