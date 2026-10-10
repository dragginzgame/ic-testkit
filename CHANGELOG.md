# Changelog

All notable, and occasionally less notable changes to this project will be
documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.31.0] - 2026-10-10

### Breaking

- Adopt Shared Tooling 0.3.0's complete setup/check aggregates: host, IC and
  Cargo tools, followed by Testkit's PocketIC setup/check. Prepare the Rust
  toolchain, run `make install-tools`, then `make tools-check`. The redundant
  `install-format-tools` target is removed; server checks require a prepared
  owner CLI and never build it. Existing installations and evidence are retained
  ([#50](https://github.com/dragginzgame/ic-testkit/issues/50),
  [shared #98](https://github.com/dragginzgame/shared-tooling/issues/98)).

## [0.30.0] - 2026-10-10

### Breaking

- Bound retained Cargo diagnostics to a 1 MiB raw prefix per stream, marking
  truncated failure text while forwarding complete observed output. Builds keep
  no deadline. Tool/workspace probes reject output above 64 KiB per stream;
  metadata permits 16 MiB stdout. Complete output is required before parsing or
  deriving identities, using Host capture
  ([#49](https://github.com/dragginzgame/ic-testkit/issues/49)).

### Fixed

- Report unavailable failed-step logs during CI inspection, retaining partial
  logs and fetch errors instead of silently treating missing evidence as success,
  through Shared Tooling 0.2.14
  ([shared #97](https://github.com/dragginzgame/shared-tooling/issues/97)).

## [0.29.0] - 2026-10-10

### Breaking

- Adopt IC Host 0.11 through the public re-exports. Use per-stream
  `OutputLimit` policies and optional deadlines in `OutputLimits`, and read
  execution cleanup failures from `ExecutionError::cleanup`
  ([#47](https://github.com/dragginzgame/ic-testkit/issues/47),
  [host #46](https://github.com/dragginzgame/ic-host-tooling/issues/46)).
  Cargo builds retain their existing output policy and have no deadline.

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

### Breaking

- Adopt IC Host 0.10's consolidated durable-write API through the public Host
  re-exports. Update writer calls and publication-error handling using the
  [migration guide](crates/ic-testkit/CHANGELOG.md#0280). Testkit write failures
  preserve publication phase and cleanup evidence; retained cache formats and
  installations require no reset.

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

## [0.23.0] - 2026-10-08

### Breaking

- Adopt IC Host 0.6 through the public host-crate re-exports. Update
  `ExecutionError` literals/destructuring for its new `group_error` field.
- Invalid-port and builder startup errors now retain bounded output and secondary
  cleanup diagnostics. Update affected variant literals and patterns, including
  the now-structured `BuilderDisconnected`; see
  [package migration notes](crates/ic-testkit/CHANGELOG.md#0230)
  ([#30](https://github.com/dragginzgame/ic-testkit/issues/30)).

### Fixed

- PocketIC version probes clean up owned wrapper descendants on exit and timeout
  using Host's group capture; probes must not launch background work intended
  to survive the check
  ([host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5)).
- Preserve startup causes while displaying secondary cleanup failures in CLI
  errors; owned builder failures also retain server output before cleanup
  ([#30](https://github.com/dragginzgame/ic-testkit/issues/30)).

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

### Breaking

- Adopt IC Host Tooling 0.5 through the public host-crate re-exports. Gzip
  encoding now takes numeric levels 0–9; update compression arguments, Wasm
  fact literals and exhaustive inspection-error matches. See
  [package migration notes](crates/ic-testkit/CHANGELOG.md#0220).

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

- Delegate managed server and command process-group cleanup to IC Host's
  shared child owner, preserving readiness, cancellation and bounded diagnostics
  ([#25](https://github.com/dragginzgame/ic-testkit/issues/25)).
- Refresh Shared Tooling to 0.1.23: recheck release payload and annotated tags
  after final hooks, verify completed-release remote identity, and query paginated
  PR results without requiring a newer GitHub CLI `--slurp` option.

## [0.21.3] - 2026-10-07

### Changed

- Refresh the canonical release engine and engineering baseline to Shared Tooling
  0.1.21, including its PR-delivery helper. This repository retains direct atomic
  branch/tag delivery; adopting the shared PR flow requires separate consumer
  adapters and qualification. See [package notes](crates/ic-testkit/CHANGELOG.md#0213)
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

- Run the canonical Rust-tool setup and path-refusal fixture in portable CI on
  Linux and both macOS architectures, covering checkout ownership, failed
  installation retention and retries without installing real tools
  ([#24](https://github.com/dragginzgame/ic-testkit/issues/24),
  [shared #54](https://github.com/dragginzgame/shared-tooling/issues/54)).

## [0.21.1] - 2026-10-07

### Fixed

- Qualify configured PocketIC paths through canonical filesystem identity on
  macOS, teach the benchmark Cargo substitute workspace discovery, and prepare
  explicit server configuration in concurrency CI
  ([#19](https://github.com/dragginzgame/ic-testkit/issues/19),
  [#18](https://github.com/dragginzgame/ic-testkit/issues/18),
  [#23](https://github.com/dragginzgame/ic-testkit/issues/23)).
- Adopt Shared Tooling 0.1.19: reject redirected Rust-tool installation paths,
  preserve complete failed-batch logs and historical changelog bytes
  ([shared #54](https://github.com/dragginzgame/shared-tooling/issues/54),
  [shared #37](https://github.com/dragginzgame/shared-tooling/issues/37),
  [shared #55](https://github.com/dragginzgame/shared-tooling/issues/55)).

### Changed

- Select compatible IC Host Tooling 0.4.2 dependencies, exposing its
  named-output, chunk-digest and installed-tool admission additions through the
  existing public host-crate re-exports. See
  [package notes](crates/ic-testkit/CHANGELOG.md#0211).

## [0.21.0] - 2026-10-07

### Breaking

- Adopt IC Host Tooling 0.4.0 across the four public host-crate re-exports.
  Move retired durable readers to `ic_host_fs::read`, supply bounded reads and
  handle typed private-file and lock errors
  ([host #14](https://github.com/dragginzgame/ic-host-tooling/issues/14),
  [#13](https://github.com/dragginzgame/ic-testkit/issues/13)).
- `workspace_root_for` now returns `io::Result<PathBuf>` and uses Cargo's actual
  workspace membership instead of a directory-name heuristic. Handle discovery
  errors and supply a real crate manifest directory
  ([#18](https://github.com/dragginzgame/ic-testkit/issues/18)).
- New Wasm build specs share `<workspace>/target/ic-testkit-incremental` across
  fingerprints instead of creating a Cargo target for each key. Select
  `with_isolated_builds()` when isolation is required, and keep retention limits
  explicit. Existing retained isolated targets are not deleted automatically
  ([#18](https://github.com/dragginzgame/ic-testkit/issues/18)).
  See [migration details](crates/ic-testkit/CHANGELOG.md#0210).

### Added

- Select PocketIC startup with `PocketIcStartupConfig::from_env`: prefer
  `IC_TESTKIT_POCKET_IC_URL`, otherwise verify the explicit `POCKET_IC_BIN`.
  Run multi-process suites through `ic-testkit-server run -- COMMAND`, retaining
  one server owner and cleaning up owned process groups on exit or interruption
  ([#19](https://github.com/dragginzgame/ic-testkit/issues/19)).
- Bound caller-owned readiness polling with `pic::tick_until`, advancing time
  and ticking only while pending and retaining typed predicate failures
  ([#20](https://github.com/dragginzgame/ic-testkit/issues/20)).

### Fixed

- Stop a pending command when its managed PocketIC server exits, including TTL
  expiry, and return the server's status and bounded diagnostics instead of
  leaving the suite waiting ([#19](https://github.com/dragginzgame/ic-testkit/issues/19)).
- Resolve dangling input symlink chains to their missing targets through the
  selected IC Host Tooling 0.4.0 libraries, keeping equivalent cache-input paths
  on one identity ([host #1](https://github.com/dragginzgame/ic-host-tooling/issues/1),
  [#13](https://github.com/dragginzgame/ic-testkit/issues/13)).
- Preserve failed CI validation logs, tool-installation candidates and portable
  tooling fixtures as downloadable artifacts, including failures during setup
  ([#17](https://github.com/dragginzgame/ic-testkit/issues/17)).

### Changed

- Live tests and the baseline example use explicit environment startup; prepare
  `POCKET_IC_BIN` or supply `IC_TESTKIT_POCKET_IC_URL`. Benchmark guidance uses
  fetched canister logs instead of the managed server's bounded diagnostic prefix
  ([#23](https://github.com/dragginzgame/ic-testkit/issues/23)).
- Document the existing compiler-selection and cached-transform contracts for
  Cargo shims, and qualify cache invalidation when the selected compiler identity
  changes ([#21](https://github.com/dragginzgame/ic-testkit/issues/21)).
- Use Shared Tooling 0.1.18 for common setup, offline tool checks and LOC reports.
  Setup and CI now share pinned ripgrep and cloc alongside jq/yq and IC tools;
  rerun `make install-tools` in existing checkouts to prepare the complete set.
- Exclude physical Cargo output reached through target aliases and reject
  validation target options/assignments before dispatch. Corrected upstream LOC
  fixtures also qualify enclosing Cargo workspaces and inherited target settings
  ([shared #31](https://github.com/dragginzgame/shared-tooling/issues/31),
  [shared #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [shared #47](https://github.com/dragginzgame/shared-tooling/issues/47),
  [shared #53](https://github.com/dragginzgame/shared-tooling/issues/53)).
- Provide optional local Rust-tool setup through `make install-rust-tools` and
  offline `make rust-tools-check`, using the shared exact tool catalog
  ([shared #51](https://github.com/dragginzgame/shared-tooling/issues/51)).
- Preserve trailing-whitespace draft headings during release preparation and
  retain the underlying Make failure status through shared validation
  ([shared #38](https://github.com/dragginzgame/shared-tooling/issues/38),
  [shared #37](https://github.com/dragginzgame/shared-tooling/issues/37)).

## [0.20.0] - 2026-10-07

### Breaking

- Adopt the published IC Host Tooling 0.3.1 split. Import artifact streams,
  digests, archives and Wasm inspection from `ic_testkit::ic_host_artifacts`,
  pathname reads and durable publication from `ic_testkit::ic_host_fs`, and
  executable operations and Git provenance from `ic_testkit::ic_host_process`.
  `ic_testkit::ic_host_tools` now exposes Candid and response helpers only.
  `read_wasm` returns `ic_host_artifacts::artifact::ArtifactError`.
  See the [migration guide](crates/ic-testkit/CHANGELOG.md#0200)
  ([#13](https://github.com/dragginzgame/ic-testkit/issues/13)).

- Durable publication now synchronizes the final parent directory. An error
  after rename can leave the complete new output visible; inspect and reconcile
  the destination before retrying. Staging collisions retry within the shared
  bound ([#13](https://github.com/dragginzgame/ic-testkit/issues/13)).

### Changed

- Share missing-suffix path resolution and observed lock acquisition with
  `ic-host-fs`, removing the local engines and redundant publication wrapper
  ([#13](https://github.com/dragginzgame/ic-testkit/issues/13)).

- Share CI/release target dispatch and failure-log retention with Shared Tooling,
  rejecting inherited Make modes that skip recipes or ignore failures
  ([Shared Tooling #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [#15](https://github.com/dragginzgame/ic-testkit/issues/15)).

- Move the unchanged `ic_testkit_perf_probe` fixture package from
  `canisters/test/perf_probe` to `crates/ic_testkit_perf_probe`, following the
  shared workspace layout; update fixture builds and release metadata together.

- Bind release push to the captured destination URL and verify snapshot digests
  independently of the inspected checksum helper
  ([#15](https://github.com/dragginzgame/ic-testkit/issues/15)).

- Adopt Shared Tooling 0.1.14 for TOML-aware version reads, release-tag validation,
  Cargo inheritance and offline formatter checks, removing duplicate local checks
  ([#11](https://github.com/dragginzgame/ic-testkit/issues/11),
  [#12](https://github.com/dragginzgame/ic-testkit/issues/12)).
  See [detailed notes](crates/ic-testkit/CHANGELOG.md#0200).

### Fixed

- Hook qualification works after the fixture-package move is committed, without
  attempting to delete a directory absent from the selected checkout.

- Cache retention releases its lock when the final record owner drops, even
  during concurrent process spawning. The macOS request-reader fixture tolerates
  a connection that is not yet ready to accept.
  See [detailed notes](crates/ic-testkit/CHANGELOG.md#0200).
- Host-tool recovery fixtures restore exact archive bytes and retain failure
  diagnostics, avoiding differences from repacking on native hosts
  ([Shared Tooling #17](https://github.com/dragginzgame/shared-tooling/issues/17)).
- Release preparation preserves undated historical changelog entries instead of
  rejecting them as competing pending releases
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
- Failed dependency-check fixtures retain their inputs and logs for diagnosis
  ([Shared Tooling #21](https://github.com/dragginzgame/shared-tooling/issues/21)).
- Changelog selection compares large version components exactly, preserving
  pending notes above floating-point integer precision
  ([Shared Tooling #23](https://github.com/dragginzgame/shared-tooling/issues/23)).

## [0.19.2] - 2026-10-06

### Changed

- Publication uses the committed Shared Tooling registry observer while keeping
  offline and upload policy local. IC tool setup shares checksum generation and
  rejects receipt traversal and filename failures before activation
  ([Shared Tooling #10](https://github.com/dragginzgame/shared-tooling/issues/10)).
  See [detailed notes](crates/ic-testkit/CHANGELOG.md#0192).
- Release guards use the shared command-adoption checker and retain the
  consumer's metadata and sequencing checks. The shared runner's fixture suite
  is maintained upstream instead of copied and rerun here
  ([Shared Tooling #8](https://github.com/dragginzgame/shared-tooling/issues/8)).

### Fixed

- Release preflight prepares missing locked dependencies using Cargo's configured
  network policy, so normal releases need no separate cache-fetch command.
  Explicit offline requests remain offline, including during recovery
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
- Linux and macOS CI explicitly prepare ripgrep and locked dependency caches
  before portable and offline release checks
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7),
  [#9](https://github.com/dragginzgame/ic-testkit/issues/9)).

## [0.19.1] - 2026-10-06

### Fixed

- Publication proceeds only after crates.io confirms the exact version is absent;
  unavailable registry observations stop the command. Release admission verifies
  the existing annotated tag requirement and its exact commit, including failed
  Git inspections.
  See [detailed notes](crates/ic-testkit/CHANGELOG.md#0191).
- Release admission rejects failed version queries and reads of saved validation
  and readiness records before changing metadata, preserving records for retry
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

## [0.19.0] - 2026-10-06

### Breaking

- Shared host APIs are available through `ic_testkit::ic_host_tools`. Replace
  `artifacts::resolve_executable` with the shared Unix resolver using explicit
  paths and search directories; Wasm reads now require a byte limit and return
  typed errors. The external-transform example now requires digest/version
  authority and uses shared verified execution. The shared dependency checker
  replaces `make actions-check`.
  See the [migration guide](crates/ic-testkit/CHANGELOG.md#0190).

### Added

- Explicit `make install-tools` setup for pinned local host parsers and IC
  executables, with offline `tools-check` and dependency declaration checks in
  CI and release validation.

### Fixed

- Release qualification isolates fixture identities from recursive Make
  overrides, so minor and major release gates can validate patch fixtures
  without inheriting the outer release candidate
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
- Release preparation stops before publishing metadata when README validation
  fails under macOS's system Bash. Portable release and hook checks enforce
  their preservation assertions, and metadata fixtures exercise the actual
  Make callbacks, including rejection of incorrect selected commits
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). See the
  [package tooling notes](crates/ic-testkit/CHANGELOG.md#0190).
- Source-tree fingerprinting reuses bounded file buffers and avoids extra native
  filename copies on Unix, reducing allocations while preserving cache digests.

## [0.18.3] - 2026-10-06

### Fixed

- Normal release commands recover an older interrupted release after committed
  fixes, then validate the requested next increment. Late checks bind saved
  metadata to that exact release commit, preserving evidence and rejecting
  changed payloads or failed Git inspection
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7),
  [Shared Tooling #4](https://github.com/dragginzgame/shared-tooling/issues/4),
  [#5](https://github.com/dragginzgame/shared-tooling/issues/5)). See the
  [package tooling notes](crates/ic-testkit/CHANGELOG.md#0183).
- Adopt Shared Tooling's formatter-failure correction for the system Bash on
  macOS, preserving selected files and the index when formatting fails.

## [0.18.2] - 2026-10-06

### Fixed

- Release validation, preparation and recovery stop when Git cannot complete
  a source or metadata digest check, even if its output matches the saved
  digest. Rejection preserves metadata and evidence; validation retries retain
  the failed attempt's log
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). See the
  [package tooling notes](crates/ic-testkit/CHANGELOG.md#0182).
- Release metadata hashes use exact file bytes, so Git filters and line-ending
  conversion cannot conceal changes to live files or retained backups
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
- Release preparation updates only the maintained README TOML dependency
  examples, preserving historical lines, other code examples and formatting
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

## [0.18.1] - 2026-10-05

### Fixed

- Rerunning the same release target recovers its saved release without another
  version increment or duplicate commit, tag or completed push. Preparation and
  recovery reject symlinks in saved metadata before changing release files or
  linked targets. Interrupted metadata publication recovers even when the
  new lockfile was written before the manifest. Leftover publication staging
  files no longer block recovery and are preserved
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
- Release and publication source checks stop when Git cannot list untracked
  files, preserving metadata and validation evidence instead of treating a
  failed inventory as a clean checkout
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).
- Hook installation uses Shared Tooling's physical-path fix directly, removing
  the local path workaround
  ([Shared Tooling #1](https://github.com/dragginzgame/shared-tooling/issues/1)).
- Release-check fixtures work with finalized or pending changelog notes, so
  checks remain usable after a release
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)). See the
  [package tooling notes](crates/ic-testkit/CHANGELOG.md#0181).
- Portable CI explicitly prepares `rustfmt` and stops at the first shell
  failure. Hook setup rejects a missing `rustfmt` before activation. CI matrices
  collect all supported hosts' results when another host fails, preserving
  macOS qualification evidence
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

## [0.18.0] - 2026-10-05

### Breaking

- Maintainer releases now use one reviewed workflow through `make release-patch`,
  `make release-minor` and `make release-major`. It runs the same complete gate
  for all three, finalizes both changelog views, stages explicit metadata, and
  atomically pushes only the selected branch and exact tag. Separate bump,
  stage, commit and push entry points are removed. Interrupted releases resume
  with `make release-resume VERSION=X.Y.Z`, retaining their exact version,
  destination, validation logs and prepared metadata. See
  [#7](https://github.com/dragginzgame/ic-testkit/issues/7) and the
  [package migration notes](crates/ic-testkit/CHANGELOG.md#0180). Use
  `make release-minor` for this batch.

### Changed

- Formatting now sorts all workspace Cargo manifests with pinned `cargo-sort`
  before formatting Rust. Developer setup activates a reviewed pre-commit hook
  that refreshes only selected files and preserves unrelated edits. CI and
  release gates enforce the same formatting independently.
- Failed preflight or validation can restart through the normal release target
  with fresh checks and retained attempt logs. Once preparation may begin,
  retries still require the exact saved release plan.

### Fixed

- Hook setup and its qualification fixtures use physical workspace paths, so
  macOS temporary-directory aliases do not cause a false repository-root
  rejection ([Shared Tooling #1](https://github.com/dragginzgame/shared-tooling/issues/1)).
- Release validation preserves the caller's network policy so its locked publish
  dry run can access the registry. Cache checks and metadata preparation remain
  offline, and explicitly offline failures are never retried online automatically
  ([#7](https://github.com/dragginzgame/ic-testkit/issues/7)).

## [0.17.3] - 2026-10-05 - Installation documentation consistency

### Fixed

- Both current installation examples now select `ic-testkit = "0.17"` rather
  than the older 0.14 line. This addresses the recurring documentation drift in
  [issue #6](https://github.com/dragginzgame/ic-testkit/issues/6).

### Tooling

- `make installation-check` checks the root and packaged README TOML dependency
  examples against the workspace package's major/minor line. Ordinary CI and
  portable-host checks run it; release preparation uses the already selected
  `CHANGELOG_VERSION` target before bumping the manifest, so minor releases
  verify their new installation line. Historical migration prose is untouched.
- Focused release-guard fixtures pass for current and patch versions, minor
  targets, whitespace/comments and historical mentions. They reject stale,
  missing, duplicate or malformed examples and invalid target versions. These
  checks use temporary workspaces and mocked release commands, with no remote
  effects or dependency changes. Native system-Bash qualification remains
  pending a matching CI run.

## [0.17.2] - 2026-10-05 - Portable teardown fixture

### Fixed

- The synthetic PocketIC HTTP peer explicitly switches accepted streams to
  blocking I/O before applying its read/write timeouts. macOS inherits the
  nonblocking listener mode; a connected client that had not yet sent its
  request could otherwise cause an immediate `WouldBlock` failure. Library
  transport and teardown behavior are unchanged.

### Testing

- A real TCP regression starts with an empty nonblocking accepted stream and
  sends the request only after the reader configures blocking I/O. It reproduces
  the failure before the fix and passes afterward on Linux. The focused suite
  passes, including the isolated instance-deletion and refused-transport probes.
- Portable CI now runs the synthetic transport/teardown suite on Linux and
  ARM64/Intel macOS without requiring a live PocketIC binary. Native confirmation
  of this fixture change remains pending a matching CI run.
- The pushed 0.17.1 portable-host checks passed on all three native hosts,
  qualifying the earlier report-publication and standalone tool-launch changes
  for that revision.

## [0.17.1] - 2026-10-05 - Benchmark tooling and pruning-test fixes

### Fixed

- Benchmark `--output` reports are encoded into private scratch beside the
  destination, synced, and atomically renamed into place. Partial writes,
  encoding errors, sync failures, observed interruption before publication,
  and rename failures leave the previous report untouched. Scratch cleanup
  removes only files owned by the driver; parent directories must already exist.
- Published reports use the staged file's private Unix permissions (`0600`).
  Replacement operates on the requested directory entry: existing readers and
  hard links keep the previous contents, and a replaced symlink's target is
  untouched. Report fields, `v1` format and library APIs are unchanged.
- Standalone benchmark launches preserve Cargo and rustc proxy names rather
  than invoking the canonical rustup binary as a different command. Cargo and
  compiler provenance run from the repository workspace so rustup applies the
  same toolchain-selection context. Explicit relative `CARGO` and
  `RUSTC` paths are anchored to the caller's directory before work begins;
  the same selected compiler is passed to Cargo and used for provenance.
  Invalid explicit tool selections fail without falling back to another tool.

### Testing

- The abandoned-staging pruning test controls its lock descriptors in an
  isolated subprocess, avoiding inheritance by unrelated parallel test spawns.
  It also verifies on Unix that a duplicate content-lock descriptor outliving
  an aborted transaction protects orphan staging until that descriptor closes.
  Holding such a duplicate reproduced the reported zero-removals assertion;
  the updated test and focused transaction suite pass. Production pruning,
  public APIs and lock/cache formats are unchanged.
- Focused Linux driver checks pass, including injected storage exhaustion after
  a partial staged write, a real rename failure, complete JSON replacement,
  retained readers/hard links, symlink target preservation and interruption
  before commit. Clippy passes with warnings denied.
- A subprocess regression reproduces the proxy-name failure before the fix and
  passes afterward. It exercises PATH lookup, absolute and relative overrides,
  workspace context, offline build flags and rejection of a missing override
  with deterministic multicall tool substitutes.
- A standalone Linux launch from outside the checkout, with `CARGO`, `RUSTC`
  and `RUSTUP_TOOLCHAIN` unset, used the real rustup proxies and completed a
  three-task live PocketIC smoke. Builds stayed locked/offline, and the atomic
  report records the workspace's Rust 1.99.0 compiler. This is dev-profile
  functional evidence, separate from the earlier 108-task performance run.
- The released 0.17.0 startup and driver checks passed natively on Linux and
  ARM64/Intel macOS. The new publication and tool-launch checks require their
  own native CI run.
- A live Linux PocketIC 16.0.0 run completed 108 tasks across fresh fixtures and
  pooled capacities one/two, with three rotating repeats, two workers and 1 MiB
  of state per canister. The relative output path received a complete `0600`
  report. Provenance records the dirty working tree and new Wasm identity;
  this is live functional evidence, not a release-to-release speedup claim.

## [0.17.0] - 2026-10-05 - Native Rust fixture benchmark tooling

### Changed

- Replaced the repository's documented Python benchmark and test commands with
  `cargo run/test -p ic-testkit --example fixture_reuse_benchmark_driver`.
  This is a hard cut to the operator entry point, recorded as a pre-1.0 minor
  release. The existing Rust workload and its measurement boundaries remain.
- The driver supports native Linux and macOS. Linux retains `/proc` RSS-page
  sampling; macOS reads process-leader RSS from native `ps` in KiB. Reports retain
  `ic-testkit-fixture-benchmark-v1` and add `provenance.rss_sampler` to identify
  the measurement source. Historical Linux measurements keep their original
  host and sampler qualification.
- Preserved fresh/pooled selection, capacity sweeps, rotated repeats, exact
  caller-supplied PocketIC 16 binaries, raw measurements and mean/p50/p95 summaries.
  Workload builds use the selected lockfile and prepared offline caches. Source,
  compiler, host and Wasm provenance is captured before cases run.
- Ctrl-C and sampler errors wait for the current worker to complete owned server
  cleanup. The driver starts no subsequent case after interruption. It preserves
  Cargo artifacts and caller-owned reports; only its private scratch is removed.
- Worker JSON is bounded to 16 MiB and checked against requested workload facts,
  task identities, phase coverage and finite nonnegative timings before summaries.
- Added focused native driver/sampler checks to the existing Linux, ARM64 macOS
  and Intel macOS CI matrix. Full live benchmark measurements remain opt-in;
  native macOS qualification for this revision remains pending CI.
- Library APIs, dependency selections, cache formats, MSRV and workspace package
  versions are unchanged. Python is no longer required for the benchmark workflow.

### Testing

- Replaced the background-reaper test's 30 ms readiness window with an explicit
  caller release after reaper handoff. The 0.16.1 Intel macOS portable-host job
  exposed that timing race; focused Linux startup checks pass and production
  startup behavior is unchanged.
- Focused Rust driver checks pass on Linux, including argument rejection,
  process-tree boundaries, native observations, report limits, malformed JSON,
  preserved raw samples/statistics, and worker completion on interruption.
- A Linux CLI smoke with substituted Cargo, server and workload programs verified
  locked offline build arguments, paths containing spaces, rotated cases, report
  emission and invalid-input exit status. This is functional fixture evidence,
  not live PocketIC performance data.
- Example/test-target Clippy checks and workflow validation pass. Native macOS
  execution of the new driver and live benchmark performance remain unmeasured.

## [0.16.1] - 2026-10-05 - Managed startup and native macOS fixes

### Fixed

- Managed startup handles Darwin's `EPERM` result when its process group contains
  only the exited, unreaped leader. A bounded native membership query verifies
  that exact state before accepting cleanup; permission failures for groups
  containing another process still propagate. Child status and captured output
  remain available through `ServerExited`.
- Startup port and output readers open nonblocking on Unix and validate the
  opened file as regular. A replaced FIFO cannot stall readiness or diagnostic
  capture: port reads return a structured `Io` failure with `InvalidData`, while
  unavailable output streams remain empty.
- Cargo input tests compare canonical package paths while retaining the caller's
  explicit Cargo-home configuration lookup path. This accommodates macOS's
  `/var` directory alias without discarding replacement detection.
- Filesystem fixtures use valid non-ASCII native names on macOS. Linux retains
  invalid-UTF-8 filename coverage, and Unix byte-conversion coverage runs without
  requiring the filesystem to accept those names.

### Changed

- Completed function-local import cleanup in startup, transport, digest, cache
  filesystem, executable-tool resolution and benchmark tests, preserving
  platform-specific conditions.
- Public APIs and repository-owned `v1` formats are unchanged. Native macOS
  confirmation requires CI for this revision; Linux checks do not qualify it.

### Testing

- A FIFO regression covers the port file and both captured-output streams,
  including whether a reader needs a writer to unblock it. The regression failed
  before the regular-file reader fix and passes with it.
- Managed process tests verify leader reaping and descendant termination on
  Linux and macOS for handle drop, readiness timeout, natural exit and background
  reaping. Process-state inspection uses each host's native `ps`, avoiding
  Linux-only `/proc` assertions. Descendants outlive the assertion deadline so
  natural timeout cannot masquerade as successful cleanup.
- Focused Linux startup, digest, cache filesystem, transaction, tool, transport,
  Cargo-input and benchmark-schema tests pass. The three affected Cargo-input
  tests also pass with a symlinked temporary directory.
- Clippy checks for the library and its test targets, and public API documentation
  builds, pass with warnings denied. Formatting and the pinned shared-tooling
  snapshot checks pass.

## [0.16.0] - 2026-10-05 - Wasm library contracts and shared engineering baseline

### Changed

- This minor release records the incompatible Wasm library contract: acquired
  packages must declare a same-name `cdylib` library, and acquisition builds
  only libraries. Non-library targets must be built separately; `--all-targets`
  and binary/example/test/bench selectors are rejected. Earlier 0.15 patch
  releases introduced related behavior cuts; future pre-1.0 contract changes
  require a minor release.
- Adopted the reviewed Shared Tooling baseline at
  `41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e` with a local overlay and checksum
  verification in CI. LOC reporting now uses the canonical Cargo-workspace
  script, replacing the stale Canic-specific implementation.
- Release CI delegates directly to ordinary CI and preserves the caller's
  temporary-directory environment. Retired the private-scratch process scanner
  and its Linux-only Python/pidfd requirements. Managed server handles retain
  child ownership; upstream background servers retain their normal lifecycle.
- Portable guard scripts support Bash 3.2. CI adds native macOS 15 ARM64 and
  Intel checks for CI, MSRV, operator tooling, managed startup and PocketIC concurrency;
  the local host matrix distinguishes required support from passing evidence.
- Centralized `libc` under workspace dependency ownership. Test modules use
  ordinary module discovery and file-top imports, and failure assertions use
  structured errors or artifact bytes. The version helper preserves selected
  external dependencies through offline lockfile updates.

### Fixed

- Wasm acquisition validates the selected packages' library output declarations
  from Cargo metadata. A package must provide a `cdylib` library whose name
  matches its expected `<package>.wasm` artifact. Missing, non-`cdylib`, and
  renamed libraries return `InvalidSpec` before cache reuse, compilation, or
  scheduled shared-target maintenance.
- Previously, changing a library to `rlib` or renaming it could leave an old
  shared-target Wasm available for publication under the changed manifest's
  fingerprint even though Cargo no longer produced that artifact.
- Validation survives batch resolution, immutable-source sessions, and prepared
  snapshots. A failed package retains its input-discovery failure and does not
  prevent compatible batch entries from succeeding. Libraries emitting both
  `rlib` and `cdylib` remain accepted; `cdylib` examples do not satisfy the
  library requirement.
- Input-only Cargo snapshots and transactional artifact recipes remain generic.
  Public signatures, fingerprint domains, and persisted layouts are unchanged;
  repository-owned formats remain `v1`.

### Testing

- A real Cargo regression reproduced stale publication after changing `cdylib`
  to `rlib`. It now checks type/name rejection, untouched shared outputs even
  with zero-byte scheduled retention, absence of compilation outputs, and
  correct reuse after restoring the declared library.
- Focused checks cover library/example distinction, dual crate types, batch
  continuation, session and prepared-reader validation, failure phases, and
  generic transactional Cargo snapshots. Full pre-push validation remains
  maintainer-owned.
- Aligned the synthetic artifact-handoff fixture's manifest with the required
  `cdylib` contract and reran its cold/warm retention and process handoff tests.
  Portable shell guards, snapshot checks, workflow lint and focused Clippy pass.

## [0.15.9] - 2026-10-05 - Library-only Wasm acquisition

### Fixed

- Wasm acquisition always invokes Cargo with `--lib`. When a binary and a
  canister library share a name, Cargo's default build can overwrite the library
  Wasm with the binary Wasm while reporting success. Acquisition now builds only
  the library in both isolated and shared targets, including observed builds.
- `--all-targets` now returns `InvalidSpec` before input resolution or cache
  work because it can reintroduce the collision. Explicit `--lib` arguments
  remain accepted and are emitted only once in the build command. Other target
  selectors remain rejected; build non-library targets separately.
- Build fingerprints include library target selection so artifacts acquired
  under the previous build behavior are not reused. This causes a fresh exact
  acquisition after upgrading; shared Cargo compilation can still reuse its
  library outputs. Public signatures and persisted layouts are unchanged;
  repository-owned formats remain `v1`.

### Testing

- Reproduced binary replacement with real Cargo before fixing, including a
  failing acquisition regression. The regression compares acquired bytes with
  a library-only baseline in isolated/shared targets and observed/silent builds,
  then checks exact-cache reuse.
- Focused checks cover early target-selector rejection and the actual Cargo
  invocation in both output modes. Full pre-push validation remains
  maintainer-owned.

## [0.15.8] - 2026-10-05 - Canister library target validation

### Fixed

- Wasm acquisition rejects binary, example, test, and bench target selectors
  before input resolution or cache work. These selectors can build successfully
  without refreshing the canister library; previously, shared-target acquisition
  could publish an existing canister Wasm under a changed source fingerprint.
- Named selectors are rejected in both separate-value and `--option=value`
  forms, along with their plural forms. Default selection, `--lib`, and
  `--all-targets` remain accepted. Public signatures and persisted layouts are
  unchanged; repository-owned formats remain `v1`.

### Changed

- Documentation uses images from the shared-assets repository; duplicated
  local image files are removed.

### Testing

- A real Cargo fixture reproduced successful binary-only acquisition with a
  stale canister output before fixing. The regression checks early rejection,
  preservation of the existing Wasm, absence of a binary build, and subsequent
  library rebuild and reuse.
- Focused validation checks cover named and plural target selectors, isolated
  and shared acquisition, and rejection before tool execution or cache creation.
  Full pre-push validation remains maintainer-owned.

## [0.15.7] - 2026-10-05 - Cargo profile and artifact-path agreement

### Fixed

- Wasm acquisition rejects a profile output directory that disagrees with the
  selected Cargo profile before input resolution or cache work. Previously, a
  shared-target build could compile changed sources in debug mode and publish
  an existing release Wasm under the new fingerprint.
- Validation recognizes default debug builds, `--release`, short `-r` flag
  clusters, both `--profile` forms, built-in profile directory mappings, and
  custom profiles. Feature values are not interpreted as profile switches.
  Empty or conflicting profile selections return `InvalidSpec`.
- Callers must pair the output directory with matching Cargo arguments;
  declaring `release` alone does not select release mode. Public signatures,
  dependencies, and persisted layouts are unchanged; formats remain `v1`.

### Testing

- Reproduced stale release-output publication with real Cargo before fixing.
  The regression checks early rejection, preservation of shared release output,
  absence of a debug build, and subsequent release rebuild and reuse.
- Focused validation checks cover profile aliases, custom profiles, compact
  flags, feature values, and rejection before tool execution or cache creation.
  Full pre-push validation remains maintainer-owned.

## [0.15.6] - 2026-10-05 - Simpler retries and artifact bookkeeping

### Changed

- Install-code retries use an explicit budget for attempts after the initial
  call, removing an unreachable panic and a one-line rejection-classification
  helper. Only install-code rate limiting triggers retries; cooldowns remain
  between attempts, and exhaustion returns the final rejection unchanged.
- Watched-input freshness tests use the existing artifact temporary-directory
  helper, removing their separate clock-based naming and atomic counter.
- Wasm acquisition constructs expected artifact paths through one flow for
  default and caller-selected targets, removing an equivalent special-case
  branch. Package order, profile directories, cache lookup, reconstruction,
  and publication retain their existing paths.
- Wasm and transactional artifact-batch metrics derive success totals from
  built plus reused counts, and failure totals from the number of report entries.
  The redundant stored counters and parallel updates are removed. Metrics use
  each report's existing successful-outcome iterator; public getters, report
  display, input-resolution counters, and successful timing totals are preserved.
- Public API signatures, dependencies, and persisted layouts are unchanged;
  repository-owned formats remain `v1`.

### Testing

- Focused retry checks cover one- and three-attempt exhaustion, final-rejection
  preservation, immediate non-rate-limit failure, immediate success, success
  after retries, cooldown ordering, and rejection of zero-attempt policies.
- Freshness checks cover matching content stamps, changed inputs, checkout-root
  and input-order independence, malformed and oversized stamps, and read errors.
- Batch checks cover empty and failed Wasm reports, transactional builds, reuse,
  and mixed failures. A real Cargo fixture checks independent feature resolution,
  cold and warm batch metrics, and retained artifact paths. Warm publication
  checks cover isolated, shared, and scheduled-maintenance modes.
- Targeted tests, Rust 1.88 compilation, Clippy, formatting, and diff checks pass.
  Full pre-push validation remains maintainer-owned.

## [0.15.5] - 2026-10-05 - Cargo build-mode and artifact-boundary validation

### Fixed

- Wasm acquisition rejects Cargo's `--unit-graph` mode before input resolution
  or cache acquisition. This mode exits successfully after printing a build
  graph without compiling; previously, shared-target acquisition could publish
  an existing Wasm output under the graph invocation's fingerprint.
- Help and build-graph flags share the existing early validation path. Public
  API signatures, dependencies, and persisted layouts are unchanged;
  repository-owned formats remain `v1`.
- Profile output directories must be one normal path component. Absolute paths,
  parent traversal, and nested paths are rejected before resolution or
  acquisition, keeping expected artifacts inside their target directories.
  Standard and custom profile names remain accepted.

### Testing

- Reproduced stale-output publication with real nightly Cargo before fixing.
  The maintained regression runs without nightly and checks early rejection,
  absence of cache publication, and preservation of existing shared outputs.
- A failing profile-boundary regression was verified before fixing. Focused
  checks cover invalid paths, accepted profile names, no filesystem side effects,
  and compatible batch input resolution.
- Focused Cargo argument checks, Rust 1.88 compilation, Clippy, documentation,
  formatting, and diff checks pass. Full pre-push validation remains
  maintainer-owned.

## [0.15.4] - 2026-10-05 - Simpler benchmark processing and cache pruning

### Changed

- Captured stdout and stderr append directly to one benchmark parse report
  through the existing parsing loop. The temporary stderr report and its vector
  and counter merge are removed. Events and diagnostics retain stdout-then-stderr
  order, per-stream line numbers, original malformed lines, and strict-mode rules.
- Benchmark aggregation borrows span labels for its temporary map, then owns
  labels in the returned rows or overflow errors. It no longer allocates a label
  key for every span update or passes a second copy of the span's label to the
  aggregation helper. Scope ordering, named `ALL` suites, checked totals, run
  counts, extrema, and peak-end counters are preserved.
- Benchmark run allocation and previous-run discovery share one numeric index
  parser. Discovery no longer reconstructs an already-split filename prefix to
  strip it again. Minimum width, ASCII digits, numeric ordering, overflow
  rejection, and metadata-based candidate ranking are preserved.
- Size-based artifact-cache pruning skips sorting when retained bytes already
  fit the budget, including after age-based removal. Above-budget pruning still
  uses last-use time and path order, respects protected and retained entries,
  and reports the same scanned and retained totals.
- Public APIs, dependencies, and persisted layouts are unchanged;
  repository-owned formats remain `v1`.

### Testing

- Focused parser checks cover both streams, strict and normal parsing, blank and
  empty inputs, malformed markers, source positions, and diagnostic ordering.
- Aggregation checks cover repeated spans, named and global scopes, numeric
  boundaries, and overflow diagnostics for every counter and scope.
- Run-discovery checks cover malformed indices, five-digit numeric ordering,
  metadata selection, and index exhaustion. Pruning checks cover exact and
  under-budget retention, least-recently-used removal, and active-entry protection.
- Report-output checks, Rust 1.88 compilation, Clippy, documentation, formatting,
  and diff checks pass. Full pre-push validation remains maintainer-owned.

## [0.15.3] - 2026-10-04 - Build validation, bounded progress, and simpler comparisons

### Fixed

- Wasm acquisition rejects Cargo help flags before input resolution or cache
  acquisition. A help-only invocation can exit successfully without building;
  previously, shared-target acquisition could publish an existing Wasm output
  under that invocation's fingerprint. Long and clustered short help flags are
  rejected, while feature values containing `h` remain accepted.

### Changed

- Benchmark comparison uses one ordered map for current and previous rows,
  removing duplicate indexes, a temporary key list, and a separate sorting pass.
  Ordering, missing rows, last-duplicate handling on both sides, and averages
  derived from current totals and run counts are preserved. Named `ALL` suites
  remain distinct from the all-suites aggregate.
- Observed Cargo builds use a bounded queue between stdout/stderr readers and
  the progress observer. Slow callbacks apply backpressure instead of allowing
  an unbounded pending-chunk backlog. Both streams continue to drain, raw bytes
  remain intact, and full diagnostics are captured even with forwarding disabled.
  Diagnostic capture itself is not truncated.
- Public API signatures, dependencies, and persisted layouts are unchanged;
  repository-owned formats remain `v1`.

### Testing

- A real-Cargo regression reproduced stale shared-target publication before the
  fix. Focused checks cover early argument rejection, compact feature arguments,
  dependency resolution, benchmark ordering, duplicates, missing and empty rows,
  updated averages, and comparison CSV scope handling.
- Output checks cover streams larger than the queue, disabled forwarding,
  consumer disconnection, interrupted reads, permanent read errors, quiet-period
  progress events, and complete failing-Cargo diagnostics with exit code 23.
- Rust 1.88 compilation, Clippy, documentation, formatting, and diff checks pass.
  Full pre-push validation remains maintainer-owned.

## [0.15.2] - 2026-10-04 - Bounded cache metadata and marker parsing

### Fixed

- Transactional artifact-cache manifests use the shared bounded file reader.
  Their size limit comes from the existing manifest writer and declared output
  set, retaining one authoritative layout. Oversized manifests are rejected
  before scanning output contents, without reading or allocating the entire file.
- Corrupt manifests still take the normal rebuild path. Entries retained by live
  consumers continue to fail closed until released; output-content verification,
  undeclared-output rejection, and atomic publication are preserved.
- The shared byte reader preserves malformed-manifest recovery, while UTF-8
  stamp readers retain their decoding errors. Public APIs, dependencies, and
  persisted layouts are unchanged; repository-owned formats remain `v1`.
- Last-use and maintenance markers also use bounded reads. Oversized last-use
  markers fall back to directory modification time; oversized maintenance markers
  make maintenance due. Normal timestamps, changed-policy scheduling, LF/CRLF
  handling, future-time behavior, and each caller's I/O error policy are preserved.

### Changed

- Benchmark markers validate their six fields with a fixed array instead of
  collecting a dynamic list of every column. Extra columns are rejected without
  collecting the remainder; counter parsing, diagnostic text, original malformed
  lines, source positions, and continued parsing of later markers are preserved.
- Destination-stamp comparison uses the shared bounded file reader. Independent
  writable-file, ownership, permission, and link checks remain in place; matching
  stamps are still preserved during warm publication.

### Testing

- Focused checks cover oversized manifests, retained-entry protection, malformed
  recovery, multi-output publication, empty-file policy, watched-input stamps,
  Wasm stamp validation, and reconstruction from verified public outputs.
- Marker checks cover bounded fallback, valid and malformed timestamps,
  maintenance intervals, policy changes, CRLF, future timestamps, and read errors.
- Benchmark checks cover malformed column counts, source diagnostics, valid
  markers after malformed input, captured streams, and strict parsing. Warm Wasm
  checks cover file preservation and replacement of linked or restricted files.
- Rust 1.88 compilation, Clippy, documentation, formatting, and diff checks pass.
  Full pre-push validation remains maintainer-owned.

## [0.15.1] - 2026-10-04 - Transport recovery and bounded freshness checks

### Fixed

- PocketIC transport classification follows the contained error inside
  `std::io::Error`, including nested wrappers. Contextual typed Candid failures
  now retain their transport classification when callers wrap them for reporting.
- Diagnostic failures already classified as `InstanceUnavailable` are recognized
  by the shared transport classifier, both directly and through error wrappers.
  Controller rejections, decode failures, and unrelated application panics remain
  distinct from transport failures.
- Watched-input freshness and Wasm-cache validation use one bounded stamp
  reader, reading at most the expected stamp length plus one byte. Oversized
  sidecars are stale without reading or allocating their full contents. Exact
  matching, artifact-content verification, and reconstruction from verified
  public outputs remain intact. Watched-input checks preserve missing-stamp
  handling, I/O errors, and `InvalidData` for invalid UTF-8 within the size limit.
  Truncated Wasm stamps are rejected before scanning artifact contents.

### Testing

- Both classification gaps were reproduced before fixing. Focused unit tests,
  a refused-server subprocess check, and live PocketIC 16 diagnostic authorization
  checks pass. Rust 1.88 compilation, Clippy, documentation, formatting, and diff
  checks pass; full pre-push validation remains maintainer-owned.
- Focused artifact checks cover missing, truncated, mismatched, invalid UTF-8,
  and oversized stamps, plus normal stamp publication and input changes.
  Wasm-cache recovery from an oversized stamp reuses verified public outputs
  without running Cargo; warm publication and copied-byte verification are checked.
- Public APIs, dependencies, and persisted cache formats are unchanged.

## [0.15.0] - 2026-10-04 - Runtime fixture capacity and one reset-policy model

### Changed

- **Breaking:** `CachedStandaloneCanisterFixturePool` takes a runtime
  `NonZeroUsize` capacity through `new(capacity, builder)`. Its capacity const
  generic is removed; capturing builders use the remaining builder type
  parameter. `capacity()` exposes the configured limit. Static construction
  stays const and fixture construction stays lazy.
- Standalone and multi-canister pools allocate slots through one lazy shared
  scheduler. The standalone initialization wrapper and duplicate capacity field
  are removed. `CachedPocketIcBaselinePool::new` and `capacity` are now const;
  unused pools do not allocate slot storage. FIFO waiting, unwind invalidation,
  phase timings, and each pool's distinct restore/recovery contract are preserved.
- **Breaking:** `ResetDomainPolicy` replaces both `ResetRequirement` and
  `ResetAchievement`. Required declarations and achieved receipts remain distinct
  `ResetRequirements` and `ResetReceipt` types. One constructor helper owns
  duplicate-domain checks, and verification still rejects missing domains,
  mismatched policies, and incorrect cycle or canister restore evidence.
- The opt-in fixture benchmark selects flows with `--modes fresh pooled` and
  arbitrary positive pool limits with `--capacities 1 2 4 8`. All cases retain
  the same task and worker counts, rotate across repeats, validate restored state,
  and report wait, preparation-inclusive time, throughput, and sampled process-tree
  RSS. The defaults still compare fresh fixtures with capacities one and two.
- Dependencies and persisted cache layouts are unchanged; repository-owned
  format identifiers remain `v1`. These are hard source/API cuts without aliases
  or compatibility adapters; the package changelog contains migration examples.

### Fixed

- Cache-directory tag checks read only the standard signature prefix from an
  ordinary file. Valid tags are preserved, including CRLF and additional comments;
  invalid tags and symlinks are replaced without changing linked referents.
  Oversized tag contents no longer require an unbounded allocation.

### Testing

- Targeted scheduler and fixture checks cover concurrent first acquisition,
  bounded overlap, FIFO waiting, waiter cancellation, explicit invalidation,
  panic propagation, and failed restoration/recovery. Reset checks cover all five
  non-snapshot domains, matching and missing receipts, policy mismatches, duplicate
  declarations/receipts, and exact restore evidence.
- Live PocketIC 16 capacity sweeps validate restored state and subsequent
  mutations. These are development smoke checks, not downstream performance
  claims. Rust 1.88 compilation, focused Clippy, formatting, and diff checks pass;
  full pre-push validation remains maintainer-owned.

## [0.14.12] - 2026-10-04 - Consistent Cargo inputs and bounded startup readiness

### Fixed

- Managed PocketIC startup reads at most 65 bytes from its port file and rejects
  files larger than 64 bytes with bounded diagnostics. Missing files and partial
  writes remain pending; valid nonzero ports, UTF-8 errors, owned-child teardown,
  and private startup-file cleanup retain their behavior.
- Cargo metadata recognizes grouped short feature options such as `-qFextra`,
  `-rF=extra`, and `-vF extra`. Enabled optional dependencies enter input discovery
  and mutation guards, and batched resolution uses the same feature context.
- Symlinked Cargo configuration resolves relative includes beside the configured
  entry, matching Cargo, rather than beside its referent. Each lookup location
  retains its nested include paths; guards detect file and directory symlink
  replacement even when the top-level configuration bytes remain identical.
  Duplicate lookup paths are suppressed and recursive cycles terminate.

### Changed

- `WasmBuildSpec` rejects profile arguments that override its workspace, package
  selection, compilation target, or configuration before input resolution or
  cache acquisition. Long, attached, and grouped short options are covered.
  Use the specification's workspace/packages, `with_target`, discovered Cargo
  configuration files, and `with_extra_env`; `--config` overrides are unsupported.
  Existing target-directory ownership checks remain enforced.
- Startup unit fixtures share the artifact tests' executable-script writer,
  which finishes writes in a child process before execution. This removes a
  separate fixture-writing path exposed to parallel Unix `ETXTBSY` failures.
- Public API signatures, dependencies, and persisted layouts are unchanged;
  owned format identifiers remain `v1`. Corrected feature/configuration inputs
  can change fingerprints and cause a fresh build without a cache migration.

### Testing

- Oversized readiness files, Cargo input-override validation, and symlinked
  configuration discovery regressions were reproduced before fixing them.
  Focused checks cover partial writes, bounded diagnostics, child cleanup,
  grouped features, optional dependency mutation, shared config referents,
  nested directory aliases, duplicate discovery, and cycle termination.
- Existing observed-output fixtures, executable resolution, batched input reuse,
  semantic workspace projection, and warm source-mutation rejection pass targeted
  checks. Clippy, formatting, and diff checks pass. No performance improvement
  has been measured; full pre-push validation remains maintainer-owned.

## [0.14.11] - 2026-10-04 - Canonical package selection and reliable discovery

### Fixed

- `resolve_executable` continues searching `PATH` when a search component is a
  regular file rather than a directory. Missing, non-executable, and directory
  candidates also leave later valid executable candidates eligible.
- Absolute executable paths resolve independently of the current directory,
  including when that directory has been removed. Relative paths still require
  the current directory, and explicitly supplied invalid paths retain their
  errors instead of falling back to `PATH`.
- Benchmark run allocation and previous-run discovery inspect the runs directory
  directly instead of probing existence first. Missing roots still begin at
  index one or return no previous run. Directory errors other than `NotFound`,
  including non-directory parents on Unix, propagate instead of being silently
  treated as missing roots.

### Changed

- `WasmBuildSpec` owns sorted, unique package names. Fingerprinting and artifact
  discovery no longer clone and normalize the package list separately; input
  resolution and Cargo invocation use the same list. `packages()` now returns
  canonical order, and repeated names produce only one `-p` argument per package.
  Existing fingerprint bytes and artifact ordering are preserved.

### Testing

- Both executable-resolution regressions were reproduced against the released
  implementation before fixing them. Three focused Unix tests cover absolute,
  relative, and empty `PATH` entries, unusable search components, missing programs, explicit-path
  errors, and a removed current directory in an isolated child process.
- A benchmark root-path regression was reproduced before fixing it. Focused
  checks distinguish missing roots from invalid directory paths and preserve
  previous-run selection and numeric index allocation. Package-selection checks
  use real Cargo input resolution and capture actual arguments in both silent
  and observed builds; existing checks cover workspace projection, batched input
  reuse, empty selections, and cold/warm artifact acquisition.
- Targeted tests, Clippy, formatting, and diff checks pass. Public API signatures,
  dependencies, fingerprints, and persisted layouts are unchanged; owned format
  identifiers remain `v1`. Existing callers need no source or cache migration.
  No performance improvement has been measured. Full pre-push validation
  remains maintainer-owned.

## [0.14.10] - 2026-10-04 - Safer atomic artifact publication

### Fixed

- Atomic publication only cleans up a temporary file after exclusive creation
  succeeds. A temporary-name collision leaves the existing file and destination
  intact and returns the creation error rather than deleting an unowned file.
- Atomic writes and copies use short sibling temporary names instead of
  appending a suffix to the destination filename. Valid long destination names
  can be published without exceeding the filesystem's filename limit. Temporary
  names stay distinct from the destination, including ASCII case variants.
- Transactional publication and cache-hit repair can replace cyclic final
  output symlinks. Validation no longer fails solely because that link's
  referent cannot be inspected. Directory targets, invalid parents, and
  input/output overlaps remain rejected; input and retained artifact bytes
  remain protected.

### Changed

- Cache maintenance marker writing delegates directory creation to the shared
  atomic writer, removing a duplicate filesystem step and error branch.
- These fixes apply to Wasm and transactional artifacts and cache metadata.
  Public APIs, fingerprint bytes, dependencies, and persisted layouts are
  unchanged; repository-owned format identifiers remain `v1`. No performance
  improvement has been measured for this release.

### Testing

- Publication regressions were reproduced against the released implementation
  before fixing them. Focused regressions cover collision ownership, subsequent
  publication, destination/temp-name separation, long filenames, and cleanup
  after write or rename failures. Existing symlink tests also cover cold
  replacement of cyclic links, warm repair of cyclic and dangling links, and
  rejection of cyclic parents and unsafe nested outputs. Transactional
  acquisition, warm Wasm reuse, partial-publication failures, maintenance
  scheduling and failure handling,
  and retained cross-process handoffs are checked.
- Targeted tests, Clippy, formatting, and diff checks pass. Full pre-push
  validation remains maintainer-owned.

## [0.14.9] - 2026-10-04 - Authoritative Wasm entries and selective publication

### Fixed

- A verified retained Wasm entry now owns the bytes for its build fingerprint.
  Public files with independently valid stamps but different bytes are repaired
  from that entry rather than left inconsistent with the returned artifact record.
- Recovery from mutable public files hashes the newly copied private files before
  constructing their stamps. Empty copies fail acquisition and remove the
  incomplete entry instead of publishing an invalid reconstructed cache entry.

### Changed

- Wasm validation retains verified lengths and digests for publication from
  exact entries. Public stamp construction no longer rehashes just-copied outputs.
  Warm acquisitions repair each public file and stamp independently; a missing
  or damaged stamp alone no longer forces a Wasm copy. On Unix, matching independent writable files
  owned by the effective user preserve their inode, modification time, and
  permissions. Linked, foreign-owned, restricted, and executable destinations
  are atomically replaced. Cold publication and non-Unix materialization use
  atomic replacement.
- Transactional and Wasm artifact publication share the destination ownership
  checks and one private file-digest representation. Public artifacts remain
  usable to reconstruct a missing or invalid exact entry, subject to retention
  locks. Complete stamp verification, input revalidation, prepared-reader
  invalidation, producer locks, and all-copies-before-public-stamps ordering
  remain enforced.
- Public APIs, fingerprint bytes, and persisted layouts are unchanged; owned
  format identifiers remain `v1`. Callers must coordinate other writers to
  mutable public paths and treat retained artifacts as read-only. No runtime
  or downstream suite speedup has been measured for this release.

### Testing

- The conflicting-public-bytes regression fails against separately compiled
  released `0.14.8` and passes with the fix. Focused checks cover selective
  repairs, matching inode/time preservation, link and permission normalization,
  partial-copy failures, exact-entry reconstruction and retained corruption,
  changed and empty public recovery sources, concurrent prepared readers, source
  races, and cross-process handoffs.
- Forty-five targeted unit checks and ten integration checks pass, including
  exact-build coordination and scheduled shared-target maintenance. Clippy,
  formatting, and diff checks pass; full pre-push validation remains
  maintainer-owned.

## [0.14.8] - 2026-10-04 - Verified artifact reuse and large-file hashing

### Changed

- Transactional cache hits retain the output lengths and digests already
  computed during complete cache-entry validation and use them to verify public
  destinations. On Unix, matching regular files owned by the effective user,
  with one link, owner read/write permissions, and no executable or special
  permission bits stay in place, avoiding redundant atomic copies. Their inode,
  modification time, and permissions are preserved; a hit does not guarantee a
  new modification time.
- Missing, changed, linked, foreign-owned, and restricted outputs still receive
  atomic replacement. Cold commits and non-Unix acquisitions retain their
  existing publication behavior. Complete cache schema and content validation,
  input revalidation, lock ordering, and artifact retention are preserved.
- Streamed file hashing uses a heap buffer sized to the opened file's length,
  between one byte and 64 KiB, preserving the thread-stack budget and avoiding
  a full-size buffer for small sources. Empty files still receive a nonempty
  read buffer to detect growth. Reading past the declared length fails
  immediately; short reads at EOF remain invalid. Digest framing and cache
  identities are unchanged.
- An earlier fixed-64-KiB-buffer comparison found about 4–5% lower hashing cost
  for 2.60/17.60 MiB files and 0.4–0.5 microseconds more for empty and 1/16 KiB
  files. Follow-up timings for the length-sized buffer varied substantially
  under high host load, so they do not establish a further speedup.
- The performance guide records a released `0.14.7` baseline and paired output
  reuse measurements. Matching-output acquisitions averaged 15.70 to 10.48 ms
  for 2.60 MiB artifacts and 77.55 to 59.09 ms for 17.60 MiB artifacts, about
  33% and 24% faster in the controlled probe. Same-size changed outputs were
  10–13% slower because verification precedes copying. Missing and different-size
  repairs stayed within about 3% of baseline. These are component measurements,
  not a measured downstream test-suite speedup, and precede the small-file
  buffer-sizing and label-ordering follow-ups.
- Public APIs and persisted layouts are unchanged. Repository-owned format
  identifiers remain `v1`; existing `0.14` callers need no source or cache
  migration. Callers remain responsible for coordinating other writers to
  mutable public destinations.
- Artifact-spec builders canonicalize identity and Cargo input labels alongside
  output names. Key calculation uses their stored order without repeatedly
  allocating and sorting reference lists. Reordered declarations now compare
  equal as specs; key bytes, duplicate-label rejection, and Cargo input guards
  are preserved. No performance gain is claimed for this ownership cleanup.

### Testing

- Thirty-eight targeted unit and integration checks pass, covering unchanged
  public inode/time, equal-length corruption, missing and explicitly valid empty
  outputs, matching symlinks and hard links, restricted permissions, cold
  publication, corrupt retained entries, Cargo input guards, batch reuse,
  cross-process coordination, retention during pruning, native digest identity,
  exclusion semantics, and streamed hashing/copying with a partial final chunk.
- Existing transaction tests additionally cover reordered identity and Cargo
  input declarations and duplicate Cargo labels rejected before cache creation.
- The streaming regression covers empty and small files, both sides of the
  buffer-size boundary, and a partial final chunk. A local read-only probe
  verifies that a file declaring zero length but yielding bytes fails closed.
- A focused read-only probe of a foreign-owned matching file reproduced the
  missing ownership check and passed after the fix. The probe is not retained as
  a host-file-dependent CI test.
- The earlier 1,440-acquisition paired experiment reused the expected key and
  returned identical public and retained bytes. Fifty direct-hashing batches covered
  302,000 operations, with every batch's final digest matching the baseline.
  Raw measurements and probe sources are retained in local measurement bundles.
- A further 75 hashing batches covered 453,000 operations with matching final
  digests across released, fixed-buffer, and length-sized-buffer implementations.
  Their timings are excluded from performance claims because of host-load variance.
- Targeted Clippy, formatting, and diff checks pass; full pre-push validation
  remains maintainer-owned.

## [0.14.7] - 2026-10-04 - Borrowed Cargo metadata and leaner artifact validation

### Changed

- Cargo metadata parsing, package selection, dependency traversal, and semantic
  projection borrow identifiers and strings from the authoritative metadata
  document instead of copying them into each intermediate collection. Package
  selection checks for zero, one, or multiple matches without collecting a
  temporary match list. Dependency closure, feature isolation, malformed-input
  errors, conservative fallback, and exact fingerprints retain their behavior.
- Baseline-derived canister restore receipts collect the already sorted,
  duplicate-checked snapshot IDs directly instead of rebuilding a set on every
  restoration. Empty receipt sets are still rejected, and caller-supplied
  receipts retain complete duplicate validation and deterministic ordering.
- Artifact schema validation checks output names directly against the declared
  output count and canonical filename formatter instead of rebuilding expected
  filename sets during commit and cache lookup. Root-entry checks use borrowed
  names. Strict filename spelling, undeclared-file rejection, regular-file
  checks, complete failure diagnostics, and filesystem errors are preserved.
- Unit and integration tests share one executable-fixture writer. Integration
  fixtures now receive the same child-process writing and launch coordination
  that prevents inherited writable script handles from causing intermittent
  Linux "Text file busy" failures. The duplicate writer is removed; real Cargo
  wrapper coverage and production command execution are preserved.
- Public APIs and persisted layouts are unchanged. Repository-owned format
  identifiers remain `v1`; existing `0.14` callers need no migration. No
  downstream suite speedup has been measured for these changes.

### Testing

- Twelve targeted unit checks pass, covering metadata reuse, semantic workspace
  projection and fallback, registry checksum identity, optional dependencies,
  source mutation guards, indexed batch failures, executable resolution, and
  caller-supplied receipt ordering and validation.
- Ten artifact-validation unit checks pass, covering staged and cached
  undeclared names, signed and noncanonical indices, integer overflow, native
  non-UTF-8 names on Unix, complete output sets, malformed manifests, retention,
  source mutation rejection, cleanup, and explicit empty-file validation.
- Ten artifact/Wasm integration checks pass, covering feature isolation,
  concurrent prepared readers, source-race rejection and invalidation,
  transactional Cargo input guards, cold and warm artifact retention,
  cross-process pruning and exact-build coordination, terminated consumers, and
  partial batch success.
- Six live PocketIC 16 baseline-pool checks pass, including exact derived
  receipt equivalence, empty-set rejection, receipt mismatch recovery, capacity
  overlap, explicit invalidation, failed recovery diagnostics, and 100
  consecutive restores without reconstruction.
- Targeted Clippy, formatting, diff, and source-package inclusion checks pass;
  full pre-push validation remains maintainer-owned.

## [0.14.6] - 2026-10-04 - Shared build inputs and explicit pool state

### Changed

- Resolved Cargo input snapshots share immutable input and exclusion lists
  across clones instead of copying every path. Sessions and prepared readers
  retain independent timing values; session reuse also removes a redundant
  timing reset. Source revalidation and lease invalidation are unchanged.
- Wasm batch resolution borrows group membership and filters pending entries
  directly, removing temporary index lists and a copied workspace path.
  Environment grouping, independent feature resolution, indexed failures, and
  progress reporting retain their existing behavior.
- Labeled-path hashing accepts borrowed or owned paths and collects them once
  for deterministic sorting. Transactional artifacts borrow filesystem paths,
  and watched ICP inputs borrow labels, removing caller-side temporary lists.
  Native path ordering, exclusions, digest framing, and cache identities are
  unchanged.
- Artifact input and tool labels share one validation rule and use borrowed
  namespaced membership keys. Duplicate labels remain invalid within each
  namespace; an input and a tool may still share the same label. Validation
  continues to precede cache acquisition.
- Bounded fixture pools represent empty, reusable, invalidated, and unwound
  slots explicitly instead of combining an optional value with validity flags.
  Panic recovery retains its cause even before a slot is populated, and stale
  values remain available for safe teardown. FIFO scheduling, cancellation,
  capacity, explicit invalidation, and restore-failure recovery are preserved.
- Public APIs and persisted layouts are unchanged. Repository-owned format
  identifiers remain `v1`; existing `0.14` callers need no migration. No
  downstream suite speedup has been measured for these changes.

### Testing

- Executable unit-test fixtures are written by a child process and awaited
  before launch, preventing concurrent test subprocesses from inheriting a
  writable script handle and causing intermittent Linux "Text file busy"
  failures. Cargo wrappers still exercise real tool identities, metadata,
  invocation counts, and build paths; production command execution is unchanged.
- Thirty-one targeted unit checks pass, covering input resolution, batch
  failures and grouping, pool scheduling and unwind recovery, native digest
  semantics, watched-input stamps, label validation, identity dimensions, and
  transactional publication and source guards.
- Seven artifact/Wasm integration checks pass, covering feature isolation,
  concurrent prepared readers, source-race rejection and invalidation,
  transactional Cargo input guards, cross-process build coordination, and
  preservation of successful entries through later batch failures.
- Ten live PocketIC 16 checks pass across generic and standalone pools,
  including 100 consecutive restores without reconstruction, capacity-scoped
  overlap, explicit invalidation, caller and recipe-hook panics, and failed
  restore and recovery behavior.
- Targeted Clippy, formatting, and diff checks pass; full pre-push validation
  remains maintainer-owned.

## [0.14.5] - 2026-10-04 - Simpler build inputs, fixture reuse, and reporting

### Changed

- Wasm batch resolution borrows caller-owned build specifications instead of
  cloning every specification into a temporary input list. Grouping keys also
  borrow workspace paths and Cargo/rustc executable names. Sessions and prepared
  snapshots retain the owned identities needed across calls; metadata argument
  filtering, environment capture, independent feature resolution, and source
  lease invalidation are unchanged.
- Cargo input discovery owns shared-target boundary validation and generated
  directory exclusions for both standalone builds and batches. Callers no
  longer repeat this safety sequence; validation still precedes hashing,
  maintenance, and builds. Unsafe batch entries retain their input-discovery
  failures without preventing compatible entries from succeeding.
- Warm baseline-pool acquisition compares restore receipts directly against
  captured canister IDs instead of allocating an expected-ID list on every
  successful restore. Mismatch diagnostics still retain both complete ID lists.
  Successful preparation also stops clearing an already-empty invalidation
  reason; lifecycle metadata is only changed when building or invalidating a
  slot. Reset checks, readiness, final validation, and recovery are unchanged.
- Canister diagnostics derive the omitted-record count from total and retained
  records instead of storing and updating a second count. Raw byte totals are
  accumulated during rendering, removing a separate traversal. Record and byte
  bounds, zero limits, upstream ordering, lossy UTF-8 conversion, and compact
  truncation text retain their existing behavior.
- Previous benchmark-run discovery retains only the latest eligible match
  instead of collecting and sorting every candidate. Metadata timestamp
  priority, numeric run-index ordering, command filtering, and skipping
  unreadable or malformed metadata retain their existing behavior.
- Public APIs and persisted layouts are unchanged. Repository-owned format
  identifiers remain `v1`; existing `0.14` callers need no migration. No
  downstream suite speedup has been measured for these changes.

### Testing

- Sixteen focused batch/input-resolution unit checks and four real Wasm build
  checks pass, covering indexed failures, progress labels, maintenance ownership,
  compatible metadata reuse, source and retained-cache boundaries, relative
  target paths, feature isolation, session reuse, concurrent prepared readers,
  and source-race rejection and invalidation.
- Five diagnostics unit checks and three live PocketIC 16 checks pass, covering
  empty records, independent zero bounds, raw-byte accounting with lossy UTF-8,
  truncation output, exact senders, and preservation of original install failures
  and caller-owned instances.
- Six focused live PocketIC 16 baseline-pool checks pass, including 100
  consecutive restores without reconstruction, exact receipt coverage,
  explicit invalidation, failed recovery diagnostics, and propagation of caller
  and recipe-hook panics before a later rebuild.
- Three focused benchmark checks pass, covering prior-run discovery, numeric
  indices beyond four digits, command filtering, metadata timestamp priority,
  missing and malformed candidates, and the metadata object schema.
- Targeted Clippy, formatting, and diff checks pass; full pre-push validation
  remains maintainer-owned.

## [0.14.4] - 2026-10-04 - Simpler snapshot capture and artifact input ownership

### Changed

- Snapshot capture owns duplicate validation and deterministic canister ordering
  in one shared preflight step. Both capture APIs validate the complete input
  before making management calls, without separate validation helpers or
  temporary sender vectors. Explicit senders still receive exactly one attempt;
  controller capture retains its ordered fallback attempts.
- Failed snapshot capture transfers its partial snapshot set to rollback,
  avoiding snapshot-ID copies that had no remaining consumer. Cleanup still
  attempts every captured snapshot and retains rejection and panic diagnostics.
- Wasm input discovery keeps one authoritative path list. Semantic hashing
  filters a borrowed view of that list instead of storing a copied subset and
  a separate fingerprint-mode wrapper. Conservative fallback for workspace-root
  and unsupported package projections is preserved.
- Resolved Cargo input revalidation hashes borrowed labels and paths directly,
  removing another temporary list of owned copies. Full raw-input mutation
  guards, native path ordering, exclusions, semantic digests, and exact cache
  identities retain their existing behavior.
- Public APIs and persisted layouts are unchanged. Repository-owned format
  identifiers remain `v1`; existing `0.14` callers need no migration. No
  downstream suite speedup has been measured for these changes.

### Testing

- Seven focused live PocketIC 16 checks and one funding-policy unit check pass.
  Coverage includes duplicate rejection before capture, deterministic first
  failure, explicit-sender rejection without fallback, controller fallback
  order, partial-capture rollback, explicit restore funding, and reuse through
  both multi-canister and standalone pools. Public behavior checks replace the
  removed private validation-helper tests.
- Six digest checks and seven artifact-input checks pass, covering exact native
  digest semantics, relevant exclusions, semantic workspace projection and
  conservative fallback, source revalidation, warm-hit source races, batched
  resolution, and transactional raw-input guards.
- Targeted Clippy, formatting, and diff checks pass; full pre-push validation
  remains maintainer-owned.

## [0.14.3] - 2026-10-04 - Leaner benchmark processing and cache finalization

### Changed

- Benchmark pairing borrows input markers and grouping keys rather than copying
  complete events into temporary stacks. Only the resulting spans and
  diagnostics retain owned data; nested pairing, suite boundaries, and
  deterministic diagnostic ordering are preserved.
- Benchmark aggregation keeps group identity in its map key instead of also
  storing it in each accumulator. Comparison and Markdown lookups borrow row
  keys, and comparison removes a redundant deduplication pass. Overflow errors,
  row ordering, missing-row handling, and duplicate-row precedence are unchanged.
- Cold Wasm builds and reconstruction of missing exact-cache entries share one
  finalization path. Successful entries are preserved; failures clean up partial
  entries while retaining the original error, cleanup diagnostics, and timings.
- Canister installation retains its existing label for failure diagnostics
  without cloning it before every installation. Install failures still preserve
  the original cause and caller-owned PocketIC instance.
- Digest formatting writes hexadecimal directly to its destination instead of
  constructing an intermediate string. The owned-string API uses the same
  formatter; lowercase text, leading zeroes, and cache identities are unchanged.
- Public APIs and persisted layouts are unchanged. Repository-owned format
  identifiers remain `v1`; existing `0.14` callers need no migration. These
  changes have no measured downstream suite speedup.

### Testing

- Twenty-five benchmark integration checks and one focused overflow check pass,
  including owned diagnostics, suite isolation, comparison ordering, missing
  rows, zero denominators, and duplicate-row precedence.
- Three focused Wasm cleanup/reconstruction checks, four artifact handoff
  checks, and two live PocketIC 16 install-failure checks pass. They cover partial
  output cleanup, successful warm reconstruction, retained artifacts, and
  preserved installation diagnostics and instance ownership.
- Six digest checks and two stamp/manifest checks pass, covering exact
  hexadecimal output, native filenames, exclusions, streaming copies, and
  persisted artifact validation. Targeted Clippy, formatting, and Wasm compile
  checks pass; full pre-push validation remains maintainer-owned.

## [0.14.2] - 2026-10-04 - Fixture reuse measurements and simpler ownership

### Added

- An opt-in Linux fixture benchmark compares fresh PocketIC instances with
  baseline pools of capacity one and two. Each task validates restored state,
  mutates two canisters, and checks the result; pooled measurements require warm
  restores and stop on unexpected rebuilding or recovery.
- The benchmark records phase timings, setup and teardown costs, sampled
  process-tree RSS, raw samples, and build/workload provenance. Selected modes
  and rotating repeat order support longer pooled comparisons. It uses a
  caller-supplied PocketIC 16 binary and runs outside the ordinary test/CI gate.

### Changed

- Standalone pools derive rebuild reasons from the shared slot state instead of
  storing a second invalidation reason. Restore failures still invalidate the
  slot, and panic invalidation retains its distinct rebuild outcome.
- Wasm build records obtain their exact cache path from the retention owner,
  removing a duplicated path allocation. Records and their clones retain the
  same immutable artifacts until their last drop.
- Public library APIs and persisted layouts are unchanged. Repository-owned
  format identifiers remain `v1`; existing `0.14` callers need no migration.

### Fixed

- Observed Cargo output tests use the existing system shell to read their
  fixture scripts, avoiding intermittent Linux `ExecutableFileBusy` / "Text
  file busy" errors when launching freshly written executables under parallel
  test load. Raw output, failure diagnostics, exit events, and quiet build
  progress notifications remain tested through real subprocesses; production
  Cargo execution is unchanged.

### Documentation

- Documents the benchmark workload, measurement boundaries, memory tradeoffs,
  and controlled PocketIC 16 results. With 100 tasks, two workers, and three
  repeats, median fixture time including preparation and teardown was 10.93 s
  at capacity one and 7.38 s at capacity two; sampled peak process-tree RSS was
  559 MiB and 770 MiB respectively. These compare pool capacities, not library
  versions or downstream suite performance.

### Testing

- Seven targeted live PocketIC 16 checks pass for standalone reuse, overlapping
  leases, capacity waits, restore-failure rebuilding, and panic recovery.
- Four artifact handoff checks pass, covering cloned cold/warm records,
  cross-process retention, terminated consumers, pruning, and batch failures.
  Two focused Python checks verify process-tree RSS boundaries and kernel stat
  parsing. Targeted Clippy and formatting checks pass; full pre-push validation
  remains maintainer-owned.
- Five focused output/progress checks pass. The two affected subprocess tests
  also pass 500 repeated runs with eight concurrent runners (1,000 test
  executions), after reproducing the executable-file race before the fix.

## [0.14.1] - 2026-10-03 - Leaner artifact validation and pool acquisition

### Changed

- Wasm cache stamps and transactional artifact manifests reject missing or
  mismatched format/build identities before hashing artifact contents. Matching
  entries still require full content validation and an exact stamp or manifest;
  corrupted entries remain subject to active-consumer retention locks.
- Artifact input hashing borrows declared paths and Unix-native filename bytes
  instead of copying them. Directory traversal computes each filename sort key
  once. Native ordering, non-UTF-8 Unix names, Windows UTF-16 little-endian
  encoding, and existing digest identities are preserved.
- Composable digest-cache lookups compare borrowed exclusion paths directly,
  avoiding temporary cloned lists on cache hits. Descendant and ancestor
  exclusions remain significant, and inputs following external symlinks retain
  their conservative exclusion checks.
- Cache size scans count files and symlinks immediately and queue only
  directories. Logical-size accounting, sparse files, missing-path errors, and
  symlink handling retain their existing behavior.
- Bounded fixture pools register a FIFO waiter and select an available slot
  under one coordinator lock. Optional tickets replace separate ticket/active
  state, and cancellation guards release the coordinator before cleanup during
  unwinding. Capacity, FIFO order, cancellation wakeups, and slot invalidation
  remain enforced.
- Public APIs and persisted cache layouts are unchanged. Repository-owned
  cache, stamp, and digest identifiers remain `v1`.

### Testing

- Adds focused native-filename and exact digest-order coverage, stamp identity
  and same-size content corruption checks, malformed-manifest recovery with live
  retention, and directory-size coverage for sparse files, external links,
  dangling links, and symlink cycles.
- Verifies relevant exclusion changes rehash cached roots, excluded input roots
  are rejected even after a cache hit, and exclusions beyond external symlinks
  cannot reuse an incompatible digest.
- Covers FIFO head-waiter cancellation, capacity, panic invalidation, warm source
  mutation rejection, pruning, and cross-process artifact handoff. PocketIC 16
  checks verify standalone reuse and 100 consecutive baseline restores.
- Targeted checks, Clippy, and formatting pass. Full pre-push validation remains
  maintainer-owned; whole-suite performance improvements are not yet measured.

## [0.14.0] - 2026-10-03 - One fixture builder and shared Cargo metadata

### Changed

- Standalone fixture pools own their builder at construction:
  `CachedStandaloneCanisterFixturePool::new(build_fixture)`. Acquire with
  `pool.acquire()`. The per-acquisition builder and `Default` constructor are
  removed as a source API hard cut; captured builders use the additional inferred
  type parameter. Capacity, snapshot funding, restore order, recovery, and guard
  types are preserved.
- Wasm batches parse Cargo package, membership, and dependency indexes once per
  resolution group. Standalone builds share this representation. Each batch
  entry still selects its own dependency closure and validates filesystem inputs;
  distinct feature graphs remain separately resolved.
- CI runs canister integration tests once through the ordinary test stage,
  removing their repeated stage and unused preliminary fixture build.
  `make test-canisters` acquires its own artifacts; the standalone manual
  `make build-test-canisters` target remains available.
- Release-push guards exercise command behavior instead of asserting exact
  Makefile recipe text. Clean, dirty, untracked, stale-tag, and failed-push cases
  verify gating and failure propagation without real publication or pushes.

### Documentation

- Updates standalone pool examples and ships the constructor/acquisition
  migration in the package changelog. Downstream callers must adopt both changes
  together when selecting `0.14`.
- Repository-owned format identifiers remain `v1`; persisted layouts are
  unchanged.

### Testing

- Focused PocketIC 16 checks cover owned builders, cold builds, warm reuse,
  failed-restore rebuilding, panic invalidation, and capacity-scoped concurrency.
- Wasm checks cover shared malformed-metadata failures, entry-specific input
  failures, feature resolution, semantic fingerprints, source-mutation rejection,
  and a cold canister build with live benchmark markers.
- Release guards and cleanup fixtures, Clippy, rustdoc, and formatting pass.
  Full pre-push validation remains maintainer-owned.

## [0.13.0] - 2026-10-03 - Authoritative benchmark and baseline state

### Changed

- Benchmark aggregate rows, comparison rows, and aggregate errors expose
  `suite()` instead of a separate writable `suite` field. Labels and comparison
  keys derive from the same scope; an authored suite named `ALL` remains
  distinct from the cross-suite aggregate.
- `BenchmarkAggregateRow::average()` replaces the stored `average` field.
  Comparisons and report writers derive averages from the current totals and
  run count, avoiding stale values after those fields change. CSV and metadata
  schemas remain unchanged.
- Baseline recipes construct reset requirements with
  `ResetRequirements::try_new(cycle_policy, non_snapshot_requirements)`.
  Snapshot restoration is an unconditional pool invariant, and the pool checks
  the captured canister set and cycle policy directly against
  `CanisterRestoreReceipt`. `ResetReceipt` carries only non-snapshot guarantees.
- Removes snapshot and cycle variants from `ResetDomainKind`, `ResetRequirement`,
  and `ResetAchievement`, along with `UndeclaredRequiredResetDomain`.
  `CyclePolicyMismatch { required, achieved }` reports cycle-policy failures;
  they retain the existing `ResetCoverageMismatch` recovery classification.
- Wasm batch failures own their errors, phases, and partial timings together.
  Acquisition and reporting share `WasmBuildFailureDetails`, removing parallel
  optional failure state, an internal consistency assertion, and an intermediate
  diagnostics conversion. Public batch result/accessor signatures are preserved.
- Release guards execute the patch, minor, and CI recipes with recorded recursive
  stages and injected failures, including parallel Make invocation. These checks
  replace exact recipe-text assertions while retaining cleanup and publication
  safeguards.

### Documentation

- Updates README guidance and the multi-canister recipe example for the source
  API hard cuts; the packaged migration guide covers field, constructor, variant,
  and receipt changes.
- Installation examples select `0.12`. Repository-owned cache, stamp, protocol,
  and digest identifiers remain `v1`; this release changes no persisted layout.

### Testing

- Observed Cargo output/heartbeat coverage releases its fixture only after an
  actual heartbeat, with a bounded timeout. This removes the fixed-sleep race
  under parallel test load while preserving raw-output and exit-event assertions.
- Verifies updated aggregate totals and run counts drive averages and comparisons,
  retaining named-`ALL` scope, arithmetic-overflow, and report-schema coverage.
- Covers canister-set and cycle-policy restore mismatches with safe slot rebuilds
  before final restored-baseline validation. Retains non-snapshot policy mismatch,
  recovery, panic invalidation, and 100-consecutive-restore checks.
- Verifies batch result, failure-details, and owned-parts views for mixed success
  and failure, captured Cargo diagnostics, retained successful outputs, and
  concurrent prepared-input readers.
- Targeted checks pass for PocketIC 16 baseline reuse and isolated dead-server
  recovery, release guards and cleanup fixtures, Clippy, rustdoc, formatting,
  and Wasm compilation. Full pre-push validation remains maintainer-owned.

## [0.12.0] - 2026-10-03 - Artifact correctness and implementation simplification

### Changed

- `aggregate_benchmark_spans` returns `Result<BenchmarkAggregateReport,
  BenchmarkAggregateError>`. Callers must handle or propagate overflow errors
  before comparing aggregates or writing reports; this is a source API hard cut.
- Benchmark metadata reads and writes derive their JSON fields from
  `BenchmarkRunMetadata`. Existing field names, optional fields and integer
  bounds are preserved; invalid inputs return `InvalidData` with Serde field
  diagnostics.
- Wasm warm hits share validation and retained-record completion. Batch input
  reuse has one internal representation for sessions and prepared snapshots,
  and silent and observed entry points call the common runner directly.
  Maintenance-policy errors use the common report-entry construction while
  retaining their failure phases and timings. Internal attempts store the phase
  and timings together, and standalone and batch resolution share toolchain
  identification.
- ICP readiness uses the watched-input snapshot's file validation directly.
- Transport errors and panic payloads use the same message classifier.
- Background server reaping retains the original managed-server owner, removing
  the separate child guard and optional startup-file state.
- Shared-target lock acquisition owns progress events for ordinary builds and
  scheduled maintenance, preserving event order and reported paths.

### Fixed

- Observed Cargo output capture retries interrupted pipe reads while preserving
  captured bytes and propagating permanent read errors.
- Compact Cargo feature arguments (`-Fextra` and `-F=extra`) reach metadata
  resolution as well as compilation. Enabled optional dependencies remain
  watched inputs, and batches keep distinct feature graphs in separate
  resolution groups.
- Shared-target maintenance rejects layouts that could remove retained exact
  Wasm artifacts. Batch maintenance identifies workspace-relative targets and
  filesystem aliases by their resolved directory.
- Relative exact Wasm targets use the caller's working directory consistently
  for Cargo output and cache operations. Conflicting Cargo target-directory
  overrides through `with_extra_env` (`CARGO_TARGET_DIR`) or
  `with_cargo_profile_args` (`--target-dir`) return `InvalidSpec` before
  acquisition; target directories must be selected through `WasmBuildSpec`.
- Artifact preparation rejects nested output destinations and validates the
  directory entry atomic publication replaces, including final symlinks.
  Declared input and tool symlink entries and their referents remain protected.
- Unix managed-server teardown terminates its owned process group on drop,
  startup failure, and natural exit before reaping the child, preventing
  descendant leaks and keeping the process-group identity reserved for cleanup.
- Benchmark aggregation checks counter and run-count arithmetic. Overflow
  rejects the complete aggregation with a typed scope/span/counter error;
  `is_all_suites()` distinguishes cross-suite overflow from a named suite `ALL`.

### Documentation

- Installation examples select the current `0.11` minor release.
- Documents target-directory and output-boundary rules, Unix process-group
  cleanup, and recipe-pool timings for profiling long suites. Refreshes upstream
  PocketIC tracking in `POCKET-IC.md`.
- Adds the aggregation API migration to the packaged changelog. Repository-owned
  cache, stamp, protocol and digest identifiers remain `v1`.

### Testing

- Adds regressions for retained exact-cache boundaries, relative exact targets,
  rejected Cargo overrides, nested output destinations and symlink publication.
- Verifies compact feature arguments discover and watch enabled optional
  dependencies and preserve separate prepared-input resolution groups.
- Covers all four benchmark counters at and beyond `u128::MAX`, suite/global
  and run-count overflow, per-span subtraction underflow, and metadata schema
  validation.
- Verifies observed session and concurrent prepared-snapshot batches retain
  per-entry results, progress order and reuse metrics. Scheduled maintenance
  progress checks include shared-target lock acquisition.
- Exercises descendant cleanup on drop, readiness timeout, natural exit and
  background reaping, and validates startup/reaping with PocketIC 16.
- Maintenance behavior tests use outcome/accessor/filesystem assertions instead
  of diagnostic wording. Zero-heartbeat rejection uses a valid fixture and a
  successful positive-interval build as its control.

## [0.11.0] - 2026-10-03 - Cache correctness and harness API consolidation

### Changed

- Consolidates baseline reuse on `CachedPocketIcBaselinePool` with a recipe and
  runtime capacity. Removes `restore_or_rebuild_cached_pocket_ic_baseline` and
  `CachedPocketIcBaselineGuard`, eliminating the mutex-slot recovery race that
  could return another caller's mutated baseline without restoring it.
- Consolidates installation on `InstallSpec`, removing
  `create_and_install_with_args` and `try_create_and_install_with_args`.
- `CanisterInstallError` identifies the creation, funding, or installation phase,
  returns an optional created canister id, and retains a `PocketIcOperationError`
  cause. Snapshot panic variants expose the same cause through `Error::source()`.
- Renames Wasm and generic artifact input-race errors to
  `InputsChangedDuringAcquisition`.
- Adds a leading `scope` column to `comparison.csv`, distinguishing named
  suites (`suite`) from cross-suite aggregates (`all`), including when a named
  suite is literally `ALL`.
- Shares batch label validation, component-aware path canonicalization, and
  silent/observed Wasm batch orchestration. All repository-owned format
  identifiers remain `v1`; API and schema changes are hard cuts.

### Fixed

- Ordinary warm Wasm hits revalidate inputs before returning artifacts and
  reject changes during acquisition, across isolated, shared incremental,
  and scheduled-maintenance modes. Explicit immutable-source sessions and
  prepared readers retain their warm reuse contract.
- One entry's Wasm input hashing failure no longer fails valid compatible
  batch entries. Deferred hashing and discovery failures retain their original
  resolution phase.
- Fallible installation captures creation and cycle-funding failures as well as
  installation failures. Standalone errors retain the caller's PocketIC
  instance at every failed stage.
- Snapshot and installation failures preserve transport classification through
  contextual wrappers and use the shared classifier for recovery.
- Server diagnostic reads allocate only the bounded log prefix instead of
  reading the complete file before truncating it.
- Benchmark run allocation continues beyond `9999`, orders previous-run
  indices numerically, and reports exhaustion rather than reusing a directory.
  Empty commit hashes use the same `unknown` prefix as directory formatting.
- Release CI cleanup recognises renamed selected PocketIC binaries by matching
  the running executable's device/inode and the invocation's private port-file
  path. The runner passes configured binary paths and PocketIC 16's exact default
  download path. Unknown executable identities retain scratch without signalling
  unrelated processes; pidfd protection, bounded termination and the original
  CI failure status are preserved.

### Documentation

- Updates README examples and API guidance against the current implementation,
  adds a documentation index, and distinguishes current usage from historical
  design records.
- Documents the 0.11.0 API and CSV migration in the packaged changelog, including
  recipe-pool acquisition, installation failure stages, optional canister ids,
  operation error causes, and input-race error names.

### Testing

- Adds regressions for independent batch hashing, warm-hit input races across
  cache modes and materialization paths, contextual transport causes, refused
  creation with retained instance ownership, sparse multi-gigabyte log reads,
  CSV scope identity, and benchmark index boundaries.
- Validates existing recipe-pool recovery and concurrency, retained artifact
  behavior, immutable-source readers, and all 27 README Rust examples with
  targeted checks.
- Expands focused cleanup regressions to cover renamed binaries, default
  downloads, relative and PATH selection, forged executable names, unavailable
  identities, ownership rechecks and servers appearing during cleanup.

## [0.10.4] - 2026-10-02 - Release cleanup and upstream PocketIC policy

### Fixed

- Stops release-owned PocketIC servers before deleting the release CI temporary
  directory, preventing missing-socket panics during HTTP adapter teardown.
  Ownership requires a port file inside that invocation's private directory;
  Linux pidfds prevent signalling a reused PID. Failed cleanup retains the
  directory, and an existing CI failure remains the returned status.

### Changed

- Removes the isolated PocketIC teardown patch and its development probe.
  Instance teardown improvements are deferred to a future upstream release;
  ic-testkit continues to use the unmodified registry dependency.

### Testing

- Adds seven process/socket regressions to the release guard checks, covering
  shutdown ordering, external-server isolation, forced termination, escaping
  paths and retained diagnostics on cleanup failure.
- Binds fixture Unix sockets using short relative paths inside their scratch
  directories so nested release `TMPDIR` paths do not exceed the socket limit.

## [0.10.3] - 2026-10-02 - PocketIC teardown experiment

### Development

- Adds an isolated PocketIC 16.0.0 upstream teardown proposal and repeatable
  synthetic HTTP probe. The proposal provides fallible shutdown deadlines,
  acknowledgement checks, retryable ownership and bounded best-effort drop.
  Seven targeted parent tests cover sync/async deletion, timeouts, retries,
  failure responses, shared-peer availability and borrowed gateway cleanup.
  The production registry dependency is unchanged; the original Busy/tick
  cause remains unproven.

### Documentation

- Records the proposal's deadline and acknowledgement semantics, probe usage,
  and the remaining persistent-state handoff review before upstream adoption.
  This release adds no production shutdown API or teardown fix.

## [0.10.2] - 2026-10-02 - Transport classification and deterministic progress tests

### Changed

- Updates the pinned internal Rust toolchain and CI from 1.96.0 to 1.99.0.
  The published MSRV remains Rust 1.88.0.
- Keeps the prepared-input reuse counter compatible with the MSRV while
  avoiding Rust 1.99's deprecated atomic `fetch_update` method.

### Fixed

- Narrows unstructured PocketIC transport classification to maintained reqwest
  error shapes with an instance URL and recognized transport source. Generic
  application messages, quoted variant names and bare I/O errors no longer
  justify dead-transport recovery. Contextual testkit call errors are recognized
  through their structured transport kind. Unrelated call panics resume with
  their original payload.

### Testing

- Updates empty/nonempty collection assertions for Rust 1.99's
  `assert_is_empty` lint, preserving their checks and showing collection values
  on failure. Warnings-denied Clippy is checked across all crate targets.
- Coordinates heartbeat and observer-unwind tests through channels instead of
  worker sleeps. Workers remain blocked until the relevant heartbeat or
  observer unwinding; timeouts are deadlock escapes rather than timing assertions.
- Adds positive transport controls, negative application/quoted-text controls,
  original-payload call-boundary coverage and a real refused HTTP request against
  a synthetic peer.
- Adds an isolated-process reproduction of PocketIC 16's synchronous deletion
  wait with parent-owned kill/reap cleanup, explicit HTTP barriers and bounded
  fixture waits. This does not reproduce the original Busy/tick cause or fix
  upstream teardown.

### Documentation

- Clarifies the classifier's heuristic limits and the distinction between
  bounded construction, operation budgets and upstream instance teardown.

## [0.10.1] - 2026-09-27 - Managed server process identity

### Added

- Adds `PocketIcManagedServer::process_id()` so callers can identify their
  owned server for resource monitoring. The PID identifies only the server
  child and does not establish liveness or retain ownership.

### Documentation

- Documents PID lifetime and reuse semantics and demonstrates the accessor in
  the managed-server example.

### Testing

- Extends managed-server coverage to check the PID against child-reported
  identity and verify cleanup on drop. The targeted test, library Clippy,
  formatting, and whitespace checks pass.

## [0.10.0] - 2026-09-17 - Retained artifact handoff

### Changed

- Hard-cuts `WasmBuildRecord::artifacts()` and
  `ArtifactCacheRecord::artifacts()` to return read-only exact-cache paths
  instead of mutable materialization destinations. Successful cold builds and
  warm hits retain their entries before producer protection ends; keeping a
  record or its clone alive protects the bytes through post-link processing,
  reading, and staging. `WasmBuildRecord::exact_cache_path()` has the same
  lifetime guarantee.
- Adds shared OS file-lock ownership to the existing cache lifecycle. Age and
  size pruning skip retained entries, then reclaim them normally after the
  final owner drops or its process exits. Retained entries may temporarily
  exceed configured limits. Invalid retained entries fail closed rather than
  being replaced; no new cache authority or permanent pin registry is added.
- Batch reports retain each successful entry independently through later
  builds and failures. Consumers must keep the report, extracted outcome, or
  cloned record until reading/staging finishes; copied paths alone do not
  retain ownership. Cache and digest formats remain `v1`.

### Fixed

- Closes the producer-to-consumer lifetime gaps reported by IcyDB in
  [#2](https://github.com/dragginzgame/ic-testkit/issues/2), for both Wasm inputs
  and generic post-link outputs. The specific operation behind the original
  intermittent missing-input failure remains unproven.
- Includes the failing filesystem path in declared-input/tool hashing errors,
  including failures while traversing a declared directory.

### Testing

- Adds pipe-coordinated process tests for cold and warm Wasm/post-link handoff,
  shared-output replacement, shared-target cleanup, age/size pruning, cloned
  ownership, and reclamation after normal release or process termination.
- Covers distinct successful batch outputs surviving a later failure,
  warm post-link reuse, fail-closed retained corruption, and missing-input/tool
  diagnostics with transaction cleanup.

### Migration

- Consume `record.artifacts()` while holding the record; stop reconstructing
  compiler or deployable paths from configuration. Keep successful batch
  results instead of reducing them to indexes. Treat retained paths as
  read-only and write transformations into transaction staging.
- This is a pre-1.0 minor hard cut. No compatibility path accessor or older
  cache-format reader is provided. IcyDB's released-dependency adoption and
  concurrent lifecycle target must be validated downstream after release.

## [0.9.1] - 2026-09-13 - TOML dependency update and CI artifact reuse

### Changed

- Updates the workspace `toml` dependency from 0.9 to 1 and refreshes
  the lockfile, resolving `toml` to 1.1.6.
- Removes `cargo clean` from successful release CI cleanup. Cargo build
  artifacts now remain available for incremental reuse after both successful
  and failed CI gates; the release wrapper still removes only its own isolated
  temporary directory.

### Testing

- Updates release-flow guards to reject direct `cargo clean` commands in CI,
  release, and publish scripts; keep the standalone `make clean` target outside
  `CI_TARGETS`; and fail if successful or failed release cleanup invokes Cargo.

### Documentation

- Clarifies that build artifacts survive CI, release, and publish flows and
  that `cargo clean` is available only through the manually invoked standalone
  target.

## [0.9.0] - 2026-09-02 - PocketIC 16 and opt-in hard lifetimes

### Added

- Re-exports the complete host-only upstream crate at
  `ic_testkit::pocket_ic`. Downstreams can use the exact PocketIC version and
  type identities selected by ic-testkit without waiting for individual types
  to be added to `ic_testkit::pic`; the existing focused `pic` conveniences
  remain available.

### Changed

- Updates the workspace `pocket-ic` dependency from 15.0 to 16.0 and refreshes
  its compatible transitive dependency graph.
- Carries PocketIC 16's upstream behavior changes through the directly
  re-exported runtime: automatic-progress startup waits for its first certified
  time update, mocked HTTP outcalls account for response-size cycle spend, and
  oversized mocked reject messages are rejected. Flexible HTTP mocking and the
  new `SubnetCoolingDown` and `CanisterStatusAccessDenied` error codes remain
  upstream-native APIs rather than new testkit wrappers.
- Stops passing a ten-minute `--hard-ttl` to `ic-testkit`-managed PocketIC
  servers by default, matching PocketIC 16's removal of the implicit absolute
  server deadline. Long-running active suites are no longer terminated solely
  because ten minutes have elapsed; PocketIC's activity-based soft TTL and
  explicit `PocketIcManagedServer` drop ownership remain in effect.
- Hard-cuts `PocketIcStartupConfig::server_hard_ttl` to return
  `Option<Duration>`. It returns `None` for the new default; callers that need
  an absolute server lifetime continue to opt in with
  `with_server_hard_ttl(duration)`.

### Documentation

- Updates the README, packaged migration guide, concurrency decision record,
  and upstream-boundary review for PocketIC 16, the complete upstream crate
  re-export, and the new managed-server lifetime policy.

### Testing

- Covers omission of the default `--hard-ttl`, forwarding of an explicit hard
  TTL, positive-bound validation, port-file ownership, bounded readiness, and
  managed-child cleanup with synthetic server processes.
- Adds consumer-path compile coverage for top-level and nested upstream types
  through `ic_testkit::pocket_ic` and proves that its `PocketIc` is identical
  to the existing `ic_testkit::pic::PocketIc` convenience export.

## [0.8.9] - 2026-08-19 - Concurrent Wasm input snapshots

### Added

- Adds `WasmBuildInputSnapshot`, a caller-owned immutable input-resolution
  snapshot prepared for a fixed set of exact Wasm specifications under a
  genuine source write-exclusion lease. Separate sequential batches can read
  it concurrently without repeating warm Cargo/rustc identity, metadata,
  discovery, or content hashing.
- Adds preparation and cumulative reader-reuse metrics plus a distinct
  per-batch prepared-reuse counter.

### Changed

- Skips the second shared-target input-resolution pass when a batch is backed
  by an explicit immutable-source session or prepared snapshot. Ordinary calls
  retain the existing locked re-resolution.
- Rejects specifications absent from snapshot preparation before progress or
  build work. A detected post-build source race invalidates every later reader;
  publication is coordinated against that shared invalidation boundary.

### Testing

- Covers concurrent warm readers, zero per-reader input-resolution time,
  undeclared-spec preflight, shared invalidation, and no publication after a
  deliberately violated source lease.

### Documentation

- Records warm IcyDB profile evidence of roughly 1.54 and 5.24 seconds spent in
  input resolution as motivation for the concurrent-reader snapshot. IcyDB
  must continue using ordinary per-call resolution until it owns a genuine
  source write-exclusion guard.

## [0.8.8] - 2026-08-18 - Explicit Wasm source assumptions

### Changed

- Hard-cuts the ambiguous `WasmBuildSession::new(&guard)` constructor to
  `WasmBuildSession::assume_sources_immutable(&guard)`. The new name exposes
  that source immutability is a caller assertion and that an arbitrary borrowed
  token is not a valid lease. No old-name alias is retained.

### Documentation

- Records a future prepared immutable resolution snapshot for concurrent
  readers as a separate design requiring a genuine source lease, shared
  invalidation, declared specifications, isolated or already-coordinated Cargo
  targets, and consumer benchmarks. Current batches and sessions remain
  sequential.

## [0.8.7] - 2026-08-18 - Explicit Wasm sessions and reliable PocketIC ownership

### Added

- Adds explicit caller-owned `WasmBuildSession` input snapshots. A session is
  lifetime-bound to a caller-held source write-exclusion guard and reuses exact
  Cargo/rustc identity, metadata, input discovery, and content digests across
  separate sequential batch calls. Ordinary batch functions remain
  independently validated and no process-global cache is introduced.
- Adds `WasmBuildFailurePhase`, `WasmBuildFailureTimings`, and
  `WasmBuildFailureDetails`. Failed batch entries now retain the primary
  specification, coordination, metadata, discovery, hashing, Cargo,
  publication, maintenance, or cleanup phase plus all phase time completed
  before return.
- Adds per-batch session-reuse metrics and session-wide retained-snapshot,
  snapshot-reuse, and invalidation counters.
- Adds caller-owned `PocketIcManagedServer` startup through
  `PocketIcStartupConfig::start_managed_server`. The handle exposes its URL and
  bounded lossy output, supports multiple bounded `connect` calls, and
  terminates and waits for its child on drop.

### Changed

- Permanently invalidates an explicit session after detecting an input race,
  discards every pending snapshot captured before that race, and rejects later
  calls with `WasmBuildBatchContractError::SourceLeaseInvalidated`.
- Hard-cuts `WasmBuildBatchEntry::into_parts` to return failure details between
  the result and entry elapsed time. No four-field alias or deprecated bridge
  is retained.
- Allocates managed PocketIC startup artifacts inside a unique private
  directory while deliberately leaving the server-owned port-file path absent
  before spawn. A missing path remains pending until the server publishes it,
  matching PocketIC 15's `--port-file` contract.

### Documentation

- Replaces the deferred session proposal with the implemented immutable-source
  contract, including the caller's obligation to coordinate every source,
  configuration, tool, declared-input, and relevant-environment mutation for
  the complete session lifetime.
- Documents partial failed-phase timing access and the hard-cut report-entry
  migration in the packaged changelog.
- Documents shared serial-suite server ownership through the managed handle
  and bounded `PocketIcStartupConfig::connect` calls, including the intentional
  process-local ownership boundary and external-server guidance for
  multi-process CI.
- Refreshes `POCKET-IC.md` and the concurrency, baseline-pool, and artifact
  orchestration designs so their current-state sections reflect managed-server
  ownership, PocketIC 15 port-file semantics, session digest reuse, and failed
  Wasm phase timings.

### Testing

- Covers cross-call exact snapshot reuse, session metrics, source-race
  invalidation, post-invalidation rejection, and partial metadata, hashing,
  Cargo, and cleanup timings.
- Covers absent pre-spawn port paths, private startup directories, synthetic
  rejection of a pre-existing `$4` port argument, managed-server URL/output,
  readiness timeout, child exit, and RAII termination.
- Adds an explicit ignored real-server test driven by
  `IC_TESTKIT_POCKET_IC_SERVER`; PocketIC 15.0.0 publishes its port, constructs
  an instance through bounded connect mode, and leaves no startup directory
  after owned shutdown.

## [0.8.6] - 2026-08-18 - Semantic Wasm identity and bounded PocketIC startup

### Added

- Adds `ResolvedCargoBuildInputs::validation_digest` as the conservative raw
  workspace/source/configuration mutation guard alongside semantic
  `input_digest` cache identity.
- Adds required `LabeledWasmBuildSpec` inputs and retains each stable caller
  label in canonical batch entries, successful outcomes, failures, progress
  events, and shared-target maintenance outcomes.
- Adds `CanisterDiagnosticsBatchContractError` for preflight rejection of
  empty or duplicate diagnostic labels before any target is contacted.
- Adds explicit `PocketIcStartupConfig::spawn` and `connect` policies plus
  structured startup errors for spawn, child exit, port readiness, invalid
  ports, builder panics, and complete-deadline expiry. Managed child output is
  retained as bounded lossy UTF-8.

### Changed

- Hard-cuts exact Cargo Wasm identity to a validated semantic workspace
  projection of the selected resolve graph, enabled features, external
  source/checksum/revision identities, effective package fields, profiles,
  resolver/lints, selected source roots, tools, configuration, and declared
  inputs. Unrelated host-only workspace dependency and lockfile changes no
  longer invalidate the selected Wasm key.
- Keeps the complete workspace manifest and lockfile in the raw validation
  digest and compares it around Cargo builds and attached artifact
  transactions. Publication still aborts on any mid-operation mutation, even
  when semantic identity is unchanged.
- Uses a conservative complete-input fallback for workspace-root packages and
  local package paths that cannot be normalized beneath the workspace. No
  global cache or cross-call immutable-source session is added.
- Changes the existing `v1` digest semantics in place. Projected workspaces
  receive new exact keys once; there is no legacy-key reader, compatibility
  alias, or dual old/new cache path.
- Hard-cuts every Wasm batch entry point to labeled specifications and a
  batch-contract `Result`. Reports now own one canonical labeled entry
  collection instead of parallel results and elapsed-time slices; structured
  iterators retain the label alongside the index and outcome or error.
- Hard-cuts diagnostics batches to validate label structure and return a
  batch-contract `Result`. Valid batches remain sequential and collect-all.
- Hard-cuts `PocketIcBuilderExt::try_build` to require an explicit bounded
  startup configuration. The managed path monitors the exact caller-resolved
  server child while awaiting both readiness and instance construction,
  terminates it at the deadline, and never enters PocketIC's implicit
  unbounded server-start path.
- Removes no-longer-used index-only batch iteration internals. No compatibility
  overloads, label sidecars, deprecated methods, anonymous fallback, or
  zero-argument startup alias are retained.

### Documentation

- Documents semantic cache identity versus conservative mutation validation,
  the projection boundary, fallback cases, and the existing requirement to
  declare build-script or tool inputs outside Cargo's graph.
- Documents the labeled Wasm migration and the caller-owned binary resolution,
  compatibility, provenance, and deadline obligations for bounded PocketIC
  startup.

### Testing

- Covers reuse after an unrelated host-only workspace dependency/lockfile
  change and invalidation after selected dependency or workspace profile
  changes.
- Covers Wasm label validation and propagation, diagnostic preflight without
  work, structured managed-server exit output, and readiness-timeout child
  termination.

## [0.8.5] - 2026-08-18 - Labeled artifact batches and failure phases

### Added

- Adds required `LabeledArtifactCacheSpec` batch inputs and retains each
  caller-owned stable label in callbacks, ordered entries, outcomes, and
  failures. Empty or duplicate labels reject the batch before work begins.
- Adds partial `ArtifactCacheBatchFailureTimings` for preparation, callback,
  explicit abort cleanup, commit, and total elapsed time, plus the primary
  `ArtifactCacheBatchFailurePhase`.
- Adds canonical `ArtifactCacheBatchEntry` and structured successful-entry
  views so multi-stage consumers can compose results by label instead of
  remapping filtered indexes.
- Adds per-target `entry_elapsed` and total wall time to
  `CanisterDiagnosticsBatchReport`, allowing deployment recovery to identify
  slow diagnostic targets without downstream timers.

### Changed

- Hard-cuts `build_artifact_caches_batch` to labeled specifications, a
  label-based population callback, and a batch-level contract `Result`.
- Hard-cuts generic artifact reports from parallel result/elapsed slices to one
  ordered labeled-entry collection. Outcome and failure iterators return
  structured entries with labels, indexes, and elapsed time.
- Hard-cuts `ArtifactCacheBatchFailure` variants to retain failure phase
  timings. Recipe panics continue unwinding and batches remain sequential.
- Hard-cuts `CanisterDiagnosticsBatchEntry::into_parts` to include retained
  elapsed time; no two-field compatibility method is retained.

### Documentation

- Records why a package dependency closure alone cannot safely discard the
  complete workspace manifest and lockfile, and defines a conservative,
  validated-success contract for any future narrower fingerprint.
- Records an explicit batch-scoped immutable-source lease or validated snapshot
  as the required boundary for sharing content digests across incompatible
  feature/metadata groups. No ambient or unsafe hash cache is added.

### Testing

- Covers stable label propagation, preflight rejection of empty/duplicate
  labels, collect-all continuation, and preparation/callback/cleanup/commit
  failure timing availability.
- Covers diagnostics entry timing against total sequential batch time for panic
  capture and real controller-rejection paths.

## [0.8.4] - 2026-08-18 - Collect-all diagnostics and failed-entry context

### Added

- Adds controller-aware `collect_canister_diagnostics_batch` for ordered,
  caller-labeled `CanisterDiagnosticsRequest` values. Every target is attempted;
  each entry retains its exact senders and independent bounded status/log
  outcomes without anonymous retry or fallback.
- Adds structured `WasmBuildBatchFailure` entries that bundle specification
  index, `WasmBuildError`, and retained entry wall time.
- Adds per-entry wall time and structured `ArtifactCacheBatchFailedEntry`
  failures to generic artifact collect-all reports.

### Changed

- Hard-cuts `WasmBuildBatchReport::failures` and
  `ArtifactCacheBatchReport::failures` from tuple items to their structured
  failed-entry types. No tuple aliases or deprecated iterators are retained.

### Documentation

- Records partial failed-phase timings as a future error-contract change.
- Records the correctness contract for possible generic-batch digest reuse:
  callers must provide a source-immutability lease or the implementation must
  revalidate rather than silently reuse hashes for matching paths.
- Records caller-supplied stable artifact entry keys as a future labeled-spec
  hard cut, not a parallel index/key sidecar.

### Testing

- Covers labeled diagnostic ordering and continuation after rejection or panic,
  exact request retention, structured Wasm failure elapsed time, and generic
  per-entry elapsed time across successful and failed batches.

## [0.8.3] - 2026-08-18 - Code hygiene and release consistency

### Changed

- Shares one saturating optional-duration aggregator across Wasm,
  transactional artifact, and baseline-pool timing reports.
- Shares ordered indexed outcome/failure iteration between Wasm and generic
  artifact collect-all reports.
- Routes Make, changelog, bump, tag, publish, and release-guard tooling through
  one workspace-version reader. The reader targets `[workspace.package]`, and
  stable release operations require an exact `major.minor.patch` version.
- Removes a stale source-section banner while keeping the crate layout and
  public surface unchanged.

### Compatibility

- Makes no public API, runtime, cache-format, schema, or compatibility-policy
  changes. No migration or pre-1.0 API hard cut is required for this patch.

### Testing

- Covers absent, present, and saturating optional-duration aggregation while
  retaining the existing focused batch-index and pool-timing coverage.
- Covers stable, bump-compatible prerelease, missing, and unrelated-section
  workspace-version inputs through the shared release helper.

## [0.8.2] - 2026-08-18 - Release flow and CI stability

### Changed

- Runs the complete release gate once, before changing version metadata;
  pushing a committed, tagged release no longer repeats fallible validation
  that can strand a local patch version after failure.

### Testing

- Makes exact-cache lock-heartbeat coverage scheduling-independent by releasing
  the fixture lock only after the expected heartbeat is observed.
- Verifies release-gate failures leave version metadata untouched and the final
  tagged push performs no second fallible validation pass.

## [0.8.1] - 2026-08-18 - Structured diagnostics and batch observability

### Added

- Adds controller-aware `CanisterDiagnosticsRequest` and a structured
  `CanisterDiagnosticsReport` that retains independent status/log outcomes,
  exact senders, bounded lossy UTF-8 logs, and explicit truncation counts.
- Adds Wasm and generic artifact batch metrics for built/reused/failed counts,
  compatible input-resolution reuse, and summed successful timings.
- Retains per-entry wall time in `WasmBuildBatchReport`, including for failed
  entries.
- Packages a `0.8` changelog and hard-cut migration guide with the crate.

### Changed

- Makes `build_artifact_caches_batch` return an ordered collect-all
  `ArtifactCacheBatchReport`; preparation, callback, and commit failures retain
  their indexes while later independent specifications continue.
- Replaces printing `PocketIcDiagnosticsExt::dump_canister_debug` with the
  structured `collect_canister_diagnostics` hard cut. Install diagnostics use
  the exact install sender and remain subordinate to the original failure.

### Removed

- Removes the fail-fast `ArtifactCacheBatchOutcome` and
  `ArtifactCacheBatchError` contract; the generic artifact batch report is the
  sole batch result.
- Removes the anonymous-only `dump_canister_debug` entry point without an alias
  or deprecated bridge.

### Documentation

- Records the required immutability/staleness contract for a future explicit
  cross-call Wasm build session. No global or partial session cache is added in
  `0.8.x`.
- Records failed phase-timing retention as a broader error-contract follow-up;
  this release retains the small per-entry elapsed completion.

### Testing

- Covers independent diagnostic senders and failures, bounded lossy log
  rendering, original install-error preservation, generic collect-all
  continuation, aggregate metrics, and elapsed retention for failed Wasm
  entries.

## [0.8.0] - 2026-08-18 - Collect-all Wasm batches and API hard cuts

### Added

- Exposes each successful record's immutable content-addressed directory
  through `WasmBuildRecord::exact_cache_path`.

### Changed

- Makes the cached-Wasm batch sequentially collect every ordered result in one
  `WasmBuildBatchReport` with indexed outcome, failure, and maintenance
  iterators. Later specifications continue after an independent failure.
- Reuses tool identity, Cargo metadata, input discovery, and memoized input
  digests across resolution-compatible batch specifications without combining
  Cargo builds or changing standalone fingerprints.
- Recreates a missing immutable entry from an already validated caller-facing
  artifact before returning its public exact-cache path.
- Updates the `v1` exact-Wasm cache semantics in place for composable input
  digests; no migration reader or second format is introduced before `1.0`.
- Makes `CachedStandaloneCanisterFixturePool::acquire` return the structured
  lifecycle outcome and timings directly.
- Moves exact-sender capture and restore onto the single
  `PocketIcSnapshotExt` trait.
- Makes the canonical Wasm and transactional artifact builders accept both
  strings and OS-native values through their unsuffixed methods.
- Makes `WasmBuildTimings::input_resolution` return the structured phase
  timings directly.

### Removed

- Removes the fail-fast `WasmBuildBatchOutcome` and `WasmBuildBatchError`
  contract; the batch report is the sole batch result.
- Removes the Wasm-specific `WasmBuildCachePrunePolicy`,
  `WasmBuildCachePruneReport`, and `WasmBuildCacheMaintenance` aliases in favor
  of the generic artifact retention types.
- Removes the duplicate `CargoHeartbeat` progress event, `_os` and alternate
  additional-input builders, boolean pool result, split
  `PocketIcCapturedSnapshotExt` trait, and panicking `build_wasm_canisters`
  wrapper without compatibility shims.

### Testing

- Covers collect-all continuation, batch snapshot reuse, standalone
  fingerprints, and public exact-cache paths.

## [0.7.6] - 2026-08-06 - Maintenance ownership and resilience

### Added

- Adds explicit strict and best-effort failure handling for integrated shared
  incremental-target maintenance. Best-effort failures remain visible through
  a typed `Failed` outcome without invalidating an otherwise successful Wasm
  acquisition.
- Adds `WasmBuildBatchConfig` orchestration that schedules maintenance once for
  each distinct shared target instead of requiring callers to modify the first
  build specification.
- Adds read-only accessors for exact and shared-target maintenance settings and
  indexed batch maintenance outcomes.

### Changed

- Keeps strict integrated maintenance as the compatibility default and rejects
  ambiguous mixtures of batch-owned and per-spec maintenance configuration.
- Preserves Cargo build artifacts when release CI fails so diagnostics and
  incremental state remain available; `cargo clean` now runs only after the
  complete CI gate succeeds, while isolated temporary files are always removed.

### Testing

- Adds focused coverage for configuration inspection, strict and best-effort
  failure behavior, per-target batch deduplication, and ownership conflicts.
- Covers successful and failed release-CI cleanup paths to prevent destructive
  cleanup from obscuring a failed gate.

## [0.7.5] - 2026-08-06 - Acquisition-wide build progress

### Added

- Adds acquisition-wide, phase-aware Wasm build heartbeats for exact and
  shared-target lock waits, Cargo input resolution, shared and exact cache
  maintenance, Cargo compilation, and artifact validation/publication.

### Changed

- Retains `CargoHeartbeat` as a compatibility event alongside the generic
  Cargo-build heartbeat, and joins active synchronous phase work before an
  observer panic propagates so no heartbeat worker is detached.

### Testing

- Adds focused regressions for quiet phase heartbeats, exact-cache lock waits,
  Cargo compatibility events, and panic-safe worker joining.

## [0.7.4] - 2026-08-06 - Integrated shared-target maintenance

### Added

- Adds `WasmBuildSpec::with_shared_incremental_target_maintenance_at_most_every`
  so scheduled caller-owned target retention participates directly in cached
  Wasm acquisition, build records, compact diagnostics, and progress events.

### Changed

- Reuses the acquisition's locked Cargo input resolution for due shared-target
  maintenance. The opt-in path coordinates and creates the target even on an
  exact hit, allowing the first acquisition to record its schedule marker.

### Testing

- Adds focused validation plus cold-build, immediate-hit, progress-ordering,
  resolution-reuse, and missing-target recreation coverage.

## [0.7.3] - 2026-08-06 - Current artifact workflow guidance

### Changed

- Updates README installation guidance to the compatible `0.7` release line,
  demonstrates interval-limited shared-target maintenance, and distinguishes
  independent per-spec batching from intentional multi-package Cargo builds.

## [0.7.2] - 2026-08-06 - Scheduled shared-target maintenance

### Added

- Adds cross-process interval-limited shared incremental-target maintenance
  with structured missing, skipped, and performed outcomes. Matching recent
  passes skip both Cargo input resolution and whole-target traversal, while
  policy changes and zero intervals remain immediately due.

### Testing

- Adds focused fast-path, policy-change, safety, missing-target, and subprocess
  coordination coverage for scheduled shared-target retention.

## [0.7.1] - 2026-08-06 - Artifact hygiene and test organization

### Changed

- Centralizes shared incremental-target existence and type validation across
  inspection and maintenance, and documents every transactional batch-error
  field without changing public behavior.
- Consolidates temporary-directory allocation, path polling, and executable
  script setup across artifact unit and integration tests, giving parallel
  test binaries consistent collision-resistant fixtures and diagnostics.
- Moves Wasm-cache and artifact-batch regressions out of production modules,
  normalizes artifact-module ordering, and replaces bare test unwraps in the
  touched paths with contextual failure messages.

## [0.7.0] - 2026-08-06 - Independent artifact orchestration

### Added

- Adds independent cached-Wasm batch orchestration that deliberately runs one
  Cargo command per `WasmBuildSpec`, preserving each package set's standalone
  dependency-feature resolution while reporting ordered outcomes and the
  successful prefix of a failed batch.
- Adds opt-in structured Wasm build progress with input-resolution and lock
  phases, raw OS-native Cargo stdout/stderr chunks, configurable quiet-period
  heartbeats, exit status, and final cache outcome events. Existing build APIs
  remain silent.
- Adds sequential independent transactional artifact batching with at most one
  live miss transaction, synchronous cleanup after caller build errors, and
  explicit non-atomic failure semantics.
- Adds explicit lock-coordinated shared Cargo target retention. Callers may
  clear the complete mutable compilation state by age or logical-size limit
  while preserving the target root, cache tag, and process-lock metadata.

### Changed

- Strengthens shared incremental-target boundary validation to reject overlap
  in either direction with exact Cargo inputs. Destructive maintenance reruns
  exact input resolution before clearing and leaves unsafe targets untouched.

### Testing

- Adds focused feature-isolation, progress-streaming, shared-target retention,
  transactional batch reuse, and failed-builder cleanup regressions.

## [0.6.1] - 2026-08-06 - Exact cache integration refinements

### Added

- Adds exact-sender-only snapshot restoration for captured snapshot sets and
  cached baselines, avoiding fallback management calls when capture already
  established the correct sender for every canister.
- Adds `ArtifactCacheSpec::with_cargo_build_inputs`, bridging exact resolved
  Cargo dependency/configuration identity into transactional artifact caching
  with full fingerprint and content revalidation through commit.
- Adds lock-coordinated shared incremental-target inspection with canonical
  path, logical size, last recorded build use, and lock-wait diagnostics.
- Adds opt-in minimum maintenance intervals for Wasm and transactional caches,
  preserving explicit retention limits while skipping repeated hit-path scans.

### Changed

- Records shared Cargo target build use without making its mutable incremental
  state part of exact-cache retention ownership.
- Restores standalone fixture-pool snapshots with their successful capture
  sender instead of retrying controller fallbacks.

### Testing

- Adds focused mixed-controller restore, exact Cargo-to-artifact transaction,
  scheduled-maintenance, and shared-target observation regressions.

## [0.6.0] - 2026-08-06 - Shared-incremental Wasm caching

### Added

- Adds an opt-in shared-incremental mode to `WasmBuildSpec`. Cargo misses build
  under a cross-process caller-owned target lock, while only revalidated final
  Wasm files enter immutable fingerprint entries. Exact hits bypass the shared
  target, failures preserve incremental state, and retention never owns it.
- Exposes `resolve_cargo_build_inputs`, `ResolvedCargoBuildInputs`, and stable
  labeled Cargo inputs, exclusions, exact digests, revalidation, and phase
  timings without requiring downstream Cargo metadata adapters.
- Adds OS-native iterator builders for Wasm and transactional cache arguments,
  environment, inherited environment, and additional paths, plus an explicit
  `resolve_executable` helper for canonical `PATH` tool fingerprinting.
- Adds `CanisterSnapshotTarget` and exact per-canister sender capture for
  mixed-controller topologies, including a matching cached-baseline capture
  constructor. Existing controller fallback behavior remains available.
- Adds compact single-line `Display` implementations for Wasm, transactional,
  standalone-fixture, and multi-canister pool outcomes and timings.

### Changed

- Documents safe pipeline invalidation through declared implementation inputs,
  explicit bounded-retention starting points, shared-incremental ownership,
  and mixed-controller snapshot capture. No cache limit becomes automatic.

### Testing

- Adds real Cargo coverage for shared incremental reuse, immutable compact
  output entries, exact old-fingerprint restoration, failed-build preservation,
  input races, and retention boundaries.
- Adds a two-process regression proving different exact-cache roots serialize
  Cargo builds that share one mutable incremental target, plus mixed-controller,
  public input-resolution, executable-resolution, and OS-native identity tests.

## [0.5.2] - 2026-08-06 - Transactional cache maintenance

### Changed

- Moves the transactional cache's unit tests into a dedicated source file and
  keeps shared test-only filesystem setup centralized.
- Preserves both source and destination paths, plus the underlying I/O cause,
  when a streamed atomic artifact copy fails.

### Testing

- Adds an actual two-process transactional-cache regression that starts both
  callers together under distinct coordination scopes and proves the shared
  exact content lock issues only one build transaction.

## [0.5.1] - 2026-08-06 - Transactional cache hardening

### Fixed

- Rejects declared inputs and tools located within the cache root instead of
  silently excluding them from exact hashing. Output destinations must remain
  outside the cache and must not alias another output or overlap a declared
  input or tool.
- Treats malformed manifest bytes, non-directory content entries, unexpected
  entry-root files, and other inspectable schema corruption as cache misses
  that are removed and rebuilt. Staged transactions likewise reject files
  outside the declared `outputs` directory.
- Removes abandoned transaction staging during retention only after
  non-blockingly acquiring the corresponding content-key lock, so pruning
  reclaims terminated builds without touching active transactions. Prune
  reports expose the removed uncommitted-directory count and logical bytes.

### Changed

- Streams input hashing, output hashing, fixed-output importing, transactional
  materialization, and Wasm-cache materialization instead of buffering entire
  artifacts in memory. The digest and on-disk cache formats remain unchanged.
- Shares atomic file copying, path removal, digest-directory recognition, and
  test temporary-directory setup across the Wasm and transactional caches.
  Transaction output declarations are normalized once instead of repeatedly
  allocated and sorted during acquisition.
- Expands regression coverage across invalid specifications, keyed and
  deliberately unkeyed identity fields, cache/input/output path boundaries,
  malformed entries, exact entry schemas, active/orphan staging, streaming
  digest compatibility, explicit aborts, and unknown outputs.

## [0.5.0] - 2026-08-06 - Transactional artifact-set caching

### Added

- Adds `ArtifactCacheSpec`, `prepare_artifact_cache`, and owned miss
  transactions for deterministic commands outside Cargo. Exact input and tool
  contents, recipe identity, ordered arguments, relevant environment, opaque
  identity fields, and the complete output schema select an immutable cache
  entry.
- Adds complete multi-output staging, checked logical output paths, fixed-path
  output importing, before/after input verification, atomic entry publication,
  caller-destination materialization, content-manifest validation, corruption
  recovery, and panic-safe cleanup. Typed records report `Built` or `Reused`,
  exact keys, materialized artifacts, phase timings, and nonfatal maintenance.
- Adds separate coordination-scope, content-key, and namespace process locks so
  recipes sharing external mutable state serialize without preventing exact
  independent cache identity. Overlapping exact acquisitions build once.
- Adds generic `ArtifactCachePrunePolicy`, `ArtifactCachePruneReport`,
  `ArtifactCacheMaintenance`, and strict `prune_artifact_cache` retention for
  transactional namespaces.
- Adds a compiled external-transform example and coverage for exact concurrent
  reuse, coordination locking, multi-output publication, corruption recovery,
  failed-build cleanup, input races, import workflows, and retention.

### Changed

- Moves cache-directory tagging, process-lock creation, last-use tracking,
  logical directory measurement, and age/size pruning beneath both the existing
  Wasm cache and the transactional artifact cache. Existing
  `WasmBuildCachePrunePolicy`, report, and maintenance names remain compatible
  aliases with no Wasm cache layout migration.
- Keeps the new cache host-only and command-agnostic, with no production Wasm
  or PocketIC runtime behavior changes.

## [0.4.2] - 2026-08-05 - Exact Cargo inputs and in-lock maintenance

### Added

- Adds optional `WasmBuildSpec::with_prune_policy` retention under the build
  operation's existing process lock. The active fingerprint is protected, and
  successful build records expose a nonfatal structured maintenance outcome
  plus its phase duration.
- Adds `WasmInputResolutionTimings` and
  `WasmBuildTimings::input_resolution_detail`, separating tool identity, Cargo
  metadata, input discovery, and content hashing while retaining the existing
  aggregate timing accessor.

### Changed

- Expands exact Cargo configuration fingerprinting to the invocation directory
  and every ancestor plus the effective Cargo home, follows recursive required
  and optional includes, and matches Cargo's extensionless `config` precedence
  when both supported configuration names exist.

## [0.4.1] - 2026-08-05 - Structured standalone fixture outcomes

### Added

- Adds `CachedStandaloneCanisterFixturePool::acquire_with_outcome` with
  structured `Built`, `Restored`, and `Rebuilt` results, rebuild reasons, phase
  timings, and timed snapshot errors. The existing `(guard, bool)` acquisition
  remains compatible and delegates to the same lifecycle implementation.

## [0.4.0] - 2026-08-05 - Bounded multi-canister baseline pools

### Added

- Adds `CachedPocketIcBaselinePool`, a caller-owned runtime-capacity pool for
  multi-canister PocketIC baselines. One structurally owned
  `PocketIcBaselineRecipe` defines build, complete snapshot restore,
  non-snapshot reset, readiness, invariant validation, and failure
  classification for the pool's lifetime.
- Adds caller-owned `FixtureRecipeId`, typed reset requirements and receipts,
  exact restored-canister-set verification, structured `Built`, `Restored`,
  and `Rebuilt` outcomes, explicit lease invalidation, one-shot recovery, and
  combined original/rebuild failures. Successful outcomes and failed
  acquisitions both retain phase timings.
- Adds `is_dead_pocket_ic_transport_error`, which searches a recipe error's
  source chain for PocketIC's currently unstructured dead-transport failure
  class, plus a public default stage-to-rebuild-reason mapping for custom
  recipe classifiers.
- Expands baseline-pool integration coverage across time advancement, cycle
  mutation, extra-canister creation, reset/readiness/validation recovery,
  built/restored validation equivalence, capacity-one queue timing, and
  an isolated manual recovery test that kills a test-owned non-reused PocketIC
  server.
- Adds `CanisterRestoreReceipt::try_from_baseline` so recipes can derive exact
  restore evidence from the captured snapshot set, plus a complete
  compile-checked two-canister recipe example covering build, restore,
  readiness, validation, failure classification, and reuse.

### Changed

- Moves `CachedStandaloneCanisterFixturePool` onto the same internal FIFO
  bounded-slot scheduler used by the multi-canister pool, preserving its public
  API while making panicked leases and partially failed restores non-reusable.

## [0.3.6] - 2026-08-05 - Consolidated cache and pooling designs

### Added

- Adds a consumer-validated follow-up design that combines Canic's external
  multi-output build caching and IcyDB's post-link transform caching into one
  proposed transactional artifact-set core, with shared locking, exact input
  verification, batch manifests, atomic publication, typed outcomes, failure
  cleanup, timings, and retention.
- Defines an opt-in shared Cargo incremental strategy that keeps the exact
  artifact store authoritative.
- Adds a proposed `0.4` bounded multi-canister baseline-pool design with runtime
  capacity, structural recipe ownership, typed reset requirements and receipts,
  uniform post-build/post-restore validation, explicit rebuild semantics, and
  one internal scheduler shared with the standalone pool.

### Changed

- Records a correctness-first delivery order for complete Cargo configuration
  discovery and per-path input change reporting before safely narrowing
  invalidation.
- Documents intentional pooled-fixture lease scope for Clippy and the `0.3.5`
  requirement to declare Cargo configuration inherited from workspace
  ancestors or the effective Cargo home.

## [0.3.5] - 2026-08-05 - Wasm cache lifecycle hardening

### Added

- Adds `WasmBuildCachePrunePolicy`, `prune_wasm_build_cache`, and structured
  `WasmBuildCachePruneReport` results for caller-controlled maximum-age and
  maximum-logical-size retention of fingerprint-specific Cargo targets.
- Adds persistent last-use markers so size pruning removes least-recently-used
  exact builds instead of relying on filesystem directory timestamps.

### Changed

- Removes a fingerprint-specific Cargo target whenever its build fails,
  including command failures, missing outputs, post-build fingerprint errors,
  and `InputsChangedDuringBuild`; cleanup failures retain both the original
  structured build error and the cleanup error.
- Writes a standards-compliant `CACHEDIR.TAG` at every caller-selected target
  root used for cached builds or pruning.
- Coordinates pruning through the same output-scoped process lock as builds
  and limits recursive removal to direct 64-hex fingerprint directories.

## [0.3.4] - 2026-08-05 - Content-addressed Wasm builds

### Added

- Adds `WasmBuildSpec` and `build_wasm_canisters_cached`, which fingerprint a
  package's local dependency closure, Cargo configuration and lockfile,
  toolchain identity, target/profile, declared environment, and additional
  inputs; coordinate builds with an output-scoped process lock; publish atomic
  per-artifact stamps; and return typed `Built` or `Reused` outcomes.
- Adds structured Wasm build errors and timings for lock wait, exact input
  resolution, Cargo execution, and the complete operation.
- Adds public `InputDigest` values and exact `WatchedInputSnapshot` artifact
  stamps for deterministic freshness across Git checkouts, CI cache restores,
  and filesystem timestamp differences.

### Changed

- Makes the existing `build_wasm_canisters` convenience function use the exact
  cache while preserving its caller-selected package, profile, environment,
  target-directory, and panic-on-failure interface.
- Replaces mtime-only `WatchedInputSnapshot::artifact_is_fresh` behavior with
  explicit content-stamp matching; callers mark an artifact only after a
  successful build with `mark_artifact_fresh`.

## [0.3.3] - 2026-08-05 - Bounded standalone fixture reuse

### Added

- Adds `CachedStandaloneCanisterFixturePool`, a caller-owned fixed-capacity
  pool that restores an independent canister snapshot per slot. Heavy suites
  can reuse one fixture recipe with bounded parallelism while keeping tests
  that depend on fresh PocketIC-wide state on directly owned fixtures.

## [0.3.2] - 2026-08-04 - Documentation refresh

### Changed

- Rewrites the README around the current 0.3.1 API, including ownership,
  startup, typed calls, installation, snapshots, scoped baselines,
  diagnostics, artifacts, benchmarking, and release behavior.
- Expands crate and public API rustdoc for the host/canister boundary, extension
  prelude, structured errors, explicit snapshot funding, and cached-baseline
  lifecycle.
- Refreshes the maintained PocketIC upstream boundary and marks older design
  documents as historical records rather than current API documentation.

## [0.3.1] - 2026-08-04 - Downstream harness ergonomics

### Added

- Adds `PocketIcBuilderExt::try_build` and `PocketIcStartupError` as a narrow,
  unclassified panic boundary for bounded downstream startup retry.
- Adds a trait-only `pic::prelude` for the PocketIC harness extension traits.
- Restores the focused `PocketIcTimeExt::current_time_nanos` conversion without
  mirroring PocketIC's broader time API.
- Adds explicit `SnapshotRestoreFunding::{Preserve, TopUpTo}` policy and cached
  baseline funding methods.

### Changed

- Makes snapshot restore preserve the current cycle balance by default instead
  of silently topping every restored canister up to 200T cycles.
- Renames standalone fixture calls to match `CandidCallExt`'s
  `update_candid*` and `query_candid*` vocabulary.
- Updates README examples and dependency guidance for the 0.3 API.

### Removed

- Removes the standalone fixture `update_call*` and `query_call*` names without
  compatibility aliases.

## [0.3.0] - 2026-08-04 - Fallible retries and upstream boundaries

### Added

- Adds `RetryPolicyError`, returned when an install retry policy is configured
  with zero attempts.

### Changed

- Replaces the asserting `RetryPolicy::new` constructor with fallible
  `RetryPolicy::try_new`.
- Records PocketIC's remaining need for fallible lifecycle and transport APIs,
  which would let ic-testkit remove panic catching and dead-instance message
  classification.
- Clarifies that reproducible benchmarks require a caller-owned explicit
  `POCKET_IC_BIN` until PocketIC exposes the resolved binary path, version, and
  digest.

## [0.2.2] - 2026-08-04 - Focused harness surface

### Changed

- Makes direct upstream construction the only public construction path: callers
  use `PocketIc::new()` or `PocketIcBuilder::build()`.
- Makes `StandaloneCanisterFixture::{install, try_install}` consume a
  caller-built `PocketIc`, allowing exact topology and server-binary selection
  without mirroring `PocketIcBuilder`.
- Returns the caller's instance inside `StandaloneCanisterInstallError` when a
  fallible fixture install fails, alongside the underlying
  `CanisterInstallError`.
- Changes `retry_install_code` operations to return `RejectResponse` and
  classifies rate limiting through
  `ErrorCode::CanisterInstallCodeRateLimited`, preserving the structured
  rejection instead of matching display text.
- Re-exports PocketIC's `LATEST_SERVER_VERSION` for benchmark metadata and
  documents that resolved binary path and digest provenance require upstream
  support or caller-owned explicit binary selection.
- Keeps install-code cooldown advancement private to `CanisterInstallExt`
  instead of exposing general time conveniences over PocketIC's native API.
- Corrects the PocketIC wishlist to recognize upstream's existing typed Candid
  helpers and describe only ic-testkit's structured-error value delta.

### Removed

- Removes the `pic()`, `try_pic()`, `build_pocket_ic()`, and
  `try_build_pocket_ic()` construction shims.
- Removes all `install_prebuilt_canister*` free-function variants and
  `StandaloneCanisterFixtureError` in favor of the two fixture methods and
  the install-only `StandaloneCanisterInstallError`.
- Removes `PocketIcStartError` and all startup panic-text classification.
- Removes `PocketIcTimeExt`; callers use PocketIC's inherent time and round
  methods directly.
- Removes the unused crate-specific `Account` type and `Fake::account()`;
  `Fake::principal()` remains the generic deterministic identity helper.

## [0.2.1] - 2026-08-04 - Ownership and diagnostics hardening

### Added

- Completes the live concurrency acceptance coverage for standalone
  `into_parts`, resource-scoped cached-baseline slots, and fresh PocketIC
  construction while another cached baseline is retained.

### Changed

- Updates the root and packaged crate documentation for the direct PocketIC
  ownership model and the released `0.2` dependency line.
- Adds rustdoc for the public standalone prebuilt-canister constructors.
- Adds `make docs-check` to the ordinary CI and release gates so rustdoc
  warnings fail before publication.
- Clarifies that benchmark run paths are caller-owned and that concurrent
  writers must use unique paths or synchronize only the shared destination.

### Fixed

- Makes install-failure status and log diagnostics best-effort so a secondary
  PocketIC or stderr panic cannot replace the original `CanisterInstallError`.

## [0.2.0] - 2026-08-04 - Direct PocketIC ownership

### Added

- Adds focused extension traits for the harness behavior that remains useful
  above PocketIC: `CandidCallExt`, `CanisterInstallExt`,
  `PocketIcSnapshotExt`, `PocketIcDiagnosticsExt`, and `PocketIcTimeExt`.
- Adds `RetryPolicy`, with `max_attempts` consistently counting the initial
  attempt, for rate-limited install-code operations.
- Adds live regression coverage proving that two independent `PocketIc`
  instances can be constructed, used, isolated, and dropped concurrently.
  The proof runs on every supported Linux and macOS PocketIC lane.
- Adds a 0.1-to-0.2 migration table and records the concurrency and API
  boundary decisions in the 0.2 design document.
- Adds guarded `make minor` and `make release-minor` commands for releases
  such as the 0.1-to-0.2 transition; `make publish` remains the separate,
  retry-safe publication step after tag CI succeeds.

### Changed

- Re-exports `PocketIc` and `PocketIcBuilder` directly and removes the `Pic`
  and `PicBuilder` forwarding wrappers. `pic`, `try_pic`,
  `build_pocket_ic`, and `try_build_pocket_ic` now return the upstream type.
- Makes ic-testkit concurrency-neutral: each test normally owns an independent
  `PocketIc`, while downstream test runners and CI control resource
  parallelism.
- Renames cached-baseline and PocketIC-specific error types for direct upstream
  ownership, including `CachedPocketIcBaseline`, `CandidCallError`,
  `CanisterInstallError`, and `PocketIcStartError`.
- Delegates PocketIC server discovery, downloading, and cache ownership back to
  the upstream crate.
- Makes controller snapshot sets deterministic, duplicate-checked, fallible,
  and transactional on capture failure, with structured rejection, panic, and
  cleanup details.

### Fixed

- Preserves upstream `RejectResponse` values as structured
  `CandidCallErrorKind::CanisterReject` failures instead of misclassifying
  canister rejections as transport errors.
- Cleans up snapshots already captured when a later capture fails and reports
  both the primary and any cleanup failures without printing from library code.
- Separates authored benchmark suites named `ALL` from the private cross-suite
  aggregation scope, preventing the authored suite from being counted twice.

### Removed

- Removes `PicSerialGuard` and all process-wide or host-wide PocketIC ownership
  locks, leases, owner records, acquisition timeouts, and retry loops.
- Removes ic-testkit's duplicate PocketIC runtime configuration, downloader,
  and binary-cache implementation, along with its direct `flate2`, `reqwest`,
  and `sha2` dependencies.
- Removes `retry_install_code_ok` and `retry_install_code_err`; callers use
  `retry_install_code` with an explicit `RetryPolicy`.

## [0.1.12] - 2026-08-04 - PocketIC 15 compatibility

### Added

- Adds the guarded `make release-patch` and `make publish` release flow used by
  `ic-query`, including changelog, clean-tree, tag-at-HEAD, CI, and retry-safe
  publication checks.

### Changed

- Updates the workspace `pocket-ic` dependency from 14.0 to 15.0.
- Updates `ic-cdk` from 0.20.1 to 0.20.2 and refreshes the compatible
  transitive Internet Computer dependency stack.

## [0.1.11] - 2026-05-29 - Rust 1.96 internal toolchain

### Changed

- Updates the pinned internal Rust toolchain from 1.95.0 to 1.96.0 while
  keeping the published MSRV at Rust 1.88.

## [0.1.10] - 2026-05-29 - PocketIC upstream wishlist

### Added

- Adds a top-level `POCKET-IC.md` working draft that tracks generic
  upstream-facing `pocket-ic` improvements suggested by current `ic-testkit`
  wrapper behavior.
- Links the PocketIC upstream wishlist from the top of the repository README.

## [0.1.9] - 2026-05-28 - Standalone InstallSpec fixtures

### Added

- Adds `install_prebuilt_canister_from_spec` and
  `try_install_prebuilt_canister_from_spec` so standalone fixtures can use
  `InstallSpec` labels and install senders while preserving the
  `StandaloneCanisterFixture` wrapper.

### Changed

- Routes existing standalone prebuilt-canister install helpers through
  `InstallSpec` internally so standalone fixture install behavior stays
  consistent across the simple and explicit APIs.

## [0.1.8] - 2026-05-28 - Structured call errors and labeled installs

### Added

- Adds `StandaloneCanisterFixture::{update_call_or_panic,
  update_call_as_or_panic, query_call_or_panic, query_call_as_or_panic}` for
  the same transport/codec-only panic behavior as the `Pic` helpers.
- Adds `PicCallErrorKind` and `PicCallContext` so downstream tests can inspect
  encode, decode, and transport failures without matching error strings.
- Adds `InstallSpec`, `Pic::{create_and_install, try_create_and_install,
  create_and_install_many, try_create_and_install_many}`, and optional install
  labels for generic labeled/batch canister installs.

### Changed

- Marks the structured call-error types and `InstallSpec` as non-exhaustive and
  adds accessor methods so the API can evolve without encouraging direct
  construction.
- Includes optional install labels in `PicInstallError` display output and
  install-trap diagnostics.
- Documents `InstallSpec` and sequential batch-install partial failure behavior
  in the README.

## [0.1.7] - 2026-05-28 - Typed call ergonomics

### Added

- Adds `Pic::{update_call_or_panic, query_call_or_panic,
  update_call_as_or_panic, query_call_as_or_panic}` for tests that should
  panic on PocketIC transport or Candid codec failures while preserving
  application-level return values such as `Result<T, E>`.
- Adds typed call forwarding helpers on `StandaloneCanisterFixture` so
  standalone prebuilt-canister tests can call the fixture canister without
  repeatedly spelling out `fixture.pic()` and `fixture.canister_id()`.
- Adds a README example for `CachedPicBaseline` with metadata and
  `restore_or_rebuild_cached_pic_baseline`.

### Changed

- Enriches Candid encode/decode `PicCallError` messages with call operation,
  canister id, caller, method, and decode byte length where available.
- Refreshes README setup guidance for `POCKET_IC_BIN`,
  `IC_TESTKIT_ALLOW_POCKET_IC_DOWNLOAD=1`, and the current `ic-testkit`
  dependency version.

## [0.1.6] - 2026-05-28 - PocketIC binary resolution

### Added

- Adds `ic_testkit::pic::ensure_pocket_ic_bin()` and
  `ic_testkit::pic::try_ensure_pocket_ic_bin()` for resolving the PocketIC
  server binary before startup.
- Adds `PicRuntimeConfig` so callers can configure PocketIC server binary
  resolution in code, including cache directory, default-off download policy,
  and optional SHA-256 verification.
- Honors existing `POCKET_IC_BIN` first and adds one env switch for opt-in
  downloads.

### Changed

- Resolves and validates the PocketIC server binary in `PicBuilder::try_build()`
  before calling into `pocket-ic`, returning `PicStartError::BinaryUnavailable`
  with setup guidance when no usable binary is available.
- Skips the repository perf-probe integration test cleanly when no PocketIC
  server binary is configured and downloads are not enabled.
- Documents the PocketIC server binary setup and cache behavior in the README.

## [0.1.5] - 2026-05-28 - Skipped

- Skipped before publication after removing extra environment-variable controls
  from the PocketIC binary resolution API.

## [0.1.4] - 2026-05-27 - Funded snapshot restore

### Fixed

- Tops up low-cycle canisters before cached baseline snapshot restore so
  `load_canister_snapshot` can pay its management-operation cost before the
  snapshot state is restored.

## [0.1.3] - 2026-05-27 - PocketIC 14 compatibility

### Changed

- Updates the workspace `pocket-ic` dependency to 14.0.
- Stops adding default extra cycles in standalone PocketIC install helpers now
  that `pocket-ic` 14 creates canisters with 100T cycles by default.

## [0.1.2] - 2026-05-24 - README and report cleanup

### Added

- Writes `comparison.csv` alongside the benchmark summary so previous-run
  comparison rows are available as a machine-readable report artifact.

### Changed

- Cleans up README and design-document wording now that canister-side
  `Performance::measure` is a normal crate dependency rather than a feature.
- Tightens the root README by removing duplicate examples and keeping a smaller
  quick-reference shape.
- Updates the crate-local README to link to the repository README on GitHub,
  which is more useful from crates.io than a package-relative path.

## [0.1.1] - 2026-05-24 - Release hygiene cleanup

### Changed

- Moves the publishable crate into `crates/ic-testkit` while keeping
  repository-level `README.md`, `CHANGELOG.md`, `canisters/`, `docs/`, and
  `images/` at the repo root.
- Adds a short crate-local `crates/ic-testkit/README.md` for Cargo packaging,
  matching the related workspace layout convention.
- Adds a root workspace manifest and moves shared dependency versions, package
  metadata, toolchain metadata, and Clippy lint policy into workspace-level
  tables for reuse by future crates.
- Updates Makefile targets and the perf-probe canister manifest for the new
  workspace layout.
- Removes the `canister` feature and makes `ic-cdk` a normal dependency so the
  `performance::Performance` marker helper is always part of the crate surface.
- Updates the README banner to use the repository-hosted image from the new
  top-level `images/` directory.

### Fixed

- Keeps the published crate package self-contained by making
  `tests/canister_benchmark.rs` skip cleanly when its repo-only fixture canister
  is absent from the packaged source.
- Defines `BenchmarkParserConfig::strict` behavior so non-empty non-marker
  lines are reported as malformed markers instead of silently ignored.
- Replaces hand-rolled benchmark metadata JSON parsing/writing with
  `serde_json` so escaped strings and externally generated metadata are handled
  correctly.
- Documents the stdout/stderr ordering limitation in
  `parse_benchmark_events_from_captured_output`.

## [0.1.0] - 2026-05-24 - Benchmark reporting and canister markers

### Added

- Starts the 0.1 benchmark-reporting surface with compact `ICTK|...` marker
  parsing, start/end span pairing, invalid/unpaired marker reporting, suite and
  `ALL` aggregation, previous-run comparison helpers, CSV report writing, and a
  Markdown analytics summary.
- Adds an optional `canister` feature with `performance::Performance::measure`
  for emitting compact benchmark markers from canister code.
- Keeps host-only PocketIC helpers out of `wasm32` builds so canisters can
  depend on the marker emitter without pulling in `pocket-ic`.
- Adds benchmark run-directory helpers for commit/date/index naming and
  previous-run discovery from report metadata.
- Adds a combined stdout/stderr parser that preserves marker source metadata
  for captured PocketIC test output.
- Adds a top-level `canisters/test/perf_probe` fixture canister plus
  `make test-canisters` / `make build-test-canisters` for exercising benchmark
  marker emission from inside this repository.
- Adds benchmark tests covering compact marker parsing, stdout/stderr source
  tracking, malformed markers, repeated/nested span pairing, invalid spans,
  aggregate rows, comparison percentages, and report file generation.
- Adds the initial 0.1 benchmarking design document under `docs/design/`.

### Changed

- Refreshes the README around the current 0.1 workflows: PocketIC wrapper
  usage, wasm installation, artifact helpers, benchmark reports,
  canister-side marker emission, and local release checks.
- Extends `make release-check` so it also runs the live PocketIC benchmark
  canister test and builds the in-repository wasm fixture.

## [0.0.6] - 2026-05-24 - Genericity audit cleanup

- Neutralizes remaining example/test specifics from the extracted harness by
  using generic fake principals in README examples instead of a real ledger
  principal.
- Changes `.icp` artifact tests to use a generic `counter` canister path instead
  of a root-canister path.
- Clarifies `.icp` artifact readiness docs so they describe freshness and
  nonempty artifact checks, not removed build-environment stamp behavior.

## [0.0.5] - 2026-05-24 - Generic artifact profiles

- Removes the hardcoded `WasmBuildProfile` enum so `ic-testkit` no longer owns
  project-specific build profile names such as `fast`.
- Changes wasm artifact helpers to accept caller-provided Cargo profile
  arguments and target profile directory names.
- Updates README examples and artifact-helper tests to show explicit caller
  profile choices instead of crate-owned profile variants.

## [0.0.4] - 2026-05-24 - README presentation cleanup

- Reworks the README header so the title remains Markdown while the tagline,
  banner image, and badges are cleanly centered with GitHub-supported HTML.
- Replaces the mixed Markdown/HTML image block with a single centered
  `images/cave.png` banner.
- Reflows README prose to remove unnecessary hard line breaks while preserving
  code blocks, lists, and badge markup.

## [0.0.3] - 2026-05-24 - Documentation and release helpers

- Clarifies that `ic-testkit` is a wrapper/helper layer around `pocket-ic` and
  links directly to the upstream `pocket-ic` crate.
- Adds the README audit warning banner while the crate surface is still being
  reviewed.
- Adds a centered README image banner and keeps the badge block at the top of
  the project page.
- Expands the Makefile with formatting, checking, Clippy, MSRV, packaging,
  publish dry-run, and aggregate release-check targets.

## [0.0.2] - 2026-05-24 - Release polish

- Removes crate-specific publishing blockers and sets the publishable MSRV to
  Rust 1.88, which is the minimum supported by the current resolved dependency
  graph without downgrading transitive dependencies.
- Reworks the README into a more readable release page with badges, install
  instructions, focused examples, feature summaries, toolchain notes, and
  application-neutral boundaries.
- Adds a small `Makefile` with `make test` as the quick local test entrypoint.
- Adds this changelog in the same Keep a Changelog/SemVer style used by related
  projects.

## [0.0.1] - 2026-05-24 - Initial release

- Adds the initial generic PocketIC test helper surface: `Pic`, `PicBuilder`,
  typed startup errors, cross-process `PicSerialGuard`, and a narrow wrapper
  around the PocketIC calls used by this crate.
- Adds Candid-aware `update_call`, `update_call_as`, `query_call`, and
  `query_call_as` helpers with contextual call errors.
- Adds generic canister install helpers, install-code rate-limit retry helpers,
  standalone prebuilt-wasm fixtures, and canister status/log diagnostics.
- Adds cached baseline primitives for snapshot/restore-heavy tests, including
  rebuild-on-dead-instance handling for stale PocketIC transports.
- Adds controller snapshot capture/restore helpers with sender fallbacks.
- Adds deterministic fake principals and account-like values for reproducible
  tests.
- Adds generic wasm artifact helpers for path resolution, readiness checks,
  package builds, artifact reads, workspace target directories, and generated
  `.icp` artifact freshness checks.
- Defines the first crate metadata and baseline README for downstream adoption.
