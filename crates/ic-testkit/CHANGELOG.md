# ic-testkit package changelog and migration guide

This file ships in the crate archive so upgrades can be completed without the
repository checkout. The complete historical changelog remains at
<https://github.com/dragginzgame/ic-testkit/blob/main/CHANGELOG.md>.

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
