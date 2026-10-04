<p align="center">
  <img src="../images/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# Documentation

The [repository README](../README.md) and generated API documentation describe
the current implementation. The [package migration guide](../crates/ic-testkit/CHANGELOG.md)
ships with the crate and records the pre-1.0 hard cuts. The
[repository changelog](../CHANGELOG.md) also covers development and release tooling.

## Current usage

| Topic | Guide |
| --- | --- |
| Setup, host and canister modules | [Install](../README.md#install) |
| Startup, server ownership, and provenance | [Startup](../README.md#startup-and-runtime-provenance) |
| Typed calls and installation | [Candid calls](../README.md#typed-candid-calls), [fixtures](../README.md#canister-installation-and-fixtures) |
| Snapshot and fixture reuse | [Baselines and pools](../README.md#snapshots-and-cached-baselines), [complete recipe](../crates/ic-testkit/examples/multi_canister_baseline_pool.rs) |
| Diagnostics | [Diagnostics and time](../README.md#diagnostics-and-time) |
| Wasm builds, retained artifacts, batches, and source leases | [Artifact helpers](../README.md#wasm-artifact-helpers), [external-tool example](../crates/ic-testkit/examples/transactional_artifact_cache.rs) |
| Benchmark markers and reports | [Benchmark guide](../README.md#benchmark-markers-and-reports), [test canister](../canisters/README.md) |
| Targeted checks and releases | [Checks](../README.md#toolchains-and-checks), [release flow](../README.md#releases) |
| PocketIC limitations and local policy | [Upstream boundary](../POCKET-IC.md) |

Build local API documentation with `make docs-check`, then open
`target/doc/ic_testkit/index.html`. Run its compile-checked example with
`cargo test -p ic-testkit --locked --doc`.

## Historical design records

These records preserve the decisions and contracts at the time of their
design. Version-specific API names and proposals in their bodies may have been
superseded; use the current README, rustdoc, and migration guide when writing code.
Successful artifact records now retain read-only exact-cache paths for their
lifetime. Configured materialization paths remain mutable, and repository-owned
cache, stamp, and digest formats remain `v1`.

| Record | Subject |
| --- | --- |
| [0.1](design/0.1-benchmarking/0.1-design.md) | Benchmark marker processing and reports |
| [0.2](design/0.2-concurrency/0.2-design.md) | Direct PocketIC ownership and concurrency |
| [0.3](design/0.3-artifact-cache/0.3-design.md) | Content-addressed Wasm build coordination |
| [0.3 follow-up](design/0.3-artifact-cache/follow-up-design.md) | Consolidated artifact and fixture cache requirements |
| [0.4](design/0.4-baseline-pooling/0.4-design.md) | Bounded multi-canister baseline pools and reset contracts |
| [0.5](design/0.5-artifact-transactions/0.5-design.md) | Transactional external artifact sets |
| [0.6](design/0.6-shared-incremental-wasm/0.6-design.md) | Shared Cargo incremental state and immutable outputs |
| [0.7](design/0.7-artifact-orchestration/0.7-design.md) | Independent batches, progress, maintenance, and source leases |
