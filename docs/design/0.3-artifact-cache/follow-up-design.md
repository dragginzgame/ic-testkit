# Consolidated Artifact And Fixture Cache Follow-Up

## Status

Proposed after the released `0.3.5` cache-lifecycle work and updated through
`0.4.2`. This document consolidates concrete Canic and IcyDB feedback. Complete
Cargo configuration discovery, in-lock retention, and detailed input timings
and the broader transactional artifact-set cache are now delivered. The
implemented transaction contract lives in the
[`0.5` transactional artifact-set design](../0.5-artifact-transactions/0.5-design.md).

## Consumer evidence

The two consumers need the same lower-level artifact transaction for different
commands:

- Canic wraps `icp build` with an operating-system lock, before/after watched
  input capture, a separate environment stamp, validation of a root Wasm plus
  a set of role artifacts, and one freshness stamp per output.
- IcyDB runs a pinned Binaryen transform after Cargo. It already identifies the
  exact input bytes, executable SHA-256 and version, ordered arguments, and a
  versioned pipeline, but it repeats the transform across build paths and
  processes.
- IcyDB uses standalone fixture-pool leases for its SQL matrix. Those leases
  intentionally live for a complete test and therefore require a documented
  `clippy::significant_drop_tightening` policy.

These are generic coordination problems. `ic-testkit` should not add separate
`icp build` and Binaryen cache implementations.

## Normalized requests

| Consumer request | Shared capability | Disposition |
| --- | --- | --- |
| Canic external artifact-set cache | Transactional artifact-set cache | One generic core |
| IcyDB post-link transform cache | Transactional artifact-set cache | One generic core |
| Canic and IcyDB retention | Cache lifecycle | Delivered for Wasm targets in `0.3.5`, with optional in-lock build maintenance in `0.4.2`; reuse in the generic core |
| Canic narrower invalidation | Exact Cargo input resolution and change reporting | Correctness-first follow-up |
| IcyDB Cargo-home configuration | Exact Cargo input resolution | Delivered generically in `0.4.2`, including ancestor discovery and recursive includes |
| IcyDB shared Cargo incrementals | Build workspace strategy | Separate opt-in layer, never cache truth |
| Canic fixture recipe validation | Typed pooled-fixture acquisition | Combine with fallible construction |
| IcyDB fallible fixture construction | Typed pooled-fixture acquisition | Combine with recipe validation |
| Canic multi-canister baseline pool | Explicit reset coverage | Defer until coverage can be represented honestly |
| IcyDB lease-lifetime lint friction | Documentation | Document the intentional scope/allowance pattern |

PocketIC client/server preflight remains an upstream compatibility concern.
It does not belong in the artifact cache. Structured timings should be emitted
by the shared transaction and build layers rather than reimplemented per
consumer.

## Exactness terminology

An exact content digest is not necessarily a minimal invalidation boundary.
The current Cargo cache hashes the contents of a conservative local package
closure. It therefore detects content changes deterministically, but a changed
README or test-only file can still invalidate a Wasm build.

Configuration resolution now covers Cargo's complete hierarchical and include
boundary. Future input resolution should retain a labeled per-file manifest and
expose added, removed, and content-changed paths. That makes invalidation
explainable without weakening it. A file may be excluded only when Cargo
semantics or an explicit build recipe prove that it cannot affect the output.

Cargo package `include` and `exclude` rules are not sufficient proof by
themselves for local builds: Rust `include!`, procedural macros, and build
scripts can read files outside the packaged set. Arbitrary caller exclusions
would make an apparently exact cache unsound and are not part of this design.
Callers continue to declare inputs that cannot be discovered from Cargo.

## Exact Cargo input resolution

One Cargo input resolver should serve `WasmBuildSpec` and future artifact
recipes that build Cargo packages. It should:

1. resolve the selected local dependency closure with Cargo metadata;
2. retain stable labels and per-file content digests in addition to the
   aggregate `InputDigest`;
3. include a selected Cargo graph/workspace semantic projection in cache
   identity while retaining complete workspace manifests and the lockfile for
   mutation validation, plus toolchain files and Cargo configuration Cargo can
   read from the workspace directory, its ancestors, and the effective Cargo
   home;
4. include the effective Cargo-home identity when relative configuration values
   make the configuration base path semantic;
5. include relevant `CARGO_*`, rustc, wrapper, and rustflags environment while
   never recording secret values in diagnostics;
6. keep caller-declared environment and additional paths as the explicit escape
   hatch for build-script and external-tool inputs;
7. produce a structured change set for diagnostics and timing reports.

The implemented resolver remains conservative without depending on prior build
state: it projects the selected graph directly from Cargo metadata, retains a
complete raw validation digest, and falls back to broad identity for package
roots that cannot be safely normalized.

## One transactional artifact-cache core

The core abstraction is a cache transaction over a named artifact set. The
complete locking, identity, publication, retention, and acceptance contract is
now specified in the accepted
[`0.5` design](../0.5-artifact-transactions/0.5-design.md); this section retains
the original normalized overview:

```text
ArtifactCacheSpec
  cache root + namespace + coordination scope
  recipe/pipeline identity
  labeled exact input snapshot
  tool content identity, ordered arguments, relevant environment
  stable logical output names and validation rules

prepare
  -> acquire the declared coordination lock
  -> capture/verify exact inputs
  -> derive the content key
  -> verify the complete cached manifest and every output digest
       -> hit: materialize requested outputs, record last use, Reused
       -> miss: return an owned build transaction and staging directory

commit
  -> import or accept every staged output
  -> validate the complete output set
  -> recapture inputs and reject changes during the build
  -> digest outputs and write one batch manifest
  -> atomically publish the cache directory
  -> atomically materialize caller-facing files, publishing stamps last
  -> record last use, Built

drop/error
  -> remove incomplete staging data
```

The caller, not `ic-testkit`, executes the command. IcyDB can direct `wasm-opt`
to a transaction staging path. Canic can run `icp build` and import its fixed
outputs into the transaction before commit. This keeps command policy,
application environment, and tool installation outside the generic crate.

The cache key must include:

- a cache-format version and namespace;
- a required caller-owned recipe or pipeline identity;
- stable logical input labels and their content digests;
- executable content digest when a tool is part of the recipe;
- ordered arguments and relevant environment;
- the logical output schema and built-in validation policy.

Tool version output is useful diagnostic metadata but is not a substitute for
the executable digest. Absolute public output paths are destinations, not cache
identity. A coordination scope is separate from the content key because two
different keys may still invoke tools that write the same external work tree.

A committed cache entry is a directory containing immutable outputs and one
manifest that lists every logical output and digest. The manifest is published
last. Partial public materialization is never considered reusable without its
matching final stamps. The core reuses the `0.3.5` failure cleanup,
`CACHEDIR.TAG`, process-lock, last-use, and age/size retention behavior.

## Relationship to the public APIs

The shared implementation should be extracted underneath existing APIs rather
than copied beside them:

- `WatchedInputSnapshot` delegates to the common labeled input snapshot;
- `WasmBuildSpec` composes Cargo input resolution with the artifact
  transaction;
- `build_wasm_canisters_cached` returns its typed `Built`/`Reused` result;
- `prune_wasm_build_cache` applies the shared retention types to the Wasm
  cache's fingerprint-entry layout;
- transform and external batch callers use the same transaction, manifest,
  errors, timings, and retention types.

One shared implementation should back each public behavior. Pre-`1.0` changes
replace old entry points directly instead of adding adapters, and internal
formats remain at `v1` while their semantics evolve in place.

## Shared Cargo incremental mode

Shared incrementals solve a different problem from exact artifact reuse and
must remain a separate, explicit build strategy:

```text
exact artifact store       authoritative, keyed by full build fingerprint
Cargo build workspace      disposable performance layer
  isolated fingerprint     current default
  shared named cohort      opt-in; Cargo owns incremental invalidation
```

A shared cohort uses a stable caller-named Cargo target directory and a cohort
lock. Final outputs are copied into the exact artifact store and verified with
the same post-build input check. The shared Cargo tree is never treated as a
cache hit by itself. Different fingerprints can reuse Cargo incrementals while
their final artifacts remain independently content-addressed.

Retention accounts for exact artifact entries and shared cohorts separately.
Fingerprint pruning must not recursively remove a live shared cohort. The
isolated strategy remains the default until downstream measurements show that
the shared mode is both faster and acceptably bounded.

## Fixture-pool follow-up

The fixture work is specified separately in the proposed
[`0.4` bounded multi-canister baseline-pool design](../0.4-baseline-pooling/0.4-design.md).
It combines structural recipe ownership, fallible construction, typed reset
requirements and receipts, precise recovery, runtime capacity, and one shared
bounded-slot scheduler. It permits scoped multi-canister reuse without
presenting named snapshots as complete simulator isolation.

## Delivery order

1. ~~Correct Cargo configuration discovery~~ (delivered in `0.4.2`), then add
   exact per-path change reports.
2. ~~Extract shared lock, cache tag, last-use, size, and retention primitives
   beneath the existing Wasm cache~~ (delivered in `0.5.0`).
3. ~~Expose the transactional artifact-set API and validate it with both a
   one-input transform and a multi-output external build fixture~~ (delivered
   in `0.5.0`).
4. Add the opt-in shared Cargo cohort strategy and measure cold, warm, disk,
   and concurrent behavior.
5. Implement the bounded multi-canister baseline pool only through the typed
   reset-coverage and recovery contract in the `0.4` design.

Each step must retain content verification, before/after input checks, bounded
cleanup, typed outcomes, and cross-process tests. Application-specific package
names, Binaryen flags, `icp` commands, and PocketIC topology recipes remain in
their owning projects.
