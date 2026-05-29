# Changelog

All notable, and occasionally less notable changes to this project will be
documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

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
