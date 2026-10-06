# Host support and qualification

The required native development and operator hosts are Ubuntu 24.04 x86-64
and macOS 15 on Apple Silicon (ARM64) and Intel (x86-64). The crate's canister
runtime target remains `wasm32-unknown-unknown`. Windows host workflows are
not qualified.

| Host | CI runner | Native qualification |
| --- | --- | --- |
| Ubuntu 24.04 x86-64 | `ubuntu-24.04` | Portable tooling and PocketIC ownership tests |
| macOS 15 ARM64 | `macos-15` | Portable tooling and PocketIC ownership tests |
| macOS 15 x86-64 | `macos-15-intel` | Portable tooling and PocketIC ownership tests |

Runner labels and architectures follow the [GitHub runner inventory](https://github.com/actions/runner-images#available-images).
The matrix declares required support; each revision needs a successful matching
native workflow run. Linux checks do not qualify macOS behavior.
CI matrices disable fail-fast so a failure on one host does not cancel checks
on the other supported hosts. Individual checks still stop on failure.
The complete CI, MSRV, archive verification and publish dry-run gates run on all
three native hosts. Qualification requires matching successful native jobs,
with incomplete workflow runs identified separately below. Do not publish or push
just to qualify tooling.

At `v0.16.0` (`5f4a850b6dc3bb5652418f85dd18ed97f63046ce`), the native
Linux gate and PocketIC concurrency checks passed, but macOS startup and
filesystem fixtures failed. See the [ARM64 startup failure](https://github.com/dragginzgame/ic-testkit/actions/runs/37317725235/job/111788601271),
[Intel startup failure](https://github.com/dragginzgame/ic-testkit/actions/runs/37317725777/job/111788604408),
and [ARM64 full-gate results](https://github.com/dragginzgame/ic-testkit/actions/runs/37317725235/job/111788601582).
These earlier runs are failure evidence, not macOS qualification for a changed
revision.

At `v0.16.1` (`e06093f9a3169383ba66ab734d36718975d7ff0c`), the
[native workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37323482068)
passed the complete gates, MSRV and concurrency checks on all three hosts,
and portable-host checks on Linux and ARM64 macOS. The
[Intel portable-host job](https://github.com/dragginzgame/ic-testkit/actions/runs/37323482068/job/111808187977)
failed because the background-reaper fixture exited before startup observed
its published port. The current test uses an explicit release after reaper
handoff.

At `v0.17.0` (`380b328717c22c6f68d27ad8a47537a281acbc1c`), the
[portable-host checks](https://github.com/dragginzgame/ic-testkit/actions/runs/37329182304)
passed on all three hosts, including managed-server lifecycle and the native
benchmark driver/sampler. This confirms the Intel test timing fix and native
driver behavior for that revision. The new report publication and tool-launch
checks were added afterward; these passes are not live benchmark measurements.
The separate [tag ARM64 full-gate run](https://github.com/dragginzgame/ic-testkit/actions/runs/37329182783/job/111828200457)
failed in the synthetic teardown fixture with an immediate `WouldBlock` while
reading an accepted request. The current fixture explicitly selects blocking
accepted-stream I/O with read/write timeouts and tests an initially empty
nonblocking stream. Later qualification is recorded below.

At `v0.17.1` (`4cf3d0d852445957ad3c1a15f6cb9268330beacd`), the
[portable-host checks](https://github.com/dragginzgame/ic-testkit/actions/runs/37333641501)
passed on all three native hosts, including report publication and standalone
Cargo/rustc proxy invocation. This qualifies those checks for that revision,
not the subsequent teardown-fixture change or live benchmark measurements.

At `v0.17.2` (`2b5a7f7630807a94a051b6de56e54a4b28bb1c33`), the
[ARM64 portable-host job](https://github.com/dragginzgame/ic-testkit/actions/runs/37335414639/job/111848825778)
and [Linux portable-host job](https://github.com/dragginzgame/ic-testkit/actions/runs/37335414639/job/111848826130)
passed, including the synthetic transport/teardown suite. This confirms the
accepted-stream fix on native ARM64 macOS. Intel qualification remains pending
the matching job; these results do not qualify the subsequent installation
documentation guard.

At `v0.17.3` (`e35a98f5786cfb5553cfeadc9ee67e6cbc7458b6`), the
[native workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37336194477)
passed the full gate, MSRV, portable-host checks and PocketIC concurrency on
all three hosts. This qualifies the teardown fixture and installation
documentation guard for that revision, including Intel macOS. It precedes the
0.18 release/hook adoption and does not qualify that later tooling.

## Prerequisites

- Bash 3.2 or newer. On macOS, use `/bin/bash` to exercise the system shell.
- GNU Make (the macOS system Make is sufficient), Git, Perl, and standard Unix
  tools including `awk`, `cat`, `find`, `grep`, `mkfifo`, `mktemp`, `ps`, `sed`,
  `sort` and `tar`. Managed process tests use `/bin/ps -p <pid> -o stat=` on both
  native Unix hosts to distinguish running processes, zombies and reaped children.
- A SHA-256 implementation: Linux `sha256sum` or macOS `shasum -a 256`.
- Rustup with the repository's pinned toolchain, `rustfmt`, `clippy`, and the
  `wasm32-unknown-unknown` target. Published MSRV checks use Rust 1.88.
- `cargo-sort` 2.1.4, prepared explicitly with `make install-format-tools`, for
  hooks, manifest sorting, and independent CI/release formatting checks.
  Hook setup and formatting checks also require an already prepared `rustfmt`
  component; use `rustup component add rustfmt` during explicit setup.
- `curl`, gzip and xz-capable tar for explicit tool setup. Run `make install-tools`
  to prepare pinned jq/yq and IC executables locally, then `make tools-check` for
  offline byte/version verification. Versions and native digests have one owner
  in `ci/tool-versions.env` and `ci/ic-tools.tsv`. See the shared
  [bootstrap instructions](local-setup.md#bootstrap-prerequisites).
  Make selects `.tools/host/bin` and `.tools/ic/bin`; direct Cargo calls use the
  explicit PATH and `POCKET_IC_BIN` exports in the README.
- `cloc` for the optional LOC report; jq comes from local host-tool setup.
- Live PocketIC tests require a compatible native PocketIC 16 binary. Set
  `POCKET_IC_BIN` to its exact local path to avoid upstream automatic download.
  Managed startup takes an explicit caller-provided binary and owns its child.
- The opt-in fixture-reuse benchmark requires the repository's probe canister,
  prepared Cargo caches, and `uname`. Its Linux RSS sampler reads native `/proc`;
  macOS uses the system `/bin/ps` process-leader RSS in KiB. The benchmark records
  its sampling source and does not download a server or dependency upgrades.
- Publishing additionally requires `curl`, registry HTTP access and Cargo
  registry credentials. Admission requires a successful exact-version HTTP 404;
  HTTP 200 skips publication, while unavailable observations stop the command.
  Explicit `CARGO_NET_OFFLINE=true` or `1` stops before the lookup. Mocked release
  and publish guard checks do not use credentials or perform remote writes.

All three maintainer release commands run the same complete `make release-check`
gate, including MSRV, with the caller's network policy and temporary-directory
environment. The locked publish dry run requires registry HTTP access without
uploading a package; the complete gate must not be forced offline. Cache checks
and metadata preparation remain offline. Explicit offline requests are preserved
and failures are not automatically retried online.
They retain each validation attempt's identity and full log in a unique Git
directory location, including failed attempts, and retain prepared metadata.
Preflight or validation-only failures rerun through the normal target with
fresh checks. Once preparation may begin, rerunning a normal release target
automatically reconciles the exact saved plan before selecting another version.
Explicit `make release-resume VERSION=X.Y.Z` uses the same checks.
Uncommitted preparation remains bound to its source and kind. After the release
commit exists, a normal target first reconciles that exact release, then runs
fresh checks for the requested increment if its kind differs or HEAD has newer
committed fixes. An unchanged same-kind retry and explicit resume finish only
the saved release. Late adapter checks compare retained metadata with regular
file blobs from the runner-selected `RELEASE_COMMIT`, rather than current HEAD
files. Fixtures cover later documentation, changed/missing/non-regular selected
files, complete-output inspection failure and retained evidence preservation.
Preparation and recovery reject symlinks in retained metadata before copying
backups or publishing files, preserving current metadata and linked targets.
README preparation explicitly checks both commands and their subshell result
before recording readiness or publishing metadata. Metadata fixture assertions
and command-stub admission use explicit rejection because Bash 3.2 does not
apply `errexit` to standalone conditional commands.
After partial metadata publication, preflight checks offline caches using the
verified saved workspace and unchanged member manifests. It does not rewrite
the mixed live manifest/lockfile to perform that check or repeat validation.
Abandoned `.release-metadata.*` publication staging files are excluded from
untracked-source checks only in the two metadata directories and are preserved.
The qualification fixture exercises this admission with a real private Git
index while retaining its substituted release/validation commands.
Consumer metadata phases and the publication clean-worktree prerequisite stop
when Git's untracked inventory fails. Command-stub checks exercise an empty
failed inventory and verify that rejection preserves metadata and validation
evidence before any gate or preparation effects.
Source and metadata digest comparisons likewise require successful Git
inspection, including inside the retained identity constructor. A failed final
source check does not publish a passing validation receipt. Retrying validation
retains its failed log; failed payload checks preserve live and retained files.
Metadata file hashing bypasses Git attributes, clean filters and line-ending
conversion. The fixture uses real Git attributes to normalize LF/CRLF and proves
that changed raw bytes in live, backup and prepared files are rejected without
rewriting current metadata or retained evidence. Restoring the exact saved
bytes permits the same prepared release to pass again. Retained identity
conflicts stop recovery; the helper does not rewrite their readiness record.
README preparation uses the installation checker's TOML example locator on its
private copies. Patch/minor/major fixtures preserve historical dependency lines,
non-TOML examples, indentation, comments and a missing final newline. Duplicate
or malformed maintained examples in either view reject preparation before live
publication, preserving the source metadata.
The reviewed runner now also rejects its failed inventory through
[Shared Tooling issue #4](https://github.com/dragginzgame/shared-tooling/issues/4).
Its command-stub fixture covers failed-empty inventory during descendant
recovery; the consumer checks preserve evidence at every adapter phase.
Release validation preserves Cargo artifacts and does not remove temporary files underneath
upstream background servers. Managed server handles own their process cleanup.

## Focused qualification

Run these commands sequentially, with no other validation modifying their
inputs. The shell guards use fixtures and mock external commands:

```bash
/bin/bash scripts/ci/verify-shared-tooling-snapshot.sh
make tools-check dependency-pins-check
/bin/bash scripts/ci/test-host-tools.sh
/bin/bash scripts/ci/test-ic-tools.sh
/bin/bash scripts/ci/test-dependency-pins.sh
make format-tools-check
make fmt-check
/bin/bash scripts/ci/check-git-hooks.sh
cargo fetch --locked
/bin/bash scripts/ci/check-release-guards.sh
/bin/bash scripts/ci/check-installation-version.sh
/bin/bash scripts/ci/check-publish-guards.sh
cargo test -p ic-testkit --locked --offline --lib pic::startup::tests
cargo test -p ic-testkit --locked --offline --lib artifacts::host_tests
cargo test -p ic-testkit --locked --offline --lib artifacts::wasm::tests
cargo test -p ic-testkit --locked --offline --test pocket_ic_teardown
cargo test -p ic-testkit --locked --offline --example fixture_reuse_benchmark_driver
cargo test -p ic-testkit --locked --offline --test pocket_ic_concurrency
```

Fetching is an explicit preparation step and preserves the selected lockfile.
For an already prepared offline host, use `cargo fetch --locked --offline`;
missing cache data is a preparation failure, not permission to select upgrades.
The focused startup tests use synthetic servers; the caller-provided live
server test is ignored unless explicitly selected as described in the README.
Process ownership checks cover leader reaping and descendant termination on
handle drop, readiness timeout, natural exit and background reaping on both
Linux and macOS.
The transport/teardown checks use a synthetic HTTP peer and isolated subprocess
probes, without a live PocketIC binary. Accepted request streams use blocking
I/O with timeouts even when the listener is nonblocking. Unix regression coverage
forces an initially empty nonblocking stream and releases request bytes only
after the reader changes its socket mode.
The driver checks include native process observations, report bounds, atomic
report replacement, standalone proxy invocation and subprocess completion on
interruption. They use synthetic workers rather than running the live benchmark.
Passing Linux checks does not qualify macOS. The 0.17.3 workflow above qualifies
the teardown fixture and installation documentation guard on all three native
hosts. The 0.18.1 results below qualify the release/hook adoption, and 0.18.2
qualifies the later digest and README checks. The 0.18.3 snapshot and selected
commit checks remain pending matching native CI.

The `portable-hosts` and `pocket-ic-concurrency` workflow jobs exercise the
declared native hosts. For LOC tooling after installing its prerequisites:

```bash
/bin/bash scripts/dev/cloc.sh
```

The maintainer owns full pre-push, release and publication gates. Agents run
only checks affected by their authorized changes.

The adopted release runner and formatting hook come from reviewed Shared Tooling
revision `a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`. Release guard checks exercise its
patch/minor/major ordering, explicit staging, atomic branch/tag push and exact
automatic recovery, including fresh validation-only retries, with command stubs.
Consumer metadata checks use the actual Make callbacks in isolated workspaces,
copied consumer helpers, real offline Cargo metadata and manifest sorting,
and substituted Git/validation commands, so they do not commit, tag,
push or publish. Consumer hook checks use isolated indexes with this repository's
actual Make targets and tracked lockfile, real Cargo/rustfmt, and a substituted
failing target for rejection. The canonical upstream fixture now preserves
existing lockfiles and checks that formatting does not create absent root or
nested lockfiles through
[Shared Tooling issue #2](https://github.com/dragginzgame/shared-tooling/issues/2).
The consumer-owned check covers
automatic refresh, unrelated-edit and lockfile preservation, partial staging,
formatter failure, idempotence and installer refusal of another hook path.
Local hook activation uses `make install-hooks`; verify the effective
`git config --get core.hooksPath` separately from `make fmt-check`.
Consumer hook assertions explicitly stop on failure under Bash 3.2, including
index preservation, activation and the offline formatter environment; strict
mode alone does not enforce standalone conditional commands on that shell.
The reviewed installer canonicalizes both its own root and Git's repository
root; qualification fixtures canonicalize their private temporary directories.
An aliased-directory setup case exercises the actual Make target, including
refusal of a conflicting hook path. This adopts the upstream fix for
[Shared Tooling issue #1](https://github.com/dragginzgame/shared-tooling/issues/1)
without patching the snapshot; symlinked `TMPDIR` is a focused Linux reproduction
of the path-identity problem, not native macOS qualification.
The 0.18.1 portable-host results below qualify this adoption on native
Linux/ARM64/Intel; the earlier release passes do not qualify it.

At `v0.18.0` (`6f77928204a1993f3d6923df8cf3f9015baa4cd6`), the
[tag workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37349040375)
failed on Linux: release-metadata fixtures depended on a pending changelog
heading that finalization had removed, and portable formatting checks lacked
the toolchain's `rustfmt` component. The portable shell also continued past an
earlier failed check. Current fixtures own their candidate notes and cover
finalized-only history; portable setup explicitly prepares `rustfmt` and its
system Bash step stops on command, pipeline and unset-variable failures.
Qualification of these fixes is recorded in the 0.18.1 results below.

At `v0.18.1` (`a71fbe5ea23a87998ac4637ea53787d2d98e602c`), the
[Linux](https://github.com/dragginzgame/ic-testkit/actions/runs/37370973393/job/111967721036),
[ARM64 macOS](https://github.com/dragginzgame/ic-testkit/actions/runs/37370973393/job/111967720746)
and [Intel macOS](https://github.com/dragginzgame/ic-testkit/actions/runs/37370973393/job/111967721157)
portable-host jobs passed. These qualify the reviewed release/hook adoption,
fixture independence, formatter prerequisites, strict system-Bash execution
and inventory-failure rejection for that source. All three main MSRV jobs
also passed.

The [main workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37370973393)
passed the complete gate on both macOS hosts; the
[tag workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37370972911)
passed it on Linux and ARM64 macOS at the same source. Both overall runs are
marked failed and contain cancelled jobs. The cancelled-job logs were unavailable
when inspected, so their cause is not established here and neither run is
reported as a complete workflow pass. Successful portable jobs do not prove
live release or registry effects. Subsequent qualification is recorded below.

At `v0.18.2` (`fef7e127f36324146aff37844aae641bad90fb60`), both the
[main workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37427644025)
and [tag workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37427644455)
completed successfully. This qualifies the digest-status, raw-byte and README
preparation checks on all three native hosts. It precedes the 0.18.3 adoption
of Shared Tooling `9437bab201bb6071da0bdc4de0336daf553113f5` and the consumer's
selected-commit metadata checks; those changes need matching native CI.

At `v0.18.3` (`b6fbfe8fbaf75bef03473325451b8d363701b52b`), the
[ARM64 macOS checks](https://github.com/dragginzgame/ic-testkit/actions/runs/37430485768/job/112159943359)
and [Intel macOS portable checks](https://github.com/dragginzgame/ic-testkit/actions/runs/37430485482/job/112159942819)
failed because a Git stub continued after receiving the wrong selected commit under
system Bash 3.2. Enforcing the fixture assertions also reproduced publication
after failed README validation in a subshell. The pending 0.18.4 checks pass
locally under privately built GNU Bash 3.2.57 on Linux, using real offline Cargo
metadata and substituted Git/validation commands. That shell reproduction does
not qualify the change on native macOS; matching CI remains required.

The developer setup prepared before host-library integration adopts Shared Tooling 0.1.6 at
`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`, whose
[upstream workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37450707625)
passed. On Linux, explicit installation of the actual pinned jq/yq and six IC
executables, offline byte/version verification, dependency declarations,
installer rejection/preservation fixtures, formatting, consumer hooks and
release guards passed. Installer and declaration fixtures also passed under
privately built GNU Bash 3.2.57 on Linux. The initial sandboxed host installation
failed DNS resolution and retained its staging directory; explicitly authorized
network setup then passed without changing pins. Upstream qualification and
Linux shell substitutes do not qualify this consumer on native macOS. Its
portable CI matrix now performs explicit installation, offline checks and those
fixtures on all three declared hosts; matching runs remain required.

Pending release 0.19.0 adds the registry `ic-host-tools` 0.1.10 dependency and
removes local executable resolution in favor of its explicit Unix resolver.
`read_wasm` uses its bounded, typed file reader; cache sidecar reads keep their
local miss/error policy and retained identities. Portable native jobs select
focused transactional admitted-tool and bounded-Wasm integration tests in
addition to the benchmark driver. These changes require their own matching
native qualification; earlier 0.18 checks do not qualify them. Rust toolchain,
MSRV and supported host requirements are unchanged. Package versions remain
maintainer-owned until the minor release is prepared.

Local Linux checks for this pending integration passed: shared admitted-tool
transaction publication/reuse and changed-executable rejection, bounded Wasm
reads, watched-input and Wasm stamp recovery, malformed manifest recovery, the
benchmark driver/sampler, all-target Clippy, Rust 1.88 host compilation and the
canister-target check. Actual runs of the external-transform example built then
reused output; wrong digest and version authority rejected before changing its
public output. The registry archive digest matches the lockfile, and its source
matches reviewed upstream commit `e9417d0afb83c2a596fada3671fe941ac01ca2ae`.
These are local consumer checks, not a complete gate or native macOS evidence.

### Pending 0.19.1 publication admission

Focused publication fixtures pass on Linux with current Bash and private GNU
Bash 3.2.57. They exercise the actual Make target with substituted Git, curl and
Cargo commands: confirmed absence, already published versions, transport and
HTTP failures, explicit offline policy, Cargo failure, annotated tags, wrong or
missing tags and failed Git inspections with apparently valid output. This is
local fixture qualification; native workflow qualification remains pending.

The reviewed Shared Tooling snapshot remains at `a37771f`. Its [exact-version
registry observer](https://github.com/dragginzgame/shared-tooling/issues/10) is
still uncommitted in the sibling checkout, so adoption awaits a reviewed canonical
commit. Its [committed release-tag checker](https://github.com/dragginzgame/shared-tooling/blob/a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3/scripts/ci/check-release-tag.sh) also
admits failed Git commands when their output matches expectations, reproduced
with both Bash versions. Consumer admission stays outside the immutable
snapshot until those prerequisites are fixed upstream.

Pending 0.19.1 also checks read success before comparing retained validation and
readiness records. Focused metadata checks pass on Linux with current Bash and
private GNU Bash 3.2.57. Fault-injection fixtures exercise complete-output read failures
through the actual Make callbacks for recovery preflight, preparation, prepared,
commit, committed, tagged and push admission. They verify unchanged live metadata,
retained records and a single original validation gate, followed by successful
admission after restoring the reader. Git and complete-gate effects are substituted;
Cargo metadata is real, locked and offline. Native qualification remains pending.

The same metadata fixtures also inject complete-output failures into source and
prepared version reads and next-version derivation. They verify rejection before
metadata or receipt changes, preserve the original gate count, and admit the
same prepared identity after restoring the helpers. These are substituted helper
failures, with native qualification still separate.
