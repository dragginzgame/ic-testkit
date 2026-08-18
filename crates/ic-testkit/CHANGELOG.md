# ic-testkit package changelog and migration guide

This file ships in the crate archive so upgrades can be completed without the
repository checkout. The complete historical changelog remains at
<https://github.com/dragginzgame/ic-testkit/blob/main/CHANGELOG.md>.

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
| Separate batch calls always resolve their inputs independently | Hold the real source write-exclusion guard and call `WasmBuildSession::new(&guard)` when the immutable-source contract can be guaranteed |

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
