# AGENTS.md

Read [DRAGGINZGAME.md](DRAGGINZGAME.md) first. The reviewed Shared Tooling
revision and file digests are recorded in [.shared-tooling.snapshot](.shared-tooling.snapshot).
This file is the local overlay for `ic-testkit`; the shared baseline governs
engineering practice. The approved physical-layout exception is recorded below.

1. Never update the workspace `Cargo.toml` `workspace.package.version` for `ic-testkit` itself. Version bumps are handled manually by the maintainer.
2. Prefer keeping this crate generic over adding application-specific test harness behavior.
3. Use focused checks under the validation cadence below; `make ci` is the full
   release suite. Release execution remains separately authorized.
4. Before `1.0`, make API, schema, and behavior changes as hard cuts. Do not add backwards-compatibility shims, aliases, deprecated bridges, dual old/new entry points, or anti-resurrection tests for removed APIs.
5. Keep every repository-owned cache, stamp, schema, protocol, digest-domain, and other format identifier at `v1` before `1.0`; change its `v1` semantics in place instead of introducing `v2` migrations or readers for older formats.
6. Follow `rules/changelogs.md` for both `CHANGELOG.md` and `crates/ic-testkit/CHANGELOG.md`. Maintain the automatically selected, numbered pending release in both views without changing package metadata; preserve published history.

7. Breaking public API or semantic changes require a minor release before `1.0`.
   Choose release notes accordingly; the maintainer still owns manifest versions.
8. In-place `v1` changes must not reinterpret retained data under an incompatible
   layout. Coordinate explicit reset/retirement of retained installations before
   a layout hard cut; do not introduce another reader or format version.
9. Before source edits or compilation, check for active validation. Do not edit
   its inputs or launch competing Cargo jobs. Use this workspace's `target/` and
   `--locked` focused checks. Prepare offline caches explicitly when validating
   offline; never resolve dependency upgrades as a side effect.

Maintainer-approved physical layout (2026-10-07): Rust packages may use both
`crates/` and `canisters/` in this repository. Keep one virtual root workspace
and selected lockfile, inherited package versions/dependencies, and complete
member coverage. Shared Tooling adoption does not require relocating packages
between these approved trees.

Host support and qualification are documented in [docs/hosts.md](docs/hosts.md).
Use `make shared-tooling-check` after snapshot changes and the focused commands
listed in that matrix after portable tooling changes. The full release suite
follows item 3; this does not authorize release targets or package-version changes.

Developer setup prepares the Rust toolchain, uses `make install-tools`, then `make install-hooks`.
`make tools-check` and `make dependency-pins-check` verify prepared selections
offline; they never install tools. Shared audit methods live in `audits/`;
product boundaries and focused qualification remain in this overlay and the
host matrix. Formatting and
manifest ordering follow `rules/git-hooks.md`; `make fmt-check` is the focused,
non-mutating check. Validation never installs formatter tools implicitly.

## Validation policy

Apply Shared Tooling's [canonical validation cadence](https://github.com/dragginzgame/shared-tooling/blob/4256171b5b7ec4c4cc4078548c1c1617ee150699/DRAGGINZGAME.md#validation-and-evidence),
adopted by exact revision `4256171b5b7ec4c4cc4078548c1c1617ee150699` (0.12.5).
This revision-bound policy reference supersedes only the validation cadence in
our older snapshot; executable tooling and all other shared rules retain their
recorded snapshot revision. Local guidance supplies commands and product
qualification requirements. Configured CI and release gates remain unchanged.
On snapshot adoption of this policy, replace this separate reference with the
local baseline link; do not keep a duplicate cadence or temporary exception.
