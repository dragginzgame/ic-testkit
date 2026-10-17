<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit package changelog and migration guide

This file ships in the crate archive so upgrades can be completed without the
repository checkout. The complete historical changelog remains at
<https://github.com/dragginzgame/ic-testkit/blob/main/CHANGELOG.md>.

## [0.52.1]

### Added

- Standalone Candid codec exports a bounded typed sequence visitor and
  `reserved::StrictReserved` for zero-skipping reply schemas. Callers select
  element ceilings, diagnostic text and decoding quotas; application DTOs,
  aggregate bounds and retained evidence stay with consumers. The implementations
  preserve Backup's typed-excess and exact reserved-marker behavior without
  adding simulator or transport dependencies.

## [0.52.0] - 2026-10-17

### Breaking

- Native tool failures use Host 0.26 types. Align directly used Host dependencies
  before constructing or downcasting the exposed tool errors.

- Diagnostic report, batch-entry and batch-report outputs expose public fields.
  Replace forwarding getters and tuple-unpacking methods with field access or
  record destructuring; bounded log internals and collect-all behavior remain.

- Remove pool recipe identifiers; each pool already owns one recipe. Remove
  `FixtureRecipeId`, recipe `id()` implementations and pool `recipe_id()` calls.
  `StandaloneCanisterRecipe::new` now takes only funding and its builder.
- Remove `try_create_and_install_many`. Map an ordered iterator through
  `try_create_and_install` and collect its results to preserve stop-on-error
  behavior and ownership of already created canisters.

- Startup uses explicit `PocketIcStartupConfig::Spawn` and `Connect` variants.
  The spawn variant owns binary, hard/idle TTLs and output paths; connect owns
  only the borrowed URL and timeout. Spawn-only options cannot accompany a URL.

- Baseline pools own canonical snapshot restoration. Successful restoration
  returns unit; funding is selected once in reset requirements. Caller reset,
  readiness and final invariant checks retain their ordering and errors.

### Migration

Replace startup constructors and fluent configuration with variant literals:

```rust
use ic_testkit::pic::PocketIcStartupConfig;
use std::time::Duration;

let config = PocketIcStartupConfig::Spawn {
    server_binary: "/prepared/pocket-ic".into(),
    timeout: Duration::from_secs(30),
    server_hard_ttl: None,
    server_idle_ttl: Some(Duration::from_secs(120)),
    server_output_files: None,
};
let borrowed = PocketIcStartupConfig::Connect {
    server_url: "http://127.0.0.1:8080/".into(),
    timeout: Duration::from_secs(30),
};
```

Match variants to read configuration fields. `from_env(timeout)` still selects
an explicit URL before a binary, refuses invalid selected values and qualifies
binary versions. `timeout()` projects the deadline for either variant. Hard TTL
`None` remains opt-in; idle TTL `None` uses PocketIC's own default. Selected TTLs
must contain at least one whole second. Output files are new, private files in
caller-controlled existing parent directories; created files survive errors and
teardown. Instance/server drop order, bounded excerpts and exact cleanup remain.

Recipe migration: pass `SnapshotRestoreFunding::Preserve` or `TopUpTo { ... }`
to `ResetRequirements::try_new(funding, domains)` instead of `CycleResetPolicy`.
Remove forwarding `restore_canisters` implementations. If a recipe must check
state before restoration (for example, cycle balances before restore charges),
implement `prepare_restore(&self, baseline) -> Result<(), Self::Error>`; its default
is successful admission. It cannot skip or substitute the captured snapshot set.
Use real guards or select fresh construction for guarantees beyond preservation
or minimum top-up; do not replace prior exact-cycle guarantees with a fabricated
receipt. Non-snapshot reset coverage is still checked after readiness.

`classify_failure` now accepts `&(dyn std::error::Error + 'static)` so it can
classify both recipe and canonical snapshot failures. Downcast to your recipe
error when needed. `BaselinePoolPreparationError::Restore` retains the original
`ControllerSnapshotError`, exact sender and rejection/transport cause. Recovery
still rebuilds at most once and retains both errors when rebuilding also fails.
`CachedPocketIcBaseline::restore` returns `Result<(), ControllerSnapshotError>`.
Remove `CanisterRestoreReceipt` imports and destructuring of its old payload.
Inspect captured IDs through `baseline.snapshot_canister_ids()` and funding
through `reset_requirements().funding()`. `PreparedBaseline::Restored { reset }`
retains the verified non-snapshot receipt.
Pool instances are process-local; this API change resets or converts no retained
installation, snapshot store or external resource.

Diagnostic outputs use `report.request`, `report.status`, `report.logs`,
`entry.label`, `entry.report`, `entry.entry_elapsed`, `batch.entries` and
`batch.total`. Use `report.status.as_ref()` / `report.logs.as_ref()` to borrow
errors and destructure records to move their outcomes. `is_success()` and
`failures()` retain their semantic projections. Log records and raw-byte totals
remain privately constructed so their bounding/truncation invariants hold.

### Changed

- Standalone pool recovery qualification checks canonical snapshot errors
  directly while retaining failed-phase timing and subsequent rebuild checks.

- Fixture benchmark programs and their live marker test belong to the private
  `ic-testkit-bench` workspace package. Run the driver with
  `cargo run -p ic-testkit-bench --bin fixture_reuse_benchmark_driver -- ...`;
  shared protocol/report code is compiled once, while workers remain separate
  processes. Existing Make/CI checks include the new package; locked archive
  verification excludes its development-only dependency graph.
- Benchmark reports retain typed fresh/pooled modes through JSON admission.
  Existing valid report bytes remain compatible; wrong requested modes and
  unknown serialized modes are rejected independently.
- Startup, snapshot and polling errors derive formatting and source chains,
  retaining their public variants, original causes and cleanup diagnostics.


- Benchmark report publication delegates existing-parent path admission to
  Host, preserving private staging, cancellation, complete replacement and
  before/after-publication error handling. Directory-required output paths
  ending in `/` or `/.` now return Host's `InvalidInput` error before staging.

- Contributor setup adopts Shared Tooling 0.20.0. Common checks cover host/Cargo
  tools and PocketIC; generic IC executables are an explicit optional install.

- Asset setup uses Host's existing directory commit engine instead of a local
  rename/sync sequence. Fresh publication requires the originally held staging
  directory; an occupied destination preserves both trees. Reuse authenticates
  the installed bundle before confirming parent durability. Publication failures
  retain Host's typed phase distinction. No retained installation is reset,
  converted or removed; archive digests, version checks and v1 names are unchanged.
  Explicit Host write options preserve diagnostic permissions and publication
  modes; archive features select gzip and Wasm
  ([#107 — Adopt Host explicit byte options and replace retired archive feature](https://github.com/dragginzgame/ic-testkit/issues/107)).

- The fixture benchmark's native worker build delegates to Build 0.14 and uses
  Cargo's reported executable/target root. Metadata and JSON messages have a
  finite per-document allowance; build duration and total diagnostic output have
  no default ceiling. The selected Cargo/RUSTC, offline lock, process-group
  cleanup and live cancellation remain caller-owned
  ([#94 — Adopt native Cargo executable selection](https://github.com/dragginzgame/ic-testkit/issues/94)).

Follow-up: wide numeric accumulation is prepared against
[Metrics PR #92 — Checked wide measurement accumulation](https://github.com/dragginzgame/ic-metrics/pull/92)
and awaits a supplying published release. Marker parsing, attribution, report
layouts and the current published benchmark arithmetic remain in Testkit until
normal adoption; no path override ships in this batch.

## [0.51.0] - 2026-10-17

### Breaking

- Adopt Host 0.25's public error types. The library compiler floor remains
  Rust 1.91.

- Pool readiness and validation hooks return `Result<(), E>`. Named receipts and
  the repeated validation recipe identity are removed; reset/canister coverage,
  actual invariant checks, failure stages and acquisition timings remain.
- Benchmark markers contain prefix, label and three measured counters:
  instructions, Wasm heap bytes and stable-memory bytes. The reserved allocation
  field is removed from public counters, averages, overflow variants and reports.

- Benchmark report data no longer embeds a run metadata record, and the report
  module no longer reads or writes metadata sidecars. Applications own run
  identities, provenance, command recording and previous-run selection.

- Diagnostic input requests, labeled requests and render limits expose public
  fields instead of infallible constructors, builders and input accessors.
  Collection retains label validation, bounded rendering and independent senders.

- Installation, retry and snapshot-target inputs use public-field records.
  Typed query/update calls require an explicit principal; use
  `Principal::anonymous()` when appropriate.
- Native CSV/Markdown directory output and its report-bundling record move to
  the repository's `benchmark_report` example. Library parsing and arithmetic
  remain available; applications own file layout and output policy.

### Migration

Select `ic-host-process` 0.25 when naming `ToolError` from
`PocketIcStartupFailure::ServerVersionProbe` or `CleanupError` returned by
`PocketIcStartupError::command_cleanup` / `server_cleanup`. Their Rust type
identities follow the adopted Host family; Testkit adds no compatibility aliases.
Build 0.12.1 is contributor benchmark tooling and still exposes Host 0.24 types.
Benchmark examples and the canister benchmark test await an aligned Build release
before their Host 0.25 output-limit inputs can compile.

Construct `CanisterDiagnosticsRequest { canister_id, status_sender, log_sender,
log_limits: CanisterLogRenderLimits::default() }` directly. Select custom limits
with `CanisterLogRenderLimits { record_limit, byte_limit }`, and label a request
with `LabeledCanisterDiagnosticsRequest { label: label.into(), request }`.
Read input fields directly. Zero render limits remain valid; empty or duplicate
batch labels are still rejected before diagnostic effects. Report/output record
accessors and the original installation failure remain unchanged.

Construct `InstallSpec { wasm, init_bytes, cycles, install_sender, label }`,
`RetryPolicy { max_attempts, cooldown }` and
`CanisterSnapshotTarget { canister_id, sender }` directly. Optional installation
fields use explicit `None`/`Some`; retry counts retain `NonZeroUsize`. Replace
input getter calls with field access. Installation ordering, rate-limit handling,
exact snapshot senders and rollback are unchanged.

Rename `update_candid_as` / `query_candid_as` to `update_candid` / `query_candid`.
Their explicit-caller signatures are retained. Existing anonymous calls add
`Principal::anonymous()` after the canister ID. Single-value and top-level reply
argument decoding retain their distinct entry points and explicit limits.

Replace `benchmark::report::write_benchmark_report_dir` and library
`BenchmarkRunReport` construction with application-owned reporting, or run
`cargo run -p ic-testkit --locked --example benchmark_report -- INPUT OUTPUT_DIRECTORY [PREVIOUS_INPUT]`
from this repository. The example preserves the existing nine CSV/Markdown file
layouts and scale-one comparisons. Existing files are historical evidence; the
move reads, rewrites and retires no retained report directory automatically.

Change recipe `drive_to_readiness` and `validate` implementations to return
`Result<(), Self::Error>`. Keep their real readiness and invariant checks, then
return `Ok(())`; final validation runs against the recipe owned by the pool.
`PreparedBaseline::Restored` contains only canister and reset receipts.

Build marker producers and consumers from the same release. The current shape is
`ICTK|label:start-or-end|instructions|heap_bytes|memory_bytes`. Update custom
producers and CSV/Markdown consumers for three counters. Before adopting this
layout, explicitly retire older marker logs and report directories from active
comparison inputs, keeping them as historical evidence. Generate fresh baselines
and use fresh output directories; do not overwrite or reinterpret retained reports.
The library selects no retained installation and supplies no old-layout reader.
Remove `metadata` from `BenchmarkRunReport` construction and replace
`BenchmarkRunMetadata` / `read_benchmark_run_metadata` with your application's
provenance record and reader. Testkit writes measurement CSV/Markdown only;
callers select and write any sidecar themselves. Existing metadata files remain
historical evidence and are not read, migrated or deleted by Testkit. Choose a
fresh output directory before using the current report contract. Fixture-reuse
worker/driver JSON remains owned by its example and is unchanged.

### Added

Unix Rust fixtures can call `pic::server_assets::check(&directory)` to authenticate
an existing reviewed bundle offline and pass its absolute executable path to
managed startup. `server_assets::setup(&directory, interrupted)` explicitly
provisions an absent bundle, with caller-owned cancellation. The CLI delegates
to this same owner. The library installs no signal handlers, and callers retain
physical-root custody and prevent namespace replacement before launch.

### Changed

- Candid reply decoders select compact diagnostics on native and Wasm hosts,
  avoiding full large-input dumps while preserving typed causes and caller limits.
  Compact diagnostics can include small inputs; callers still own disclosure.

- Git provenance uses Host's observed-version admission instead of capturing and
  admitting the version separately. Executable/output budgets, explicit environment,
  cancellation checks and typed Git/tool causes remain selected by the driver
  ([#112 — Single-probe Host observed-version admission](https://github.com/dragginzgame/ic-testkit/issues/112)).
- Startup log reading shares Host's prefix operation, retaining the 16 KiB cap,
  lossy UTF-8 decoding, omitted-byte count and nonblocking special-file refusal.
- Snapshot funding is selected and calculated once before restore. Preserve mode
  performs no balance read, and an already satisfied top-up minimum adds no
  cycles. The live restore case covers preserve, needed and satisfied funding.
- Testkit's direct JSON dependency and Serde derives are limited to native
  tests/examples; native Candid calls retain their deserialization dependency.

- Release preparation uses the reviewed shared Cargo metadata helper in a retained
  private workspace; local exact-byte recovery and additional release views remain.

### Testing

- Asset fixtures re-exec before writing executables so unrelated library forks
  cannot inherit their writable handles. Production admission and no-retry
  behavior are unchanged
  ([#115 — Isolate server-asset fixtures from parallel process tests](https://github.com/dragginzgame/ic-testkit/issues/115)).

- Qualify exact Git status bytes with portable control/quote filenames on macOS
  and Linux; retain invalid UTF-8 directory-entry coverage on Linux
  ([#111 — Portable Git dirty-path byte qualification](https://github.com/dragginzgame/ic-testkit/issues/111)).
- Keep the output-limit producer alive until capture terminates and reaps it,
  removing the natural-exit/signalling race from cleanup qualification
  ([#114 — Deterministic live-producer cleanup qualification](https://github.com/dragginzgame/ic-testkit/issues/114)).

## [0.50.0] - 2026-10-16

### Breaking

- Snapshot capture and restore failures carry `source: Box<PocketIcCallFailure>`;
  cleanup failures use the same source in one contextual record. Match rejection
  or operation failure through that source. Snapshot errors no longer implement
  `Clone`, `Eq` or `PartialEq`; exact senders and rollback evidence remain available.

- All typed PocketIC query/update helpers require a final
  `ic_testkit_candid::DecodeLimits` argument and use that crate's bounded decoder.
  `CandidCallError::Decode` retains a boxed `DecodeError` in `source` instead of
  a message string; `CandidCallError` no longer implements `Clone`, `Eq` or
  `PartialEq`. Top-level reply-argument helpers share the same decoding owner.

- Remove `ic_testkit::testing` and its duplicate HTTP request readers. Native
  fixture callers use `ic_host_process::testing::http` with `test-support`;
  response policy and PocketIC lifecycle stay with their callers.
- Benchmark metadata rejects duplicate declared fields rather than silently
  accepting the last value. Object-only input and optional-field rules remain;
  metadata now deserializes directly without an intermediate JSON tree.
- Adopt Host 0.23's exposed types. Process creation selects the previous cleanup
  policy explicitly, and setup retains descriptor-based locking and archive
  authentication through the current Host APIs.

### Migration

Match `ControllerSnapshotError::CaptureFailed` or `::RestoreFailed` for either
rejection or panic, then inspect `source.as_ref()` as
`PocketIcCallFailure::{Rejected, Operation}`. These variants both retain the
selected `sender`; capture failures also retain all rollback failures.
`SnapshotCleanupFailure` is a record with public `canister_id`, `sender` and
boxed `source` fields instead of separate rejection/panic variants. Use the typed
source or `Error::source()` to retain transport classification, rather than
parsing error text. Snapshot errors no longer supply cloning or equality traits.

Add `ic-testkit-candid` as a direct dependency and supply caller-selected
`DecodeLimits` after the Candid argument tuple in `query_candid`,
`update_candid`, their `_as` variants and the `query_candid_args` /
`update_candid_args` free functions. Select wire and work limits for the expected
reply shape; the library supplies no universal default. These limits govern
reply decoding and do not impose a build or transport deadline.
Match `CandidCallError::Decode { source, .. }` or downcast `Error::source()` to
`ic_testkit_candid::DecodeError` for wire-admission and original decoder causes.
Use `ic_testkit_candid::decode_args` when decoding top-level reply arguments
outside PocketIC; `(T, U)` denotes two arguments and `(T,)` denotes one.


Select `ic-host-process` directly with its `test-support` feature for synthetic
HTTP peers. Replace `testing::read_http_request` with
`testing::http::capture_http_request(..., HttpContentLength::Optional).into_result()`.
Import phased `read_http_request_line` and `read_http_request_with_line` from
Host's `testing::http` module. Keep each peer's existing deadline and byte limits.

Metadata files keep their current object layout and optional fields. Remove
ambiguous duplicate declared keys before reading; existing files are neither
rewritten nor reinterpreted under another format. Output remains unchanged.

### Development

Benchmark aggregation borrows suite identities while grouping and allocates
owned names only for completed rows or errors. Suite/label sorting, the distinct
cross-suite scope and transactional overflow rejection remain unchanged.

Package checks and publication dry runs build the exact coordinated archives
using the selected lockfile and a temporary dependency on the packaged Candid
source. This avoids checking against an older published copy with the current
workspace version. The verification override is confined to a disposable tree;
archives retain registry dependency declarations. Release preparation uses
Shared Tooling's renderer to advance the internal requirement with the workspace
version; package versions remain maintainer-owned.

## [0.49.0] - 2026-10-16

### Breaking

- Startup builder panic failures carry `source: PocketIcOperationError` instead
  of a message string, preserving the typed cause through `Error::source()`.
  Synchronous and asynchronous construction share operation panic capture;
  startup deadlines, cancellation and owned-server cleanup remain unchanged.

- Diagnostics expose `PocketIcCallFailure::{Rejected, Operation}` in place of
  `CanisterDiagnosticFailure`. Diagnostics and snapshot capture share rejection
  and panic translation; snapshot context, rollback and original causes remain
  available through their existing outer reports and errors.

- Pool acquisitions return common `slot` and `timings` fields with a
  `disposition: BaselinePoolDisposition`. Match `Built`, `Restored` or
  `Rebuilt(reason)` through that field; read timings and slot directly instead
  of calling their former accessors.

- `CandidCallError::Transport` carries `source: PocketIcOperationError` instead
  of a message string. Inspect the typed source directly or through
  `Error::source()`; unrelated application panics still resume unchanged.
- Invalid benchmark spans retain both original events without an owned `reason`
  field. The report writer emits the existing counter-decrease explanation.

- Benchmark aggregation and comparison return row vectors directly. Read and
  populate `BenchmarkRunReport::aggregates` and `::comparison` without `.rows`;
  replace the removed report wrapper types with `Vec<BenchmarkAggregateRow>`
  and `Vec<BenchmarkComparisonRow>`. Report files and arithmetic stay unchanged.

### Changed

- Release and evidence fixtures reuse Shared Tooling's physical-directory
  allocation, completion admission and failure-preserving cleanup.

- The benchmark driver delegates preparation commands to Host for cancellation,
  child cleanup and typed execution failures. Cargo builds keep inherited output
  and no elapsed-time deadline; version/platform stdout capture is limited to
  64 KiB. Interrupted preparation still exits with status 130, while worker
  interruption continues to wait for the workload's server cleanup.

- Hook qualification uses the reviewed Shared Tooling checker with Testkit's
  actual workspace inputs, including manifest sorting and preservation checks.
- Environment-selected PocketIC URLs use the same parser and typed errors as
  explicit startup configuration, preserving validation before server effects.

### Fixed

- Teardown integration tests use Host's bounded HTTP capture directly, removing
  their dependency on Testkit's synthetic-request helper export.

- Align benchmark tooling on Build 0.10 and Host 0.22 so workspace discovery
  and metadata observation accept the same output-limit types. The fake Cargo
  fixture emits artifact messages required by Build's current Wasm discovery.

- Keep PocketIC setup and failure-evidence recording compatible with Host 0.22's
  explicit byte-write options, preserving replacement and permission choices.
  Enable the separate gzip feature used to authenticate and decode server bundles.

## [0.48.0] - 2026-10-16

### Breaking

- Diagnostic operation failures retain `PocketIcOperationError` directly in
  `CanisterDiagnosticFailure::Operation`; inspect its cause and transport flag.
- Remove `BenchmarkSpan::start`, retaining end counters, checked deltas and
  source lines. Invalid spans still retain both original events.

- Match installation error variants for their original cause and label; funding
  and installation failures carry a required canister ID. Remove the separate
  phase enum and optional-field accessors.
- Benchmark events retain `span_label` and `kind` without a duplicate `label`
  string. CSV output reconstructs the existing label and retains its columns.

- Match `SnapshotCleanupFailure::Rejected` or `::Panicked` for exact sender and
  cause details. Panic cleanup failures retain the original operation error and
  transport classification; remove the optional-field accessors.
- Read unpaired benchmark events directly from
  `BenchmarkSpanReport::unpaired_markers`. Remove the duplicate marker wrapper
  and kind enum; event order and report CSV columns remain unchanged.
- Installation helpers return structured failures without fetching diagnostics
  or printing to stderr. Fixture callers choose diagnostic requests and output,
  retaining the original installation error if diagnostic collection fails.
- Read comparison sides through `current` and `previous`, each containing its
  run count and average together. Missing measurements remain distinct from zero.
- Format diagnostic reports with `Display` or `to_string()` instead of
  `render_compact()`; bounded output and retained labels are unchanged.

### Migration

Replace diagnostic `InstanceUnavailable`/`Panicked` message matches with
`CanisterDiagnosticFailure::Operation(source)`. Read `source.message()` and
`source.is_transport()`; the same typed cause is exposed through the standard
error source chain. Rejections remain distinct, status/log outcomes remain
independent, and the bounded diagnostic display is unchanged.

Valid benchmark spans retain `end` and `delta` instead of a third starting-counter
copy. Use the original parsed event identified by `start_line` when raw starting
counters are needed. Pairing still rejects subtraction underflow before creating
a span; invalid spans retain both raw events. CSV layouts are unchanged and no
retained report reset is needed.

Match `CanisterInstallError::CreateCanister { label, source }`,
`::AddCycles { canister_id, label, source }` or
`::InstallCode { canister_id, label, source }` instead of reading `phase()` and
unwrapping an optional ID. The latter two variants always retain the created
ID. Inspect `source.message()` or `source.is_transport()` directly; the standard
error source chain and standalone instance ownership remain intact.

Raw benchmark event labels are formed as `span_label` plus `:start` or `:end`
according to `kind`. Native report output reconstructs and escapes those labels
without changing raw-event or unpaired-marker CSV layouts. Retained reports
need no reset.

`BenchmarkComparisonRow::current` and `::previous` are each an
`Option<BenchmarkMeasurement>`. Read `side.runs` and `side.average` after matching
the side once, replacing the independent optional run-count and average fields.
The comparison CSV retains its existing columns, ordering and blank cells for
missing measurements; zero measurements still render as zero. Retained reports
need no reset.

Use `format!("{report}")`, `report.to_string()` or direct formatting of a single
or batch diagnostics report. Both use the existing bounded `Display` output.

### Changed

- Benchmark summaries traverse the samples retained in each run directly, without
  staging a second vector of references. Sample ordering, missing-phase handling,
  numeric checks and published summary fields remain unchanged.
- Contributor fixtures and benchmarks use published Build 0.8 so its exposed
  Host types match the selected Host 0.21 family. This is a development dependency
  alignment; the library's consumer compiler floor and package version are unchanged.

- Baseline-pool invalidation records the caller-selected reason in the invalid
  slot itself. Repeated invalidation retains the latest reason; unwinding still
  takes precedence and invalidated instances remain owned until safe teardown.

### Other migration details

Replace cleanup accessor calls with variant matches. A `Panicked` variant
carries `source: PocketIcOperationError`, so callers can inspect the original
message and transport classification without reparsing formatted text.

Unpaired entries are now `RawBenchmarkEvent` values: use `entry.kind` and
`entry.source_line` directly rather than `entry.event`. CSV layouts stay v1
with their existing columns and start/end values; no retained report reset is
required.

After installation fails, callers can match the error variant for its ID and
use the retained instance to collect `CanisterDiagnosticsRequest` with their selected
status/log senders. Standalone failures retain the same instance/server record.
Keep diagnostic failures subordinate to the original install error, including
when diagnostic output fails. No new diagnostic mode or installation retry is
introduced.

## [0.47.0] - 2026-10-15

### Breaking

- Return `CanisterRestoreReceipt` directly from cached baseline restores and
  reject empty cached captures before effects. Return that receipt from ordinary
  recipe hooks; custom restore contracts still use `try_new`.
- Match `CandidCallError` variants for context and rejection details. Remove
  `CandidCallErrorKind`, duplicated formatted messages and constructor wrappers;
  contextual codec/transport text and complete rejections remain available.

- Remove `BenchmarkRunDirectory`, `benchmark_run_directory_name`,
  `next_benchmark_run_directory` and `find_latest_previous_run`. Benchmark
  commands now own naming, allocation and previous-run selection; report
  rendering and metadata IO remain available.
- Build the fixture benchmark worker and driver together: they now exchange one
  complete typed report, requiring startup, cold-preparation and teardown fields
  and rejecting unknown fields. Retained report files are left unchanged.

- Require Rust 1.91 for both published packages, matching the selected Metrics
  0.11 dependency and the workspace's coordinated compiler floor.

### Added

- Share `pic::advance_and_tick` for caller-selected time advancement and exact
  tick batches. Zero duration skips time reads/writes; zero ticks permits
  advancement alone. Polling and install cooldowns reuse the same operation.
  [#98 — Share caller-selected simulated-time and tick batches](https://github.com/dragginzgame/ic-testkit/issues/98).

### Changed

- Use Host 0.20's explicit trusted-digest admission and borrowed Git observations,
  preserving authenticated offline server checks and byte-safe provenance.

- Capture benchmark Git provenance through Host's existing admitted observation
  API, with 16 MiB stdout and 64 KiB stderr limits per command and no deadline.
  Include untracked files and all submodule changes explicitly, preserve native
  pathname bytes, and retain typed Git/tool failure evidence.

- Use published Metrics 0.11.1 for sample mean, median and nearest-rank statistics,
  removing the driver's private calculation. Preserve count/mean/p50/p95 report
  fields; numeric failures retain their original cause and refuse publication.
  [#103 — Adopt published Metrics sample statistics](https://github.com/dragginzgame/ic-testkit/issues/103).

- Keep one worker report for admission, summaries and output, removing the raw
  JSON duplicate and the worker-only outer report type. Preserve all lifecycle
  measurements and driver-owned RSS fields.

- Observe the benchmark driver's Cargo catalog through IC Build's typed API,
  retaining selected tools, locked/offline policy, complete bounded output and
  original command/schema failures. Cargo builds retain unlimited duration.

For Candid failures, replace `error.kind` and optional field checks with variant
matches such as `CandidCallError::CanisterReject { context, response }`.
Use `error.to_string()` for rendered context; `message` in codec/transport
variants is the original failure text. The enum retains `Clone` and equality.
Harness failures unrelated to a Candid operation should use the harness's error
rather than the removed unclassified constructor.

Cached `baseline.restore(funding)` now returns the receipt after every snapshot
restores successfully. Empty inputs to cached capture return
`ControllerSnapshotError::EmptyCanisterSet`; low-level snapshot capture retains
its empty-set semantics. Custom recipes reporting stricter cycle guarantees must
still construct their own exact receipt after establishing those guarantees.

## [0.46.0] - 2026-10-15

### Breaking

- Adopt Host 0.19 for native process and filesystem operations. Callers using
  Host types exposed by Testkit must select the same Host minor line. Private
  parent preparation now rejects ancestor symlinks and `..`; supply canonical
  physical paths when intentionally selecting an alias.

### Changed

- Use Build 0.6's Cargo observation API for workspace discovery, preserving
  selected tools, bounded diagnostics and unlimited build duration.
- Refresh the reviewed Shared Tooling snapshot to committed 0.15.0, carry its
  release-input validator into isolated metadata qualification, and share
  formatter failure-evidence selection with the common owner.
- Bound macOS RSS command capture with driver-selected, configurable output
  limits, retaining complete-table admission, original causes and worker cleanup.
  No elapsed-time limit is added.
  [#101 — Adopt explicit Host RSS command limits](https://github.com/dragginzgame/ic-testkit/issues/101).

### Added

- Decode multiple top-level Candid reply arguments with `query_candid_args` and
  `update_candid_args`, preserving explicit callers and contextual failures.
  Existing single-result calls keep their decoding semantics.

### Fixed

- Skip simulated-time reads and same-time writes during zero-duration readiness
  polling; preserve nonzero advancement, tick budgets and predicate ordering.
- Give successful version-probe fixtures scheduling headroom on loaded hosts,
  retaining the short deadline for the deliberately stalled probe.
- Include MIT license text in both published crates and inherit the existing
  workspace license in the private performance probe.

Pass a tuple of reply types, such as `(u128, u128)`, to the new functions.
This describes two top-level arguments; `CandidCallExt` still decodes one value,
including tuple-valued replies. Both paths share encoding and transport/error
handling. Use `()` for no replies and `(T,)` for one reply.

## [0.45.0] - 2026-10-15

### Breaking

- Use fallible Candid query/update methods and choose caller-side `expect` where
  desired; remove the four `_or_panic` convenience methods.
- Import native benchmark directory, metadata and report-output APIs from
  `benchmark::report`. Parsing, counters and aggregation remain portable;
  filesystem reporting and Testkit's JSON dependency are native-only.
- Return `std::io::Result<StandaloneCanisterFixture>` from standalone recipe
  builders. Setup failures retain their causes and build-stage timings through
  the existing pool error path.

- Restrict `performance::Performance` and the direct IC CDK dependency to Wasm;
  native callers emit supplied counters through `benchmark::format_marker`.
- Use only `try_create_and_install`, `try_create_and_install_many` and
  `StandaloneCanisterFixture::try_install`. Handle the result or choose an explicit
  caller-side `expect`; remove the panic-only installation entry points.
- Construct retry policies with `RetryPolicy::new(NonZeroUsize, cooldown)`;
  `max_attempts()` returns that non-zero value. Remove `RetryPolicyError`.
  Caller preparations: [Canic PR #572](https://github.com/dragginzgame/canic/pull/572)
  and [Jobs PR #51](https://github.com/dragginzgame/ic-jobs/pull/51).
- Import generic Host APIs directly from `ic-host-artifacts`, `ic-host-fs` and
  `ic-host-process`; remove their complete re-exports through `ic_testkit`.
  Existing Testkit signatures that expose Host-owned types retain those types.

### Added

- Add the independent `ic-testkit-candid` package for native/Wasm reply decoding
  with caller-selected wire, work, type-table and header limits. Choose ordinary
  single-result decoding or require exactly one wire argument without trailing
  bytes. Decoder errors retain their original causes; existing PocketIC call
  defaults are unchanged.
  [#87 — Own a simulator-independent caller-bounded Candid reply codec](https://github.com/dragginzgame/ic-testkit/issues/87).

### Changed

- Share typed benchmark worker arguments and mode parsing between the driver
  and workload, preserving native paths and independent report admission.
- Check and package both published libraries. Publication observes each package
  before uploading and skips completed uploads when retrying a partial release.

### Fixed

- Fix release-guard qualification for standalone Candid tests, retaining
  PocketIC admission and package-specific failure ordering. Surface the failing
  substituted recipe log when a product target unexpectedly fails.

## [0.44.0] - 2026-10-15

### Breaking

- Read `CandidCallError` and `CandidCallContext` through their existing public
  fields; remove their duplicate getters. Use `context.as_deref()` and
  `reject_response.as_deref()` when borrowing boxed optional values.
  Canic caller preparation is in [PR #569](https://github.com/dragginzgame/canic/pull/569).

- Capture snapshots through `capture_snapshots(targets)` and restore through
  `restore_snapshots(snapshots, funding)`. Select each sender explicitly; restore
  reuses the captured sender without fallback. Cached baselines use
  `capture(instance, targets, metadata)` and `restore(funding)`. Snapshot rejection
  errors expose one `sender` and `response` instead of an attempt list.
  Canic preparation is in [draft PR #570](https://github.com/dragginzgame/canic/pull/570);
  normal adoption awaits publication and its pending upstream integration.
- Match aggregate/comparison rows and aggregate errors through the public
  `scope: BenchmarkScope::{Suite, All}` field instead of `suite()` and
  `is_all_suites()`. Existing report formats remain unchanged.

### Changed

- Share catch-all PocketIC panic conversion across installation, snapshots and
  diagnostics, preserving contextual failures, cleanup and recovery decisions.
  Candid calls still resume unrelated panics with their original payload.
- Create and admit private server setup directories through IC Host FS,
  retaining the directory descriptor until verified bundle publication.
  Failed attempts and installed bundles retain their existing layout and policy.

## [0.43.0] - 2026-10-14

### Breaking

- Remove the redundant `PocketIcTimeExt` trait and its prelude export. Read
  nanoseconds directly with `pic.get_time().as_nanos_since_unix_epoch()`;
  update trait imports and call sites. Simulated polling remains unchanged.
  Jobs caller preparation is in [PR #49](https://github.com/dragginzgame/ic-jobs/pull/49);
  Canic caller preparation is in [PR #565](https://github.com/dragginzgame/canic/pull/565).

### Changed

- Share the fixture benchmark worker/driver report types and decode borrowed
  JSON without cloning the complete report. Preserve raw worker fields, required
  phases, workload matching and driver-owned sampling admission.
- Render benchmark terminal columns through IC Host Text 0.18.2 with computed
  widths and escaped cells. Numeric formatting and JSON reports remain local.
- Serialize benchmark metadata directly from its typed record, retaining JSON
  fields and the trailing newline; key presentation follows record order.

## [0.42.0] - 2026-10-14

### Breaking

- The public IC Host reexports now select 0.18. Following file reads/hashes
  admit their opened descriptor instead of checking metadata before opening.
  Native open failures therefore precede type/size refusal; callers relying on
  error precedence must update their handling. No stored-format migration is
  required for this library contract change.

### Changed

- Select reviewed Shared Tooling 0.12.7 and its local-lockfile rewriter for
  release preparation. Only exact unsourced workspace package versions and
  local dependency references change; all external selections remain intact.
  Saved manifests still support cache preparation after interrupted metadata
  installation, and final locked Cargo admission remains independent
  ([#73 — Adopt snapshot-owned common tool catalogs](https://github.com/dragginzgame/ic-testkit/issues/73)).

### Added

- Use `testing::read_http_request_line` to classify a synthetic request without
  waiting for headers/body, then `read_http_request_with_line` on that same
  socket to complete it under the original absolute deadline. Both phases
  retain caller-selected bounds, response policy and failure injection; the
  existing whole-request helper remains available for complete peers
  ([Blob #70](https://github.com/dragginzgame/ic-blob-storage/issues/70)).

### Fixed

- Allocate retained idle-lifetime evidence atomically with a randomized temporary
  directory under `target/`. Previous logs remain intact when CI restores its
  cache; the server lifetime and evidence upload paths retain their contracts
  ([#91](https://github.com/dragginzgame/ic-testkit/issues/91)).

## [0.41.0] - 2026-10-14

### Breaking

- Use `average(scale)` and `compare_benchmark_aggregates(current, previous, scale)`
  for fallible integer projections through IC Metrics 0.9.4. Current/previous
  scaled means replace approximate percentage-change fields. Aggregate CSV/text
  uses whole original units; comparison CSV carries each side's scale. Keep
  retained reports separate and write the changed columns to new directories
  ([#84](https://github.com/dragginzgame/ic-testkit/issues/84)).

### Added

- Capture bounded raw native requests with `testing::read_http_request` for
  synthetic peers. Response policy and failure injection remain caller-owned
  ([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

### Testing

- Share bounded synthetic HTTP request capture across startup and teardown
  fixtures, retaining one read deadline and explicit macOS blocking socket mode
  ([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

- Run shared guards, formatting and ordinary startup, teardown and concurrency
  tests through the existing complete CI gate once on each supported native host.
  That gate now uses system Bash with the portable CDPATH and temporary-directory
  context. Portable installer and release-runner fixtures, benchmark examples,
  ignored real sync/async instance lifetime and recovery cases, CLI provisioning,
  idle-lifetime evidence and hosted archive readback retain their distinct steps.
  The full release aggregate is unchanged
  ([#81](https://github.com/dragginzgame/ic-testkit/issues/81)).

- Run each synthetic release network-policy case in its own subshell. The
  unset case no longer exposes an inherited `CARGO_NET_OFFLINE=true`; existing
  unset/false/true/1 cache and gate assertions still apply. Production fetch
  continues to honor the operator's policy, and real fixture Cargo calls stay
  offline
  ([#83](https://github.com/dragginzgame/ic-testkit/issues/83)).

## [0.40.0] - 2026-10-14

### Breaking

- Keep the instance and optional server together using
  `PocketIcStartupConfig::build_instance`. Remove `PocketIcBuilderExt::try_build`
  and its detached successful-server reaper. Standalone install, install-error
  recovery and cached baseline capture now transfer `PocketIcInstance<PocketIc>`;
  dropping their retained owner deletes the instance before reaping its server
  ([#79](https://github.com/dragginzgame/ic-testkit/issues/79)).

### Changed

- Share Host's regular-file opening and `tempfile`'s ephemeral test-directory
  ownership while preserving bounded reads, private permissions and retained
  CLI failure evidence
  ([#79](https://github.com/dragginzgame/ic-testkit/issues/79)).

Pass the returned owner record directly to `StandaloneCanisterFixture::install`
or `try_install`, and to `CachedPocketIcBaseline::capture` or
`capture_with_senders`. Both standalone `into_parts()` methods return that record,
so recovery preserves the server. For caller-built instances on externally owned
servers, construct `PocketIcInstance { instance, server: None }` explicitly.
Retain a separate `PocketIcManagedServer` for shared servers and connect instances
through its URL. Connect mode never takes server authority.

There is no detached server lifetime after successful managed construction.
Keep the owner live for the intended fixture or pool lifetime. The startup
worker still bounds synchronous construction; upstream synchronous instance
Drop still has no request deadline. Retained data and installations do not
change, so no format migration or reset is required.

### Testing

- Set accepted sockets to blocking mode in the synthetic async startup peer,
  retaining bounded reads and request/cancellation barriers. A controlled
  initially nonblocking stream now proves that delayed bytes keep the peer
  open, covering the Intel macOS premature-close failure
  ([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

- Use Host 0.17.9's opt-in Unix test-support writer directly in native unit and
  integration fixtures. Testkit no longer owns copies of the executable writer;
  script content and fixture assertions remain local. The parent still holds
  only a pipe while an owned child writes complete bytes before chmod/use,
  protecting parallel fork/exec. Failures remain explicit at the test caller
  ([#78](https://github.com/dragginzgame/ic-testkit/issues/78),
  [Host #87](https://github.com/dragginzgame/ic-host-tooling/issues/87)).

- Run the existing prepared-server startup tests explicitly in the native
  concurrency matrix. The ignored cases cover real managed construction,
  synchronous and asynchronous owned topology/lifetimes, and borrowed-server
  ownership. Ordinary `cargo test` did not execute these cases. No public API,
  startup policy or runtime format changes from this CI selection
  ([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

## [0.39.0] - 2026-10-14

### Breaking

- Call Candid extensions on the standalone fixture's `pocket_ic()` with its
  `canister_id()` explicitly. Remove the fixture's forwarding methods while
  retaining owned installation and decomposition
  ([#78](https://github.com/dragginzgame/ic-testkit/issues/78)).

Import `ic_testkit::pic::CandidCallExt` and pass the installed canister ID to
the existing extension, for example
`fixture.pocket_ic().update_candid(fixture.canister_id(), "method", args)`.
Explicit-sender and panicking calls use that same PocketIC/ID boundary. This
changes no retained format; standalone fixtures and startup directories are
process-local.

### Changed

- Delegate private startup directory allocation and cleanup to `tempfile`,
  retaining port readiness, private output files and caller-owned log retention
  ([#78](https://github.com/dragginzgame/ic-testkit/issues/78)).

- Require Host 0.17.8 or newer, including its native permission conversion fix
  for macOS CLI builds
  ([#77](https://github.com/dragginzgame/ic-testkit/issues/77)).

- Adopt Shared Tooling 0.12.0 for contributor tooling, sharing exact artifact
  readback and checked release-version rendering. Release preparation now refuses
  unnumbered or misplaced candidates while preserving metadata and history. Tool
  selections and library contracts are unchanged
  ([#73](https://github.com/dragginzgame/ic-testkit/issues/73),
  [#59](https://github.com/dragginzgame/ic-testkit/issues/59),
  [Shared #162](https://github.com/dragginzgame/shared-tooling/issues/162)).

## [0.38.0] - 2026-10-13

### Breaking: one fixture-pool lifecycle

Single-canister reuse uses `StandaloneCanisterRecipe` with
`CachedPocketIcBaselinePool`; the standalone pool, guard, outcome, error, stage,
rebuild-reason and timing family is removed. Construct the recipe with an explicit
`FixtureRecipeId`, `SnapshotRestoreFunding` and caller-owned builder. For static
suites use `LazyLock`; construction remains lazy and the pool's constructor is
still const. Builders must own their captures and satisfy `Send + Sync + 'static`.
The baseline metadata is its installed canister ID. Use
`baseline.pocket_ic().query_candid(*baseline.metadata(), method, args)` rather
than calling the standalone fixture's forwarding methods through a pooled guard.

Match `BaselinePoolOutcome` and stage-specific `BaselinePoolError` values.
Snapshot and receipt failures retain their typed causes in the recipe's
`std::io::Error`; its `get_ref()` exposes the original cause. Non-transport
restore failure is fatal and drops the affected baseline before returning,
recording teardown in that failed acquisition. The next acquisition builds an
empty slot. Dead transport still permits only one recovery build and retains
both causes if replacement fails. Snapshot senders, funding policy, capacity,
FIFO waiting, unwind invalidation and completed phase timings remain explicit.

Fresh `StandaloneCanisterFixture` values remain available. This recipe restores
only the captured canister; it does not reset time, extra canisters, pending work
or external resources. No retained cache or protocol layout is changed. Existing
pools are process-local; stop/restart their test processes with the new caller
code. Canic's payload-limit suite must adopt the new guard/call boundary after
0.38.0 publication ([#76](https://github.com/dragginzgame/ic-testkit/issues/76),
[Canic #540](https://github.com/dragginzgame/canic/issues/540)).

### Changed: convenience and worker-file ownership

The six anonymous/panicking Candid convenience methods have trait defaults;
PocketIC implements only explicit-sender query/update operations. Existing call
signatures, structured failures, caller tracking and application-level Result
handling remain unchanged. Worker stdout uses an explicitly private temporary
File with automatic handle-lifetime cleanup instead of a named scratch directory.
Output remains file-backed and read with the same complete-report limit; worker
completion and server cleanup still precede interruption or sampling failure.
No elapsed-time build or worker deadline is added
([#76](https://github.com/dragginzgame/ic-testkit/issues/76)).

### Fixed: benchmark artifact handoff

The repository-local fixture benchmark acquires its probe through IC Build and
holds the retained result until all workers and report publication finish. Both
the Wasm hash and every worker use the same exact artifact rather than Cargo's
mutable output path ([#72](https://github.com/dragginzgame/ic-testkit/issues/72)).
The driver retains its native workload recipe, profiles, sampling, report schema,
selected Cargo/compiler and locked/offline preparation. Ordinary foreground build
handling and interruption checks remain; no build deadline is added. Exact cache
entries live under the selected Cargo target's `ic-testkit-fixture-benchmark-wasm`
directory, with shared incremental compilation under `ic-testkit-incremental`.
Cleanup remains explicit; earlier targets and retained artifacts are preserved.
Build failures keep the typed IC Build cause, including status and diagnostics.

### Changed: development dependency and helper ownership

The tracked lockfile selects published IC Build 0.2.3 for native development,
aligning its Host dependencies with Testkit's 0.17.4 selection and removing the
separate Host 0.16 graph ([IC Build #9](https://github.com/dragginzgame/ic-build/issues/9)).
This grouping API type change stays outside Testkit's exposed library boundary;
IC Build remains a development dependency. Diagnostics validates
labels directly at its contract owner, preserving empty/duplicate indices and
rejection before collection. Pool phase accumulation uses standard saturating
duration arithmetic, preserving unrecorded versus zero-duration phases
([IC Build #7](https://github.com/dragginzgame/ic-build/issues/7)). These changes
preserve library APIs and stored formats; no consumer migration is required.

### Changed: fixture lifecycle flow

Canister installation executes directly in its owning extension method.
Snapshot capture shares partial-set rollback before projecting a rejection or
panic, retaining both the original cause and independent cleanup failures.
Creation, funding and install order, exact senders, fallback policy and failed
standalone instance custody are unchanged; no public API or stored format changes
([#74](https://github.com/dragginzgame/ic-testkit/issues/74)).

### Changed: Host mechanics

PocketIC executable admission uses Host's no-follow identity/version boundary,
removing the extra whole-file hash while retaining alias and special-file
rejection before execution. Native process observations and checked descendant
RSS summation now have their canonical implementation in Host; sampling cadence,
workload selection, native source identities and report fields remain unchanged.

Completed benchmark reports use Host's checked descriptor publication with
private permissions and an existing parent. Cancellation after file sync rejects
before replacement. Producer/prepublication errors preserve any earlier report;
a final directory-sync failure warns after the complete new report is visible,
without inviting a retry. Atomic visibility remains the contract. Worker stdout
capture retains its separate private scratch lifetime. No cache-format, library
API, compiler/MSRV or build-deadline change is introduced
([#75](https://github.com/dragginzgame/ic-testkit/issues/75),
[Host #79](https://github.com/dragginzgame/ic-host-tooling/issues/79)).

### Changed: Shared Tooling 0.10 catalog ownership

The reviewed snapshot selects `ed9c26f015691859f67b50b512a94709dcdc0c6c`.
The existing common version/digest selections are unchanged; setup, formatting,
freshness inspection and evidence collection use the catalogs owned by that
snapshot. Redundant Testkit Make defaults are removed. Host, Rust and IC catalog
exceptions must be selected independently and qualified explicitly
([#73](https://github.com/dragginzgame/ic-testkit/issues/73),
[Shared #153](https://github.com/dragginzgame/shared-tooling/issues/153)).
Compiler/MSRV, application dependencies and PocketIC selection retain their
existing owners. This is compatible contributor tooling, with no library API,
stored-format or retained-installation change.

The same snapshot rejects line-break LOC directory aliases before selecting a
checkout, and preserves release-lock filesystem diagnostics and exit status
([Shared #151](https://github.com/dragginzgame/shared-tooling/issues/151),
[Shared #152](https://github.com/dragginzgame/shared-tooling/issues/152)).

## [0.37.0] - 2026-10-13

### Breaking: exposed Host dependency types

The public `ic_host_artifacts`, `ic_host_fs` and `ic_host_process` re-exports and
typed startup diagnostics now select IC Host 0.17. Align direct Host dependencies
to 0.17 wherever their types cross the Testkit boundary. These subcrates retain
their source APIs; their new crate identities require a Testkit minor release.
The aggregate Host crate's Candid feature split does not affect Testkit, which
does not depend on it. The selected IC Build 0.1.3 development dependency retains
its separate Host 0.16 graph; no conversion or compatibility bridge is added.

### Changed: Shared Tooling 0.9.15

The reviewed snapshot selects `a9a8ea5ddea0db279d8c139ef5a951474c50f742`.
CI inspection retains empty failed-step observations and recovers full logs only
from failed/timed-out jobs in the selected attempt. Partial API/download failures
remain errors; no rerun or attempt switching is introduced
([Shared #149](https://github.com/dragginzgame/shared-tooling/issues/149)).
Maintenance rejects CR/LF in literal and physically resolved paths before CLI
probes or state writes, preserving existing reports and schedules
([Shared #150](https://github.com/dragginzgame/shared-tooling/issues/150)).
This adoption does not change the executable-tool build policy or activate tasks.

### Changed: managed-server ownership

Managed servers always own their child process. Port readiness and bounded log
capture belong to the startup files, so file-reader fixtures no longer construct
childless servers. Termination still preserves the primary and typed cleanup
failures and finishes child cleanup before reading final diagnostics. Background
reaping retains the child and files together, including thread-spawn failure.

### Changed: fixture-pool acquisition

Each fixture pool now uses one construction path for empty/invalid slots and
recoverable restore failures, removing repeated teardown, build and result
handling ([#71](https://github.com/dragginzgame/ic-testkit/issues/71)). The standalone
acquisition forwarding layer is removed. Restoration still follows each pool's
own policy: ordinary standalone restore failures invalidate for the next lease,
while recognized dead transport can rebuild immediately; recipe baselines retain
caller classification and fatal errors. Recovery makes at most one build attempt
and preserves both original and replacement failures with phase timings.

Dedicated-server recovery probes use Testkit's managed child lifecycle, and CI
runs both probes on every supported native host. The added standalone probe also
checks that failed recovery leaves an empty slot that can build again. FIFO
capacity, lazy construction and unwind invalidation retain their existing owner.
The pool and startup simplifications preserve public behavior and stored formats.
The Host dependency boundary above is the reason for this minor release.

## [0.36.0] - 2026-10-13

### Breaking: exposed Host dependency types

Testkit's public `ic_host_artifacts`, `ic_host_fs` and `ic_host_process` re-exports
now select IC Host 0.16. Align direct Host dependencies to 0.16 when sharing their
types with Testkit, including `PocketIcStartupFailure` probe errors and cleanup
reports. The three Host subcrates have unchanged source APIs from 0.15.2; their
new Rust crate identities still require a Testkit minor release at this exposed
consumer boundary. The aggregate `ic-host-tools` change to empty default features
does not apply here: Testkit does not depend on or re-export that crate.
IC Build's existing development-only dependency retains its separate Host 0.15
selection; no bridge, type conversion or implicit dependency update is introduced.

### Changed: Shared Tooling 0.9.13

The reviewed snapshot selects `35cad65677ff989b5fb24a88d947fda37d2366e8`.
Tool-store and checkout locks preserve filesystem diagnostics and exit statuses,
and source-acquisition failures stop before build/activation without losing the
previous selection ([Shared #145](https://github.com/dragginzgame/shared-tooling/issues/145),
[Shared #146](https://github.com/dragginzgame/shared-tooling/issues/146)). Explicit
`make install-rust-tools` prepares the changed build-helper identity; old tool
generations remain retained.

Authorized development may run `cargo fetch --locked` for the existing selected
lockfile without another approval; offline settings, executable installation and
release authority remain separate
([Shared #147](https://github.com/dragginzgame/shared-tooling/issues/147)). The CI
inspection helper refuses failed or empty current-branch observations instead of
selecting unrelated runs, while explicit branch/commit/run selection remains
available ([Shared #148](https://github.com/dragginzgame/shared-tooling/issues/148)).

### Added: retained instance construction

`PocketIcStartupConfig::build_instance(builder)` returns
`PocketIcInstance<PocketIc>` whose public `instance` and optional `server` fields
retain the exact owned server in spawn mode. Field order deletes the synchronous
instance before server teardown. Connect mode returns no server ownership.
Caller topology, state/request settings, output paths and TTL selections stay
on the existing builder/configuration
([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

`build_instance_async(builder).await` exposes the upstream nonblocking instance
on the caller's Tokio runtime with its time driver enabled. Readiness and instance
waits yield, sharing typed startup failures, bounded output and independent
cleanup diagnostics. Dropping construction reaps its owned server during either
phase; explicit output files retain cancellation diagnostics. A borrowed server
is untouched, and an instance created before timeout or a lost response requires caller
reconciliation ([#69](https://github.com/dragginzgame/ic-testkit/issues/69)).
Process spawning, bounded file reads and owned-child cleanup remain synchronous;
startup deadlines cannot preempt synchronous work within upstream future polls.
Async instances require explicit `instance.drop().await` while retaining the
server. Neither API bounds subsequent instance operations or upstream deletion.
The existing one-shot `try_build` behavior remains unchanged. These construction
additions are compatible; no stored format changes are required. The exposed Host
dependency change above selects the minor release for this batch.

### Fixed: benchmark protocol admission

The fixture-reuse driver uses `pic::supports_pocket_ic_server` for its selected
binary instead of maintaining another version parser. All admission callers
share stable 16.x.y syntax, including leading-zero and overflow rejection
([#70](https://github.com/dragginzgame/ic-testkit/issues/70)).

### Testing

Prepared-server lifecycle qualification observes its exact child's published path
instead of predicting it from a shared counter, avoiding a parallel-test race.

## [0.35.1] - 2026-10-13

### Changed: Shared Tooling and CI tool reuse

The reviewed snapshot selects Shared Tooling 0.9.10
`7e55adecf6f0b196c050f104b37f8c69b6bf434e`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). Cargo-tool setup
rejects non-boolean Cargo `net.offline` configuration, preserving explicit
environment overrides ([Shared #140](https://github.com/dragginzgame/shared-tooling/issues/140)).
The changed build-policy digest requires explicit `make install-rust-tools`
before offline tool/formatter checks; previous builds remain retained.

The checks and portable CI jobs use the canonical installer identity to cache
Cargo sources and physical Rust tool generations on each native host
([Shared #142](https://github.com/dragginzgame/shared-tooling/issues/142)). Every
run still performs complete setup and receipt/byte admission. Only main pushes
save verified generations, before later validation; PRs only restore. Cache
transport is bounded and nonfatal, while setup/admission failures remain fatal.
Checkout selections and credentials are excluded; there are no broad restore
prefixes. The existing library/Wasm/MSRV gates and artifact readback remain.
Hosted cache hit rates and timing improvements need matching CI observations.

The selected Rust-tool fixture isolates its compiler context and retains real
staging/receipt checks without creating commits
([Shared #143](https://github.com/dragginzgame/shared-tooling/issues/143),
[Shared #144](https://github.com/dragginzgame/shared-tooling/issues/144)). The
baseline adds scoped change-reporting rules; Testkit's library API and manifest
versions are unchanged by this contributor-tooling adoption.

### Changed: selected native dependencies

The tracked lockfile selects published IC Build 0.1.1 for native development.
Cargo grouping retains the original typed Host failure, exit status and admitted
stdout/stderr, and rejects duplicate or empty package names before command
effects ([IC Build #3](https://github.com/dragginzgame/ic-build/issues/3),
[IC Build #4](https://github.com/dragginzgame/ic-build/issues/4)). Testkit's
existing artifact imports and benchmark recipes need no changes. IC Build stays
a development dependency; its grouping API is not re-exported by Testkit.

The selected IC Host artifact, filesystem and process dependencies advance to
0.15.1 without library API changes. Testkit's public API, Rust 1.88 floor, retained
cache contracts and build-deadline policy are unchanged.

### Testing: retained build handoff

The canister benchmark integration test acquires its Wasm with locked Cargo
inputs, reads the exact artifact path from IC Build's retained record and keeps
that record alive through PocketIC installation and marker verification. It
releases the fixture and build record before removing its temporary output.
Generic cache reuse, corruption repair, environment invalidation, maintenance
and timing checks stay in IC Build's behavioral coverage, so this test needs
only one build acquisition
([IC Build #1](https://github.com/dragginzgame/ic-build/issues/1)).

## [0.35.0] - 2026-10-12

### Breaking: generic build ownership

Generic Cargo input resolution, Wasm/artifact caches, retained records,
transactions, batching and explicit maintenance now belong to IC Build.
Add a direct native dependency and use `ic_build::artifacts`; no aliases or
compatibility reexports remain. Source code keeps the existing v1 stamp bytes,
digest domains and cache/lock namespaces. Retain build/cache records through
consumption and keep recipe, source custody, budgets and cleanup policy explicit.

PocketIC startup, provisioning, snapshots, diagnostics and benchmark/marker
contracts stay in Testkit. Native development uses published IC Build 0.1 from
crates.io and the tracked lockfile, so staged formatting needs no sibling
checkout. Manifest versions remain maintainer-owned.
[IC Build #1](https://github.com/dragginzgame/ic-build/issues/1) owns acceptance.


### Breaking: Host 0.15 and direct IC-tool imports

The three artifact, filesystem and process re-exports select IC Host 0.15.
Align direct dependencies to that line wherever their types cross the Testkit
boundary ([#67](https://github.com/dragginzgame/ic-testkit/issues/67)).
`ic_testkit::ic_host_tools` is removed: this crate had no internal callers, so
Testkit no longer depends on it solely to expose its API. Add `ic-host-tools`
directly for IC-specific helpers, using its current split imports:

| Previous convenience path | Direct Host 0.15 path |
| --- | --- |
| `ic_testkit::ic_host_tools::candid::{normalize, NormalizationError}` | `ic_host_tools::candid::text::{normalize, NormalizationError}` |
| `ic_testkit::ic_host_tools::candid::{extract, ExtractionError, ExtractedCandid}` | `ic_host_tools::candid::extraction::{extract, ExtractionError, ExtractedCandid}` |
| Other APIs under `ic_testkit::ic_host_tools` | Their corresponding `ic_host_tools` paths |

Pure Candid text APIs also work with Host's default features disabled; extraction
requires Unix and the `candid-extraction` feature, enabled by default. See the
[Host migration guide](https://github.com/dragginzgame/ic-host-tooling/blob/5253af17427ec5f7054d6fe670993bb0722f924e/docs/changelog/0.15.md)
for feature selection and optional interface inspection. No compatibility bridge
is retained. PocketIC/cache contracts and the Rust 1.88 floor are unchanged.

### Fixed: uploaded artifact discovery

The reviewed Shared snapshot selects 0.9.5
`b1630b89482cc6071fc928a86962d79c9c11fdb7`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[#59](https://github.com/dragginzgame/ic-testkit/issues/59),
[Shared #63](https://github.com/dragginzgame/shared-tooling/issues/63)).
Portable CI uses the canonical discovery helper for the exact artifact ID returned
by its upload. Six successful API listings can retry absence with bounded backoff;
the step has a two-minute timeout including requests. Authentication/API errors,
malformed responses, expired or conflicting identities and exhausted discovery
fail the gate. The pinned download runs only after discovery succeeds and retains
digest-mismatch refusal and independent archived-payload verification. Uploads
are not repeated and no other artifact, run or attempt is selected.

## [0.34.7] - 2026-10-12

### Fixed: transient contributor-tool downloads

The reviewed snapshot selects Shared Tooling 0.9.4
`4532b66c1b43bab331978523ee97d29fe11a2f50`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138)).
Explicit host and IC asset setup uses curl's transient retry classification,
with at most three retries, a two-second delay and a 60-second retry-start
window. Each attempt retains its 15-second connection and 300-second transfer
limits; an in-flight attempt can finish after the retry window.

Permanent HTTP failures and checksum/admission errors remain terminal. Exhaustion
preserves curl's final failure status, failed candidates and the previous active
tool selection. HTTPS restrictions and checksum verification remain in effect.
Offline checks and library behavior are unchanged; existing tools need no
reinstall solely for this adoption.

### Fixed: snapshot and release identities

Snapshot parsing, refresh observations and direct/PR release recovery require
complete 40- or 64-character hexadecimal Git identities
([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
Malformed intermediate-length identities refuse before refresh or release
effects, preserving saved plans and evidence. Valid v1 snapshots and recovery
records need no conversion.

## [0.34.6] - 2026-10-12

### Fixed: installer cleanup and evidence admission

The reviewed snapshot selects Shared Tooling 0.9.2
`c118062a070a2e238f1eb3989b6fd700f369ce1e`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). Host and IC setup
share lock acquisition, verified-cache reuse and cleanup at the shared store
owner. Both lock releases are attempted even when checkout cleanup fails.
An earlier installation error retains its original exit status and failed
candidate; a cleanup failure after successful installation returns nonzero.
Inspect any remaining checkout lock before retrying, even if a verified bundle
was activated successfully.

Evidence checksum verification rejects NUL bytes before hashing referenced files,
then parses the same captured manifest. Malformed digest or filename bytes can
no longer disappear during Bash parsing. Rejected manifests and evidence remain
intact. Host, IC and Rust installers reject CR/LF in both supplied and resolved
consumer paths before loading configuration, probing tools or preparing stores,
including aliases that hide invalid physical paths.

Tool pins, retained installations, offline admission and library contracts are
unchanged. Existing tools need no reinstall solely for this adoption.

### Testing: controlled cache-lock release

The nonblocking cache-lock regression runs in an exact-test subprocess with one
test thread ([#66](https://github.com/dragginzgame/ic-testkit/issues/66)). Other
parallel tests can no longer fork while this fixture holds its lock and briefly
extend that lock's lifetime before exec. Contention, immediate reacquisition,
content preservation and redirected-file assertions remain in the isolated
child. Production locking behavior is unchanged.

## [0.34.5] - 2026-10-12

### Changed: Shared Tooling 0.9.0

The reviewed snapshot selects Shared Tooling 0.9.0
`aec7a149f581a42b9531fc747b46024fc0314fcf`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). The selection adds
the canonical freshness checker, its task guide and fixture. Tool pins and
prepared selections remain unchanged.

Snapshot refresh and verification now share one manifest reader, which rejects
NUL bytes before parsing the captured input
([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
`bash scripts/ci/verify-shared-tooling-snapshot.sh --read-manifest FILE` emits
validated records only after the entire manifest passes syntax checks, preserving
record order and literal field bytes. It establishes syntax, while ordinary
verification still checks payload bytes and modes and refresh separately proves
ownership. Existing v1 snapshots need no conversion or tool reinstall.

Formatting admission now reads `RUST_TOOL_VERSIONS`, matching installation even
when host and Rust catalogs are separate
([Shared #135](https://github.com/dragginzgame/shared-tooling/issues/135)). Failed
Cargo builds refuse redirected observation files before recording diagnostics,
preserving outside bytes, failed evidence and the previous tool selection
([Shared #136](https://github.com/dragginzgame/shared-tooling/issues/136)).

Snapshot refresh atomically replaces each admitted payload before publishing
the manifest, so an interrupted copy preserves complete destination bytes and
can be retried. Minimal Make selections now derive their common installers and
default LOC helper automatically
([Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)). Ignore
`.shared-tooling-payload.*` scratch files after an ungraceful interruption.
Successful ripgrep PCRE2 checks are quiet; failed probes retain diagnostics.

Run `bash scripts/dev/cloc.sh --json` for numeric workspace LOC and test counts,
including per-crate rows and totals. JSON appears only after all counts succeed;
a failed count preserves its failure status and emits no partial document. The
default human-readable report remains available through `make cloc`. Snapshot
guidance now consistently distinguishes explicitly selected roots from the
script companions derived by the exporter.

Run `perl scripts/dev/check-cargo-tool-versions.pl --consumer "$PWD"` for an
explicit online report, or add `--json` for structured output. It compares catalog
pins, checkout receipts and user Cargo receipts with non-yanked stable crates.io
releases, retaining registry responses and reporting missing or unavailable
observations separately. It uses Perl core modules and curl 8.4 or newer and
does not execute installed tools. Receipts describe versions; `make rust-tools-check`
continues to authenticate prepared bytes and build identities. See the
[task guide](../../tasks/cargo-tool-versions.md) for statuses and alternate inputs.
Portable CI exercises the checker with substitute registry responses, including
outdated/missing selections and incomplete observations.

The maintainer's lock update selects all four Host crates at 0.14.7. This patch
changes contributor tooling; its crate implementation is unchanged from 0.14.6.
Library contracts and MSRV remain compatible.

## [0.34.4] - 2026-10-12

### Changed: Shared Tooling 0.8.1 and locked Host 0.14.6

The reviewed Shared snapshot advances to 0.8.1
`c97f9f9c7b7c706fffdcfc5925cf58759b58093f`, retaining the same entrypoints and
90 files ([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). Snapshot
refresh and verification reject malformed or empty extra fields while preserving
literal spaces in filenames
([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).

When a Cargo tool's compiler, native inventory or build policy changes during
installation, activation still refuses. The failed attempt now retains both
identity observations, names the changed fields and identifies evidence to compare
([Shared #134](https://github.com/dragginzgame/shared-tooling/issues/134)). The
normal failure collector includes those observations. Prepared selections and
tool pins are unchanged; no reinstall is required for this adoption alone.

The maintainer's lock update selects all four Host crates at 0.14.6. This patch
adopts contributor tooling and leaves the crate implementation unchanged from
0.14.5. Library contracts and MSRV remain compatible.

### Fixed: interrupted PocketIC downloads

`ic-testkit-server setup` now observes SIGINT, SIGTERM and SIGHUP during downloads
and cancels the downloader's owned process group before returning `128 + signal`
([#65](https://github.com/dragginzgame/ic-testkit/issues/65)). The partial archive,
captured stderr and original cancellation failure remain in the attempt directory;
a failure to save stderr is reported separately without replacing cancellation.
The CLI emits no successful executable path after observing interruption.

Setup reuses Host's owned-child communication and the CLI's signal policy. HTTPS
admission, output budgets, explicit setup-only downloads and offline check/run
behavior are unchanged. Descendants that leave the owned group remain caller-owned.
This compatible fix requires no installation reset, dependency update or library
migration.

### Testing: build-identity artifact readback

The maintained normal artifact proof compares exact bytes for both build-identity
and native-environment observations
([#59](https://github.com/dragginzgame/ic-testkit/issues/59)). Missing build identity
or changed native observations refuse readback while preserving extracted evidence.
The compact collector is checked against the original observations; existing
rejected-selection proofs retain their admission boundaries. This extends the
existing upload/download proof rather than adding another artifact interface.

## [0.34.3] - 2026-10-12

### Changed: locked IC Host 0.14.5

All four Host crates select 0.14.5 in the maintainer's lock update. Host 0.14.4
adopts Shared Tooling 0.6.6 and the fixed derive formatter; 0.14.5 only changes
release metadata. Their crate implementation is unchanged. Public Host
re-exports retain the compatible 0.14 contract, with no library migration or
MSRV change.

### Changed: Shared Tooling 0.8.0 snapshots

The reviewed snapshot selects Shared Tooling 0.8.0 at
`045a6f22a9de497bfb3af90cb2571957384806f2`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)). The
generated version annotation now agrees with the upstream 0.8.0 source metadata.

The v1 manifest now records intended entrypoints separately from derived file
records. Refreshes reuse those roots and automatically include transitive
companions. The existing 90-file selection is retained. To change roots, use
`--entrypoint` or `--add-entrypoint` with the new exporter and review `--preview`;
refresh automatically retires unchanged owned files outside the selection after
proving their prior source, digest and mode. Local edits, staging and concurrent
changes still refuse. Remove `--retire-file` from any external refresh commands;
directly used helpers must remain selected entrypoints. See the
[adoption guide](../../docs/consuming-snapshots.md#retiring-obsolete-payloads).
Prepared tools, pin selections and library contracts are unchanged.

### Fixed: failure evidence for linked checkout directories

Both CI evidence modes retain unrelated logs, step outcomes and a diagnostic
when `target` or `.tools` is a symlink
([#59](https://github.com/dragginzgame/ic-testkit/issues/59)). The archive records
the link itself without traversing its target. Collection still fails after
emitting the completed archive path, allowing the existing uploader to retain
the evidence. A concurrent shared-selector failure preserves its original
status and diagnostics; partial selections remain diagnostic data only.

## [0.34.2] - 2026-10-12

### Changed: locked IC Host 0.14.3

All four Host packages select 0.14.3 in the maintainer's lock update. The reviewed
release changes contributor tooling; its crate implementation is unchanged from
the preceding 0.14 patches. Public Host re-exports remain on the same compatible
0.14 contract, with no consumer migration or new build requirement.

### Changed: Shared Tooling 0.6.6 validation and formatting

The reviewed snapshot selects Shared Tooling 0.6.6
`a15eb9d406d287697df9bc8fa96db8cda6721a71`. Staged formatting can admit prepared
Rust-tool receipts from the checkout while checking pins and compiler selection
from the exported worktree; mismatches still refuse without installation
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).

Both `ci` and `release-check` checkpoint the Makefile, complete Cargo manifests,
lockfile, declared toolchain, tool pins and Shared snapshot. If an input changes
or disappears, the gate retains observations and stops before the next stage.
Nested gates preserve the original checkpoint; target failures and their logs
remain intact
([Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129)).
Restart explicit preparation and the complete gate against the current inputs
after a refusal. This contributor-only change preserves package versions,
library APIs, installed tools and release recovery state.

The common derive formatter selects `cargo-sort-derives` 0.14.1, fixing literal
`$crate` paths in direct and conditional macro derives
([Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113)).
Explicitly run `make install-rust-tools`, then `make rust-tools-check` after
adopting the pin. Building this tool requires Rust 1.88 or newer; the library's
MSRV is unchanged. Older tool bundles remain retained. Testkit has no affected
exported macros or corruption-workaround markers to remove, and its formatter
policy remains unchanged.

### Fixed: PocketIC store path admission

Setup, offline checks and owned runs reject LF/CR in a store directory or its
ancestors, including a working directory used to resolve a relative selection
([#64](https://github.com/dragginzgame/ic-testkit/issues/64)). Admission checks the
complete absolute path before normalization, lock creation or external tools.
This enforces the existing directory policy; retained artifacts and installed
bundles are not renamed or removed. Other path bytes and library APIs are unchanged.

The host prerequisite guide now selects `make install-rust-tools`, offline
`make rust-tools-check` and `.tools/rust/common/bin`, matching the maintained
shared setup contract
([#57](https://github.com/dragginzgame/ic-testkit/issues/57)).

## [0.34.1] - 2026-10-12

### Changed: Shared Tooling 0.6.4 Make interface

The reviewed snapshot advances to Shared Tooling 0.6.4
`6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d`. Its common Make entrypoint now owns
setup/check, formatting, hook installation, snapshot verification and release
routing, replacing separate includes and duplicate local recipes
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).
Hook setup still requires prepared formatters. Ordered PocketIC installation and
offline verification, the default help command, product gates and release and
publication adapters stay local. Isolated hook/release exports include the new
entrypoint. Existing tools need no reinstall for this adoption, and no public
command, library API or retained format changes.

### Fixed: local Make job controls

Product Cargo recipes now inherit Make's parallel job controls, including test
shell chains and the publication adapter
([#63](https://github.com/dragginzgame/ic-testkit/issues/63)). The local owner CLI
has one absolute executable path shared by setup, offline checks and tests.
Existing locked dependency selection, setup ordering, offline admission and
separate-process test coverage remain intact. Contributor tooling changes are
compatible; library APIs and package versions are unchanged.

### Fixed: primary PocketIC download failure

Failed setup retains curl's original nonzero or signalled process error even
when its stderr cannot be saved. The recording failure is reported separately;
`failure.txt` and the CLI retain the download failure as the primary cause
([#62](https://github.com/dragginzgame/ic-testkit/issues/62)). Partial archives,
failed attempts and existing bundles remain intact on retries. A successful
download still stops before installation if recording stderr fails. This is a
compatible error-reporting fix; no CLI arguments, formats or limits change.

### Testing: hosted failed-selection evidence readback

CI's existing artifact round trip now includes the normal qualification archive
and the checks/portable archives produced by rejected shared-tool selection
([#59](https://github.com/dragginzgame/ic-testkit/issues/59)). The separate verifier
consumes their proof directory and applies common checks for local build
logs/receipts, formatter and validation diagnostics, links, original selector
status, incomplete collection and step outcomes. It also checks validation logs
for checks archives and portable fixture/readback files for portable archives.
Rejected shared payloads, unrelated link targets and the other mode's diagnostics
stay excluded.

Corrupt payload, outcome and selector-status controls reject the proof and retain
extracted evidence. Readback compares exact expected bytes, including trailing
newlines; added-newline controls cover normal payloads and rejected-selection
build logs. Missing validation diagnostics and portable payloads also reject the
proof while retaining extracted evidence. All three archives use the existing
pinned upload/download actions, exact returned artifact identity and digest
admission. Library, server
CLI, installed tool selections and retained formats are unchanged.

## [0.34.0] - 2026-10-11

### Breaking: IC Host 0.14 completed process results

All four public Host re-exports select IC Host 0.14. Align direct dependencies
when their types cross the Testkit boundary
([#61](https://github.com/dragginzgame/ic-testkit/issues/61),
[Host #66](https://github.com/dragginzgame/ic-host-tooling/issues/66)).
Capture, communication and `AdmittedTool::run` return `CompletedExecution`, whose
`status` is a required `ExitStatus`. Nonzero and signalled completion are results;
spawn, IO, cancellation and cleanup failures still retain optional-status
`ExecutionEvidence`. Replace completed-status assertions and nonzero-exit error
unpacking with direct status handling. Completed evidence in version admission,
Git provenance and Candid extraction also uses the new type.

Call `require_success()` before treating an external producer's output as valid
or committing its artifact transaction. Use `require_complete()` on completed
results before parsing bounded output as an identity or structured response.
Output observers return `io::Result<()>`; infallible progress observers finish
with `Ok(())`. Host now retains a sink failure as `ExecutionFailure::Observer`
with the stream, original cause and secondary cleanup evidence; update exhaustive
failure matches. No compatibility entry points are retained.

Testkit's Cargo paths now project completed status directly while preserving
phase errors, all output events, bounded diagnostic retention and builds with no
deadline. PocketIC downloads and server-version probes, as well as the artifact
composition example, explicitly require success before admission/publication.
Failed download retries retain each partial archive, stderr and original process
failure. Existing retained caches and installed bundles need no reset or reinstall.

Native artifact composition coverage rejects valid staged Wasm from nonzero and
signalled producers, preserves the prior public artifact, and verifies that each
failed acquisition is executed again rather than reused.

### Fixed: restart committed release preparation

Developer tooling adopts Shared Tooling 0.6.3. A normal direct release can archive
a stale preparation intent after a clean descendant commit, select the requested
increment from current package metadata, and repeat preflight and full validation.
Dirty or divergent source, staging evidence and identity/destination conflicts
remain refusals. Explicit resume stays bound to its saved source and version
([Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126)).

Consumer qualification covers rejection of the previous source's validation,
retained receipts/logs/build evidence after failed revalidation, and fresh-source
binding before metadata preparation. Existing contributor tool installations need
no reinstall; this tooling update leaves the product CLI unchanged.

## [0.33.8] - 2026-10-11

### Fixed: Cargo-tool installation failures

Developer tooling adopts Shared Tooling 0.6.2. Common Cargo-tool setup now
propagates lock, validation and activation failures instead of continuing and
reporting success. Cleanup releases only acquired locks, and failed setup keeps
the previous common selection and retained attempts
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120)).

Run `make install-rust-tools`, then `make rust-tools-check` to select the corrected
build policy. Contributor PATHs remain unchanged, previous installations stay
retained, and library and product CLI contracts are unchanged.

### Fixed: interrupted-release source diagnostics

Release retries report the saved plan, phase, base/candidate versions and both
source commits when HEAD differs from the validated source. Preparation conflicts
point to the [guarded maintainer recovery procedure](https://github.com/dragginzgame/ic-testkit/blob/main/docs/releases.md#source-changes-after-saved-preparation-intent).
Old validation cannot qualify the changed source, and the runner preserves the
saved intent and evidence instead of automatically discarding them
([Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126)).

### Changed: IC host dependencies

The locked IC host crates advance to the compatible 0.13.5 release, including
Host's additive private regular sidecar opener.

## [0.33.7] - 2026-10-11

### Changed: shared Cargo-tool storage

Developer tooling adopts Shared Tooling 0.6.0. Cargo-built tools now use the
canonical per-user store with exact source/packaged-lock, compiler, native
environment, executable and profile identities. Tool builds use a standard
environment; application flags and wrappers remain local. Explicit offline
policy is preserved. Refresh setup with `make install-rust-tools`, then run
`make rust-tools-check`; the common PATH stays `.tools/rust/common/bin`.
Previous installations and failed attempts remain retained, while offline checks
require the new shared selections.

The canonical selector owns local Rust evidence and referenced shared Cargo
generations, including failed builds. Consumer archives exclude unrelated
generations without duplicating local Rust selections
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120)).

### Fixed: failure evidence after rejected shared selection

CI retains local installation logs, Rust receipts/build attempts, tool references
and step outcomes even when the shared selector fails. The archive records the
original selector status, diagnostic log and partial output, marks shared
collection incomplete, and excludes rejected shared payloads and link targets.
Partial selections never authorize filesystem reads. Both failure collectors
publish the completed archive path before returning the original failure status,
so the upload step can retain it without reporting collection as successful
([#59](https://github.com/dragginzgame/ic-testkit/issues/59),
[Shared #121](https://github.com/dragginzgame/shared-tooling/issues/121)).

This is contributor tooling only; library and product CLI contracts are unchanged.

## [0.33.6] - 2026-10-11

### Changed: shared host and IC tool storage

Developer setup adopts Shared Tooling 0.5.0's per-user verified store. Matching
host and IC selections reuse physical bundles across checkouts, while each
checkout keeps its pins and its `.tools/host/bin` and `.tools/ic/bin` paths.
After refreshing, explicitly run:

```sh
make install-host-tools
make install-ic-tools
make host-tools-check ic-tools-check
```

Use the same `SHARED_TOOLING_STORE` setting for setup, checks and evidence
collection when overriding the default per-user store. Previous local bundles
and failed attempts remain available; offline checks do not accept the old
physical layout or repair installations. Rust tool slots and PocketIC storage
keep their existing owners and paths. The library API, consumer build
requirements and product CLI contract are unchanged.

Consumer archive qualification now creates selections through the canonical
installer. Failure archives retain referenced failed/corrupt shared generations
and exclude unrelated store entries; verified active sets can remain compact
([#58](https://github.com/dragginzgame/ic-testkit/issues/58),
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120)).

## [0.33.5] - 2026-10-11

### Changed: prepared developer tool layout

Shared Tooling's common Rust tools now use verified, versioned installation
slots, activated together at `.tools/rust/common/bin`. Update interactive PATHs
to that directory and explicitly run `make install-rust-tools`, followed by
`make rust-tools-check`. CI and formatting hooks use the same selection. The
installer retains earlier executables, receipts and build evidence; offline
checks do not accept the previous `.tools/rust/bin` layout or repair it.
Existing product CLI and PocketIC selections keep their paths
([#57](https://github.com/dragginzgame/ic-testkit/issues/57),
[Shared #118](https://github.com/dragginzgame/shared-tooling/issues/118)).

CI failure archives retain the physical Rust installation tree, including
admitted slots and failed attempts, independently of compact host/IC evidence.
Links remain archived as links without reading their external targets
([#57](https://github.com/dragginzgame/ic-testkit/issues/57)).

## [0.33.4] - 2026-10-11

### Fixed: release and publication temporary paths

Release-guard, release-metadata and publication fixtures physically anchor their
scratch roots and pass that parent to child helpers before changing directory.
Relative or symlinked `TMPDIR` selections preserve temporary-file placement,
substitute command selection and failed evidence
([#56](https://github.com/dragginzgame/ic-testkit/issues/56)).
Qualification still substitutes Git, registry, installation and release effects.

### Fixed: Host 0.13 upgrade guidance

Testkit 0.33.3 already selects Host 0.13 for all four public host-crate re-exports.
This documentation correction changes no dependency selection or API
([#54](https://github.com/dragginzgame/ic-testkit/issues/54)).

When upgrading from 0.33.2 or earlier, align direct `ic-host-artifacts`,
`ic-host-fs`, `ic-host-process` and `ic-host-tools` dependencies with 0.13 wherever
their types cross the Testkit boundary. Host 0.12 and 0.13 types have distinct
dependency identities.

Host 0.13 removes `ic_host_fs::durable::lock_file_with_progress`. Open the lock
with `open_regular_lock_file_with_parents`, then pass that retained descriptor to
`lock_exclusive_with_wait`, selecting polling and progress-reporting intervals
in the caller. Testkit's cache already uses these descriptor APIs. Existing lock
contents, cache formats and server installations require no reset.

The public Host cut shipped under 0.33.3, although a breaking pre-1.0 change
requires a minor release. These notes document that existing incompatibility;
they do not reclassify the published release or restore the removed API.

## [0.33.3] - 2026-10-11

### Fixed

- Adopt Shared Tooling 0.3.12 so snapshot exports report the corrected source
  version, retaining the existing portability fixes
  ([shared #114](https://github.com/dragginzgame/shared-tooling/issues/114)).
- Keep temporary files from child hooks and formatters under the selected physical
  parent when qualification starts with a relative or symlinked `TMPDIR`; preserve
  failure diagnostics and source/index checks
  ([#55](https://github.com/dragginzgame/ic-testkit/issues/55),
  [shared #116](https://github.com/dragginzgame/shared-tooling/issues/116)).

## [0.33.2] - 2026-10-11

### Fixed: downloaded-evidence completion

Successful verification of the workflow's downloaded archive marks the fixture
complete before its EXIT guard runs. All mandatory archive comparisons fail
explicitly under Bash 3.2. The maintained fixture invokes this separate entry
point with valid and deliberately corrupted payload/outcome archives, requiring
valid verification to clean its scratch and failures to retain extracted bytes
([#53](https://github.com/dragginzgame/ic-testkit/issues/53)).
Archive selection, hosted artifact identity and production publication remain
unchanged.

### Fixed: shared setup and validation diagnostics

The adopted Shared Tooling revision runs Rust preflight, installation and offline
probes from the selected consumer's directory. Caller-relative version catalogs and
consumer-relative lockfiles preserve their documented resolution; explicit
Rustup toolchain overrides remain authoritative
([shared #101](https://github.com/dragginzgame/shared-tooling/issues/101)).
Live highlighting and retained validation summaries share one diagnostic
predicate, so fatal, LLVM and missing-file errors before a long output tail
remain visible. Neutral Rust test names and context remain unclassified
([shared #110](https://github.com/dragginzgame/shared-tooling/issues/110)).

### Fixed: portable temporary-directory selection

Host-tool fixtures normalize temporary roots before comparing physical installer
paths. Release-command qualification anchors relative and symlinked temporary
roots before changing directory and passes the same physical parent to nested
Make probes. Failure assertions, retained evidence and substitute-only release
routing remain intact
([shared #111](https://github.com/dragginzgame/shared-tooling/issues/111),
[shared #112](https://github.com/dragginzgame/shared-tooling/issues/112)).

### Changed: Host selection

All four Host crates select 0.12.9. Their Rust source and public APIs are unchanged
from 0.12.7; these upstream releases improve native development-tool qualification
and retention. No public Testkit API, retained format or installation reset changes
are required.

## [0.33.1] - 2026-10-10

### Fixed: formatting-hook observation failures

Shared Tooling 0.3.7 admits the status of each Git index-tree observation before
comparing its output. A failed observation stops formatting or publication of
formatted files even when Git prints the expected tree ID; the error status and
original working/index bytes are preserved
([shared #106](https://github.com/dragginzgame/shared-tooling/issues/106)).

### Fixed: malformed durable-publication paths

All four Host crates select 0.12.6; 0.12.6 adds CI tooling changes only over
0.12.5. Durable writes reject NUL-containing filenames
and parent components before creating directories; Testkit's atomic artifact
copy retains the original `InvalidInput` publication cause and existing output.
Public Host signatures and retained artifact formats are unchanged
([host #54](https://github.com/dragginzgame/ic-host-tooling/issues/54)).

### Fixed: portable assertion failures

Mandatory portable fixture comparisons fail explicitly on Bash 3.2. Selected
Cargo executable installation also refuses an absent host identity or a
non-executable candidate before activation, preserving failure evidence
([shared #107](https://github.com/dragginzgame/shared-tooling/issues/107)).

### Added: locked Cargo executable selection

The shared Rust tool installer accepts `--lockfile FILE` instead of an explicit
`--version` for a selected published Cargo executable. Setup and offline admission
share exact package/source/version selection; failed or changed selections retain
build evidence. Testkit continues building its own owner CLI from source, without
installing its published CLI into its own workflow
([shared #96](https://github.com/dragginzgame/shared-tooling/issues/96)).
No Rust API, retained-format or installation-reset changes are required.

### Changed: CI cancellation

- Keep only the newest CI run per workflow and branch or PR, cancelling older
  queued and running checks while retaining the existing host matrix and gates
  ([Shared #108](https://github.com/dragginzgame/shared-tooling/issues/108)).

## [0.33.0] - 2026-10-10

### Changed

- Select Host 0.12.4 for all four Host crates; their Rust source and public APIs
  are unchanged from 0.12.3.

### Breaking: connection-reset recovery classification

`is_dead_pocket_ic_transport_error` and `PocketIcOperationError` now recognize
OS `ConnectionReset` inside maintained reqwest/hyper HTTP errors for numeric
PocketIC instance URLs. The error retains its classification through installation
and snapshot causes and contextual I/O wrappers. macOS may report a reset when
an instance peer disappears instead of refusing the connection
([#52](https://github.com/dragginzgame/ic-testkit/issues/52)).

Recovery recipes can now choose `DeadPocketIcTransport` and rebuild for these
errors. Review custom classification/rebuild policy when upgrading; bare I/O
errors, quoted application text and non-instance URLs do not gain classification.
No API signature, dependency selection or retained-artifact format changes;
existing caches and installations need no reset.

### Testing: reset boundaries

Synthetic native peers now reset fully received query and installation requests,
verifying real reqwest error classification and preserved standalone ownership.
Diagnostic capture also checks that scoped resets become `InstanceUnavailable`
through contextual I/O wrappers
([#52](https://github.com/dragginzgame/ic-testkit/issues/52)).

## [0.32.2] - 2026-10-10

### Changed: common IC tools and Host selection

Shared Tooling 0.3.4 selects Binaryen 133 for the common development toolset.
Prepare it explicitly with `make install-ic-tools`; offline checks do not install
or replace tools, and earlier bundles remain retained. Testkit does not select a
production optimizer policy. Consumers must qualify their own optimized Wasm
before changing that policy
([shared #102](https://github.com/dragginzgame/shared-tooling/issues/102)).
All four Host crates select 0.12.3; their Rust source and public APIs are unchanged
from 0.12.2, so no API migration or retained-artifact reset is required.

### Fixed: validation runner admission and completion

Shared Tooling 0.3.3 rejects malformed inherited `VALIDATION_RUNNER_DEPTH` before
creating target logs or dispatching validation. Values must be canonical
non-negative decimal integers of at most 18 digits; unset or empty selects zero.
Premature runner exits, including Bash 3.2 expansion errors and early zero exits,
fail and preserve available runner-source/log evidence. Completed target failures
retain their original status; fail-fast routing and saved failure evidence are
unchanged ([shared #104](https://github.com/dragginzgame/shared-tooling/issues/104)).
No Rust API, installation reset or persisted-format change is required.

## [0.32.1] - 2026-10-10

### Fixed: portable tooling and qualification evidence

Shared Tooling 0.3.2 keeps Cargo's jobserver descriptors available during common
Rust setup/checks, formatting and LOC commands. The tool include admits safe
Make execution itself, so recursive recipe marking cannot bypass unsafe-mode
refusal. Testkit's preflight dispatch also retains jobserver access for its
locked Cargo preparation. Tool selection, setup order and offline checks are
unchanged.

Shared and consumer-owned qualification fixtures now distinguish completed
assertions from premature exits. Bash 3.2 expansion errors and early zero exits
must fail and preserve evidence; only completed successful fixtures clean up
([#51](https://github.com/dragginzgame/ic-testkit/issues/51),
[shared #99](https://github.com/dragginzgame/shared-tooling/issues/99),
[shared #103](https://github.com/dragginzgame/shared-tooling/issues/103)).
No Rust API, retained installation reset or persisted-format change is required.

## [0.32.0] - 2026-10-10

### Breaking: Host crate identity

All four public Host re-exports now select 0.12.1. Update direct Host dependencies
to the 0.12 line when passing their types into Testkit or handling its Host
errors. The Host Rust source is unchanged from 0.11; no caller-method migration,
retained artifact reset or Cargo build deadline is introduced.

### Fixed: tool setup admission

Shared Tooling 0.3.1 checks complete-set platform/catalog support and the selected
Rust/Cargo toolchain before aggregate setup downloads anything, with automatic
Rustup installation disabled. Prepare the declared toolchain explicitly, then
run `make install-tools`. Missing, changed or invalid host-tool checks identify
the selected tool, expected version, path and repair command. Offline admission
and retained installation/evidence policy are unchanged
([shared #101](https://github.com/dragginzgame/shared-tooling/issues/101)).

## [0.31.0] - 2026-10-10

### Breaking: complete tool setup and offline admission

Adopt Shared Tooling 0.3.0. Prepare the declared Rust toolchain, then use
`make install-tools` to prepare host, IC and Cargo tools in order, followed by
the owner CLI and PocketIC. `make tools-check` checks those same selections
offline. Product targets are registered through the shared ordered lists.
Replace `make install-format-tools` with `make install-tools`; the redundant
target is removed. `make server-check` now requires the prepared owner CLI
instead of compiling it. Missing prerequisites stop checks before builds or
installation. Release preflight and native CI use the same aggregate, preserving
source/candidate admission and explicit setup boundaries
([#50](https://github.com/dragginzgame/ic-testkit/issues/50),
[shared #98](https://github.com/dragginzgame/shared-tooling/issues/98)).

Direct host-installer callers must stop passing `--with-ripgrep`/`--with-cloc`;
the complete host set is mandatory. Existing incomplete toolsets need explicit
setup; older selections, receipts, failed attempts and server bytes are retained.
No Rust library API, persisted format or installation reset changes are required.

## [0.30.0] - 2026-10-10

### Breaking: bounded diagnostics and complete probe output

Cargo build errors retain the first 1 MiB of raw output per stream, followed by
an explicit truncation marker when needed. Lossy UTF-8 rendering may expand that
raw prefix. Enable Cargo output progress events to receive every raw byte, on
success or failure. Excess diagnostic output never terminates a build, and no
deadline is introduced. Silent builds retain the same bounded failure prefixes.

Tool identity and workspace discovery require complete streams of at most
64 KiB each. Cargo metadata allows 16 MiB stdout and 64 KiB stderr. Oversized
probes fail through typed Host evidence before partial output can become an
identity or JSON result. Workspaces producing larger metadata must reduce their
graph/output to the supported bound. Nonzero statuses and original spawn I/O
causes remain distinguishable. These policies belong to Testkit, not Host
defaults ([#49](https://github.com/dragginzgame/ic-testkit/issues/49)).

Successful bounded tool identities, cache framing and retained installations
are unchanged; no cache reset or compatibility reader is needed.

### Fixed

- Report unavailable failed-step logs during CI inspection, retaining partial
  logs and fetch errors instead of silently treating missing evidence as success,
  through Shared Tooling 0.2.14
  ([shared #97](https://github.com/dragginzgame/shared-tooling/issues/97)).

## [0.29.0] - 2026-10-10

### Breaking: IC Host 0.11 process contracts

The public Host re-exports select 0.11. Replace `CommunicationLimits` with
`OutputLimits`, and replace `stdout_bytes: n` / `stderr_bytes: n` with
`stdout: OutputLimit::Terminate(n)` / `stderr: OutputLimit::Terminate(n)`.
Wrap finite deadlines in `Some(duration)`; `None` permits long-running work.
For forwarding every byte while retaining a bounded prefix, use
`OutputLimit::Truncate(n)`. Read secondary execution failures through
`ExecutionError::cleanup` rather than its former individual cleanup fields
([#47](https://github.com/dragginzgame/ic-testkit/issues/47),
[host #46](https://github.com/dragginzgame/ic-host-tooling/issues/46)).

Testkit Cargo builds retain their existing output policy with no new quota or
deadline. A failed Cargo invocation with any cleanup failure preserves the whole
typed Host error. No stored format or retained installation needs a reset.

### Fixed

- Adopt Shared Tooling 0.2.13's snapshot path safeguards and selected-tool
  diagnostics. Release preflight prepares required tools after locked cache
  preparation; offline admission precedes standalone validation
  ([#48](https://github.com/dragginzgame/ic-testkit/issues/48),
  [shared #95](https://github.com/dragginzgame/shared-tooling/issues/95),
  [shared #96](https://github.com/dragginzgame/shared-tooling/issues/96)).
- Inherit IC Host's pathname admission fix to refuse directory-suffixed
  publication targets without replacing the stripped filename
  ([host #44](https://github.com/dragginzgame/ic-host-tooling/issues/44)).
- Reject unsafe Make execution modes even when flag variables are cleared or
  replaced, adopting Shared Tooling 0.2.11
  ([#45](https://github.com/dragginzgame/ic-testkit/issues/45),
  [shared #30](https://github.com/dragginzgame/shared-tooling/issues/30)).
- Coordinate mutable provisioning test fixtures to prevent executable-busy
  races between parallel tests, preserving concurrent setup coverage
  ([#46](https://github.com/dragginzgame/ic-testkit/issues/46)).

## [0.28.1] - 2026-10-10

### Changed

- Adopt Shared Tooling 0.2.10 for recorded snapshot versions and concise
  formatting output, retaining failed formatter diagnostics in CI artifacts
  ([shared #92](https://github.com/dragginzgame/shared-tooling/issues/92)).

### Fixed

- Use IC Host 0.10.1's parent-directory sync fix so durable publication completes
  directory syncing even when a competing writer creates the parent first
  ([host #43](https://github.com/dragginzgame/ic-host-tooling/issues/43)).
- Bind authenticated CI artifact readback to the current repository and run,
  preserving exact artifact IDs and payload verification
  ([shared #93](https://github.com/dragginzgame/shared-tooling/issues/93)).

## [0.28.0] - 2026-10-09

### Breaking: IC Host 0.10 durable publication

The public Host re-exports now select 0.10. Replace two-argument
`ic_host_fs::durable::write_with(path, producer)` with
`write_with(path, WriteOptions { mode: PublicationMode::Replace,
permissions: 0o666 }, producer)`. Existing three-argument `write_typed_with`
calls use the same options and producer with the name `write_with`.

Byte writers now return `NamedWriteError<io::Error>` and regular lock-file
creation can return `RegularFileLockError::Publication`. Match publication
phase before deciding whether retry is safe: `Producer` and
`BeforePublication` retain separate cleanup errors; `AfterPublication` means
the new output is already visible and requires reconciliation. Preserve the
whole error instead of extracting only its underlying I/O cause.

Testkit's existing I/O error surfaces wrap the complete publication error
with `io::Error::other`; downcast the contained error to
`ic_host_fs::durable::NamedWriteError<io::Error>` to inspect its phase and
cleanup evidence. Atomic-copy errors retain an additional path-context layer
in their source chain. Write failure kinds are now `Other` at these publication
boundaries; source-file open failures retain their native kind. The cache,
stamp and provisioning formats, permissions and replacement/create-only policy
are unchanged. No retained-data migration, reinstall or reset is required.

No Testkit function, method or type was removed. The upstream writer removal
is a Host API hard cut; Testkit adds no compatibility shim.

### Fixed

- Adopt Shared Tooling 0.2.8 so Make admission uses the selected snapshot even
  with an inherited tooling root and accepts recursive Make commands containing
  extra arguments while still rejecting modes that skip or hide failures
  ([shared #30](https://github.com/dragginzgame/shared-tooling/issues/30)).

## [0.27.2] - 2026-10-09

### Fixed

- Adopt Shared Tooling 0.2.7 so hook setup preserves existing path selections
  with trailing newlines instead of treating them as the default hook path
  ([shared #89](https://github.com/dragginzgame/shared-tooling/issues/89)).
- Refuse Make modes that skip commands or hide failures, and keep isolated
  release checks bound to their fixture even with an inherited tooling root
  ([shared #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [shared #7](https://github.com/dragginzgame/shared-tooling/issues/7)).

### Changed

- Use shared release and Rust formatting Make definitions, removing duplicate
  recipes while retaining Testkit's validation, tool pins and release policy
  ([shared #91](https://github.com/dragginzgame/shared-tooling/issues/91),
  [shared #92](https://github.com/dragginzgame/shared-tooling/issues/92)).

## [0.27.1] - 2026-10-09

### Fixed

- Build the local server CLI before Make checks and tests use it, so a cleaned
  build directory no longer causes a missing-executable failure. Server
  provisioning remains explicit
  ([#38](https://github.com/dragginzgame/ic-testkit/issues/38)).

### Changed

- Run native CI once per release source through main, instead of repeating the
  heavy matrices for its tag. Preserve all three hosts, gate families and PR
  checks ([#42](https://github.com/dragginzgame/ic-testkit/issues/42)).

## [0.27.0] - 2026-10-09

### Breaking

- Replace startup-error variant matching with `error.failure()` and
  `PocketIcStartupFailure`. Read bounded server output through `error.output()`
  and inspect separate typed command/server cleanup reports. Preserve the
  original failure and a failed command's exit status when server cleanup also
  fails ([#30](https://github.com/dragginzgame/ic-testkit/issues/30)).

### Fixed

- Adopt Shared Tooling 0.2.2 so IC setup and offline verification include the
  final selected tool even when the pin matrix has no final newline
  ([shared #87](https://github.com/dragginzgame/shared-tooling/issues/87)).

### Changed

- Qualify Host 0.9.1, which releases captured pipes at EOF and simplifies durable
  publication while preserving the existing APIs and behavior.

## [0.26.0] - 2026-10-09

### Breaking

- Adopt Shared Tooling 0.2.0 and its five-tool IC bundle. Prepare PocketIC
  separately with `make install-server`; use `make server-check` for offline
  admission. Test and CI callers obtain the server from Testkit instead of the
  shared bundle. Rerun explicit IC setup; prior bundles and evidence remain
  intact ([#38](https://github.com/dragginzgame/ic-testkit/issues/38)).

### Fixed

- Clean up observed Cargo descendants when the compiler leader exits, without
  waiting first for inherited output pipes to close. Use Host communication for
  both execution modes while preserving live output, heartbeats, diagnostics and
  builds without a deadline ([#36](https://github.com/dragginzgame/ic-testkit/issues/36)).

### Changed

- Prepare formatters through Shared Tooling's checkout-local Rust installer and
  discover prepared tools in isolated formatting hooks without a shell PATH
  export ([#41](https://github.com/dragginzgame/ic-testkit/issues/41)).
- Check prepared PocketIC bundles without retaining the decoded executable in
  memory, preserving archive authentication, gzip integrity and size limits
  ([#38](https://github.com/dragginzgame/ic-testkit/issues/38)).

## [0.25.5] - 2026-10-09

### Changed

- Adopt Shared Tooling 0.1.35's Cargo binary/example installer, with offline
  receipt and byte checks and retained failed installations
  ([shared #65](https://github.com/dragginzgame/shared-tooling/issues/65)).

### Added

- Select managed PocketIC operation-idle lifetime through `run --idle-ttl`
  or `PocketIcStartupConfig::with_server_idle_ttl`, allowing long pre-client
  work independently of the hard lifetime while retaining existing defaults
  ([#37](https://github.com/dragginzgame/ic-testkit/issues/37)).

## [0.25.4] - 2026-10-09

### Changed

- Adopt Shared Tooling 0.1.34 and retire the unused local fleet reporter;
  run fleet reports centrally in Shared Tooling. Local workspace LOC and tool
  checks remain available ([#39](https://github.com/dragginzgame/ic-testkit/issues/39),
  [shared #83](https://github.com/dragginzgame/shared-tooling/issues/83)).
- Verify prepared PocketIC executables through Host 0.8.7's streaming file
  admission, avoiding a full executable-sized verification buffer while retaining
  checksum and redirected-file refusal
  ([#38](https://github.com/dragginzgame/ic-testkit/issues/38)).
- Specify approximate floating-point benchmark averages and percentage changes,
  retaining exact aggregate totals and existing report formats
  ([#40](https://github.com/dragginzgame/ic-testkit/issues/40)).

### Added

- Add explicit authenticated PocketIC `setup` and offline `check` commands to
  the published server CLI. Managed `run` can use Testkit's prepared server
  without consumer version or asset selection
  ([#38](https://github.com/dragginzgame/ic-testkit/issues/38)).

### Fixed

- Admit compatible stable 16.x server overrides independently of the Rust
  client's latest downloadable server; provisioning retains an exact reviewed
  selection ([#34](https://github.com/dragginzgame/ic-testkit/issues/34)).

- Allow native host and PocketIC concurrency CI jobs to complete long builds
  within GitHub Actions' default job limit, removing the 15/10-minute caps.
  Individual runtime and server-startup deadlines are unchanged.
- Preserve CI qualification for each pushed commit while allowing newer PR
  revisions to replace older review runs
  ([shared #80](https://github.com/dragginzgame/shared-tooling/issues/80)).
- Reuse verified IC tool bundles after comment-only or reordered pin edits,
  adopting Shared Tooling's 0.1.32 changes
  ([shared #79](https://github.com/dragginzgame/shared-tooling/issues/79)).
- Use Host's regular-file admission for cache locks while retaining shared
  record ownership and heartbeat timing; reject redirected lock entries without
  touching their targets ([#35](https://github.com/dragginzgame/ic-testkit/issues/35)).

## [0.25.3] - 2026-10-08

### Changed

- Adopt Shared Tooling 0.1.29 and restore its ownership of the IC tool catalog,
  retaining the selected PocketIC 16.1 versions and hashes
  ([shared #76](https://github.com/dragginzgame/shared-tooling/issues/76)).
- Use shared release-source admission to identify staged, unstaged and untracked
  paths that block release, while preserving consumer metadata checks
  ([shared #74](https://github.com/dragginzgame/shared-tooling/issues/74)).

## [0.25.2] - 2026-10-08

### Changed

- Use IC Host 0.8.1's nonblocking regular-file locks for cache pruning and replacement,
  preserving busy-entry skipping and refusing redirected or special lock files
  ([Host #24](https://github.com/dragginzgame/ic-host-tooling/issues/24)).
- Adopt Shared Tooling 0.1.28, rejecting malformed active tool-selection links
  before execution or downloads and adding its opt-in maintenance task catalog
  ([shared #75](https://github.com/dragginzgame/shared-tooling/issues/75)).

### Testing

- Qualify release-runner behavior with the shared simulation-only fixture in
  native CI ([shared #70](https://github.com/dragginzgame/shared-tooling/issues/70)).
- Record the actual minimum Rust compiler and check both native and Wasm paths,
  including the Wasm performance probe, at the unchanged Rust 1.88 floor.

## [0.25.1] - 2026-10-08

### Testing

- Exercise real managed-server CLI execution with the pinned PocketIC server
  on Linux and both supported macOS hosts in native CI.

## [0.25.0] - 2026-10-08

### Breaking

- Adopt the selected IC Host 0.8 re-exports. Update struct literals and exhaustive
  destructuring for `ic_host_process::{child::CleanupError, tool::ExecutionError}`
  to include the new `term_error` field. Existing default cleanup remains
  immediate KILL with synchronous reaping.

### Fixed

- Make consumer release, publication and qualification adapters resolve their
  helper paths safely under inherited `CDPATH`. Exercise release, installation,
  publication and hook qualification with that environment in native CI
  ([#31](https://github.com/dragginzgame/ic-testkit/issues/31)).

### Changed

- Select PocketIC 16.1 for the Rust client and managed server, with verified
  native release assets. Run `make install-ic-tools` to prepare the new server.
- Bind CI archive qualification downloads to the returned upload ID, reject
  missing identities and occupied destinations, and require digest verification
  ([#33](https://github.com/dragginzgame/ic-testkit/issues/33)).
- Run the maintained artifact-helper integration checks in native CI instead
  of an empty selection left after the Wasm reader API removal.
- Adopt Shared Tooling 0.1.27, including canonical path fixes and corrected
  snapshot discovery in tooling counts
  ([shared #67](https://github.com/dragginzgame/shared-tooling/issues/67),
  [shared #69](https://github.com/dragginzgame/shared-tooling/issues/69)).
- Use shared tool-evidence selection in CI failure archives. Freshly verified
  active bundles contribute pins, check logs and IC receipts; changed or
  unselected bundles remain in full
  ([shared #66](https://github.com/dragginzgame/shared-tooling/issues/66)).

## [0.24.0] - 2026-10-08

### Breaking

- Adopt IC Host 0.7 through the public host-crate re-exports. Update exhaustive
  matches for `ExecutionFailure::Cancelled` and
  `ExecutionOperation::{StdinPipe, WriteInput}`; remove references to
  `ExecutionOperation::{StdoutPipe, StderrPipe}`.
- Remove `artifacts::read_wasm`. Compose `artifacts::wasm_path` with
  `ic_host_fs::read::read_file` and retain the caller-selected byte limit.

### Fixed

- Make CI evidence qualification work with inherited `CDPATH` and physical
  checkout, workspace and temp paths ending in newlines. Exercise these paths
  in the native archive round-trip checks
  ([#31](https://github.com/dragginzgame/ic-testkit/issues/31)).
- Adopt Shared Tooling 0.1.26, including the release-tracking transaction that
  preserves concurrent symbolic refs and guarded refresh of unchanged,
  uncommitted snapshots
  ([shared #62](https://github.com/dragginzgame/shared-tooling/issues/62),
  [shared #64](https://github.com/dragginzgame/shared-tooling/issues/64)).

### Migration

Replace `read_wasm(target_dir, crate_name, profile_target_dir, max_bytes)` with:

```rust
use ic_testkit::{artifacts::wasm_path, ic_host_fs::read::read_file};

let path = wasm_path(target_dir, crate_name, profile_target_dir);
let bytes = read_file(&path, max_bytes)?;
```

This preserves bounded allocation, typed `ArtifactError` failures and the
caller-controlled path policy, including allowed symlinks. Reading does not
validate Wasm. Cargo layout helpers remain in `artifacts`; file reads belong to
Host. There is no compatibility alias or retained-cache format change.

Host 0.7 adds `tool::communicate_child`, cancellation and stdin IO error
categories. Existing Testkit command execution still preserves the caller's IO,
returns nonzero command statuses and imposes no command deadline. Cargo progress
events remain streamed; the new Host helper does not supply output callbacks.

## [0.23.0] - 2026-10-08

### Breaking

- Adopt IC Host 0.6 through the public host-crate re-exports. Update
  `ExecutionError` literals/destructuring for its new `group_error` field.
- Invalid-port and builder startup errors now retain bounded output and secondary
  cleanup diagnostics. Update affected variant literals and patterns, including
  the now-structured `BuilderDisconnected`
  ([#30](https://github.com/dragginzgame/ic-testkit/issues/30)).

### Fixed

- PocketIC version probes clean up owned wrapper descendants on exit and timeout
  using Host's group capture; probes must not launch background work intended
  to survive the check
  ([host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).
- Preserve startup causes while displaying secondary cleanup failures in CLI
  errors; owned builder failures also retain server output before cleanup
  ([#30](https://github.com/dragginzgame/ic-testkit/issues/30)).

### Migration

`PocketIcStartupError::InvalidServerPort` gains `termination_error`.
`BuilderThreadSpawn`, `BuilderPanicked` and `BuilderDisconnected` carry
`stdout`, `stderr` and `termination_error`; use `BuilderDisconnected { .. }`
when matching only the failure kind. Update literals with the new fields and
use `..` in patterns where diagnostics are not consumed. Bounded output remains
limited to the first 16 KiB per stream. Borrowed-server builder failures carry
empty streams and no owned-server termination error.

The original kind/message/source remains primary; cleanup failure is appended
by `Display`. Managed invalid-port and builder-failure projections preserve the
actual cleanup evidence. `ExecutionError::group_error` separately reports Host
process-group signalling failures; direct-child fixtures use `None`. See
[Host 0.6 notes](https://github.com/dragginzgame/ic-host-tooling/blob/6f066e727c977e0b7ec8d3d77821df8508b95c64/docs/changelog/0.6.md).
There are no cache/schema changes or retained-data resets.

## [0.22.3] - 2026-10-08

### Added

- Retain complete PocketIC stdout/stderr independently of server teardown with
  `PocketIcStartupConfig::with_server_output_files` or the runner's paired
  `--server-stdout` / `--server-stderr` flags. Callers own the new file paths,
  retention and disk budget; status propagation and bounded excerpts are unchanged
  ([#29](https://github.com/dragginzgame/ic-testkit/issues/29)).

### Changed

- Adopt Shared Tooling 0.1.25 and its corrected evidence archiver, replacing
  local archive mechanics while preserving Testkit's CI selection and retention
  ([#28](https://github.com/dragginzgame/ic-testkit/issues/28),
  [shared #59](https://github.com/dragginzgame/shared-tooling/issues/59)).

## [0.22.2] - 2026-10-08

### Changed

- Adopt Shared Tooling 0.1.24: completed direct releases refresh the matching
  local upstream observation without repeating delivery; tooling reports include
  `bin/` and repositories awaiting their first commit
  ([shared #62](https://github.com/dragginzgame/shared-tooling/issues/62),
  [shared #61](https://github.com/dragginzgame/shared-tooling/issues/61)).
  Keep the existing failure collector pending the corrected shared archive helper.

## [0.22.1] - 2026-10-08

### Fixed

- Archive failed CI evidence before upload so legal Unix filenames, file modes
  and symlinks survive retention. Preserve logs, step outcomes and failed
  qualification artifacts; add CI upload/download qualification on every
  supported native host
  ([#28](https://github.com/dragginzgame/ic-testkit/issues/28),
  [shared #59](https://github.com/dragginzgame/shared-tooling/issues/59)).

## [0.22.0] - 2026-10-08

### Breaking and migration

- The public `ic_host_artifacts`, `ic_host_fs`, `ic_host_process` and
  `ic_host_tools` re-exports now select Host 0.5. Update gzip encoder calls to
  pass a numeric compression level from 0 through 9, update Wasm fact literals
  for the expanded facts, and review exhaustive inspection-error matches.
  See the [Host 0.5 migration notes](https://github.com/dragginzgame/ic-host-tooling/blob/db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7/docs/changelog/0.5.md).
  This is a pre-1.0 hard cut; no deprecated aliases or compatibility bridges
  are provided. Repository-owned retained cache layouts are unchanged.

- Observed Cargo builds now run in an owned process group. Observer unwinding
  stops the compiler group as well as Cargo; callers must not rely on those
  builds sharing the parent terminal process group. Unobserved builds retain
  their existing command-output behavior.

### Fixed

- Stop fingerprinting when an excluded cache root cannot be resolved, rather
  than silently dropping the exclusion. Not-yet-created roots remain accepted;
  valid digest framing and retained cache layouts are unchanged
  ([#26](https://github.com/dragginzgame/ic-testkit/issues/26)).

### Changed

- Managed PocketIC servers and commands now use `ic_host_process::child::OwnedChild`
  for process-group ownership, polling and termination instead of local cleanup
  implementations ([#25](https://github.com/dragginzgame/ic-testkit/issues/25)).
  Startup readiness, cancellation, TTL and output bounds remain consumer-owned.
  Cleanup signals the owned group and reaps its leader; it has no wall-clock
  bound and cannot contain descendants that escape that group.
- Adopt Shared Tooling 0.1.23 release helpers. Final checks now revalidate the
  committed payload and exact annotated tag object before delivery, and completed
  releases verify their remote tag and branch history. PR lookup aggregates
  paginated responses with jq without requiring GitHub CLI `--slurp`. This
  repository continues to use direct atomic branch/tag delivery.

## [0.21.3] - 2026-10-07

### Changed

- Refresh the canonical release engine and engineering baseline to Shared Tooling
  0.1.21, including its PR-delivery helper. This repository retains direct atomic
  branch/tag delivery; adopting the shared PR flow requires separate consumer
  adapters and qualification
  ([shared #42](https://github.com/dragginzgame/shared-tooling/issues/42)).

### Testing

- Remove duplicated generic path-resolution cases now owned by IC Host and
  delegate release-tag hash admission entirely to Shared Tooling. Retain local
  cache-confinement, relative-target and publication-refusal qualification.

## [0.21.2] - 2026-10-07

### Changed

- Select IC Host Tooling 0.4.6, including the macOS durable-writer compilation
  repair and bounded gzip hash/compare helpers through existing re-exports
  ([host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18),
  [host #12](https://github.com/dragginzgame/ic-host-tooling/issues/12)).
  Native filename qualification now follows actual filesystem admission
  ([host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19)).
  Refresh the canonical Shared Tooling snapshot to 0.1.20.
- Expand the compiled shim-Cargo/post-link recipe using the existing Wasm and
  transactional caches. Declare compiler inputs, optimizer bytes, arguments,
  environment and working directory; retain input ownership through publication
  and bound transformed output before reading it
  ([#21](https://github.com/dragginzgame/ic-testkit/issues/21)).

### Testing

- Select Shared Tooling's `test-rust-tools.sh` in the reviewed 0.1.20 snapshot
  and portable native CI. Substitute Cargo proves normal setup/check/retry,
  retained failed installation evidence, and refusal of redirected directories,
  executable paths and Cargo receipts before probes or writes. These checks
  do not install the optional Rust tool set or change ordinary setup policy
  ([#24](https://github.com/dragginzgame/ic-testkit/issues/24),
  [shared #54](https://github.com/dragginzgame/shared-tooling/issues/54)).

## [0.21.1] - 2026-10-07

### Fixed

- Compare selected server paths after canonicalization, accounting for macOS
  temporary-directory aliases without weakening the production path contract.
  The standalone benchmark's Cargo substitute handles offline workspace
  discovery separately from locked metadata/build calls. Concurrency CI prepares
  pinned tools and exports `POCKET_IC_BIN` before its live tests
  ([#19](https://github.com/dragginzgame/ic-testkit/issues/19),
  [#18](https://github.com/dragginzgame/ic-testkit/issues/18),
  [#23](https://github.com/dragginzgame/ic-testkit/issues/23)).
- Refresh the reviewed snapshot to Shared Tooling 0.1.19, including the required
  common IC pin parser. Rust installation rejects redirected directories,
  executables and receipts before probing or installation. Failed validation
  batches retain `latest-combined.log` alongside individual logs; release-note
  preparation preserves historical bytes even without a terminal newline
  ([shared #54](https://github.com/dragginzgame/shared-tooling/issues/54),
  [shared #37](https://github.com/dragginzgame/shared-tooling/issues/37),
  [shared #55](https://github.com/dragginzgame/shared-tooling/issues/55)).

### Changed

- Select compatible IC Host Tooling 0.4.2 in the lockfile. Existing public
  re-exports expose named durable output, bounded chunk digests and exact-version
  admission for trusted installed tools. Testkit retains its existing stream
  publication and long-lived server owners; these additions do not replace
  multi-file directory transactions or add a compiler post-link mode. See the
  [host release notes](https://github.com/dragginzgame/ic-host-tooling/blob/6501d0e9fa7ba0439ec7a4010ca7bf0205e1d712/docs/changelog/0.4.md).

## [0.21.0] - 2026-10-07

### Breaking: IC Host Tooling 0.4 re-exports

All four publicly re-exported host crates now select the published 0.4 line.
Move retired `ic_host_fs::durable` read calls to the bounded `read` module:
`read_file_no_follow` and `read_optional_file_no_follow` return `ArtifactError`.
`read::read_private_bytes` returns `Result<Option<_>, PrivateFileReadError>`;
only missing files return `None`, and failures must not trigger key replacement.
Replace `durable::lock_file` with `lock_regular_file_with_parents` and handle
`RegularFileLockError` for progress locks. See the
[host migration details](https://github.com/dragginzgame/ic-host-tooling/blob/6b171744def811882ba6c71d50135efa898302a9/docs/changelog/0.4.md)
([host #14](https://github.com/dragginzgame/ic-host-tooling/issues/14),
[#13](https://github.com/dragginzgame/ic-testkit/issues/13)). Testkit's cache
budgets, stale-sidecar handling and retained layouts remain unchanged. Typed
host-to-I/O conversion and bounded prefix capture now reuse the host owners.

### Breaking: Cargo-owned workspace discovery and shared build state

`workspace_root_for` accepts a manifest directory and returns `io::Result<PathBuf>`.
Propagate or handle the result. It invokes Cargo's offline `locate-project
--workspace` against the actual manifest; App paths, explicit workspace selection,
excluded packages and independent workspaces follow Cargo's contract. Missing or
unlisted manifests fail instead of falling back to the supplied directory. Cargo
must already be prepared; discovery does not resolve dependencies or install tools.

`WasmBuildSpec::new` now defaults to the workspace-relative
`target/ic-testkit-incremental` Cargo target. Misses across fingerprints share
compiler state under the existing process lock; exact final Wasm entries keep
their existing identities and retention guards. Override the shared target when
needed, or call `with_isolated_builds()` for intentionally independent compiler
state. Shared-target maintenance configured on an isolated spec is rejected.

Configure byte/age limits through the existing exact-entry pruning and shared-target
maintenance builders. No universal limits are imposed. Existing isolated targets
and retained exact entries are preserved: prune obsolete targets explicitly under
their existing maintenance contract. There is no format reinterpretation, new
reader or automatic retained-installation reset
([#18](https://github.com/dragginzgame/ic-testkit/issues/18)).

### Added

- `PocketIcStartupConfig::from_env(timeout)` selects `IC_TESTKIT_POCKET_IC_URL`
  before `POCKET_IC_BIN`. Invalid selected values fail without fallback; missing
  configuration returns `NotConfigured`. The explicit executable's bounded
  `--version` probe requires the selected PocketIC server identity and successful
  exit. It never installs, downloads or performs cache discovery.
- `ic-testkit-server run [--ttl SECONDS] [--startup-timeout SECONDS] -- COMMAND`
  exports the shared URL to a command and retains managed server ownership until
  completion. Command status is preserved; SIGINT/SIGTERM/SIGHUP return `128 +
  signal` after cleanup. `--ttl` is optional and only applies to owned servers.
  Startup defaults to 30 seconds per version/startup phase; interruption is
  observed after bounded startup. External URL mode never terminates the borrowed
  server. `PocketIcStartupConfig::run_command` uses the existing owned-group
  lifecycle engine and accepts caller-owned cancellation without installing
  signal handlers in library code
  ([#19](https://github.com/dragginzgame/ic-testkit/issues/19)).
- `pic::tick_until` checks a caller-owned predicate before mutation, then advances
  time and ticks for at most the supplied number of pending rounds.
  `TickUntilError<E>` distinguishes exhausted progression from the original
  predicate failure. Zero rounds still allow an initial check, and completion
  on the last round succeeds. The round budget does not impose a wall-clock
  timeout ([#20](https://github.com/dragginzgame/ic-testkit/issues/20)).

### Fixed

- `PocketIcStartupConfig::run_command` monitors its owned server while the
  command is pending. Server exit, including hard TTL expiry, stops the owned
  command group and returns `ServerExited` with the server's exit status and
  bounded output. External URL mode does not monitor or terminate the borrowed
  server ([#19](https://github.com/dragginzgame/ic-testkit/issues/19)).
- Resolve dangling input symlink chains to their missing targets through the
  selected IC Host Tooling 0.4.0 libraries, keeping equivalent cache-input paths
  on one identity ([host #1](https://github.com/dragginzgame/ic-host-tooling/issues/1),
  [#13](https://github.com/dragginzgame/ic-testkit/issues/13)).
- Upload retained validation logs and host/IC installer candidates after failed
  native CI jobs. Portable tooling checks use an explicit fixture directory that
  is included in the failure artifact. Setup diagnostics and complete CI output
  are retained without changing failure propagation; artifacts expire after
  14 days ([#17](https://github.com/dragginzgame/ic-testkit/issues/17)).

### Changed

- Live integration tests and the multi-canister baseline example require explicit
  environment startup instead of permitting upstream binary downloads. Prepare
  `POCKET_IC_BIN` or configure `IC_TESTKIT_POCKET_IC_URL`; dedicated-server probes
  require the binary. Overlap checks identify instances by server URL and ID.
  Benchmark documentation selects `FetchedLog` and requires complete samples;
  managed output remains a bounded diagnostic prefix
  ([#23](https://github.com/dragginzgame/ic-testkit/issues/23)).
- Clarify `with_rustc_program` as an identity probe for a shim's actual compiler.
  Explicit `RUSTC` through `with_extra_env` selects both the Cargo environment
  and fingerprint probe and takes precedence. Regression coverage checks reuse,
  invalidation after compiler identity changes, and that precedence. Post-link
  work continues to use the existing cached external artifact transaction
  ([#21](https://github.com/dragginzgame/ic-testkit/issues/21)).
- Use the reviewed Shared Tooling 0.1.18 `make/tools.mk` for setup, offline tool
  verification, workspace Rust LOC and read-only sibling tooling reports.
  `make install-tools` and CI prepare pinned ripgrep with PCRE2 and cloc as well
  as jq/yq and IC executables. Existing checkouts must rerun explicit setup;
  `make tools-check` remains offline and never installs missing tools.
- Cargo target aliases exclude their physical output from LOC totals. Validation
  target options and assignments fail before gate dispatch. Canonical fixture
  qualification covers enclosing workspaces and inherited target settings
  ([shared #31](https://github.com/dragginzgame/shared-tooling/issues/31),
  [shared #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [shared #47](https://github.com/dragginzgame/shared-tooling/issues/47),
  [shared #53](https://github.com/dragginzgame/shared-tooling/issues/53)).
- Optional `make install-rust-tools` prepares the shared pinned cargo-sort,
  cargo-sort-derives and candid-extractor set under `.tools/rust`; its offline
  check is `make rust-tools-check`. The common PATH selects that directory when
  prepared. Ordinary validation and common setup retain their existing selections
  ([shared #51](https://github.com/dragginzgame/shared-tooling/issues/51)).
- The shared release finalizer accepts trailing spaces/tabs on draft headings,
  keeping their notes under the selected finalized release. The shared validation
  runner preserves the first failed Make invocation's exit status rather than
  normalizing it to 1. Its default evidence selection remains unchanged here
  ([shared #38](https://github.com/dragginzgame/shared-tooling/issues/38),
  [shared #37](https://github.com/dragginzgame/shared-tooling/issues/37)).

## [0.20.0] - 2026-10-07

### Breaking: explicit shared host owners

The complete published IC Host Tooling 0.3.1 crates are re-exported under their
own names. Replace imports from the former all-purpose `ic_host_tools` modules:

| Former path under `ic_testkit` | New owner under `ic_testkit` |
| --- | --- |
| `ic_host_tools::artifact` stream, identity and digest APIs | `ic_host_artifacts::artifact` |
| `ic_host_tools::artifact` pathname read/hash APIs | `ic_host_fs::read` |
| `ic_host_tools::archive`, `ic_host_tools::wasm` | `ic_host_artifacts::archive`, `ic_host_artifacts::wasm` |
| `ic_host_tools::tool`, `ic_host_tools::provenance` | `ic_host_process::tool`, `ic_host_process::provenance` |
| `ic_host_tools::candid`, `ic_host_tools::response` | Unchanged |

Archive and Wasm features are enabled on the artifact owner. `read_wasm` returns
`Result<Vec<u8>, ic_host_artifacts::artifact::ArtifactError>`; its caller-selected
size limit and fallible bounded reads are preserved. Cache sidecar overflow still
means a cache miss. No compatibility aliases or older host dependency remain.
Update downstream imports and explicit error type annotations. Retained cache
bytes, digest domains and lock namespaces are unchanged; no reset is required.

### Breaking: durable publication errors

The split and generic filesystem adoption are part of
[#13](https://github.com/dragginzgame/ic-testkit/issues/13). Byte writes and streamed
copies use the shared durable commit engine, including final-parent directory
synchronization. The local publication engine and its temporary-name sequence are
removed. Short staging names support maximum-length destinations and retry
collisions within the shared bound without touching unowned files. A failure
before rename preserves the destination; a final-parent synchronization failure
can return an error after the complete new bytes become visible. Inspect and
reconcile the destination before retrying after such a failure.

Shared path resolution handles missing suffixes against an explicit base;
Testkit retains selection of its current directory and cache identity policy.
Shared descriptor acquisition handles contention and interrupted locks; Testkit
retains opening, the 25ms polling cap, phase heartbeat mapping and retention
lifetime. Digest framing, cache limits, ownership checks and lock namespaces
remain local.

Publication callers invoke the shared byte writer directly; the redundant
local adapter and shared-engine-only failure test are removed. The consumer
retains atomic-copy context and long-name copy coverage, while the shared owner
qualifies producer/rename cleanup and collision ownership.

### Shared validation dispatch

CI and release checks delegate their ordered local target selections to the
canonical shared validation runner with `--fail-fast`. Failed target output is
retained under `target/validation-failures`, and later targets do not execute.
The shared Make-execution admission check rejects ignore-errors, dry-run, query
and touch modes before validation, release preparation or hook formatting;
ordinary release variables and jobserver settings remain inherited. The local
dispatch loops and repeated generic version-reader fixtures are retired
([Shared Tooling #30](https://github.com/dragginzgame/shared-tooling/issues/30),
[#15](https://github.com/dragginzgame/ic-testkit/issues/15)).

### Shared metadata and admission checks

The reviewed snapshot selects Shared Tooling `25e7ce8` (0.1.14). Version,
installation, release and publication callers use its TOML-aware workspace
version reader, accepting valid inline comments and rejecting malformed or
failed observations. Stable release admission remains explicit. The former
local reader is removed; fixtures copy the shared reader to its canonical path
and retain workspace targets needed for Cargo's offline manifest validation
([#11](https://github.com/dragginzgame/ic-testkit/issues/11)).

The normal dependency declaration gate also checks Cargo workspace inheritance.
Tag admission selects the current commit locally and delegates annotated-tag
and exact-commit validation to the fixed shared checker. Release preparation,
metadata mutation, interruption recovery and publication policy remain local.
Existing host setup keeps its jq/yq selection; shared ripgrep support is optional.

Release dispatch observes one remote URL, verifies it after validation and again
before push, and pushes the exact saved branch/tag refs through that captured
URL. Snapshot verification hashes inspected files directly instead of executing
the inspected checksum helper. The selected governance export includes its
maintained file list and the read-only exact-commit GitHub CI helper
([#15](https://github.com/dragginzgame/ic-testkit/issues/15)).

Formatting, hook installation and release admission use the shared offline
formatter prerequisite checker with the existing explicit cargo-sort pin. The
duplicate local admission comparisons are removed; tool installation remains
an explicit setup step. Failed formatter observations are rejected even when
they print an apparently matching version
([#12](https://github.com/dragginzgame/ic-testkit/issues/12)).

Refreshed host-tool recovery fixtures restore exact authenticated archive bytes
and retain failure diagnostics, avoiding host-dependent repacking differences
([Shared Tooling #17](https://github.com/dragginzgame/shared-tooling/issues/17)).

Failed dependency declaration fixtures retain their scratch inputs and checker
output for diagnosis; successful fixtures still remove their own temporary data
([Shared Tooling #21](https://github.com/dragginzgame/shared-tooling/issues/21)).

Changelog finalization passes the saved previous release identity to the shared
selector, preserving imported undated historical sections at or below that
version. Future competing candidates still reject preparation. Both maintained
changelog views are covered through the actual release callbacks
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

The shared finalizer compares both equality and ordering of version components
as exact strings, keeping adjacent components above 2^53 distinct. Pending notes
remain in the single dated candidate while undated historical sections retain
their original identity
([Shared Tooling #23](https://github.com/dragginzgame/shared-tooling/issues/23)).

### Hook qualification after committing the package move

Consumer hook fixtures project the current selected checkout and prepared hook
inputs without deleting the historical `canisters` directory. The removed
unconditional deletion failed after the package move entered HEAD, preventing
formatting qualification from starting. Package locations remain unchanged; the
fixture does not enforce a layout by removing a directory.

### Cache retention and portable test synchronization

Cache retention explicitly unlocks after the final record clone drops. A file
descriptor inherited briefly during concurrent process spawning no longer keeps
an otherwise unowned entry protected from pruning. Independent acquisitions
continue to protect the entry until their own records drop; process exit still
releases locks automatically. Regression coverage holds a duplicated descriptor
open while checking both shared record ownership and independent acquisitions.

The initially nonblocking request-reader fixture uses the same bounded accept
loop as the synthetic PocketIC peer, tolerating `WouldBlock` before the connection
is available on macOS. Its socket-mode barrier and I/O timeout checks remain.

## [0.19.2] - 2026-10-06

### Shared publication and setup checks

The reviewed snapshot now selects Shared Tooling `47cd2cc` (0.1.7).
Publication delegates exact-version registry observation to the canonical helper
from [Shared Tooling #10](https://github.com/dragginzgame/shared-tooling/issues/10),
removing the temporary inline HTTP implementation. Confirmed presence skips
publication, confirmed absence permits Cargo, and unavailable results stop the
command. Package selection, explicit offline policy and upload authority stay
in the consumer adapter; the existing publication failure fixtures exercise the
shared helper through the actual Make target.

IC installation now uses the shared checksum generator for receipts, rejects
filenames the receipt cannot represent and checks traversal before activation.
Existing tool selections, receipt layouts and installed tools remain unchanged.

### Release guard ownership

`make release-guards-check` delegates standard release entry-point verification
to the canonical checker from
[Shared Tooling #8](https://github.com/dragginzgame/shared-tooling/issues/8).
The checker covers patch/minor/major/resume arguments, runner success and failure,
and every pair of conflicting release selections using a substituted runner.
It clears inherited Make and validation identity controls and retains failed
fixtures and logs. No release effects occur during the check.

The local entry-point smoke block and vendored shared-runner fixture suite are
removed. Shared runner recovery tests stay with their upstream implementation.
Consumer version, installation, cleanliness, gate sequencing and metadata
recovery checks remain in this repository and run through the existing target.

### CI prerequisites

Portable Linux and macOS jobs explicitly install ripgrep before running shared
tool fixtures. Complete-gate jobs now fetch the selected locked dependencies
before consumer metadata fixtures need them offline. This fixes the 0.19.1
`rg: command not found` and missing cached `ic-host-tools` failures without
changing dependency selection or allowing validation to retry online
([#7](https://github.com/dragginzgame/ic-testkit/issues/7),
[#9](https://github.com/dragginzgame/ic-testkit/issues/9)).

### Release cache preparation

Normal release preflight runs a locked fetch under Cargo's configured network
policy. It can download missing selected dependencies before the gate and
offline metadata preparation; maintainers no longer need a separate cache-fetch
command for ordinary releases. Explicit offline environment or Cargo configuration
still rejects missing cache entries without an online fallback.

Recovery fetches against the verified saved workspace when live metadata is
partially published. Fetch failure preserves live metadata, receipts and retained
identity, and stops before validation or version mutation. Focused fixtures cover
missing-cache policy, failure and an explicit retry using the same retained
identity. This corrects the consumer's forced-offline preflight; Shared Tooling's
locked cache-preparation contract is unchanged
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

## [0.19.1] - 2026-10-06

### Publication admission

`make publish` queries the exact crates.io package/version endpoint before Cargo
publication. A successful HTTP 404 permits publication; HTTP 200 skips an already
published version. Transport failures, unexpected responses and explicit
`CARGO_NET_OFFLINE=true` or `1` stop admission without uploading. Cargo publication
errors retain their exit status.

The existing release contract requires an annotated `v<version>` tag at `HEAD`.
Admission now verifies the annotation and exact commit and rejects failed Git
commands even if they print apparently valid output. Focused fixtures exercise
registry and tag failures through the actual Make publication target using
substituted commands, without credentials or remote writes.

### Retained release evidence

Release preparation, recovery and later admission checks reject failed reads of
validation receipts and readiness records, even when the reader prints matching
bytes before failing. Validation identity and input digest are derived from one
successful receipt read. Rejection preserves live metadata, saved old/new files,
receipts and readiness without dispatching another validation gate; a successful
retry uses the same retained identity. The retained `v1` layout is unchanged.
This extends the release-admission work in
[#7](https://github.com/dragginzgame/ic-testkit/issues/7).

Source/prepared version reads and next-version derivation must also complete
successfully before their output is compared with the selected release. A helper
that prints the expected version and then fails stops admission without changing
metadata, invalidating the saved receipt or running another validation gate.

## [0.19.0] - 2026-10-06

### Shared host APIs and migration

The host-only dependency `ic-host-tools` 0.1.10 is selected from crates.io and
re-exported as `ic_testkit::ic_host_tools`; no sibling checkout enters builds.
Consumers use its canonical bounded reads, Wasm inspection, archive/response
decoding and verified executable APIs directly. Unix-only executable, Candid
and provenance operations retain the upstream platform contract.

The local `artifacts::resolve_executable` API and implementation are removed.
Use `ic_testkit::ic_host_tools::tool::resolve_executable(&requested, &absolute_cwd,
&search_directories)`. The shared resolver reads no ambient PATH, returns
`ResolutionError`, and stops on filesystem errors such as a non-directory search
entry instead of silently selecting a later executable. Resolution alone does
not admit execution. For transforms, use shared `AdmittedTool` with exact
digest/version authority and explicit environment, output bounds and deadline;
declare the same executable in `ArtifactCacheSpec::with_tool`.

`read_wasm` requires a fourth `max_bytes: usize` argument and returns
`Result<Vec<u8>, ic_host_tools::artifact::ArtifactError>` instead of panicking.
Handle or propagate the typed failure. Reads follow links in caller-controlled
target directories, reject non-regular files, and enforce the bound independently
of metadata. They do not validate Wasm. The shared reader also replaces local
cache-sidecar allocation/read loops while preserving oversized-data cache misses,
UTF-8 checks, digest framing and retained `v1` cache layouts and identities.

The fixture benchmark uses shared bounded descriptor reads for its 16 MiB report
budget and shared streaming raw SHA-256 for its 128 MiB Wasm budget. Report
overflow is now a shared typed artifact error; reported digests remain raw SHA-256.
The shared parser-based `dependency-pins-check` replaces the local action scanner
and `actions-check` target. Native portable jobs exercise shared admitted tools
through transactional output publication and bounded Wasm reads. Cargo build
progress, benchmark worker ownership and PocketIC process-tree cleanup remain
with their consumer owners; shared execution owns direct-child capture only.

The transactional external-transform example removes direct `Command` execution
and delegates admission and capture to shared `AdmittedTool`. Its arguments now
include the reviewed raw executable SHA-256 and exact trimmed version stdout:
`<tool-path> <sha256> <exact-version-stdout> <input> <public-output> <cache-root>`.
Admission is required before cache lookup and rechecked before running. The
example uses an empty environment, a 256 MiB executable bound, 64 KiB per output
stream and a 60-second deadline. Its recipe identity becomes
`example/admitted-transform/v1`, so outputs produced by the former inherited
environment are rebuilt rather than reused. Existing cache entries remain
preserved under the unchanged layout. Failures preserve the uncommitted transaction's
normal recovery disposition. Callers retain interpreter/library trust and
responsibility for effects outside the declared staging output.

### Developer setup and retained fixes

Release metadata qualification clears inherited Make flags and command-line
overrides when invoking the actual callbacks in its private workspaces.
Previously, `make release-minor` passed its selected version through recursive
Make and overrode the patch fixture's environment, so finalization rejected
the fixture notes as ambiguous. Each fixture now supplies its own identity
while the real maintainer release keeps its selected identity unchanged
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

Developer setup adopts Shared Tooling 0.1.6 at
`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`. `make install-tools` explicitly
prepares pinned jq/yq and quill, icp, didc, ic-wasm, PocketIC and wasm-opt under
the checkout's `.tools/` directory. Make selects the local executable paths
and PocketIC binary. `make tools-check` verifies installed bytes and versions
offline; `make dependency-pins-check` checks dependency declarations and tracked
workspace lockfiles without changing dependency selections. Both checks are in
the complete CI/release gates, and CI explicitly prepares tools first. Native
portable jobs also exercise installer rejection and preservation fixtures.
Existing developers should run `make install-tools` before those complete gates;
ordinary focused Cargo checks do not require this toolset. Rust toolchain,
MSRV, cargo-sort version and PocketIC client major version are unchanged.

Release metadata preparation explicitly checks README rewriting and validation,
including the subshell result, before recording readiness or publishing files.
This preserves live metadata when either maintained installation example is
malformed or duplicated under Bash 3.2, the macOS system shell.

The portable metadata fixtures also reject unexpected Git/Make commands and
enforce their file, version, validation-log and gate-count assertions explicitly.
Bash 3.2's `set -e` alone does not stop standalone conditional commands; relying
on it caused the selected-commit rejection fixture to fail in native macOS CI
and left other assertions unenforced. Consumer hook qualification now explicitly
enforces formatting, index preservation, hook activation and formatter-policy
assertions too. Metadata fixtures use the repository's actual Make callbacks
with copied consumer helpers, while substituting Git effects and the complete
validation gate
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). Those release-metadata
fixes preserve package metadata and retained `v1` formats; the public API cuts
in this release are listed above.

Source-tree fingerprinting reuses one file-read buffer per hashing pass, bounded
at 64 KiB, instead of allocating it for each file. Growth reserves the required
space explicitly to avoid geometric over-allocation near that limit.
Directory traversal retains
filenames once and compares borrowed native bytes on Unix; other platforms keep
cached encoded sort keys. Digest domains, field encoding, native filename order,
size-change detection and retained cache keys are unchanged. File-digest coverage
includes successive small, large and empty fields to verify that reused scratch
bytes never enter a later field's digest.

## [0.18.3] - 2026-10-06

The reviewed Shared Tooling snapshot advances to
`9437bab201bb6071da0bdc4de0336daf553113f5`, including its release recovery,
failed-inventory rejection, system-Bash formatter correction and user-triggered
maintenance rules. No mutable sibling source is used by CI or release commands.

Normal release targets first reconcile an older committed release, even when
newer fixes have been committed or another increment is requested. They then
perform fresh preflight and complete validation for the requested increment from
the actual local version. An unchanged same-kind retry finishes only its saved
release; explicit resume selects only that release. Exact original commit/tag
identities and evidence are retained, and unknown or conflicting remote state
stops recovery ([Shared Tooling #4](https://github.com/dragginzgame/shared-tooling/issues/4),
[#5](https://github.com/dragginzgame/shared-tooling/issues/5)).

The consumer's committed, tagged and push checks compare the six saved metadata
files with regular-file blobs from the runner-selected `RELEASE_COMMIT`, which
may precede HEAD. Newer documentation does not substitute for the original
release payload. Missing, changed or symlinked committed files, a missing
selection, failed tree inspection and changed retained evidence stop the check
without changing live metadata or the saved receipt. Preparation and pre-commit
checks still verify the live prepared workspace
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

The formatting hook also preserves the index and working files when its
formatter fails under macOS system Bash. Focused Linux fixtures cover selected
commit metadata and preserved rejection evidence, while shared runner fixtures
exercise descendant recovery with substituted Git effects. Native qualification
of this adoption remains separate from the released 0.18.2 results in the
[host matrix](https://github.com/dragginzgame/ic-testkit/blob/main/docs/hosts.md).
These are compatible tooling corrections; package versions, library APIs,
dependency selections and retained v1 layouts are unchanged.

## [0.18.2] - 2026-10-06

Release source and payload inspection requires each Git command to complete
successfully before comparing its digest. Previously, a failed diff or hash
command that had already emitted matching bytes could pass an inline comparison;
failures inside the saved metadata identity constructor could also be lost.
Validation now rejects a failed final source inspection without publishing a
passing receipt, and preparation, recovery and later release checks reject
failed digest reads before their next effects
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

Metadata file hashing bypasses Git attributes, clean filters and line-ending
conversion. Previously, those transformations could make different live or
retained file bytes produce the same identity, admitting an unvalidated change.
Readiness and recovery checks now bind the actual file contents, including
changes between LF and CRLF under normalizing attributes
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). Identity mismatches
stop recovery while preserving its record; they do not rewrite saved evidence.

README requirement updates use the installation checker's TOML example locator.
Its `--rewrite VERSION` mode operates on preparation's private copies, replacing
only the one validated requirement before the adapter checks the result.
Previously, a broader replacement also changed historical dependency lines
outside TOML blocks. Patch, minor and major preparation now preserve those lines,
other code examples, indentation, comments and a missing final newline. Missing,
duplicate or malformed maintained examples still stop preparation before live
metadata publication ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

Rejection preserves current metadata, retained old/new files and validation
evidence. A validation retry runs fresh checks while retaining the failed
attempt's log. Focused fixtures inject failures after complete output, including
individual live, backup and prepared files. Real Git attribute fixtures also
check raw-byte rejection and restoration of the exact saved payload. These are
compatible tooling fixes:
library APIs, dependency selections and the retained `v1` layout are unchanged.
Native qualification of this correction is separate from the 0.18.1 results
recorded in the [host matrix](https://github.com/dragginzgame/ic-testkit/blob/main/docs/hosts.md).

## [0.18.1] - 2026-10-05

The reviewed Shared Tooling snapshot advances to
`f52c0e2476aee094359ed21de91c468540d3969f`. Rerunning the same
`make release-patch`, `make release-minor` or `make release-major` target now
reconciles an unfinished release at its saved version before computing an
increment, including when preparation already changed metadata
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). Recovery preserves
validation evidence, rejects conflicting identities, payloads and destinations,
and avoids duplicate matching commits, tags and completed pushes. Explicit
`make release-resume VERSION=X.Y.Z` uses the same checks.

Recovery preflight checks offline caches against the verified saved workspace
when preparation has published only part of its metadata. A new Cargo.lock
with the old Cargo.toml previously made the repeated `cargo fetch --locked`
fail before recovery could finish. The adapter now verifies retained evidence
and admits only exact saved old/new file identities before checking that
consistent private workspace. It preserves the live manifest and lockfile,
dependency selections and original validation rather than resolving again
against mixed live metadata
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

Interrupted atomic publication can leave `.release-metadata.*` staging files
beside root or packaged metadata. Those helper-owned names are now excluded
from Git's untracked-source checks only in the repository root and
`crates/ic-testkit`, so recovery can continue without deleting those files.
It publishes from verified saved metadata using fresh staging files. Unrelated
untracked files, including the same prefix in other directories, still block
the release ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

Release metadata phases and `make ensure-clean`, the publication prerequisite,
require Git's untracked inventory to complete successfully before admitting
source. A failed command with empty output previously passed as a clean
checkout. Rejection now preserves release files and existing validation
evidence without running another gate or beginning preparation
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

Preparation and recovery reject symlinks anywhere in the retained metadata
tree before copying backups or publishing release files. A linked readiness
file could previously be rejected only after metadata had already changed;
linked backup files and preparation directories are now rejected at the same
boundary. Current metadata and linked targets remain unchanged on rejection
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). Corrupt retained
state stays available for inspection rather than being followed or removed.
Qualification failures also retain their private fixture directories for
inspection.

Hook installation delegates physical-path normalization to the reviewed
installer, including macOS temporary-directory aliases; the consumer Makefile
no longer repeats that workaround
([Shared Tooling #1](https://github.com/dragginzgame/shared-tooling/issues/1)).

Release-metadata qualification constructs its own candidate notes instead of
depending on an undated heading in the checkout. It exercises preparation
from finalized notes, preserves published history, and rejects conflicting
candidates in either changelog view
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). This fixes checks
that failed after the 0.18.0 notes were finalized.

Portable CI explicitly installs the pinned toolchain's `rustfmt` component
before formatting-hook checks. Its system Bash step uses strict mode so
command, pipeline and unset-variable failures stop the step immediately.
Hook setup and formatting checks verify `rustfmt` availability with automatic
installation disabled; setup fails before changing hook configuration if that
component is missing. All four CI matrices collect results on every supported
host even if another host fails, rather than cancelling macOS qualification
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
These are compatible tooling fixes; library APIs and runtime behavior are
unchanged. Native qualification still requires matching successful runs on
Linux and both supported macOS architectures.

## [0.18.0] - 2026-10-05

The repository's maintainer release CLI makes a hard cut to the reviewed
Shared Tooling runner ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
Use `make release-patch`, `make release-minor` or `make release-major`; all run
the complete release gate, including MSRV, before changing metadata. The old
`patch`, `minor`, `release-ci`, `release-stage`, `release-commit`, `release-push`
and `changelog-check` targets are removed. Library APIs and runtime behavior
are unchanged. The command changes require a minor release before 1.0.

Maintain one numbered, undated pending release in both changelog views. Infer
its version from the latest finalized release and the complete pending batch;
these command changes select 0.18.0 from 0.17.3. Use `make release-minor` for
this batch. The workflow rejects a conflicting candidate and finalizes both
views with its saved version and UTC date, updates both installation
examples, and stages only Cargo.toml, Cargo.lock, both changelogs and both READMEs.
It preserves dependency selections and build artifacts. Commits and tags remain
maintainer-owned; agents do not invoke the one-shot release commands.

`make release-resume VERSION=X.Y.Z` reconciles the exact retained plan, source,
commit, branch and remote rather than bumping again. Prepared old/new metadata
and validation logs remain in the Git directory. Atomic pushes use the selected
branch and tag with `--no-follow-tags --atomic`; package publication remains a
separate `make publish` action. Native qualification is documented in the
[host matrix](https://github.com/dragginzgame/ic-testkit/blob/main/docs/hosts.md).

Developer setup uses `make install-format-tools install-hooks` to prepare pinned
`cargo-sort` 2.1.4 and activate the reviewed repository-local pre-commit hook.
Both `make fmt` and `make fmt-check` sort all workspace Cargo manifests before
Rust formatting; the latter checks without mutation. The hook formats an export
of the exact index, refreshes only selected files, rejects partial staging, and
preserves unrelated working edits. CI and release gates independently check
formatting. Prepared release manifests are sorted before their exact payload is
saved and staged, without changing dependency selections.

Preflight or validation-only failures restart through the normal release target
after correcting inputs; both gates run afresh against the current source.
Each attempt retains its identity and full validation log, and a failed retry
cannot reuse an earlier successful receipt. Once preparation may begin, the
retained exact plan owns recovery via `make release-resume VERSION=X.Y.Z`.

The complete gate preserves the caller's network policy. Its locked
`cargo publish --dry-run` check requires registry HTTP access without uploading;
forcing the whole gate offline previously prevented validation from finishing
([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). Cache availability and
metadata preparation remain offline without changing dependency selections.
Explicit offline requests remain effective, with no automatic online retry.

`make install-hooks` invokes the unchanged reviewed installer from the physical
workspace path. Its consumer qualification checks also canonicalize private
temporary directories and exercise setup through an aliased directory, while
preserving refusal of a different hook path. This prevents the false root
rejection described in [Shared Tooling #1](https://github.com/dragginzgame/shared-tooling/issues/1),
including macOS temporary-path aliases; native confirmation remains required.

## 0.17.3

The root and packaged README installation examples now select
`ic-testkit = "0.17"`; copying them no longer selects the older 0.14 line. Library APIs and
runtime behavior are unchanged, and no migration is required.

Repository CI checks both maintained TOML examples against the workspace
package version. Release preparation checks the selected target minor line
before the manifest bump, while historical migration prose remains outside
the check. Focused shell fixtures pass, including rejection of stale, missing,
duplicate and malformed examples. Native system-Bash confirmation of this
guard remains pending matching CI.

## 0.17.2

The synthetic HTTP fixture used by the PocketIC transport and teardown tests
explicitly selects blocking I/O on accepted streams before setting read/write
timeouts. This prevents an immediate `WouldBlock` when macOS inherits the
listener's nonblocking mode and the client has connected without sending data.
Library APIs and runtime behavior are unchanged; no migration is required.

A real TCP regression reproduces the failure with an initially empty,
nonblocking stream and passes after the fix. The focused Linux suite passes.
Portable CI now selects these synthetic tests on Linux and both macOS
architectures without needing a live PocketIC server. Native confirmation of
this change remains pending matching CI. The released 0.17.1 portable-host
checks passed on all three hosts, including the earlier benchmark report and
tool-launch checks.

## 0.17.1

The repository-local benchmark publishes `--output` reports with an atomic
replacement after encoding and syncing private scratch on the destination's
filesystem. Failed writes, encoding, syncing, observed interruption before
commit and failed renames preserve the previous report. The parent directory
must already exist; only driver-owned scratch is cleaned up.

Published Unix reports have mode `0600`. Replacement changes the requested
directory entry, preserving old readers, hard links and any symlink target.
Library APIs, report fields, the `v1` format and dependency selections are
unchanged. Atomic visibility is not a power-loss durability guarantee.

Standalone driver launches retain the Cargo/rustc invocation names required by
rustup proxies. Cargo and compiler provenance use the repository's workspace
context. Explicit relative `CARGO`/`RUSTC` overrides resolve from the caller's
directory, and the selected compiler feeds both builds and provenance. Invalid
explicit tools propagate failures without selecting another tool.

The abandoned-staging pruning test now runs in an isolated subprocess, so
unrelated parallel test spawns cannot inherit its lock descriptors. Unix
coverage explicitly retains a duplicate content-lock descriptor after abort:
pruning preserves orphan staging while that descriptor is open and removes it
after it closes. This reproduced the reported zero-removals failure; the updated
test and focused transaction suite pass. Production pruning and lock/cache
formats are unchanged. Native macOS confirmation remains pending CI.

Focused Linux driver tests and Clippy pass. The released 0.17.0 startup and
driver checks passed on all three native CI hosts; the new publication and
tool-launch checks still require a matching native CI run. A deterministic
subprocess regression failed before the proxy fix and passes afterward.
A standalone launch from outside the checkout also completed a three-task live
Linux PocketIC smoke with Cargo/compiler/toolchain overrides unset, using the
real rustup proxies and locked offline builds. The dev-profile report records
the workspace-selected Rust 1.99.0 compiler.

A live Linux PocketIC 16.0.0 benchmark completed 108 tasks with three rotating
repeats over fresh fixtures and pooled capacities one/two, two workers and
1 MiB of state per canister. A relative output path received the complete
private report. Its dirty-source provenance and changed Wasm identity describe
this working-tree run, not a clean released performance baseline.

## 0.17.0

The repository-local fixture benchmark driver is now a Rust example supporting
native Linux and macOS. Replace the retired Python benchmark command with
`cargo run -p ic-testkit --locked --offline --example fixture_reuse_benchmark_driver --`
followed by the existing benchmark options. Focused checks use `cargo test`
with the same example. This operator CLI hard cut is recorded as a minor release
before 1.0; library APIs and the existing workload are unchanged.

Linux retains native `/proc` RSS-page sampling, and macOS uses process-leader RSS
from `/bin/ps`. The report stays `ic-testkit-fixture-benchmark-v1` and records its
sampling source in `provenance.rss_sampler`. Host and sampler changes require
new performance baselines; previous Linux measurements retain their original
qualification. Fresh/pooled cases, capacity sweeps, rotated repeats, raw samples
and summaries retain their measurement boundaries.

Builds use locked, prepared offline dependencies. Provenance is captured before
measurement. Interrupted runs and sampler errors wait for worker-owned server
cleanup; caller-owned artifacts are preserved. Worker JSON is bounded and
validated before summaries. CI adds focused native checks on Linux and both
macOS architectures; macOS qualification remains pending those runs.

Dependency selections, cache formats, MSRV and workspace package versions are
unchanged. The [benchmark guide](https://github.com/dragginzgame/ic-testkit/blob/main/docs/fixture-reuse-benchmark.md)
describes the current Cargo commands and prerequisites.

Focused Linux driver checks pass. A synthetic CLI smoke verifies build flags,
case rotation and report emission without executing a live PocketIC workload.
Clippy and workflow validation pass; new native macOS and live performance
evidence remains pending.

The managed-server background-reaper test now waits for an explicit caller
release after handoff instead of exiting within a 30 ms readiness window.
The 0.16.1 Intel macOS portable-host job exposed this test timing race;
focused Linux startup checks pass and production startup behavior is unchanged.

## 0.16.1

Native macOS CI for 0.16.0 exposed startup exit-reporting and test-fixture
portability failures. Managed startup now recognizes Darwin's group-signal
`EPERM` result only after verifying that the sole member is its exited,
unreaped child. Groups containing another process still propagate permission
failures; the child exit status and captured output retain their existing
`ServerExited` contract.

Port and output readers validate regular files after opening nonblocking on
Unix. Replaced FIFOs cannot stall startup or captured-output reads. Non-regular
port files return `PocketIcStartupError::Io` with `InvalidData`; unavailable
output streams are omitted.

Cargo fixtures account for canonical package paths and preserve explicit
Cargo-home lookup paths across directory aliases. Non-ASCII filesystem fixtures
work on macOS while Linux retains invalid-UTF-8 filename coverage; Unix native
byte conversion is exercised without requiring filesystem support. Remaining
function-local imports move to their owning module's import group with their
platform conditions retained.

No consumer migration is required. Public APIs, dependency selections and
repository-owned `v1` formats are unchanged. Native macOS qualification for
this revision remains pending CI.

Managed ownership tests now exercise leader reaping and descendant termination
on both Linux and macOS, including drop, timeout, exit and background reaping.
Native process-state checks distinguish running processes from zombies and
reaped children; descendants cannot naturally expire within the cleanup deadline.

Focused Linux tests pass, including the affected Cargo-input fixtures under a
symlinked temporary directory. Library/test-target Clippy checks and public API
documentation builds pass with warnings denied; formatting and shared-tooling
snapshot verification pass.

## 0.16.0

This is a minor release because the acquired-package contract excludes outputs
that previous versions accepted. Acquisition builds only libraries; remove
`--all-targets` and binary/example/test/bench selectors from acquisition specs,
and build those targets separately. Each acquired package must declare the
same-name `cdylib` library described below. Related cuts shipped in earlier
0.15 patches; future incompatible pre-1.0 behavior changes use minor releases.

Repository tooling now adopts a pinned, verified Shared Tooling baseline.
Release CI uses ordinary CI and the caller's temporary-directory environment;
the private scratch/process scanner and Linux-only Python/pidfd dependency are
retired. Caller-owned temporary directories must outlive upstream background
servers; explicit managed handles own their child cleanup. The host matrix and
native macOS CI cover Bash 3.2 guard tooling, startup and concurrency. Test helper
modules follow ordinary Rust discovery, error assertions use structured failures,
and `libc` is workspace-owned. Release version helpers preserve existing
external dependency selections using offline metadata updates.

Wasm acquisition now checks Cargo metadata for a `cdylib` library whose name
matches the package's expected `<package>.wasm` output. Missing, non-`cdylib`,
and renamed libraries return `WasmBuildError::InvalidSpec` before cache reuse,
compilation, or scheduled shared-target maintenance. Previously, changing a
library to `rlib` or renaming it could publish an old shared-target Wasm under
the changed manifest's fingerprint after a successful Cargo build.

Ensure each acquired package declares the expected library name and includes
`cdylib` in `[lib].crate-type`. Libraries emitting both `rlib` and `cdylib` are
accepted; a `cdylib` example cannot replace the required library. This check also
applies to batches, immutable-source sessions, and prepared snapshot readers.
Invalid packages retain input-discovery failures while compatible batch entries
continue to succeed.

`resolve_cargo_build_inputs` and transactional artifact recipes still support
input snapshots for arbitrary Cargo targets. Public signatures, fingerprint
domains, and persisted layouts are unchanged; repository-owned formats remain
`v1`.

A real Cargo regression reproduced stale publication after switching to `rlib`.
It checks type/name rejection, preservation of shared outputs before scheduled
retention, absence of compilation outputs, and reuse after restoring the
library. Focused checks cover library/example distinction, dual crate types,
batch/session/prepared validation, and generic transactional snapshots.
Full pre-push validation remains maintainer-owned.

## 0.15.9

Wasm acquisition now always builds package libraries with Cargo's `--lib` flag.
Previously, a package containing a binary and a canister library with the same
name could build successfully while the binary overwrote the library Wasm.
Both isolated and shared acquisition now publish the library artifact, including
builds with progress observers.

Remove `--all-targets` from `with_cargo_profile_args`: it now returns
`WasmBuildError::InvalidSpec` before input resolution or cache work because it
can reintroduce the collision. Binary, example, test, and bench selectors remain
rejected. Run those builds separately from canister acquisition. Packages used
for acquisition must provide a library that produces the declared Wasm output;
binary-only packages are no longer built by acquisition. Explicit `--lib` remains
accepted and is included only once in the Cargo invocation.

Fingerprints now include library target selection, triggering a fresh exact
acquisition after upgrading. Shared Cargo targets can still reuse library
compilation outputs. Public signatures and persisted layouts are unchanged;
repository-owned formats remain `v1`.

A real Cargo regression reproduced binary replacement before fixing. It checks
library bytes and exact-cache reuse for isolated/shared targets and
observed/silent builds. Focused checks cover early selector rejection and actual
Cargo arguments in both output modes. Full pre-push validation remains
maintainer-owned.

## 0.15.8

This patch prevents stale shared-target canister Wasm publication when Cargo
arguments select binary, example, test, or bench targets. These builds can finish
successfully without refreshing the canister library. Acquisition now returns
`WasmBuildError::InvalidSpec` before input resolution or cache work for `--bin`,
`--bins`, `--example`, `--examples`, `--test`, `--tests`, `--bench`, and `--benches`.
Named selectors are rejected in both separate-value and `--option=value` forms.

Remove these selectors from `with_cargo_profile_args`; use default target
selection or `--lib` to acquire canister library artifacts. `--all-targets`
remains accepted. Run separate binary, example, test, or bench builds outside
Wasm acquisition. Public signatures and persisted layouts are unchanged;
repository-owned formats remain `v1`.

A real Cargo regression reproduced successful binary-only acquisition of an old
canister output after source changes. It checks early rejection, preservation
of shared output, absence of a binary build, and correct library rebuild and
reuse. Focused validation checks cover named and plural selectors, isolated and
shared acquisition, and rejection without filesystem side effects.
Full pre-push validation remains maintainer-owned.

Documentation images now come from the shared-assets repository instead of
duplicated local files.

## 0.15.7

This patch prevents stale shared-target Wasm publication when the declared
profile output directory disagrees with the Cargo build arguments. Mismatches
now return `WasmBuildError::InvalidSpec` before input resolution or cache work.

Pair `WasmBuildSpec::new(..., "release")` with
`.with_cargo_profile_args(["--release"])` or an equivalent explicit release
selection. Default builds, `--profile dev`, and `--profile test` use `debug`;
`--release`, `-r`, and `--profile bench` use `release`; custom profiles use their
own name. Both `--profile name` and `--profile=name` are supported, including
release flags clustered before feature arguments. Empty or conflicting profile
selections are rejected.

Previously, Cargo could compile changed sources in debug mode while acquisition
certified an old release output with the new fingerprint. A real Cargo
regression covers rejection and subsequent correct release rebuild and reuse.
Focused checks cover profile matching and rejection without filesystem side
effects. Public signatures, dependencies, and persisted layouts are unchanged;
repository-owned formats remain `v1`.

## 0.15.6

This patch simplifies install retries, artifact paths, and batch bookkeeping.
Public API signatures, dependencies, and persisted layouts are unchanged;
repository-owned formats remain `v1`. No consumer migration is required.

- Install-code retries use an explicit budget after the initial attempt. The
  unreachable panic and a one-line classification helper are removed. Only
  install-code rate limiting is retried, cooldowns occur between attempts, and
  the final rejection is returned unchanged when the budget is exhausted.
- Watched-input freshness tests share the existing artifact temporary-directory
  helper instead of maintaining a separate clock-based name and atomic counter.
- Wasm acquisition uses one artifact-path construction for default and
  caller-selected targets, removing an equivalent special-case branch.
- Both artifact-batch metrics types derive successful and failed totals instead
  of storing and updating redundant counters. Successful outcomes remain the
  authoritative source for built/reused counts and timing sums. Public metric
  getters and report display retain their values.

Focused checks cover exhaustion, final rejection, non-retryable failures,
immediate and eventual success, cooldown ordering, zero-attempt policies, and
freshness-stamp behavior.

Focused batch checks also cover empty reports, failures, transaction builds and
reuse, and cold/warm Wasm metrics through a real Cargo feature-resolution fixture.
Warm Wasm publication is checked in isolated, shared, and scheduled-maintenance
modes, including repair of changed outputs and preservation of matching files.

Rust 1.88 compilation, Clippy, formatting, and diff checks pass.
Full pre-push validation remains maintainer-owned.

## 0.15.5

This patch prevents build-graph Cargo invocations from publishing stale Wasm
and confines profile output paths to their target directories.
Public API signatures, dependencies, and persisted layouts are unchanged;
repository-owned formats remain `v1`.

- Cargo's `--unit-graph` flag now returns `WasmBuildError::InvalidSpec` before
  resolution or cache acquisition. It prints a graph and exits successfully
  without compiling; previously, an existing shared-target Wasm could be
  published under a new fingerprint. Remove this flag from build specifications
  and run graph inspection separately from Wasm acquisition.
- Help and build-graph modes use the same early validation path.
- Profile output directories must be one normal path component. Absolute paths,
  parent traversal, and nested paths now return `WasmBuildError::InvalidSpec`
  before resolution or acquisition. Pass Cargo's output subdirectory name, such
  as `debug`, `release`, or a custom profile name, rather than a path.

The failure was reproduced with real nightly Cargo. Maintained regressions need
no nightly and verify early rejection, no cache publication, and unchanged shared
outputs. A profile-boundary regression also failed before fixing; targeted checks
cover invalid paths, accepted names, absence of filesystem side effects, and
compatible batch resolution. Focused Cargo argument checks, Rust 1.88 compilation,
Clippy, documentation, formatting, and diff checks pass. Full pre-push validation
remains maintainer-owned.

## 0.15.4

This patch simplifies benchmark processing and avoids unnecessary cache-pruning
work.
Public APIs, dependencies, and persisted layouts are unchanged; no consumer
migration is required. Repository-owned formats remain `v1`.

- Captured stdout and stderr append to one parse report. The temporary stderr
  report and its merge are removed. Stream ordering, source line numbers,
  malformed-line contents, ignored counts, and strict-mode behavior are preserved.
- Aggregation borrows labels from spans while building its temporary map and
  creates owned labels for final rows or overflow errors. The helper derives the
  label from the span instead of accepting it again as a separate argument.
  Named suites remain distinct from the global aggregate, including suites named
  `ALL`; ordering, checked totals, run counts, extrema, and peak-end counters are
  unchanged.
- Run allocation and previous-run discovery share one numeric index parser,
  removing prefix reconstruction and reparsing. Index width, ASCII digit
  validation, numeric ordering, overflow rejection, and metadata ranking remain
  unchanged.
- Size pruning skips sorting when retained bytes already fit the limit. Age
  pruning still runs first; above-budget removal retains its last-use/path order,
  active-entry and live-consumer protection, and report totals.

Focused checks cover both streams, strict parsing, empty and blank inputs,
malformed markers, source positions, repeated spans, numeric boundaries, and
overflow diagnostics for every counter and scope.
Run-discovery checks cover malformed and large indices, metadata selection, and
index exhaustion. Pruning checks cover entries at and below the size limit,
least-recently-used removal, and active-entry protection.
Report-output checks, Rust 1.88 compilation, Clippy, documentation, formatting,
and diff checks pass. Full pre-push validation remains maintainer-owned.

## 0.15.3

This patch prevents help-only Cargo invocations from publishing stale Wasm and
simplifies benchmark comparison. Public API signatures, dependencies, and
persisted layouts are unchanged. Repository-owned formats remain `v1`.

- Cargo help flags now return `WasmBuildError::InvalidSpec` before resolution
  or cache acquisition. Previously, a successful `cargo build --help` could
  publish an existing shared-target output under a new fingerprint. Remove help
  flags from build specifications; long and clustered short forms are rejected.
  Feature values containing `h`, including compact forms, remain accepted.
- Benchmark comparison uses one ordered map instead of two indexes, a key list,
  and explicit sorting. Ordering, missing rows, last-duplicate handling on both
  sides, averages from current totals and runs, and the distinction between named
  `ALL` suites and all-suites aggregates remain intact.
- Observed Cargo output uses a bounded pending-chunk queue. Slow callbacks can
  delay Cargo's writes instead of growing an unbounded forwarding backlog.
  Raw stream contents and complete failure diagnostics are preserved, including
  when output forwarding is disabled; diagnostic capture itself is not truncated.

The stale-publication regression was reproduced using real Cargo before fixing.
Focused argument, feature-resolution, and benchmark checks pass, along with
large-stream, disabled-forwarding, reader-disconnection, read-error, and failing
Cargo checks. Rust 1.88 compilation, Clippy, documentation, formatting, and diff
checks also pass.
Full pre-push validation remains maintainer-owned.

## 0.15.2

This patch bounds cache metadata reads and simplifies benchmark marker parsing.
Public APIs, dependencies, and persisted layouts are unchanged. Repository-owned
formats remain `v1`; no consumer migration is required.

- The shared bounded file reader rejects oversized manifests before inspecting
  output contents. The existing writer and declared output set determine the
  limit, retaining one authoritative manifest layout.
- Malformed manifests still trigger rebuilding. A corrupt entry retained by a
  live consumer cannot be replaced until released. Content verification,
  undeclared-output rejection, and atomic publication remain intact.
- Manifest byte comparison keeps its existing malformed-data behavior. UTF-8
  stamp readers retain their decoding errors through the shared read helper.
- Last-use and maintenance markers share the bounded stamp reader. Oversized
  last-use markers fall back to directory modification time; oversized maintenance
  markers make maintenance due. Normal timestamps, changed-policy scheduling,
  LF/CRLF handling, future-time behavior, and I/O error policies remain intact.
- Benchmark marker parsing uses a fixed array for six fields and the absence of
  a seventh, removing the dynamic column list. Extra columns still report the
  same error, preserve the original line and source position, and do not prevent
  later valid markers from being parsed.
- Destination-stamp comparison also uses the bounded file reader, preserving
  writable-file, ownership, permission, and link checks and keeping matching
  stamps during warm publication.

Focused checks cover oversized and malformed manifests, retained-entry protection,
multiple outputs, empty-file validation, watched-input and Wasm stamps, and
Wasm-cache reconstruction. Marker checks cover timestamps, bounded fallback,
policy intervals, CRLF, and read errors. Benchmark checks cover malformed and
valid markers, source diagnostics, and strict parsing; warm Wasm checks cover
file preservation and linked or restricted file replacement.
Rust 1.88 compilation, Clippy, documentation, formatting, and diff checks pass.
Full pre-push validation remains maintainer-owned.

## 0.15.1

This patch fixes transport recovery classification without changing public APIs,
dependencies, or persisted cache formats. No consumer migration is required.

- Typed Candid transport errors retain their classification through contextual
  `std::io::Error` wrappers, including nested wrappers.
- `CanisterDiagnosticFailure::InstanceUnavailable` is recognized by
  `is_dead_pocket_ic_transport_error`, directly and through error wrappers.
  Diagnostic controller rejections, decode failures, and unrelated application
  panics still do not qualify as transport failures.
- Watched-input freshness and Wasm-cache validation share a bounded stamp reader.
  Reads stop at the expected stamp length plus one byte; oversized stamps are
  stale. Exact matching, Wasm content verification, and recovery from verified
  public outputs remain intact. Watched-input checks preserve missing-stamp
  handling, I/O errors, and `InvalidData` for invalid UTF-8 within the size limit.
  Truncated Wasm stamps are rejected before scanning artifact contents. Stamp
  formats remain `v1`.

Focused unit and integration checks cover both reproduced failures, a refused
server, live PocketIC 16 diagnostic authorization, and artifact stamp boundaries.
Rust 1.88 compilation, Clippy, documentation, formatting, and diff checks pass.
Full pre-push validation remains maintainer-owned.

## 0.15.0

This minor release makes standalone fixture capacity configurable at runtime and
consolidates reset policies into one model. It contains hard source/API cuts.
Dependencies and persisted cache layouts are unchanged; owned format identifiers
remain `v1`. Existing repository-owned cache entries need no migration.

| Previous API | Replacement |
| --- | --- |
| `CachedStandaloneCanisterFixturePool<N>` | `CachedStandaloneCanisterFixturePool`, passing `NonZeroUsize` to `new`. |
| `CachedStandaloneCanisterFixturePool::<N, _>::new(builder)` | `CachedStandaloneCanisterFixturePool::new(capacity, builder)`; the builder type is inferred. |
| `ResetRequirement::Domain(policy)` | `ResetDomainPolicy::Domain(policy)` in `ResetRequirements`. |
| `ResetAchievement::Domain(policy)` | `ResetDomainPolicy::Domain(policy)` in `ResetReceipt`. |

A standalone pool can still be initialized statically without constructing any
fixtures or allocating slots:

```rust
use std::num::NonZeroUsize;
use ic_testkit::pic::{CachedStandaloneCanisterFixturePool, StandaloneCanisterFixture};

static POOL: CachedStandaloneCanisterFixturePool =
    CachedStandaloneCanisterFixturePool::new(
        NonZeroUsize::new(2).unwrap(),
        build_fixture,
    );

// Supply the same fixture recipe on every invocation.
fn build_fixture() -> StandaloneCanisterFixture {
    // Existing application-owned construction, installation, and seeding.
    todo!()
}
```

Local pools can use runtime capacity and a capturing builder:

```rust
let capacity = std::thread::available_parallelism()?;
let pool = CachedStandaloneCanisterFixturePool::new(
    capacity,
    move || build_fixture_with_config(&config),
);
```

Capacity is an explicit host resource budget; CPU availability alone does not
establish that enough memory exists for that many PocketIC instances. Callers
still decide which tests can safely reuse snapshots.

When a statically typed function-pointer pool also selects restore funding,
coerce the builder before chaining the policy method:

```rust
use ic_testkit::pic::SnapshotRestoreFunding;

static FUNDED_POOL: CachedStandaloneCanisterFixturePool = {
    let pool: CachedStandaloneCanisterFixturePool =
        CachedStandaloneCanisterFixturePool::new(
            NonZeroUsize::new(2).unwrap(),
            build_fixture,
        );
    pool.with_restore_funding(SnapshotRestoreFunding::TopUpTo {
        minimum_cycles: 5_000_000_000_000,
    })
};
```

Standalone and multi-canister pools now share lazy slot allocation.
`CachedPocketIcBaselinePool::new` and both pools' `capacity()` accessors are
const. The scheduler still enforces exclusive leases, FIFO waiting, cancellation,
and unwind invalidation. Snapshot funding, non-snapshot reset, readiness,
validation, and recovery behavior are preserved.

Reset policy declarations and completion evidence use one domain-policy enum:

```rust
use ic_testkit::pic::{
    CycleResetPolicy, ResetDomainPolicy, ResetReceipt, ResetRequirements,
    TimeResetPolicy,
};

let requirements = ResetRequirements::try_new(
    CycleResetPolicy::PreserveCurrent,
    [ResetDomainPolicy::PocketIcTime(TimeResetPolicy::PreserveCurrent)],
)?;

// Report the policy actually achieved by the recipe's reset operation.
let receipt = ResetReceipt::try_new([
    ResetDomainPolicy::PocketIcTime(TimeResetPolicy::PreserveCurrent),
])?;
```

`ResetRequirements` and `ResetReceipt` remain distinct types. Both reject
repeated domains. Warm preparation still checks every required domain and policy,
cycle policy, the exact captured canister set, readiness, and recipe validation.
The change introduces no automatic reset or implicit fresh-instance guarantee.

Cache-directory tag validation now reads only the standard signature prefix.
Valid ordinary tags are preserved, including CRLF and additional comments.
Invalid tags and symlinks are replaced without changing linked referents.

The repository's opt-in benchmark now uses `--modes fresh pooled` and accepts
`--capacities 1 2 4 8`; replace `--modes pooled-1 pooled-2` with
`--modes pooled --capacities 1 2`. Default comparisons remain fresh fixtures and
capacities one/two, with the same task and worker counts across cases. Each warm
acquisition still validates restored state. Capacity sweeps are recorded with
phase timings and sampled process-tree RSS; development smoke results do not
establish downstream speedups.

Focused scheduler, fixture recovery, and reset-contract checks pass, including
all five non-snapshot policy domains. Rust 1.88 compilation, focused Clippy,
formatting, and diff checks pass. Full pre-push validation remains
maintainer-owned.

## 0.14.12

This patch release aligns Cargo input discovery with execution and bounds
managed PocketIC readiness-file reads. Public API signatures, dependencies,
and persisted layouts are unchanged; owned format identifiers remain `v1`.

- Managed startup reads at most 65 bytes and rejects port files larger than
  64 bytes with bounded diagnostics. Missing files and partial writes remain
  pending. Nonzero decimal ports, UTF-8 errors, owned-child teardown, and private
  startup-file cleanup retain their behavior.
- Grouped short feature arguments such as `-qFextra`, `-rF=extra`, and `-vF extra`
  reach Cargo metadata. Enabled optional dependencies are watched, and batch
  resolution shares the same feature context.
- Cargo configuration includes resolve beside the configured entry when that
  entry is a symlink. Input paths preserve file and directory lookups, so guards
  observe link replacement and nested include changes. Separate aliases retain
  their include paths; duplicate lookup paths and recursive cycles are handled.
- Startup unit fixtures use the same child-process executable writer as artifact
  tests, removing a separate writing path susceptible to parallel Unix
  `ETXTBSY` launch failures.

`with_cargo_profile_args` can no longer redirect inputs outside the build
specification's discovery context. These options return `InvalidSpec` before
resolution or acquisition, including their attached and grouped short forms:

| Previously supplied argument | Required replacement |
| --- | --- |
| `--manifest-path`, `-m`, or `-C` | Set the invocation directory through `WasmBuildSpec::new`'s `workspace_root`. |
| `--package`, `-p`, `--workspace`, `--all`, or `--exclude` | Supply the exact package list to `WasmBuildSpec::new`. |
| `--target` | Use `with_target`. |
| `--config` | Use discovered Cargo configuration files or explicit `with_extra_env` values. |

Existing target-directory ownership checks remain enforced. Corrected feature
and configuration inputs can change fingerprints and trigger fresh builds;
no persisted-cache migration is required.

Readiness, input-override validation, and symlink discovery regressions were
reproduced before fixing them. Focused tests cover partial writes and child
cleanup, grouped features and dependency mutation, configuration lookup and
alias replacement, duplicate discovery, cycles, batched reuse, workspace
projection, warm mutation rejection, and observed Cargo diagnostics. Targeted
checks, Clippy, formatting, and diff checks pass. No performance improvement
has been measured. Full pre-push validation remains maintainer-owned.

## 0.14.11

This patch release fixes executable and benchmark-run discovery and consolidates
Wasm package selection. Public API signatures, dependencies,
fingerprints, and persisted layouts are unchanged; repository-owned format
identifiers remain `v1`.

- `resolve_executable` skips non-directory `PATH` components and continues to
  later executable candidates. Missing, non-executable, and directory candidates
  also remain skippable during search.
- Absolute executable paths no longer require a readable current directory.
  Relative paths still depend on it, and explicitly supplied invalid paths
  retain their errors without falling back to `PATH`.
- Benchmark run discovery reads directories directly. Missing roots still begin
  at index one or have no previous run; other directory errors propagate rather
  than being concealed by an existence probe.
- `WasmBuildSpec` sorts and deduplicates package names once. Fingerprints,
  resolved inputs, artifact paths, and Cargo commands use that canonical list;
  `packages()` returns sorted, unique names. Keep a separate caller-owned list
  if original request ordering is needed. Fingerprint bytes and artifact ordering
  remain unchanged, and repeated packages produce only one `-p` argument each.

Existing callers need no source or cache migration. Both executable-resolution
regressions were reproduced before fixing them. Three focused Unix tests cover
search-entry semantics, invalid candidates, explicit-path errors, and removed-directory
handling in an isolated child process. A benchmark root-path regression was
also reproduced before fixing it. Focused checks preserve run ordering and
allocation, package identity and Cargo arguments, workspace projections,
batched input reuse, empty selections, and cold/warm acquisitions.
Targeted tests, Clippy, formatting, and diff checks pass. No performance gain
has been measured. Full pre-push validation remains maintainer-owned.

## 0.14.10

This patch release fixes artifact publication and repair failures. Public APIs,
fingerprint bytes, dependencies, and persisted layouts are unchanged;
repository-owned format identifiers remain `v1`.

- Failed exclusive temporary-file creation preserves the existing temporary
  file and destination. Collisions still return a creation error; cleanup only
  removes a temporary file successfully created by this operation.
- Short sibling temporary names allow atomic writes and copies to valid long
  destination filenames. The temporary name remains distinct from the
  destination, including ASCII case variants. Same-directory rename, file
  synchronization, and cleanup after write or rename failure are preserved.
- Transactional publication and cache hits can replace cyclic final output
  symlinks without requiring readable referents. Directory targets,
  invalid parent paths, and input/output overlap remain rejected. Declared
  inputs and retained artifacts are unchanged by output-link repair.
- Maintenance marker writing uses the shared atomic writer's directory creation
  instead of repeating it. Scheduling and maintenance failure handling remain
  enforced.

Existing callers need no source or cache migration. Temporary names are private
implementation details. Publication regressions were reproduced before fixing
them; focused tests cover ownership, filename limits, failure cleanup, cyclic
and dangling output-link repair, invalid parents, artifact reuse, maintenance,
and cross-process retention. Targeted tests, Clippy,
formatting, and diff checks pass. No performance gain has been measured. Full
pre-push validation remains maintainer-owned.

## 0.14.9

This patch release gives each verified retained Wasm entry authority over the
bytes for its fingerprint and removes redundant publication work. Public APIs,
fingerprint bytes, and persisted layouts are unchanged; repository-owned format
identifiers remain `v1`.

- Public Wasm files carrying valid stamps but different bytes from the retained
  entry are repaired to match the artifact paths returned by acquisition.
- Publication from verified exact entries reuses lengths and digests rather than
  hashing public copies again to construct stamps. Warm files and stamps are
  repaired separately; a stamp-only failure no longer forces a Wasm copy.
  Matching independent, writable Unix files owned by the effective user retain their inode, time, and
  permissions. Linked, foreign-owned, restricted, and executable destinations
  are replaced. Cold and non-Unix publication use atomic replacement.
- Wasm and transactional publication share destination ownership checks and a
  private file-digest representation. Public artifacts can still reconstruct
  missing or invalid exact entries when no consumer retains them. Full stamp
  validation, source guards, producer locks, prepared-reader invalidation, and
  copying every output before publishing public stamps remain enforced.
- Recovery from mutable public files hashes the newly copied private files before
  stamping them. Empty copies fail acquisition and remove the incomplete entry.

The conflicting-public-bytes regression fails against released `0.14.8` and
passes with this fix. Focused checks cover warm repairs, ownership, timestamps,
partial-copy failure, reconstruction, retained corruption, prepared readers,
changed and empty public recovery sources, source races, and process handoffs.
Callers remain responsible for coordinating other writers to public paths and
treating retained paths as read-only. No runtime or downstream suite speedup has
been measured for these changes.
Forty-five targeted unit checks and ten integration checks pass, along with
Clippy, formatting, and diff checks. Full pre-push validation remains
maintainer-owned.

## 0.14.8

This patch release avoids replacing already-correct public artifact outputs on
Unix cache hits, improves streamed hashing of large files, and gives artifact
builders ownership of label ordering. Public APIs and
persisted layouts are unchanged; existing `0.14` callers need no source or cache
migration. Repository-owned format
identifiers remain `v1`.

- Cache-entry validation returns its already computed output lengths and
  digests for destination verification. Matching regular files owned by the
  effective user, with one link, owner read/write permissions, and no executable
  or special permission bits remain in place. Their inode, modification time,
  and permissions are retained; callers must not rely on every cache hit
  refreshing the modification time.
- Missing, changed, linked, foreign-owned, and restricted destinations still
  receive atomic replacement. Cold commits and non-Unix hosts retain their
  existing copying behavior. Complete schema and content checks, input
  revalidation, lock ordering, retention, and the responsibility to coordinate other destination
  writers are preserved.
- Streamed file hashing uses a heap buffer sized to the opened file's length,
  between one byte and 64 KiB. Small sources avoid full-size buffers, and empty
  files retain a nonempty read buffer so growth cannot pass unnoticed. Growth
  beyond the declared length fails immediately; short reads at EOF remain
  invalid. Digest framing, input identities, and the thread-stack budget are
  preserved. An earlier fixed-buffer probe measured about 4–5% less hashing
  time for 2.60/17.60 MiB files and 0.4–0.5 microseconds more for small files.
  Length-sized-buffer follow-up timings varied under high host load, so no
  further speedup is claimed.
- Artifact-spec builders canonicalize identity and Cargo input label order,
  just as they already canonicalize output names. Key calculation reads that
  order directly instead of allocating and sorting temporary reference lists on
  each source check. Reordered declarations now also compare equal as specs;
  key bytes, duplicate-label rejection, and Cargo input guards are preserved.
  No performance gain is claimed for this small ownership cleanup.
- Paired release-profile measurements found matching-output acquisition about
  33% faster for 2.60 MiB artifacts and 24% faster for 17.60 MiB artifacts.
  Same-size changed outputs were 10–13% slower because they require verification
  before copying; missing and different-size repairs stayed within about 3% of
  baseline. These controlled component measurements do not establish downstream
  suite speedups. The repository performance guide records the methodology and
  the released `0.14.7` fixture and artifact baselines. These measurements precede
  the small-file buffer-sizing and label-ordering follow-ups.

Thirty-eight targeted unit and integration checks pass, including public file
identity and timestamp preservation, missing/changed/empty outputs, detachment
of matching symlinks and hard links, restricted permissions, cold publication,
cache corruption, Cargo input guards, process coordination, batch reuse, and
retention during pruning, native digest identities, exclusion semantics, and
streaming across empty/small files, buffer boundaries, and a partial final chunk.
Existing transaction tests also cover
reordered identity/Cargo declarations and duplicate Cargo labels rejected before
cache initialization. A read-only local probe also confirmed
foreign-owned matching files take the replacement path; it is not a
host-file-dependent CI test. Another read-only local probe verified rejection
when a zero-length file yields bytes. The earlier 1,440-acquisition paired experiment reused the
expected key and returned identical public and retained bytes. Fifty hashing
batches covered 302,000 operations, with every final digest matching the baseline.
A further 75 batches covered 453,000 operations across the released, fixed-buffer,
and length-sized-buffer implementations with matching final digests; timings are
excluded from performance claims because of host-load variance.
Targeted Clippy, formatting, and diff checks pass; full pre-push validation
remains maintainer-owned.

## 0.14.7

This patch release avoids Cargo metadata copies and repeated validation of
baseline receipt IDs, simplifies artifact schema checks, and gives executable
test fixtures one shared owner. Public APIs and persisted layouts are unchanged;
existing `0.14` callers need no source migration. Repository-owned format
identifiers remain `v1`.

- Cargo metadata indexes, dependency traversal, and semantic projection borrow
  identifiers and strings from the parsed document. Package selection no
  longer collects a temporary match list. Feature isolation, malformed-input
  diagnostics, conservative projection fallback, source guards, and exact
  fingerprints are preserved.
- Baseline-derived restore receipts reuse snapshot capture's deterministic
  ordering and duplicate validation without rebuilding a set. Empty receipt
  sets remain invalid; caller-supplied IDs still receive full validation.
- Artifact validation checks canonical output names against the declared count
  without rebuilding expected filename sets. Root-entry checks borrow allowed
  names. Strict schema and filename rejection, regular-file checks, failure
  diagnostics, filesystem errors, and persisted filenames are preserved.
- Unit and integration tests use one executable-fixture writer. Child-process
  writing and waiting prevent inherited writable script handles from causing
  intermittent Linux "Text file busy" launches in integration fixtures too.
  Real Cargo wrapper coverage and production command execution are unchanged.

Twenty-two targeted unit checks, ten artifact/Wasm integration checks, and six
live PocketIC 16 baseline-pool checks pass. Coverage includes concurrent prepared
readers, source-race invalidation, cross-process artifact retention and exact
build coordination, staged and cached schema rejection, native non-UTF-8 names
on Unix, receipt validation and recovery, and 100 consecutive restores without
reconstruction.
Targeted Clippy, formatting, diff, and source-package inclusion checks also pass.
No downstream suite speedup is claimed; full pre-push validation remains
maintainer-owned.

## 0.14.6

This patch release shares immutable Cargo input lists, simplifies batch and
artifact hashing ownership, and makes fixture-pool states explicit. Public APIs
and persisted layouts are unchanged; existing `0.14` callers need no source
migration. Repository-owned format identifiers remain `v1`.

- Cargo input snapshot clones share immutable input and exclusion lists while
  retaining independent timing values. Sessions no longer repeat an already
  established timing reset. Source revalidation and prepared-reader lease
  invalidation retain their behavior.
- Wasm batches borrow group membership and filter pending entries without
  temporary index lists or a copied workspace path. Environment grouping,
  feature isolation, indexed failures, and progress reporting are preserved.
- Labeled-path hashing owns the single collection needed for deterministic
  sorting. Artifact transactions borrow filesystem paths, and watched ICP
  inputs borrow labels. Native path ordering, exclusions, digest framing, and
  cache identities are unchanged.
- Input and tool label validation shares one rule with separate namespaces.
  Duplicates within a namespace are rejected before cache acquisition; an
  input and a tool may still have the same label.
- Bounded pools replace validity flags with explicit slot states. Unwind
  recovery preserves both its cause and any retained value, including panics
  before population. Safe teardown, FIFO scheduling, cancellation, capacity,
  explicit invalidation, and restore-failure recovery remain intact.
- Executable unit-test fixtures are written in a child process and awaited
  before use. Parallel test subprocesses cannot inherit a writable script
  handle, avoiding intermittent Linux "Text file busy" launches while retaining
  real Cargo wrapper coverage. Production command execution is unchanged.

Thirty-one targeted unit checks, seven artifact/Wasm integration checks, and
ten live PocketIC 16 checks pass. Coverage includes concurrent prepared readers,
source-race invalidation, cross-process artifact coordination, pool scheduling,
panic and restore-failure recovery, and 100 consecutive restores without
reconstruction. Targeted Clippy, formatting, and diff checks also pass. No
downstream suite speedup is claimed; full pre-push validation remains
maintainer-owned.

## 0.14.5

This patch release consolidates Cargo input safety checks and simplifies warm
baseline reuse, temporary Wasm specification ownership, diagnostic state, and
benchmark discovery. Public APIs and persisted layouts are unchanged; existing
`0.14` callers need no source migration. Repository-owned format identifiers
remain `v1`.

- Wasm batches borrow caller specifications and grouping paths instead of
  copying them for input resolution. Sessions and prepared snapshots still own
  identities retained across calls. Feature isolation, metadata/environment
  grouping, concurrent prepared readers, and source-lease invalidation retain
  their existing behavior.
- Standalone builds and batches share input discovery, shared-target boundary
  validation, and generated-directory exclusions. Checks still precede hashing,
  maintenance, and builds; unsafe entries preserve their input-discovery errors
  without blocking compatible batch entries or deleting source files.
- Warm baseline reuse compares captured canister IDs directly with restore
  receipts, allocating complete diagnostic ID lists only on mismatch.
  Successful preparation leaves lifecycle metadata unchanged instead of
  clearing an already-empty invalidation reason. Reset coverage, readiness,
  validation, explicit invalidation, and unwind recovery retain their behavior.
- Diagnostic log rendering derives omitted-record counts and accumulates raw
  byte totals in one traversal. Empty records, independent record/byte bounds,
  lossy UTF-8 handling, compact truncation text, and failure reporting are
  preserved.
- Previous benchmark-run discovery selects the latest eligible match without
  collecting and sorting all candidates. Metadata timestamp priority, numeric
  indices, command filtering, and skipping unreadable or malformed metadata
  remain unchanged.

Sixteen batch/input-resolution unit checks, four real Wasm build checks, five
diagnostics unit checks, three benchmark checks, and nine live PocketIC 16
checks pass. Baseline-pool coverage includes 100 consecutive restores without
reconstruction, receipt
mismatch recovery, explicit invalidation, failed recovery diagnostics, and
caller and recipe-hook panics. Benchmark coverage includes metadata schema,
timestamp priority, numeric indices, command filtering, and missing or malformed
candidates. Targeted Clippy, formatting, and diff checks also pass. No downstream
suite speedup is claimed; full pre-push validation remains maintainer-owned.

## 0.14.4

This patch release consolidates snapshot preparation and removes duplicated Wasm
input state. Public APIs and persisted layouts are unchanged; existing `0.14`
callers need no source migration. Repository-owned format identifiers remain
`v1`.

- Both snapshot capture APIs share complete duplicate validation and
  deterministic ordering before issuing management calls. Temporary sender
  vectors are removed while preserving explicit-sender and controller-fallback
  contracts.
- Partial-capture rollback consumes its snapshot set instead of copying IDs.
  Cleanup continues across failures and retains rejection and panic diagnostics.
- Wasm semantic hashing derives its path subset from the authoritative
  validation list. A duplicated path list and fingerprint-mode wrapper are
  removed; conservative fallback and workspace projection semantics remain.
- Cargo input revalidation borrows existing labels and paths rather than
  reconstructing owned copies. Raw-source mutation guards, exclusions, native
  path ordering, semantic digests, and exact cache identities are unchanged.

Seven focused live PocketIC 16 checks, one funding-policy unit check, six digest
checks, and seven artifact-input checks pass. Public snapshot behavior tests
replace private validation-helper tests. Targeted Clippy, formatting, and diff
checks also pass. No downstream suite speedup is claimed; full pre-push
validation remains maintainer-owned.

## 0.14.3

This patch release simplifies benchmark processing and Wasm cache finalization.
Public APIs and persisted layouts are unchanged; existing `0.14` callers need
no source migration. Repository-owned format identifiers remain `v1`.

- Benchmark pairing borrows markers until constructing owned spans or
  diagnostics. Aggregation stores group identity only in map keys, and
  comparison and Markdown lookups borrow existing row keys. Nested pairing,
  suite boundaries, deterministic ordering, overflow rejection, missing rows,
  and duplicate-row precedence retain their existing behavior.
- Cold builds and missing exact-cache reconstruction use one success/failure
  finalization path. Successful entries survive, incomplete entries are cleaned
  up, and original errors, cleanup diagnostics, and timings are preserved.
- Canister installation moves the existing label into failure diagnostics
  instead of cloning it before every install. Original causes and caller-owned
  PocketIC instances remain available after failures.
- Digest text has one hexadecimal formatter that writes directly to its
  destination. The owned-string API and persisted stamps, manifests, and cache
  paths keep the same lowercase, zero-padded representation.

Twenty-five benchmark integration checks, one overflow check, three Wasm
cleanup/reconstruction checks, four artifact handoff checks, two live PocketIC
16 install checks, six digest checks, and two stamp/manifest checks pass.
Targeted Clippy, formatting, and Wasm compile checks also pass. No downstream
suite speedup is claimed; full pre-push validation remains maintainer-owned.

## 0.14.2

This patch release simplifies fixture and artifact ownership and adds opt-in
fixture performance measurements. Public library APIs and persisted layouts are
unchanged; existing `0.14` callers need no source migration. Repository-owned
format identifiers remain `v1`.

- Standalone fixture pools derive rebuild reasons from shared slot state,
  removing duplicated invalidation metadata while preserving restore-failure
  rebuilding and distinct panic recovery outcomes.
- Wasm build records expose the cache path held by their retention owner instead
  of allocating a second copy. Cloned records continue retaining immutable
  artifacts until their last drop.
- The repository includes a Linux benchmark comparing fresh fixtures with
  baseline pools of capacity one and two using a caller-supplied PocketIC 16
  binary. It validates every task and reports phase timings, setup/teardown,
  sampled process-tree RSS, raw samples, and provenance. The workload and
  measured capacity tradeoffs are documented in the repository's
  [fixture benchmark guide](https://github.com/dragginzgame/ic-testkit/blob/main/docs/fixture-reuse-benchmark.md).
  This tooling is outside ordinary tests/CI.
- Observed Cargo subprocess tests read their fixtures through the existing
  system shell, avoiding intermittent Linux "Text file busy" launch failures
  under parallel load. Output forwarding, captured diagnostics, exit events,
  and host build-progress notifications retain real subprocess coverage;
  production Cargo execution is unchanged.

Seven targeted PocketIC checks and four artifact handoff checks pass, covering
reuse, capacity, restore failures, panic recovery, cloned records, cross-process
retention, pruning, and terminated consumers. Two Python sampler checks,
targeted Clippy, and formatting also pass. The measurements compare fixture
strategies; they do not establish a library-version or downstream suite speedup.
Five focused output/progress checks pass, along with 500 parallel repetitions
of the two affected subprocess tests (1,000 test executions).

## 0.14.1

This patch release reduces artifact acquisition and fixture-pool overhead without
changing public APIs or persisted layouts. Existing `0.14.0` callers need no
source migration. Repository-owned cache, stamp, and digest identifiers remain
`v1`.

- Wasm stamps and transactional manifests reject mismatched format/build
  identities before hashing outputs. Matching entries still require full content
  validation and exact metadata; live retained entries cannot be replaced during
  corruption recovery.
- Input hashing borrows declared paths and Unix-native filename bytes, and
  computes directory sort keys once per entry. Native ordering and Windows
  UTF-16 little-endian encoding retain their existing digest semantics.
- Digest-cache hits compare borrowed exclusion paths without reconstructing
  cloned lists. Changes to relevant exclusions still rehash inputs, excluded
  input roots are rejected, and external symlinks retain conservative checks.
- Cache size scans queue only directories while preserving logical file sizes
  and counting symlink bytes without following their targets.
- Bounded fixture pools register and claim FIFO slots under one coordinator
  lock, with one optional cancellation ticket and safe unwind cleanup. Capacity,
  waiter ordering, cancellation wakeups, and invalidation remain unchanged.

Targeted checks cover native filenames, exact digests and stamps, corruption
recovery, retained consumers, pruning, cross-process handoff, FIFO cancellation,
and PocketIC 16 reuse with 100 consecutive baseline restores. Whole-suite
performance improvements are not yet measured.

## 0.14.0

Standalone fixture pools own one builder, removing the per-acquisition builder
that warm slots ignored. This is a source API hard cut. Snapshot funding,
capacity, restoration, recovery, guard types, and persisted formats are unchanged.
Repository-owned format identifiers remain `v1`.

### Migration

| Previous usage | Replacement |
| --- | --- |
| `CachedStandaloneCanisterFixturePool::<N>::new()` or `default()` | `CachedStandaloneCanisterFixturePool::<N>::new(build_fixture)` |
| `pool.acquire(build_fixture)` | `pool.acquire()` |
| Pass a closure capturing configuration to every acquisition | Own it once with `CachedStandaloneCanisterFixturePool::<N, _>::new(move || build_fixture(&config))`. |

Static pools can use a function pointer or a noncapturing closure:

```rust
use ic_testkit::pic::{CachedStandaloneCanisterFixturePool, StandaloneCanisterFixture};

static POOL: CachedStandaloneCanisterFixturePool<8> =
    CachedStandaloneCanisterFixturePool::<8>::new(build_fixture);

fn build_fixture() -> StandaloneCanisterFixture {
    // Install and seed the canister here.
    todo!()
}

let (fixture, outcome) = POOL.acquire()?;
```

The builder must produce the same Wasm, initialization, topology, and seeded
state each time it runs. Use separate pools for different recipes. Cold and
replacement slots invoke the owned builder; warm acquisitions restore the
captured snapshot. Apply the constructor and acquisition changes together when
downstream suites adopt `0.14`.

For statics that chain `with_restore_funding`, specify the constructor capacity
as above so Rust selects the default function-pointer builder type before
applying the funding policy.

### Implementation simplification

Wasm batches parse Cargo package, membership, and dependency indexes once per
resolution group. Each specification still selects its own dependency closure
and validates its filesystem inputs; differing features remain separate groups.
Standalone Wasm builds use the same parsed representation.

CI runs the canister integration target through the ordinary test stage, removing
its separate repeat and unused preliminary fixture build. `make test-canisters`
remains a focused entry point whose tests acquire their own artifacts;
`make build-test-canisters` remains available for manual builds.

Release-push guards exercise clean, dirty, untracked, stale-tag, and failed-push
behavior using harmless command doubles, replacing the exact recipe-text check.

## 0.13.0

This release gives benchmark identity and averages one authoritative
representation and verifies canister restore evidence without copying it into
non-snapshot reset receipts. The source API changes are hard cuts. Report schemas
and persisted cache layouts are unchanged; cache, stamp, protocol, and digest
identifiers remain `v1`.

### Migration

| Previous usage | Replacement |
| --- | --- |
| `BenchmarkAggregateRow::suite`, `BenchmarkComparisonRow::suite`, or `BenchmarkAggregateError::suite` field | Call `suite()`. The label derives from the private scope; use `is_all_suites()` to distinguish an authored `ALL` suite from the cross-suite aggregate. |
| Read or assign `BenchmarkAggregateRow::average` | Read `average()`. Averages derive from `total` and `runs`; there is no separately writable average. |
| `ResetRequirements::try_new([CanisterSnapshots, CanisterCycles(policy), ...])` | `ResetRequirements::try_new(policy, [...])`, passing only non-snapshot requirements in the collection. Read the explicit cycle policy with `cycle_policy()`; `get()` and `iter()` cover non-snapshot domains. |
| Snapshot/cycle variants of `ResetDomainKind`, `ResetRequirement`, or `ResetAchievement` | Snapshot restoration is unconditional. Report restored canisters and achieved cycle policy in `CanisterRestoreReceipt`; report other guarantees in `ResetReceipt`. |
| Inspect snapshot/cycle achievements in `PreparedBaseline::Restored::reset` | Inspect its `canisters` receipt with `canister_ids()` and `cycle_policy()`. The `reset` receipt contains only non-snapshot achievements. |
| Match cycle failures through `ResetPolicyMismatch` | Match `CyclePolicyMismatch { required, achieved }`. `ResetPolicyMismatch` still reports non-snapshot policy mismatches. |
| Match `UndeclaredRequiredResetDomain` | Remove this constructor-error branch. The required cycle policy is a constructor argument, and complete snapshot restoration is enforced during preparation. |

For example:

```rust
use ic_testkit::pic::{
    CycleResetPolicy, ResetRequirement, ResetRequirements, TimeResetPolicy,
};

let requirements = ResetRequirements::try_new(
    CycleResetPolicy::PreserveCurrent,
    [ResetRequirement::PocketIcTime(TimeResetPolicy::PreserveCurrent)],
)?;
```

Update downstream recipes when adopting this release. Restore, non-snapshot
reset, readiness, and final validation keep their ordering. Canister-set and
cycle-policy mismatches retain the `ResetCoverageMismatch` rebuild reason;
recoverable preparation failures still permit one rebuild, and failed recovery
retains both failures.

### Simplification and verification

The observed Cargo output/heartbeat test keeps its fixture running until the
observer receives a heartbeat, with a bounded timeout. This removes a scheduling
race caused by a fixed sleep under parallel test load; runtime heartbeat behavior
is unchanged.

Benchmark labels and comparison keys derive from one scope. Report writers and
comparisons calculate averages from totals and runs, preserving CSV columns and
named-`ALL` identity. Arithmetic overflow checks remain in place.

Wasm batch acquisition and reporting use one failure-details representation,
owned alongside each error. Existing public result/accessor and `into_parts()`
signatures, partial timings, captured output, entry order, and retained successful
records are preserved.

Targeted checks cover benchmark updates and report schemas, restored canister-set
and cycle-policy mismatches, recovery and panic invalidation, 100 consecutive
restores, mixed Wasm batch results, captured Cargo diagnostics, retained-output
handoff, and concurrent prepared readers. PocketIC 16 baseline reuse and isolated
dead-server recovery, Clippy, rustdoc, formatting, and Wasm compilation pass.

## 0.12.0

This update tightens artifact path boundaries, fixes Unix managed-server
descendant cleanup, and rejects benchmark aggregate overflow. Cache, stamp,
protocol and digest identifiers remain `v1`.

### Migration

These source API and validation changes are hard cuts:

| Previous usage | Replacement |
| --- | --- |
| `aggregate_benchmark_spans(...) -> BenchmarkAggregateReport` | `Result<BenchmarkAggregateReport, BenchmarkAggregateError>`; handle or propagate overflow before comparing aggregates or writing reports. |
| Override `CARGO_TARGET_DIR` in `with_extra_env` or pass `--target-dir` in `with_cargo_profile_args` | Select the exact target with `WasmBuildSpec::new` or the shared target with `with_shared_incremental_target`; command overrides return `InvalidSpec` before acquisition. |
| Configure one public artifact output beneath another | Use distinct, non-nested output destinations; overlapping destinations are rejected during preparation. |

### Fixes and simplification

Observed Cargo output capture retries interrupted pipe reads, preserving captured
bytes while propagating permanent read errors.

Compact Cargo feature arguments (`-Fextra` and `-F=extra`) now reach metadata
resolution as well as compilation. Optional dependencies enabled by these
arguments are watched inputs, and batches resolve distinct feature graphs
separately.

Benchmark aggregation rejects counter and run-count overflow with a typed
error identifying the scope, span and counter. No partial, wrapped or saturated
totals are returned. `is_all_suites()` distinguishes a cross-suite failure from
a named suite `ALL`.

Shared-target maintenance now rejects layouts that could delete retained exact
Wasm artifacts. Batches maintain each resolved target directory once, including
workspace-relative paths and filesystem aliases. Relative exact targets use the
caller's working directory consistently for Cargo output and cache operations.
Conflicting `CARGO_TARGET_DIR` environment or `--target-dir` command overrides
are rejected before acquisition; select target directories through the build spec.

Artifact preparation rejects nested file destinations. Output boundary checks
validate the directory entry replaced by atomic publication rather than a final
symlink's referent; declared input and tool symlink entries remain protected.

On Unix, managed-server teardown terminates descendants in its owned process
group on handle drop, startup failure, and natural server exit. Cleanup reserves
the leader's PID until after signaling the group, then reaps the child.

Benchmark metadata now derives its JSON fields from `BenchmarkRunMetadata`.
Existing JSON field names, object shape, field types, optional fields and integer
bounds are preserved; invalid metadata returns `InvalidData` with Serde field
diagnostics.
Wasm warm hits share input validation and retained-record completion, and batch
input reuse has one internal representation for sessions and prepared snapshots.
Batch entry points call the same runner directly, and transport errors and panic
payloads share one message classifier.
Batch maintenance-policy errors use the common report-entry construction while
retaining their failure phases and timings. Internal attempts store the phase
and timings together; standalone and batch resolution share toolchain
identification.
ICP readiness delegates file validation to the watched-input snapshot.
Background server reaping retains the original managed-server owner and its
startup files. Shared-target lock acquisition owns progress events for both
ordinary builds and scheduled maintenance.

### Verification and documentation

Targeted regressions cover retained-cache and output boundaries, relative target
paths, Cargo override rejection, benchmark overflow and metadata validation,
compact-feature dependency discovery and batch grouping, and observed
session/prepared-reader batches. Unix descendant cleanup is checked
on drop, startup timeout, natural exit and background reaping, with a real
PocketIC 16 startup/reaping check. Maintenance behavior tests use outcome and
filesystem assertions, with a positive-interval control for heartbeat validation.

Installation examples select `0.11`. The README documents the updated path and
cleanup rules and explains recipe-pool timings for profiling long suites.
`POCKET-IC.md` refreshes upstream tracking.

## 0.11.0

This minor release fixes cache-hit input races and batch failure isolation,
consolidates baseline reuse and installation APIs, and makes operation failures
and benchmark aggregate identity explicit. The API and CSV schema changes are
hard cuts; cache, stamp, protocol, and digest identifiers remain `v1`.

### Migration

Update downstream code for these API and report-schema changes:

| Previous API or schema | 0.11.0 replacement |
| --- | --- |
| `restore_or_rebuild_cached_pocket_ic_baseline` and `CachedPocketIcBaselineGuard` | `CachedPocketIcBaselinePool::new(capacity, recipe)` and `acquire()`. Use capacity one for sequential reuse; acquisition returns a typed outcome and an exclusive lease. |
| `create_and_install_with_args` and `try_create_and_install_with_args` | `create_and_install(InstallSpec::new(...))` and `try_create_and_install(InstallSpec::new(...))` |
| `CanisterInstallError::canister_id() -> Principal` | `Option<Principal>`; creation failures have no id. Inspect `phase()` for `CreateCanister`, `AddCycles`, or `InstallCode`. |
| `CanisterInstallError::new(id, message)` / `labeled(...)` | `new(phase, optional_id, optional_label, PocketIcOperationError)` |
| Snapshot panic variants' `message` field | `source: PocketIcOperationError`; inspect `message()` and `is_transport()` on the cause, or follow `Error::source()`. |
| `WasmBuildError::InputsChangedDuringBuild` and `ArtifactCacheError::InputsChangedDuringBuild` | `InputsChangedDuringAcquisition` |
| `comparison.csv` without aggregate scope | Leading `scope` column, containing `suite` or `all`. Named suite `ALL` and the cross-suite aggregate are distinct. |

`CachedPocketIcBaseline<T>` remains the owned snapshot-and-metadata value used
by recipe pools. A recipe declares its restore/reset/readiness/validation
contract and recovery policy; pool leases prevent another acquisition from
replacing or mutating the slot during recovery.

Fallible installation now captures upstream failures during creation and cycle
funding as well as installation. Standalone errors retain the caller's PocketIC
instance at every failed stage. Snapshot and installation errors retain a shared
operation cause so `is_dead_pocket_ic_transport_error` can classify it through
contextual wrappers without broadening the strict transport parser.

Ordinary warm Wasm acquisitions reject inputs that change after initial
resolution, including conservative workspace inputs. Immutable-source sessions
and prepared readers retain their explicit lease contract and skip repeated
warm validation. Batches report input hashing/discovery failures per entry and
continue with valid compatible entries. Input-race errors invalidate leased
readers as before.

Server diagnostic reads allocate only the bounded log prefix. Benchmark indices
continue past `9999`, previous-run selection compares them numerically, and
exhaustion returns an error rather than reusing an existing path. Empty commit
hashes consistently use the `unknown` directory prefix.

### Repository tooling and documentation

Release CI cleanup now matches the selected server binary's device/inode,
including renamed binaries, alongside its private port-file path. Unknown or
unavailable executable identities retain scratch without signalling unrelated
processes. Focused process/socket regressions cover configured and default
binary selection, identity failures and ownership races. This changes repository
release tooling; the runtime changes are described above.

README examples and API guidance are updated against the implementation, with
all 27 Rust examples checked. The documentation index separates current usage
from historical design records. Targeted regressions cover the cache, transport,
installation, output-read, and benchmark boundaries described above, alongside
existing live PocketIC recovery and concurrency checks.

## 0.10.4

The repository's release CI runner now stops invocation-owned PocketIC servers
before removing its temporary directory. This prevents server HTTP adapter
teardown from panicking on sockets already deleted by release cleanup. The
published crate's runtime API and dependency selection are unchanged.

The isolated PocketIC teardown patch and development probe are removed.
Instance teardown improvements will wait for a future upstream release; the
repository does not maintain a patched PocketIC client. Seven targeted
process/socket regressions cover the repository's release cleanup behavior.
Fixture sockets use short relative bind paths to support nested release
temporary directories without exceeding the Unix socket pathname limit.

## 0.10.3

The repository includes an isolated PocketIC 16.0.0 upstream teardown proposal
and repeatable synthetic HTTP probe. Seven targeted parent tests qualify
fallible shutdown deadlines, acknowledgement checks, ownership retained for
retry, bounded best-effort drop and borrowed gateway cleanup.

This is a development experiment. The published crate still uses the registry
PocketIC dependency and adds no production shutdown API or teardown fix.
The original Busy/tick cause remains unproven. The experiment was subsequently
removed in 0.10.4 in favor of waiting for an upstream release.

## 0.10.2

The repository and CI now use Rust 1.99.0. The published MSRV remains Rust
1.88.0.

Transport classification now requires a maintained reqwest error shape with a
PocketIC instance URL and recognized transport source, or a structured testkit
call transport kind. Generic `channel closed` / `ConnectionRefused` application
text, quoted error variants and bare I/O errors do not qualify. Use the public
classifier only for PocketIC-originating errors; it remains a heuristic rather
than proof of a dead instance. Unrelated call panics retain their original
payload.

Empty/nonempty collection assertions comply with Rust 1.99's `assert_is_empty`
lint and show collection values on failure.

Heartbeat tests now use event coordination. A synthetic HTTP/subprocess test
demonstrates that PocketIC 16's instance destructor waits for DELETE, independently
of the construction deadline and operation budget. This records an upstream
limitation; no bounded teardown API or simulator wrapper is added.

## 0.10.1

`PocketIcManagedServer::process_id()` exposes the owned server child's OS PID
for caller-managed resource monitoring alongside its URL and captured output.
It identifies only the child, does not establish liveness, and retains no
ownership when copied. The OS may reuse it after the child exits and is reaped.

## 0.10.0

This minor release hard-cuts artifact consumption to retained exact outputs,
fixing the lifetime gaps reported by IcyDB in issue #2. The original missing
input's precise racing operation has not been established.

`WasmBuildRecord::artifacts()` and `ArtifactCacheRecord::artifacts()` now return
read-only exact-cache paths instead of caller-selected materialization paths.
A successful cold build or warm hit acquires retention before releasing its
producer locks. Keep its record, outcome, or batch report alive while consuming
those paths. Cloning a record shares retention; cloning a path or an
`ArtifactCacheArtifact` descriptor does not. `exact_cache_path()` is also
protected for the Wasm record's lifetime.

Age/size pruning skips live entries across threads and processes. Configured
bounds may be exceeded while consumers retain entries; after the last owner
drops, the next maintenance pass can reclaim them normally. OS locks release
on process exit, including a crash, without stale pins. A corrupt retained
entry fails closed instead of being replaced. Treat exact paths as read-only;
manual cache deletion and external mutation are outside the ownership contract.
All cache and digest formats remain `v1`.

### Hard-cut migration

| Previous consumption | 0.10 consumption |
| --- | --- |
| Read the configured compiler output after a build | Read `outcome.record().artifacts()` and keep the outcome or a cloned record alive through post-link commit |
| Read the configured post-link destination later | Keep the returned `ArtifactCacheRecord` and read its artifacts until staging/reading finishes |
| Reduce successful batch results to indexes or paths | Keep the report, move out successful outcomes, or clone their records; later failed entries do not invalidate successful records |
| Transform an artifact in place | Write into the post-link transaction's output staging paths |

Materialization still populates configured destinations, but those remain
mutable and may be replaced by another acquisition. No compatibility accessor
or alternate cache protocol is added. Declared-input/tool hashing errors now
include the failing path. Source-mutation and input-identity checks remain in
force.

The `artifacts` module documentation demonstrates retained Wasm-to-post-link
handoff. IcyDB must adopt the release in both single and batch flows and rerun
its concurrent lifecycle tests; that downstream validation is not claimed here.

## 0.9.1

The workspace `toml` dependency moves from 0.9 to 1, with the refreshed
lockfile resolving `toml` to 1.1.6.

Release CI no longer runs `cargo clean` after a successful gate. Cargo build
artifacts are preserved for incremental reuse after success as well as for
diagnosis after failure; only the release wrapper's isolated temporary
directory is removed. Release-flow guards keep the standalone `make clean`
target outside CI and reject Cargo cleanup from CI, release, and publish
scripts.

## 0.9.0

The workspace now uses PocketIC 16. Managed servers started through
`PocketIcStartupConfig::spawn` no longer receive a ten-minute `--hard-ttl` by
default, matching the upstream lifetime policy. An active test suite is not
terminated merely because ten minutes have elapsed; PocketIC's activity-based
soft TTL still bounds an orphaned server, and a caller-owned
`PocketIcManagedServer` is still terminated and waited for on drop.

PocketIC 16 also waits for the first certified time update before returning
from automatic-progress startup, accounts for mocked HTTP response cycle spend,
rejects mocked HTTP reject messages larger than 1 KiB, and adds flexible HTTP
mocking plus the `SubnetCoolingDown` and `CanisterStatusAccessDenied` error
codes. These remain part of the direct upstream runtime surface; ic-testkit
does not add parallel wrappers for them.

The complete host-only upstream crate is now re-exported as
`ic_testkit::pocket_ic`. Use that path for native PocketIC types outside the
focused `ic_testkit::pic` convenience surface:

```rust
use ic_testkit::pocket_ic::{
    CanisterSettings, CreateCanisterParams, PocketIc,
    common::rest::{BlobCompression, IcpFeatures, IcpFeaturesConfig},
};
```

These are the types from the exact PocketIC dependency selected by ic-testkit;
there is no copied type or parallel wrapper. The complete re-export, like
`ic_testkit::pic`, is unavailable on `wasm32`.

Call `with_server_hard_ttl(duration)` when an absolute server deadline is
required. Subsecond explicit values remain invalid.

### Hard-cut migration

| 0.8 API | 0.9 API |
| --- | --- |
| `PocketIcStartupConfig::server_hard_ttl() -> Duration` returned the default ten-minute hard TTL | `server_hard_ttl() -> Option<Duration>` returns `None` by default and `Some(duration)` after `with_server_hard_ttl(duration)` |

There is no compatibility accessor or implicit ten-minute fallback. Startup
and instance-creation deadlines remain independently bounded by
`PocketIcStartupConfig::timeout`.

## 0.8.9

`WasmBuildInputSnapshot::prepare_assuming_sources_immutable` resolves a fixed
set of exact `WasmBuildSpec` values once under a caller-held source
write-exclusion guard. Its `build_batch` and `build_batch_with_progress`
methods take `&self`, so separate sequential batches may read the prepared
inputs concurrently. Reader metrics distinguish prepared reuse from ordinary
batch and mutable-session reuse.

Every reader specification must have been declared during preparation;
`SpecificationNotPrepared` rejects an undeclared entry before progress or build
work. A detected post-build input mutation invalidates the snapshot for every
later reader, and publication is coordinated with that shared invalidation
boundary. Ordinary batch calls remain independently resolved. Do not construct
a snapshot with an unrelated token: the borrowed value is a lifetime boundary,
not guard-provenance validation performed by `ic-testkit`.

## 0.8.8

`WasmBuildSession::new(&guard)` is hard-cut to
`WasmBuildSession::assume_sources_immutable(&guard)`. The constructor name now
makes the caller assertion explicit: `ic-testkit` lifetime-binds the session to
the supplied reference but cannot prove that the value is a genuine workspace
write-exclusion guard. An unrelated token still violates the contract and can
permit stale reuse. There is no `new` alias or deprecated bridge.

A concurrent-reader resolution snapshot remains a documented future design,
not an ambient cache or parallel batch mode. It would prepare a declared spec
set under one genuine source lease, freeze resolution state before sharing,
propagate invalidation to every reader, retain existing Cargo target locking,
and require consumer benchmarks before implementation.

## 0.8.7

Managed spawn now allocates stdout, stderr, and the server-owned port path
inside a unique private directory, but creates only the output files before
launch. The actual `--port-file` path remains absent until PocketIC publishes
it; a missing path is treated as pending readiness. This fixes PocketIC 15,
which exits successfully without starting when the supplied port path already
exists.

`PocketIcStartupConfig::start_managed_server` returns a caller-owned
`PocketIcManagedServer`. Its `url()` can feed any number of bounded
`PocketIcStartupConfig::connect` calls in a serial suite, `output()` returns
the first 16 KiB of lossy stdout/stderr per stream with an omitted-byte marker,
and dropping the handle terminates and waits for the managed child. Keep the
handle alive until its connected instances are dropped. This is explicit
process-local ownership rather than a process-global server or an implicit
retry path. CI spanning multiple Cargo or test-runner processes should retain
an external runner-owned server and use bounded connect mode in each process.

An ignored real-server regression test accepts the exact caller-resolved
binary through `IC_TESTKIT_POCKET_IC_SERVER`. It verifies port publication,
bounded instance construction, owned shutdown, and startup-directory cleanup
without adding binary discovery or download behavior to the crate.

`WasmBuildSession` is an explicit caller-owned cross-call input snapshot. Its
constructor borrows a source write-exclusion guard for the session lifetime;
the caller must ensure that all Cargo/rustc executables, manifests and Cargo
configuration, discovered sources, declared additional inputs, and relevant
environment values are immutable while the session exists. Exact resolution
snapshots and content digests may then be reused by separate sequential
`build_batch` or `build_batch_with_progress` calls. Ordinary batch functions
retain their current per-call validation, and there is no ambient or
process-global cache.

If revalidation around a Cargo build detects an input mutation, the session is
permanently invalidated, all pending pre-race snapshots are discarded, and a
later call returns `WasmBuildBatchContractError::SourceLeaseInvalidated`.
`WasmBuildSession::metrics` exposes retained snapshots, successful snapshot
reuses, and invalidation state; each batch separately reports
`input_resolution_session_reuses`.

Failed Wasm batch entries now expose `WasmBuildFailurePhase` and partial
`WasmBuildFailureTimings` through `WasmBuildBatchFailure::phase` and `timings`.
The timings retain completed exact/shared coordination, tool identity, Cargo
metadata, input discovery, content hashing, shared maintenance, Cargo,
publication, exact-cache maintenance, explicit cleanup, and total wall time.
Successful build-record timing remains unchanged.

### Hard-cut migration

| 0.8.6 API | 0.8.7 API |
| --- | --- |
| `WasmBuildBatchEntry::into_parts() -> (usize, String, Result<_, _>, Duration)` | Destructure `(index, label, result, failure_details, entry_elapsed)`; failed entries carry `Some(WasmBuildFailureDetails)` |
| `WasmBuildBatchFailure` exposes only label/index/error/elapsed | Use `phase()` and `timings()` for the primary failed phase and partial work |
| Separate batch calls always resolve their inputs independently | Hold the real source write-exclusion guard and call `WasmBuildSession::assume_sources_immutable(&guard)` when the immutable-source contract can be guaranteed |

There is no four-field `into_parts` alias, deprecated session-free overload,
implicit guard, global cache, or reset-after-race shim. Batches remain
sequential and collect-all; recipe and observer panics continue unwinding.

## 0.8.6

Wasm batches now require `LabeledWasmBuildSpec`. Labels must be nonempty and
unique and are retained in canonical report entries, successful outcomes,
failures, progress events, and shared-target maintenance outcomes. Label
preflight completes before metadata resolution, progress, or build work;
labels do not alter exact Wasm fingerprints. Valid entries remain sequential
and collect-all.

Diagnostic batch labels now follow the same contract. Empty or duplicate
labels return `CanisterDiagnosticsBatchContractError` before any status or log
call starts. Valid targets retain their exact controllers and continue after
independent failures as before.

`PocketIcBuilderExt::try_build` now requires an explicit
`PocketIcStartupConfig`. `spawn` launches one exact caller-resolved binary,
detects child exit while waiting for readiness or instance construction,
terminates the child at the complete startup deadline, and returns structured
errors with bounded lossy stdout/stderr. `connect` applies the same construction
deadline to a caller-owned existing server. Both policies force an explicit
server URL onto the upstream builder, so it cannot spawn an unobservable child.
ic-testkit does not discover, download, cache, or compatibility-check server
binaries.

Exact Cargo Wasm identity now uses a validated semantic workspace projection.
The projection retains selected resolve nodes, enabled features, exact external
source/checksum/revision identity, effective package fields, workspace
profiles/resolver/lints, selected local sources, tools, Cargo configuration,
and declared inputs. An unrelated host-only workspace dependency or lockfile
update can therefore reuse the same selected Wasm entry.

The complete workspace manifest and lockfile remain conservative validation
inputs. `ResolvedCargoBuildInputs::validation_digest` exposes that raw mutation
guard; `input_digest` is now semantic cache identity. Cargo builds and attached
artifact transactions reject any raw input change during their operation.
Workspace-root packages and local packages outside the normalizable workspace
boundary fall back to the complete input identity.

### Hard-cut migration

| 0.8.5 API | 0.8.6 API |
| --- | --- |
| Wasm batch functions accept `&[WasmBuildSpec]` and return `WasmBuildBatchReport` | Wrap every spec with `LabeledWasmBuildSpec`; handle `Result<WasmBuildBatchReport, WasmBuildBatchContractError>` |
| `results()`, `into_results()`, and parallel `entry_elapsed()` access | Use canonical `entries()` or `into_entries()`; each `WasmBuildBatchEntry` owns index, label, result, and elapsed time |
| Wasm `outcomes()` and shared maintenance yield indexed tuples; failures have no label | Use the structured entry accessors `index()`, `label()`, `outcome()` or `error()`, and `entry_elapsed()` where available |
| Batch progress variants contain only an index | Match the required `label` field as well, or use `..` when the label is intentionally ignored |
| Diagnostics batch returns `CanisterDiagnosticsBatchReport` directly | Handle `Result<CanisterDiagnosticsBatchReport, CanisterDiagnosticsBatchContractError>`; labels must be nonempty and unique |
| Configure `with_server_binary`/`with_server_url`, then call `try_build()` | Call `try_build(PocketIcStartupConfig::spawn(path, timeout))` or `try_build(PocketIcStartupConfig::connect(url, timeout))` |
| Read `PocketIcStartupError::message()` | Match the structured `PocketIcStartupError` variants and their public fields |

No index-only overloads, parallel label slices, deprecated report accessors,
zero-argument `try_build`, implicit binary fallback, or compatibility aliases
are retained.

### Semantic identity migration

- Treat `ResolvedCargoBuildInputs::input_digest` and
  `WasmBuildRecord::input_digest` as semantic selected-build identity. Use the
  new `validation_digest` when retaining or comparing a conservative raw input
  snapshot.
- Expect one new exact key for workspaces eligible for projection. Repository
  digest domains and cache formats remain `v1`; no legacy key lookup, shim,
  alias, or dual reader is retained.
- Continue declaring build-script, procedural-macro, source-include, or tool
  inputs that live outside Cargo's selected package graph.

## 0.8.5

`0.8.5` hard-cuts generic artifact batches to caller-labeled specifications.
`LabeledArtifactCacheSpec` labels must be nonempty and unique; they are retained
in cache-miss callbacks and every ordered report entry. Labels are report and
composition identity only and do not alter exact artifact-cache keys. Invalid
label structure returns `ArtifactCacheBatchContractError` before any entry
starts.

`ArtifactCacheBatchFailureTimings` distinguishes preparation, callback,
explicit abort cleanup, commit, and total time. The failure also exposes its
primary `ArtifactCacheBatchFailurePhase`. Commit timing includes cleanup owned
internally by a failed commit. Recipe panics still unwind, and independent
entries remain sequential and collect-all.

`CanisterDiagnosticsBatchEntry::entry_elapsed` retains each target's complete
diagnostic collection time, while `CanisterDiagnosticsBatchReport::total`
retains total sequential batch time. The compact renderer includes both. Exact
controllers, bounded logs, collect-all behavior, and the absence of anonymous
fallback remain unchanged.

### Hard-cut migration

| 0.8.4 API | 0.8.5 API |
| --- | --- |
| `build_artifact_caches_batch(&[ArtifactCacheSpec], FnMut(usize, ...)) -> ArtifactCacheBatchReport<E>` | Wrap specs with `LabeledArtifactCacheSpec`; the callback receives `&str`; handle `Result<ArtifactCacheBatchReport<E>, ArtifactCacheBatchContractError>` |
| `results()`, `into_results()`, and `entry_elapsed()` parallel report slices | `entries()` and `into_entries()` return canonical `ArtifactCacheBatchEntry<E>` values with `index()`, `label()`, `result()`, and `entry_elapsed()` |
| `outcomes()` yields `(usize, &ArtifactCacheOutcome)` | Yields `ArtifactCacheBatchOutcomeEntry`; use `index()`, `label()`, `outcome()`, and `entry_elapsed()` |
| `ArtifactCacheBatchFailure` without phase timing fields | Match the hard-cut variants with `..`, then use `phase()` and `timings()`; failed-entry views also expose `label()` and `timings()` |
| `CanisterDiagnosticsBatchEntry::into_parts() -> (String, CanisterDiagnosticsReport)` | Returns `(String, CanisterDiagnosticsReport, Duration)`; borrowed callers may use `entry_elapsed()` and the batch `total()` |

No index-only overloads, parallel label sidecars, tuple aliases, or deprecated
bridges are retained.

The Wasm resolver already discovers the selected local dependency closure, but
the complete workspace manifest and lockfile remain exact inputs because
workspace inheritance, profiles, patches, resolver state, external revisions,
build scripts, proc macros, and includes can cross the apparent closure. A
future narrower fingerprint must use a validated semantic projection with a
conservative fallback. Likewise, digest reuse across incompatible batch groups
requires an explicit caller-held immutable-source lease or validated snapshot;
`0.8.5` adds no ambient or unsafe digest cache.

## 0.8.4

`0.8.4` adds sequential collect-all diagnostics for caller-labeled exact
requests. `PocketIcDiagnosticsExt::collect_canister_diagnostics_batch` attempts
every target and returns ordered `CanisterDiagnosticsBatchEntry` values. Each
entry retains its label, exact controller-aware request, independent status and
log outcomes, bounded lossy UTF-8 log content, and omitted-byte/record counts.
An earlier rejection, dead PocketIC transport, or panic does not prevent later
entries from being attempted. There is no anonymous retry and
`dump_canister_debug` is not restored.

Wasm and generic artifact collect-all reports now expose structured failed
entries with specification index, error or failure, and complete entry wall
time. Generic reports also retain an ordered `entry_elapsed` value for every
success and failure. Detailed partial phase timings for failed preparation or
commit paths remain a future error-contract change.

### Hard-cut migration

| 0.8.3 API | 0.8.4 API |
| --- | --- |
| `WasmBuildBatchReport::failures()` yields `(usize, &WasmBuildError)` | Yields `WasmBuildBatchFailure`; use `index()`, `error()`, and `entry_elapsed()` |
| `ArtifactCacheBatchReport::failures()` yields `(usize, &ArtifactCacheBatchFailure<E>)` | Yields `ArtifactCacheBatchFailedEntry<E>`; use `index()`, `failure()`, and `entry_elapsed()` |

The tuple iterators have no aliases or deprecated bridges. Generic batch input
hashing is not memoized across entries because current preparation rehashes to
detect mutations. Safe reuse requires a caller-supplied source-immutability
lease or explicit revalidation. Caller-supplied stable artifact entry keys are
likewise reserved for one future labeled-spec hard cut rather than a parallel
key sidecar.

## 0.8.3

`0.8.3` is a behavior-preserving code-hygiene patch. It consolidates optional
phase-timing aggregation and the indexed result iteration shared by collect-all
batch reports, removing duplicate internal implementations.

Release tooling now uses one reader for the `[workspace.package]` version
across Make, changelog, bump, tag, publish, and release guards. Exact stable
versions are required for release operations while bump preparation retains
its prerelease-compatible parsing.

There are no public API, cache-format, schema, or runtime behavior changes in
this patch, and no migration or pre-1.0 API hard cut is required.

## 0.8.2

`0.8.2` is a release-process and CI-stability patch. It makes exact-cache
lock-heartbeat coverage scheduling-independent and ensures the complete release
gate runs before version metadata changes. A committed, tagged release is no
longer subjected to a redundant second validation pass that can strand a local
patch version after failure.

There are no public API, cache-format, schema, or runtime behavior changes in
this patch.

## 0.8.1

`0.8.1` continues the pre-1.0 hard-cut policy with structured controller-aware
diagnostics, collect-all generic artifact batches, and aggregate batch
observability. Removed APIs have no aliases, deprecated bridges, dual entry
points, or compatibility readers.

### Changes

- `build_artifact_caches_batch` is sequential collect-all and returns
  `ArtifactCacheBatchReport<E>`. Preparation, callback, and commit failures are
  indexed and later independent entries continue.
- Wasm and generic artifact reports expose aggregate built/reused/failed
  counters and successful timings. Wasm metrics also distinguish compatible
  input-resolution runs from reused snapshots.
- `WasmBuildBatchReport::entry_elapsed` retains wall time for every entry,
  including failures. Detailed phase timings remain available on successful
  records; partial failed-phase timing is a documented follow-up.
- `PocketIcDiagnosticsExt::collect_canister_diagnostics` takes exact,
  independent status and log senders and returns a structured report. Status
  and logs retain separate success/failure results. Log content is bounded
  lossy UTF-8 with explicit omitted-record and omitted-byte counts.
- The anonymous-only, printing `dump_canister_debug` entry point is removed.
  Install-failure diagnostics pass the install sender through and remain
  best-effort so they cannot replace the original failure.

### Additional hard-cut migration

| Earlier API | 0.8.1 API |
| --- | --- |
| `Result<ArtifactCacheBatchOutcome, ArtifactCacheBatchError<E>>` | `ArtifactCacheBatchReport<E>` with indexed `ArtifactCacheBatchFailure<E>` values |
| `PocketIcDiagnosticsExt::dump_canister_debug(canister_id, context)` | `collect_canister_diagnostics(CanisterDiagnosticsRequest::new(canister_id, status_sender, log_sender))`; inspect `status` and `logs`, then use `Display` or `render_compact` when text is needed |

Compatible Wasm input memoization remains scoped to one batch call. There is no
silent global cache or cross-call session in `0.8.1`. A proposed explicit
session must require a caller-guaranteed source-immutability lease (or pay for
revalidation); it is documented rather than partially implemented.

## 0.8.0

`0.8.0` is a pre-1.0 hard cut. It adds collect-all Wasm batches, compatible
input resolution reuse within one batch, and public immutable exact-cache
paths. Removed APIs have no aliases or deprecated bridges.

### Migration

| Before 0.8 | 0.8.0 |
| --- | --- |
| Wasm batch returned `Result<WasmBuildBatchOutcome, WasmBuildBatchError>` | It returns `WasmBuildBatchReport`; inspect `results`, indexed `outcomes`/`failures`, and `is_success` |
| `WasmBuildCachePrunePolicy`, `WasmBuildCachePruneReport`, `WasmBuildCacheMaintenance` | `ArtifactCachePrunePolicy`, `ArtifactCachePruneReport`, `ArtifactCacheMaintenance` |
| `CargoHeartbeat { elapsed }` | `Heartbeat { phase: WasmBuildProgressPhase::CargoBuild, elapsed }` |
| Wasm builders ending in `_os` | Use `with_cargo_profile_args`, `with_extra_env`, and `with_inherited_env` directly |
| `with_additional_input_paths` | `with_additional_inputs` |
| Transaction builders ending in `_os` | Use `with_arguments`, `with_environment`, and `with_unset_environment` directly |
| `CachedStandaloneCanisterFixturePool::acquire_with_outcome` | `acquire`, which returns the structured lifecycle outcome |
| Boolean result from fixture-pool `acquire` | Call `outcome.is_reused()` on the structured result |
| `PocketIcCapturedSnapshotExt` | Import `PocketIcSnapshotExt`; it owns both controller-fallback and exact-sender methods |
| `WasmBuildTimings::input_resolution_detail` plus aggregate `input_resolution` | `input_resolution` returns `WasmInputResolutionTimings` directly; call `.total()` for the aggregate |
| Panicking `build_wasm_canisters` wrapper | Construct `WasmBuildSpec` and call `build_wasm_canisters_cached` |

Wasm batch functions no longer return a top-level error. Handle all entries
after the sequential batch completes:

```rust,no_run
# use ic_testkit::artifacts::{WasmBuildSpec, build_wasm_canisters_cached_batch};
# let specs: Vec<WasmBuildSpec> = Vec::new();
let report = build_wasm_canisters_cached_batch(&specs);
for (index, error) in report.failures() {
    eprintln!("Wasm build {index} failed: {error}");
}
```

All repository-owned cache, stamp, and digest-domain identifiers remain at
`v1`. No migration reader is provided; disposable build caches may rebuild
under the current `v1` semantics.
