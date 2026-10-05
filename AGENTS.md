# AGENTS.md

Read [DRAGGINZGAME.md](DRAGGINZGAME.md) first. The reviewed Shared Tooling
revision and file digests are recorded in [.shared-tooling.snapshot](.shared-tooling.snapshot).
This file is the local overlay for `ic-testkit`; the shared baseline governs
engineering practice. At the maintainer's request, rule 6 follows the reviewed
upstream working-tree policy for numbered pending changelogs; importing that
policy into the snapshot awaits its committed revision.

1. Never update the workspace `Cargo.toml` `workspace.package.version` for `ic-testkit` itself. Version bumps are handled manually by the maintainer.
2. Prefer keeping this crate generic over adding application-specific test harness behavior.
3. Run only targeted tests relevant to the files and behavior changed. Do not run the broad `make test` or `make release-check` gates; the maintainer runs full pre-push validation.
4. Before `1.0`, make API, schema, and behavior changes as hard cuts. Do not add backwards-compatibility shims, aliases, deprecated bridges, dual old/new entry points, or anti-resurrection tests for removed APIs.
5. Keep every repository-owned cache, stamp, schema, protocol, digest-domain, and other format identifier at `v1` before `1.0`; change its `v1` semantics in place instead of introducing `v2` migrations or readers for older formats.
6. Maintain one numbered, undated pending release in both `CHANGELOG.md` and `crates/ic-testkit/CHANGELOG.md`. Infer the next version from the latest finalized release and the complete pending batch: patch for compatible changes, minor for breaking changes before `1.0`. Reuse that entry until released, keeping both views and links aligned. Do not change package metadata while preparing notes. The maintainer-owned release workflow must match that candidate and finalize both views with its saved version and UTC date; preserve published history.

7. Breaking public API or semantic changes require a minor release before `1.0`.
   Choose release notes accordingly; the maintainer still owns manifest versions.
8. In-place `v1` changes must not reinterpret retained data under an incompatible
   layout. Coordinate explicit reset/retirement of retained installations before
   a layout hard cut; do not introduce another reader or format version.
9. Before source edits or compilation, check for active validation. Do not edit
   its inputs or launch competing Cargo jobs. Use this workspace's `target/` and
   `--locked` focused checks. Prepare offline caches explicitly when validating
   offline; never resolve dependency upgrades as a side effect.

Host support and qualification are documented in [docs/hosts.md](docs/hosts.md).
Use `make shared-tooling-check` after snapshot changes and the focused commands
listed in that matrix after portable tooling changes. Full CI and release gates
remain maintainer-owned unless explicitly requested.
