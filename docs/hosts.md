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
uploading a package; the complete gate must not be forced offline. Preflight
prepares the locked dependency cache under Cargo's configured network policy;
metadata preparation remains offline. Explicit offline requests are preserved
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

### 0.19.1 publication admission

Focused publication fixtures pass on Linux with current Bash and private GNU
Bash 3.2.57. They exercise the actual Make target with substituted Git, curl and
Cargo commands: confirmed absence, already published versions, transport and
HTTP failures, explicit offline policy, Cargo failure, annotated tags, wrong or
missing tags and failed Git inspections with apparently valid output. This is
local fixture qualification; native workflow qualification remains pending.

Release 0.19.1 selected Shared Tooling `a37771f`; the exact-version registry
observer was then uncommitted, so publication used a consumer adapter. Its
[committed release-tag checker](https://github.com/dragginzgame/shared-tooling/blob/a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3/scripts/ci/check-release-tag.sh)
admits failed Git commands when their output matches expectations, reproduced
with both Bash versions. Tag admission remains in the consumer adapter until
that prerequisite is fixed upstream.

Release 0.19.1 also checks read success before comparing retained validation and
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

### 0.19.2 Shared Tooling adoption

The snapshot selects clean committed Shared Tooling `47cd2ccaf0e8b428f06e6db0262df76cfc1581de`
(0.1.7), exported from a separate pinned checkout. All 44 files verify against
that revision. The canonical registry observer replaces the consumer's inline
HTTP lookup; publication policy stays local. The tag checker is unchanged
upstream and is still excluded from adoption.

Focused publication fixtures pass on Linux with current Bash and private GNU
Bash 3.2.57. Offline installed-tool checks, IC setup/retention/activation fixtures,
consumer metadata fixtures, ShellCheck and upstream checksum/registry fixtures
also pass locally. Consumer metadata fixtures pass with both Bash versions.
Registry and installation requests are substituted in those
fixtures; offline tool checks inspect the existing native installation. Native
qualification for the changed consumer revision remains pending. Released
0.19.1 source `af200d4` has failed [main](https://github.com/dragginzgame/ic-testkit/actions/runs/37460659223)
and [tag](https://github.com/dragginzgame/ic-testkit/actions/runs/37460660218)
workflows. All six portable jobs failed because the IC tool fixture lacked
`rg`; all six complete-gate jobs failed because offline metadata fixtures could
not resolve uncached `ic-host-tools`. Main MSRV and both workflows' concurrency
jobs passed on all three hosts. Those successes do not qualify the failed gates.

The 0.19.2 release guard also adopts the canonical command checker and
retires the vendored shared-runner suite. The checker passes with current Bash
and private GNU Bash 3.2.57; its upstream nested-Make and retained-failure fixtures
pass locally. The focused `make release-guards-check` passes with real locked,
offline consumer Cargo metadata and substituted release effects. Consumer-owned
sequencing, metadata recovery and admission tests remain local. These results
do not qualify native macOS execution or execute a release.

The 0.19.2 workflow correction installs ripgrep explicitly on Linux and macOS
and performs `cargo fetch --locked` before the complete gate's offline fixtures.
Portable jobs retain their existing locked fetch. Actionlint and the dependency
declaration checker pass locally; the focused release guard passes against
prepared caches. Native CI for this correction remains outstanding; no full
gate, workflow rerun or package publication was executed locally.

The 0.19.2 release adapter also removes forced offline mode from preflight
fetching. Normal releases prepare missing locked dependencies automatically;
explicit offline requests still fail on cache misses. Recovery selects the
verified saved manifest rather than mixed live metadata. Metadata fixtures pass
with current Bash and private GNU Bash 3.2.57 for default/online/offline policy,
failed fetching with retained records and an explicit successful retry.
One scoped policy fixture checks both cache admission and the complete gate.
Network/cache-miss effects are
substituted; other metadata operations use the prepared real offline cache.

At `v0.19.2` (`827157434eb8b2d6c13c4b8e47493bd6a38678b9`), the
[main workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37477026029)
completed with successful portable-host, MSRV and PocketIC concurrency jobs on
all three hosts, plus successful complete gates on both macOS architectures.
The [Linux complete gate](https://github.com/dragginzgame/ic-testkit/actions/runs/37477026029/job/112314986282)
failed because an artifact handoff test still observed cached paths after its
final pruning pass. The same source's
[tag workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37477026703)
passed the Linux complete gate but failed the
[ARM64 macOS complete gate](https://github.com/dragginzgame/ic-testkit/actions/runs/37477026703/job/112314988175)
when the nonblocking request-reader fixture's single accept returned `WouldBlock`.
At inspection, the tag's Intel complete gate and Intel concurrency job were still
running. These observations qualify the completed jobs at that source, rather
than a successful complete workflow or the subsequent local changes.

Pending 0.19.3 explicitly releases retention locks after the final record owner
drops and reuses bounded accept polling in the request-reader fixture. A Linux
regression holding a duplicated retention descriptor open failed before the lock
fix and passed afterward, including clone and independent-acquisition protection.
The original artifact handoff failure did not recur in 30 local repetitions
before the fix; concurrent descriptor inheritance remains the inferred trigger
for that CI failure. Focused cache filesystem, pruning, retained-corruption,
artifact handoff and teardown checks pass locally after the fixes, as does
`make fmt-check`. Native macOS and complete workflow qualification of the changed
source remain outstanding. No broad gate or workflow rerun was executed locally.

The pending 0.19.3 snapshot refresh selects clean committed Shared Tooling
`d957d1f8801885c5b69e4a9ef900155f5f2a8a9d` (0.1.8), with 47 verified files.
Consumer version, installation, release and publication callers use the shared
TOML-aware version reader, and annotated-tag admission uses the shared checker.
The normal pin gate enables Cargo inheritance enforcement
([#11](https://github.com/dragginzgame/ic-testkit/issues/11)). Consumer metadata
writes, release source selection and recovery remain local. Existing manifest
and lockfile edits were preserved rather than selected by this tooling refresh.

Focused Linux snapshot, inheritance, installation, formatting, ShellCheck and
documentation-link checks pass. Publication and release guards pass with real
offline Cargo metadata and substituted Git, registry and release effects.
The metadata fixture driver also passes under GNU Bash 3.2.57 on Linux; Make
callbacks retain their configured shell. Shared Cargo metadata fixtures pass
against the matching committed upstream helpers, and refreshed host/IC setup
fixtures plus existing installed-tool offline verification pass. The shared
ripgrep capability remains opt-in; this consumer's setup still selects jq/yq.
The consumer hook fixtures also pass actual manifest sorting and Rust formatting,
partial-stage refusal, formatter failure isolation, unrelated-edit and lockfile
preservation, and private hook activation. Shared dependency declaration fixtures
pass locally through the adopted structured checker. These checks do not activate
the live checkout's hooks or provide native macOS qualification.

The [upstream 0.1.8 workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37484175750)
passed Linux portable regression and lint/security but failed both macOS portable
jobs. Neither upstream Linux results nor local consumer fixtures qualify native
macOS for this changed source. Consumer native qualification remains outstanding.

The subsequent pending 0.19.3 refresh selects Shared Tooling
`b32d3038c850a7c53470c326b0f7f11263b31669` (0.1.9), with 48 verified files
exported through the canonical helper from a separate clean pinned checkout.
Concurrent dirty upstream edits were excluded. Formatter admission now delegates
to the shared checker with the consumer's existing pin
([#12](https://github.com/dragginzgame/ic-testkit/issues/12)). Snapshot, formatting,
inheritance, consumer hook, release guard and refreshed host-tool fixtures pass
locally. The shared formatter failure fixtures also pass with GNU Bash 3.2.57
selected for the driver and subprocesses on Linux; formatter commands are
substituted in those fixtures. No tools are installed implicitly.

A consumer regression reproduced rejection of imported undated historical notes
in both changelog views. Passing the saved previous release to the shared
finalizer now preserves those sections; the release fixtures still reject
competing future candidates and preserve retained metadata. These tests use
real offline Cargo metadata with substituted Git and release effects.

At inspection, the [upstream 0.1.9 workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37489483879)
passed Linux and ARM64 macOS portable regression plus lint/security; Intel macOS
portable regression was still running. This is upstream evidence only. Native
consumer qualification remains outstanding, and no complete CI/release gate,
workflow rerun, release or publication was executed locally.

The next pending 0.19.3 refresh selects Shared Tooling
`21f3ec3dd97f2968c9f0b08924451bb2f71770d1` (0.1.10), retaining the same 48-file
selection through a clean pinned export. Dependency declaration fixtures now
preserve failed inputs and output. Snapshot, actual declarations, the normal
fixture suite and ShellCheck pass locally. An injected parser failure confirms
nonzero fixture status and retention of the private manifest and checker output;
the retained qualification directory is `/tmp/ic-testkit-0193-retention.6QXcox`.
No compiler-cache installer or tag-maintenance command was adopted.

The [upstream 0.1.10 workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37491682760)
passed Linux portable regression and lint/security. Both macOS portable jobs
passed dependency, Cargo metadata, formatter and hook fixtures before failing
in the new upstream fixture-retention test, which is outside this snapshot.
The logs identify retained failure directories but not the precise assertion.
These component results do not establish a passing complete upstream workflow
or native qualification of the consumer's uncommitted changes.

The latest pending 0.19.3 refresh selects Shared Tooling
`46c02774a8335cb3949d6f04284c4f53375353c1` (0.1.11), with 48 verified files
from a clean pinned export. Snapshot, dependency declarations, formatting and
isolated release guards pass locally. The finalizer also preserves notes for
adjacent SemVer components above the exact integer range of floating-point
numbers. Offline dependencies were explicitly prepared with
`cargo fetch --locked --offline`; manifest and lockfile selections were preserved.

The [upstream 0.1.11 workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37500153922)
passed Linux, macOS 15 Intel and macOS 15 ARM portable regression, plus
lint/security. This supersedes the earlier upstream failure for the selected
source, but native qualification of the consumer's uncommitted changes remains
outstanding. No complete consumer CI/release gate, workflow rerun, release or
publication was executed locally.

The maintainer subsequently selected pending 0.20.0 for the breaking public
IC Host Tooling split ([#13](https://github.com/dragginzgame/ic-testkit/issues/13)).
The locked graph now selects the four published 0.3.0 registry crates, re-exported
under their actual owner names. Focused Linux Wasm-read, digest, admitted-tool,
oversized cache-sidecar and malformed-manifest tests pass, as do example
compilation, host library checks with Rust 1.88 and the canister library check
for `wasm32-unknown-unknown`. Strict Rustdoc and the benchmark driver's bounded
report-read and standalone tool-resolution fixtures also pass. The isolated
release-adapter guards pass again against this selected lockfile, using real
offline Cargo metadata and substituted release effects. Package versions remain
maintainer-owned.

The [upstream host-library workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37497150858)
passed Linux, macOS 15 Intel, macOS 15 ARM and MSRV for the published split's
library sources. Matching native consumer qualification remains outstanding.
The additional streamed durable writer is still uncommitted pending upstream
0.3.1; the consumer's atomic publication engine is retained until that reviewed
API is available. No dirty sibling source or compatibility reader was adopted.

The subsequent shared extraction prepares explicit-base path resolution and
descriptor lock waiting, plus a fix for maximum-length atomic destinations in
the pending upstream 0.3.1 source. A private consumer replacement fixture at
`/tmp/ic-testkit-host-extraction.b9rs0cq9` passes focused digest/publication,
path, heartbeat, transaction and Wasm materialization checks using explicit
local-source selections with unchanged package versions and verified source
hashes. This is preparation evidence, not registry adoption or native macOS
qualification. The actual checkout retains its published dependencies and local
engines until reviewed shared publication. Follow-up remains on
[Testkit #13](https://github.com/dragginzgame/ic-testkit/issues/13).


The current pending 0.20.0 adoption selects the four published IC Host Tooling
0.3.1 registry crates. Every packaged Rust source file matches owner commit
`38a2a5127be064014e6d39d72d0300ffb2cf20be`. Its
[owner workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37580017649)
passed native Linux, macOS 15 Intel, macOS 15 ARM and MSRV. The consumer now
uses shared streamed durable publication, explicit-base missing-suffix path
resolution and descriptor lock waiting; the earlier private-fixture and
unpublished-source paragraphs record historical preparation only.

Actual registry-backed focused Linux checks pass: digest/publication (10), cache
filesystem (8), phase-aware lock heartbeat (1), transactions (28), Wasm
materialization (3), and admitted shared-tool output (1). Example compilation,
Rust 1.88 host library, wasm32 library, strict Rustdoc and library Clippy pass.
The selected graph was explicitly fetched before locked offline validation.
Cache framing, v1 identifiers and lock namespaces remain local and unchanged.

The latest reviewed Shared Tooling snapshot is 0.1.13,
`e378671d90afa237ff63a4b0e3b9551eb2c222b6`, exported from a clean committed
source with 51 explicitly selected files. The dirty sibling's pending changes
were preserved and excluded. Its
[owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37581058940)
passed Linux, macOS 15 Intel and macOS 15 ARM portable regression and
lint/security. The canonical distribution fixture passes locally, and the
isolated selected export passes documentation checks (92 references across
26 documents). Release-runner and exact-commit CI-helper fixtures passed for
0.1.12; those helpers are unchanged in 0.1.13.

The maintained fixture package moved from `canisters/test/perf_probe` to
`crates/ic_testkit_perf_probe`, retaining its name, source, package version,
features, unpublished status and selected lockfile. Consumer build, integration
and benchmark references, release metadata and hook fixtures use the new path.
Locked metadata, examples/integration-test compilation, fixture Wasm checking,
formatting, isolated release-adapter guards and consumer hook qualification pass.
The declaration checker inventories indexed paths, so the unstaged retirement
was qualified with a private index and private object store containing the
current working tree; the maintainer's index was not changed.

These local checks do not establish native consumer CI or full release
qualification. The committed shared snapshot still has the inherited Make
execution-mode gap tracked by
[Shared Tooling #30](https://github.com/dragginzgame/shared-tooling/issues/30).
Release all-clear remains blocked until its canonical fix is committed,
reviewed and adopted. No complete consumer CI/release gate, workflow rerun,
commit, tag, push or publication was performed.


A subsequent authorized local ownership cleanup removes the redundant
`digest::write_atomic` adapter; cache, ICP freshness, transaction-manifest and
Wasm stamp callers invoke `ic_host_fs::durable::write_bytes` directly. The test
`atomic_publication_failures_remove_only_the_owned_temporary_file`, which called
the shared writer rather than a consumer boundary, is retired; published owner
producer/rename cleanup coverage remains upstream. Consumer copy-context and
long-destination checks remain. This batch removes 33 net lines across its five
Rust files. Compared with Testkit HEAD
`827157434eb8b2d6c13c4b8e47493bd6a38678b9`, the combined digest/cache-filesystem
production sections now have 72 fewer lines, including the earlier shared
extractions and retained final-owner lock correction. These are scoped source
counts, not a whole-repository or performance claim.

After cleanup, focused registry-backed digest (9), cache filesystem (8), ICP
freshness (3), transaction (28) and warm Wasm publication (1) tests pass, as do
example compilation and strict library Clippy. The previous 10-test digest count
records the earlier source state. Digest framing/stream buffer reuse, retention,
namespace, destination admission and typed copy context intentionally remain
consumer policy; source inspection does not justify replacing them with the
shared libraries' different raw-identity or filesystem-admission contracts.
The native-consumer qualification and Shared Tooling #30 limits above remain.


The subsequent authorized duplication review uses
`audits/flow-convergence-and-duplication.md` and `audits/module-cleanup.md`
against the pending Testkit tree based on
`827157434eb8b2d6c13c4b8e47493bd6a38678b9`. It adopts committed Shared Tooling
0.1.14, `25e7ce83149e081e4dcc52c55c33724e44153f2a`, with 53 verified files
from a clean explicit export. The sibling's dirty LOC-helper work was preserved
and excluded. No package metadata or selected Cargo dependency was changed.

The two local Make validation loops now project their ordered target lists into
`run-validation-targets.sh --fail-fast`. The canonical runner owns dispatch,
failure propagation and retained diagnostic output; the Make targets still own
their validation selections. The shared `check-make-execution.sh` is included
in the snapshot dependency closure for validation, release and formatting hooks.
Repeated inline version-reader fixtures in the consumer release guard are
removed; the unchanged reader's generic contract is qualified by the owner's
Cargo metadata suite. Actual consumer ordering, failure at each stage, release
metadata recovery, installation requirements, publication/tag admission and
hook/index preservation retain their consumer checks. No named function, method
or type was removed in this tooling batch.

Focused Linux qualification passes: canonical validation-target runner, release
runner, owner release-metadata recovery, Cargo metadata, snapshot distribution,
shared hook fixtures, actual consumer release guards and consumer hook fixtures.
Snapshot and formatting checks also pass. Failed preparation evidence remains
in `/tmp/ic-testkit-0200-shared0114-guards.log` (the private fixture initially
lacked the newly selected runner) and
`/tmp/ic-testkit-git-hooks.kqSqgm` (missing Make-check helper projection); the
consumer fixture closures were corrected and reruns passed. These were fixture
projection failures, not failures of production validation or formatting.

The [0.1.14 owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37586649650)
passed Linux, macOS 15 Intel, macOS 15 ARM and lint/security. This reviewed
adoption supersedes the earlier outstanding canonical-fix status for
[Shared Tooling #30](https://github.com/dragginzgame/shared-tooling/issues/30).
Native consumer CI and maintainer-owned full pre-push validation remain pending.
No complete consumer CI/release gate, commit, tag, push or publication was run.

Intentional local separation: the Cargo metadata adapter owns manifest/lockfile
preparation, both maintained changelog views and interruption evidence; the
upstream repository's adapter handles its different VERSION-file contract.
Publication policy owns registry absence/uncertainty and the ic-testkit package
selection, while canonical helpers own registry observations and tag facts.
These trust and recovery boundaries were retained rather than consolidated
with semantically different upstream implementations. Existing selected
installers, pinning fixtures and release-command checks already match their
canonical owner and were not reimplemented. The scoped review supports this
convergence; it is not a whole-product or performance audit.
