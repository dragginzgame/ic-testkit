# ic-testkit package changelog and migration guide

This file ships in the crate archive so upgrades can be completed without the
repository checkout. The complete historical changelog remains at
<https://github.com/dragginzgame/ic-testkit/blob/main/CHANGELOG.md>.

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
