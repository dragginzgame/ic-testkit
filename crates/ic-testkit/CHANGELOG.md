<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-testkit/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# ic-testkit package changelog and migration guide

This file ships in the crate archive so upgrades can be completed without the
repository checkout. The complete historical changelog remains at
<https://github.com/dragginzgame/ic-testkit/blob/main/CHANGELOG.md>.

## [0.25.2]

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
