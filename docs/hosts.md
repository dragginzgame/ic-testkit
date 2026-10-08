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
  to prepare pinned jq/yq, ripgrep with PCRE2, cloc and IC executables locally,
  then `make tools-check` for
  offline byte/version verification. Versions and native digests have one owner
  in `ci/tool-versions.env` and `ci/ic-tools.tsv`. See the shared
  [bootstrap instructions](local-setup.md#bootstrap-prerequisites).
  Make selects `.tools/host/bin` and `.tools/ic/bin`; direct Cargo calls use the
  explicit PATH and `POCKET_IC_BIN` exports in the README.
- Perl for the prepared standalone cloc payload and read-only sibling tooling
  report; core modules supply the report's JSON and digest support.
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
cargo test -p ic-testkit --locked --offline --test server_runner
POCKET_IC_BIN="$PWD/.tools/ic/bin/pocket-ic" cargo test -p ic-testkit --locked --offline --test server_runner real_server_runs_a_separate_process_using_environment_startup -- --ignored --exact
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
make cloc
make cloc-tooling
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


After the maintainer released 0.20.0 at
`5ed6a2e62c06fb13f90891b0c53c7b393688e57e`, the existing dirty lockfile selected
all four published host crates at 0.3.2. Pending 0.20.1 preserves that selection
without resolving another upgrade or changing package versions. Offline cache
preparation succeeds with `cargo fetch --locked --offline`. Packaged Rust sources
match owner commit `c7c0d85765054909c05d86f6d3fd2c9965510335`.

The consumer regression covers equivalent dangling input aliases, symlink chains
and direct missing-target paths through its path-policy adapter, without creating
the missing input tree. Focused Linux cache-filesystem (9), transaction (28) and
digest (9) tests pass, with retained logs at `/tmp/ic-testkit-0201-cache-fs.log`,
`/tmp/ic-testkit-0201-transactions.log` and `/tmp/ic-testkit-0201-digest.log`.
Examples, Rust 1.88 host library and wasm32 library checks pass. This is local
qualification of the prepared graph, not native macOS acceptance.

The exact-source [host 0.3.2 workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37589678525)
passes Linux and MSRV but fails both macOS 15 architectures in
`existing_file_traversal_errors_survive_missing_suffix_normalization`: an expected
error instead returns a canonical temporary-directory path. Public failure
evidence and the native-contract investigation are recorded on
[host #1](https://github.com/dragginzgame/ic-host-tooling/issues/1#issuecomment-6033745092).
The pending consumer selection is not release-ready until that owner failure is
resolved and native consumer qualification succeeds. Existing 0.20.0 release
CI and the new patch's dependency qualification remain distinct.

### 0.20.1 updated owner qualification (2026-10-07)

The macOS owner blocker above is superseded by published IC Host Tooling 0.3.3,
`3d18ca9a9ed0ac5935a16c5bac99694d8e9a7d0a`. Its path fixture now compares with
native canonicalization instead of requiring Linux's traversal error on Darwin;
the production path implementation is unchanged. The
[exact-source owner workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37595113180)
passes Linux, macOS 15 ARM, macOS 15 Intel and MSRV. All four selected packaged
Rust source trees match that committed revision. Explicit selected-cache
preparation and subsequent `cargo fetch --locked --offline` succeed.

Shared Tooling 0.1.15, `bfb50bd0884b5e6c5ee9592056531c6108f96d73`, is adopted
through a clean committed export with 55 verified files. Unrelated sibling edits
are excluded and preserved. Its
[owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37593142226)
passes all configured jobs. The reviewed `make/tools.mk` replaces six copied
setup/check recipes and supplies LOC reporting. CI removes separate package-manager
ripgrep provisioning and uses the same checksum-pinned local tool set. Hook and
release-command fixtures project the include explicitly. No named Rust function,
method or type was removed in this batch; tool ownership moved to the snapshot.

Focused Linux qualification passes: cache-filesystem (9), transaction (28) and
digest (9) tests; example, Rust 1.88 library and wasm32 library checks; snapshot,
formatting, consumer hook preservation, consumer release adapters and metadata,
host installation/rejection fixtures, shared Make-command fixtures and tooling
LOC fixtures. Explicit host setup prepares pinned ripgrep with PCRE2 and cloc;
`make tools-check dependency-pins-check` then passes offline. `make cloc` reports
both workspace members. Logs use `/tmp/ic-testkit-0201-host033-*.log` and
`/tmp/ic-testkit-0201-shared0115-*.log`. Shellcheck and Perl syntax checks pass.

Both exact 0.20.0 consumer runs now pass:
[branch CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37591084435) and
[tag CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37591084619).
Those results qualify the released source, not this pending 0.20.1 graph and
snapshot. Matching native consumer CI and full maintainer validation remain the
delivery checks. No full consumer gate, package-version change, commit, tag,
push or publication was run during this update.

The subsequent compatible CI evidence change addresses
[#17](https://github.com/dragginzgame/ic-testkit/issues/17). Final failure-only
uploaders in `checks` and `portable-hosts` include installer diagnostics,
`target/validation-failures` and hidden host/IC installation candidates.
Portable operator checks select a retained directory under the runner's temporary
root. Upload scopes exclude the general build directory and expire after 14 days;
the original failing setup/check step stays failed.

Focused local evidence is retained at
`/tmp/ic-testkit-0201-ci-retention.iqV1QV`. An unchanged shared installer receives
a substitute incomplete download and returns status 23 through the same Bash
pipefail/tee setup as CI. Its failed candidate remains, no active tool selection
is published, and commands after the pipeline do not run. A controlled failing
Make target exercises the unchanged shared logger and retains complete failure
logs and temporary fixtures. Parsed workflow upload paths include those actual
files and enable hidden-file collection. Host-installer and consumer-hook fixtures
pass with the new explicit temporary-directory selection. Actionlint, dependency
declaration checks and whitespace checks pass. This proves local retention and
scope selection, not hosted artifact delivery; no remote workflow was dispatched
and no complete consumer gate was run.

### Pending 0.21.0 qualification (2026-10-07)

The maintainer selected 0.21.0 for the complete pending batch because workspace
discovery and default Cargo-state ownership change public behavior. Package
metadata remains 0.20.0 and the four selected host crates remain 0.3.3. Published
history is preserved. Host 0.4 is not adopted while its breaking reader/error
cleanup remains unpublished; the public host re-exports require minor-release
coordination even when direct Testkit calls do not use the removed APIs.

`workspace_root_for` delegates to prepared Cargo's offline workspace locator and
returns a fallible result. Real dependency-free Cargo fixtures cover App members,
explicit workspace selection, excluded packages, independent roots, unlisted
packages and missing paths. Discovery creates neither lockfiles nor build output.
All maintained Rust callers handle the result. New Wasm specs share workspace
compiler state across fingerprints under existing locks; explicit isolation and
caller-selected pruning/maintenance remain supported. A synthetic Cargo build
fixture demonstrates two distinct exact fingerprints using the same retained
compiler directory. Existing persisted layouts and byte identities are unchanged;
old isolated targets are preserved for explicit maintenance rather than reset.

The new `tick_until` predicate helper has four focused tests for initial success
and failure, exact round/poll ordering, completion at the budget boundary,
exhaustion and retained typed error causes. Progression is tested with controlled
round callbacks, not a live PocketIC server. Its public adapter calls upstream
time advancement and tick operations; it adds no wall-clock timeout or application
readiness policy.

Reviewed Shared Tooling 0.1.16 at
`b69507367d45e3db9543359e689e1fcba0467ff4` is exported through a private clean
consumer after verifying the existing 0.1.15 snapshot, preserving pending local
adoption and sibling work. The new snapshot has 56 verified files, including the
linked canister audit addendum. Canonical runner, release-metadata/finalizer,
tool-command and tooling-LOC fixtures pass locally. Consumer release adapters
exercise whitespace-bearing draft headings through both changelog views.
Snapshot, hooks, formatting, pins, Actionlint and local documentation links pass.
The [0.1.16 owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37598153506)
was still queued at the latest inspection; native owner acceptance remains open.

Focused Linux runtime results: 4 time-helper, 4 artifact-helper, 58 Wasm-cache,
12 Wasm-batch and 4 artifact-handoff tests pass (one optional handoff worker test
is ignored). The first batch attempt found a fixture relying on implicit isolation;
it now selects that maintained mode explicitly, and the full focused module
passes. Original failure evidence remains at `/tmp/ic-testkit-0210-wasm-batch.log`;
the passing retry and subsequent logs use `/tmp/ic-testkit-0210-*.log`.
All test/example callers compile, strict library Clippy, Rust 1.88, rustdoc and
wasm32 library checks pass. Consumer hook and release-adapter checks pass after
selecting the new draft. No full consumer CI/release gate, commit, tag, push or
publication was executed. Matching native consumer CI remains maintainer-owned.

### Shared Tooling 0.1.17 snapshot review

The reviewed clean source at `88f1d70cdf671aefb9507d7a81411ed5daa358b3`
refreshes the existing 56-file selection through the canonical distribution
helper. Selected file bytes and modes are unchanged from 0.1.16; its correction
is in the upstream LOC fixture, which this consumer does not vendor. Snapshot,
prepared offline host/IC tools, dependency declarations and whitespace checks
pass. No Rust source or dependency selection changed during this refresh.

The canonical upstream LOC fixture passes locally with checkout-local `TMPDIR`
and an inherited consumer `CARGO_TARGET_DIR`. A separate disposable enclosing
checkout with an ancestor Cargo target configuration still causes that fixture
to fail; removing only that configuration makes the control pass. This remaining
fixture isolation gap is reported on
[Shared Tooling #47](https://github.com/dragginzgame/shared-tooling/issues/47).
Evidence is retained under the checkout's `target/shared-0117-review.*`, including
the failing attempt and passing control. These are Linux fixture results, not
production reporter failure or native macOS qualification. The
[exact-source owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37601115116)
was queued at inspection. Full consumer gates and release effects were not run.

### Environment startup and command-scoped server qualification

Pending 0.21.0 adds environment selection and the command-scoped server runner
for [#19](https://github.com/dragginzgame/ic-testkit/issues/19). It reuses published
IC Host Process 0.3.3 for bounded version execution and the existing Testkit
process-group cleanup engine. Library command cancellation does not install
signal handlers; only the CLI owns those process-wide actions. The native test
gate explicitly selects the real cross-process probe after its ordinary tests.

Focused Linux results: 16 startup tests and six synthetic runner subprocess
tests pass. The subprocess cases cover URL selection, version/configuration
refusal, explicit TTL, borrowed server ownership, command spawn failure, status
propagation and owned descendant cleanup after completion or SIGTERM. Startup
cases additionally cover cancellation before effects and callback-panic cleanup.
The live runner starts the prepared PocketIC 16.0.0 binary and a separate
process creates and deletes an instance through the exported environment URL.
Its initial sandbox attempt failed at loopback bind; the authorized retry outside
that restriction passes. Both attempts remain in
`/tmp/ic-testkit-0210-server-live*.log`.

Focused strict Clippy, Rust 1.88 library/binary checks, wasm32 library checks,
rustdoc, formatting, snapshot verification and pin declarations pass. The earlier
Clippy attempts are retained separately from the corrected final check. Logs
use `/tmp/ic-testkit-0210-startup-final.log` and
`/tmp/ic-testkit-0210-server-*.log`. Full consumer CI/release gates and matching
native macOS delivery remain maintainer-owned; no commit or release was made.

Shared Tooling now has an active uncommitted candidate for the remaining LOC
fixture gap on base `88f1d70cdf671aefb9507d7a81411ed5daa358b3`. A frozen copy of
its two fixture scripts and reporter passes both focused fixtures with enclosing
Cargo/Git scratch, ancestor target configuration and inherited target selection.
Input hashes and logs are retained in `/tmp/ic-testkit-shared0117-candidate.*`.
The active owner checkout was preserved, and the current snapshot still records
the committed 0.1.17 source; candidate evidence does not establish committed
adoption or native host qualification.

### IC Host Tooling 0.4.0 consumer qualification

All four selected registry crates are 0.4.0. Their Cargo VCS metadata records
`6b171744def811882ba6c71d50135efa898302a9`, and each packaged Rust source tree
matches that clean owning revision. The
[exact-source owner workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37602699181)
passes Linux, macOS 15 ARM64/Intel and MSRV. These owner results qualify the
published source, separately from the pending Testkit adoption.

The existing locked 0.4 selection was preserved and explicitly prepared with
`cargo fetch --locked --offline`. Consumer changes reuse the host's typed I/O
conversion while retaining stale-sidecar/invalid-file policy, and use its bounded
reader for startup log prefixes. Live tests and the baseline example use explicit
environment startup. A compiler-shim regression qualifies reuse, invalidation
after selected compiler identity changes, and explicit `RUSTC` precedence with
real Cargo metadata and a synthetic Wasm producer. It does not qualify an actual
nightly build-std compiler or post-link optimizer.

Focused Linux results: 16 startup, nine cache-filesystem, one exact-stamp and one
compiler-identity test pass. The affected live integration targets pass 37 tests,
and the real cross-process runner creates/deletes an instance using the prepared
PocketIC binary. The first integration attempt exposed two assertions comparing
server-local numeric IDs across different servers; overlap checks now use URL
plus ID, and rebuild checks use outcome and recipe-call evidence. Both the failed
attempt and passing retry are retained under
`/tmp/ic-testkit-0210-host040-live-tests*.log`.

Strict Clippy for the changed library, binary, integration and example callers,
Rust 1.88 library/binary checks, wasm32 library, rustdoc, formatting, snapshot and
pin checks pass. Qualification logs use `/tmp/ic-testkit-0210-host040-*.log`; the
compiler regression uses `/tmp/ic-testkit-0210-shim-identity.log`. Full Testkit
CI/release gates and matching native macOS consumer qualification remain
maintainer-owned. No package metadata version, commit, tag or release changed.

The adopted Shared Tooling 0.1.17
[owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37601115116)
has since completed successfully. That result covers the committed snapshot;
the later LOC fixture corrections remain uncommitted owner work and are not
included in this consumer's snapshot.

### Shared Tooling 0.1.18 follow-up adoption

The awaited correction is committed at
`a3430b34b32a60f3b245a2b4f7e2f5321556fe56`. The canonical exporter refreshes a
private clean consumer after verifying the accepted 0.1.17 selection; copying
only its declared export preserves pending consumer work and active sibling
edits. The 57-file snapshot includes the newly required Rust setup helper used
by `make/tools.mk`. Its complete Rust set remains optional here; ordinary
validation does not install or select additional tools implicitly.

Committed-source LOC context, tooling-inventory, validation-runner and
tool-command fixtures pass. The LOC context combines enclosing Git/Cargo
membership, ancestor Cargo target configuration and inherited output selection.
Snapshot, prepared offline host/IC tools and dependency pins pass, as do consumer
hook/formatting and release-adapter fixtures and the actual manifest-selected
workspace LOC report. Export and fixture evidence is retained under
`/tmp/ic-testkit-shared0118-export.*`; consumer adapter logs use
`/tmp/ic-testkit-0210-shared0118-*.log`. The
[exact-source owner workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37604299590)
was still running at inspection; this local qualification does not establish
native macOS acceptance. Full consumer gates and release effects were not run.

### 2026-10-07: pending 0.21.0 server-exit follow-up

Against consumer base `665434f4b348f44dc3d3977245fae464524e3841` plus the
uncommitted server-monitoring follow-up, Linux startup tests pass (16), runner
tests pass (7), and the exact prepared real-server cross-process probe passes
(1). The controlled server-exit case verifies typed status/output and cleanup
of the pending command and descendant; it does not simulate a live TTL expiry.
Strict focused Clippy, Rust 1.88 library/binary checks and `make fmt-check` pass.
Locked offline cache preparation and all results, including the initial Clippy
failure before correction, are retained in
`/tmp/ic-testkit-0210-server-exit*.log`. No symbols, retained layouts or package
versions change. Full consumer gates remain maintainer-owned; the
[committed-base CI run](https://github.com/dragginzgame/ic-testkit/actions/runs/37607010214)
is queued and cannot qualify these uncommitted edits or native macOS behavior.

The Shared Tooling 0.1.18 owner workflow cited above subsequently completed
successfully. Source review confirms that our optional Rust setup commands
expose the installer route reported in
[shared #54](https://github.com/dragginzgame/shared-tooling/issues/54); this
checkout has a physical `.tools` directory and no `.tools/rust` installation.
No redirected installation or Rust-tool probe was performed, and the reviewed
snapshot remains intact pending an upstream repair.

### 2026-10-07: pending 0.21.1 tooling and CI follow-up

Released consumer base `13df6bcf7ca913fc045e10f2060de3968f1b870b` failed
[native CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37619749873):
macOS compared a canonical `/private/var` server path with a `/var` fixture
alias; the portable benchmark's Cargo substitute rejected workspace discovery;
the concurrency job omitted explicit server setup. The pending fixes retain
canonical production selection, qualify discovery separately from build calls,
and prepare pinned tools and `POCKET_IC_BIN` in that job.

The accepted lockfile selects all four IC Host crates at 0.4.2. Their cached
Rust sources match clean owner `6501d0e9fa7ba0439ec7a4010ca7bf0205e1d712`,
and each package records that commit and its correct path. Explicit
`cargo fetch --locked` prepares this selection before offline checks. Linux
startup (16), benchmark-driver (18), cache-filesystem (9), host integration (1)
and real prepared-server concurrency (9) tests pass. Strict focused Clippy,
Rust 1.88 library/binary compilation, wasm32 library compilation and formatting
pass; the initial formatting failure and corrected result are retained
under `/tmp/ic-testkit-0211-*.log`. The
[exact 0.4.2 owner run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37624014360)
was queued at inspection, so native owner acceptance remains separate.

Shared Tooling 0.1.19 is adopted from clean local commit
`a06e4719e3839b8eefcfb88ec8923aa88eb63ccc` through the canonical exporter.
The 58-file selection explicitly adds the common IC pin parser required by the
installer. Snapshot, prepared tools, pins, consumer release adapters and hooks
pass. Upstream Rust-route, combined-log and common Make-command fixtures pass in
an isolated copy, using substitutes rather than installing Rust tools or releasing.
Evidence is retained under the directory named by
`/tmp/ic-testkit-shared0119-evidence`. That commit was not retrievable from GitHub
and had no matching owner run at inspection; local adoption does not establish
remote delivery or macOS qualification. New optional alignment/disk checks are
not selected automatically. Package versions, retained layouts and published
changelog history remain unchanged; full gates remain maintainer-owned.

### 2026-10-07: released 0.21.1 and pending Rust-fixture qualification

Released source `a98d751fb6991c12a5d0cd8faaddabe1e879199d` passed
[branch CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37635582990):
checks, portable tooling and live concurrency succeeded on Linux and macOS 15
ARM64/Intel. MSRV was skipped in that run; prior focused MSRV evidence remains
separate. This supersedes the outstanding consumer-native status of the 0.21.1
fixes, while retaining their earlier failed runs.

Pending 0.21.2 exports the canonical Rust-tool fixture at the same reviewed
Shared Tooling revision, bringing snapshot coverage to 59 files, and selects it
in each portable-host job. Linux system-Bash runs pass with ordinary and aliased,
trailing-slash temporary roots. Snapshot, declaration pins, shell syntax and
workflow parsing pass; logs use `/tmp/ic-testkit-0212-*.log`. Substitute Cargo
qualifies path refusal/setup/retry without installing tools. These new working
bytes still require matching native CI under
[#24](https://github.com/dragginzgame/ic-testkit/issues/24). No Rust compilation
or full local gate is needed for this fixture/wiring change.

### 2026-10-07: pending 0.21.2 upstream adoption and cache composition

Shared Tooling 0.1.20 is exported from clean revision
`3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`, preserving the canonical Rust-tool
fixture selection and adding the required contribution rules. The 60-file
snapshot, prepared tools, dependency pins and formatting checks pass. The
repository description matches its current PocketIC testing scope. The owner
[0.1.20 run](https://github.com/dragginzgame/shared-tooling/actions/runs/37641211708)
passed Linux and lint/security but failed both macOS jobs. Those jobs expose no
steps through the jobs API, and log downloads return Azure `BlobNotFound`;
the cause and native acceptance remain unresolved.

The maintainer-selected lockfile contains all four IC Host crates at 0.4.3.
Explicit `cargo fetch --locked` prepared the cache; package provenance and every
cached source file match committed owner
`644d49c096ae05c2e17e1b6aacf14770988c5cf6`. Unpublished owner gzip edits were
preserved. Linux host integration (2), cache-filesystem (9), atomic-copy (1)
and the compiled artifact recipe (1) pass, as do strict library/test Clippy,
Rust 1.88 library/binary and wasm32 library compilation. The new composition
fixture uses real Cargo metadata, controlled compiler identities, synthetic
minimal Wasm and an admitted copy producer. It proves warm reuse, compiler
identity invalidation and independent optimizer-byte invalidation; it does not
qualify real nightly build-std or a production optimizer.

The host [0.4.3 run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37639415895)
passed Linux/MSRV but failed both Darwin architectures with E0308 at
`durable/mod.rs:371`: `Mode::from_raw_mode` expects Darwin's `u16`, whereas
`WriteOptions.permissions` is `u32`. This is a consumer macOS compilation
blocker. The selected lockfile is preserved pending upstream correction, without
a vendored dependency fork. A proposed checked conversion is retained in
`/tmp/ic-host-043-darwin-permissions.patch`; it is not applied or natively
qualified. GitHub issue writes failed with connector internal errors at that
inspection; no remote delivery was claimed.

Released 0.21.1 now also has successful
[tag CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37635583503).
The pending working tree has no matching native consumer run. Logs for these
focused checks use `/tmp/ic-testkit-0212-host043-*.log` and
`/tmp/ic-testkit-0212-shared020-*.log`; the initial compile failure and corrected
retry are retained. Full gates and manifest version changes remain
maintainer-owned.

IC Host 0.4.4 was committed during this review at
`d7785db667db0c2fd5135f4aaa0bf02a847c3126`. Its gzip hash/compare additions are
compatible, but this repository owns no gzip implementation to replace, and the
Darwin permission conversion remains unchanged. It is reviewed without changing
the maintainer-selected 0.4.3 lockfile or claiming the compilation blocker fixed.

### 2026-10-07: selected IC Host 0.4.5 qualification

The maintainer-selected lockfile now contains all four IC Host packages at 0.4.5.
Explicit `cargo fetch --locked` prepares that exact selection, and package
provenance plus every cached source file matches clean owner
`93a905b048bcaa2a0aed4214ac2f13f065dc2905`. Existing compatible `0.4` requirements,
workspace version and retained cache layouts remain unchanged. The additive
0.4.4 gzip helpers need no local replacement: this repository owns no gzip
implementation. No host source fork or cross-repository edit is used.

The [exact owner run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37645681743)
passes Linux and MSRV. Both macOS architectures now compile and run filesystem
tests, resolving the permission-width defect tracked in
[host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18). Both then
pass 57 filesystem tests and fail the invalid UTF-8 filename publication unwrap
with typed `BeforePublication`/EILSEQ at `durable/stream/tests.rs:205`. That native
filesystem assumption is tracked separately in
[host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19); full native
owner acceptance remains outstanding. IC Testkit's consumer evidence was posted
to that issue, without claiming a matching native consumer run.

Linux host integration/cache composition (2), cache-filesystem (9), atomic copy
(1), strict library/test Clippy, Rust 1.88 library/binary compilation and the
compiled artifact recipe (1) pass against the selected published 0.4.5 graph.
Evidence is retained in `/tmp/ic-testkit-0212-host045-*.log`. Earlier 0.4.3
failures remain bound to their original inputs. Full gates remain maintainer-owned.

### 2026-10-07: IC Host 0.4.6 native acceptance and consumer selection

Pending 0.21.2 selects the four IC Host packages at published 0.4.6, owner
`0fb05f9e18f032425188d68e1d69317a0f0127d5`. Scoped exact-version updates and
explicit `cargo fetch --locked` prepare the graph without changing other package
versions or retained Windows dependency selections. Every cached source byte
and package provenance matches that committed owner. No package manifest version,
sibling source or reviewed Shared Tooling snapshot is changed by this adoption.

The [matching owner run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37648086908)
passes Linux, MSRV and both macOS 15 architectures. This establishes native owner
acceptance of the permission-width repair and filesystem-specific filename
qualification, superseding the outstanding 0.4.5 owner status while preserving
its original failure evidence. Final filename refusal can occur after producing
ASCII staging bytes; `BeforePublication` denotes absence of final publication,
not absence of producer effects. Production admission is unchanged.

Focused Testkit host/cache composition (2), strict library/test Clippy and Rust
1.88 library/binary compilation pass locked/offline against the published 0.4.6
graph. Logs use `/tmp/ic-testkit-0212-host046-*.log`, including the retained
registry-index timeout/retry during selection. The consumer result was posted to
[host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19).
Remote Testkit main still identifies released 0.21.1 `a98d751` at inspection;
working 0.21.2 bytes have no matching committed native consumer run. Full gates
and manifest version ownership remain with the maintainer. Shared Tooling's new
PR-release changes are dirty work after its adopted 0.1.20 revision and are not
consumed as a moving baseline.

### 2026-10-07: Shared Tooling 0.1.21 adoption

Shared Tooling is refreshed through its canonical exporter from clean revision
`45e34e92b43edb9543d5b7212774f87f8334079f`. The 61-file snapshot selects the new
PR helper alongside the common release runner. The current consumer retains
its direct delivery policy and existing metadata/evidence adapters. Selecting
PR delivery still requires the consumer merged-preflight/receipt work described
in the shared release contract; the common fixture does not qualify those absent
adapters or real GitHub permissions.

Snapshot, prepared tools, dependency pins, formatting, consumer release-metadata
qualification and hook qualification pass. A private exact-source clone also
passes the canonical direct-runner and PR-runner fixtures: isolated Git histories
and bare destinations exercise recovery, merge/squash/rebase, fresh validation
and conflicts using substituted GitHub responses. No real release, PR creation,
merge or remote push is performed. Rust source and the selected dependency graph
are unchanged, so no Rust compilation is required for this adoption.

Evidence uses `/tmp/ic-testkit-0213-*.log`; the private source is identified by
`/tmp/ic-testkit-shared0121-evidence`. The matching
[Shared Tooling run](https://github.com/dragginzgame/shared-tooling/actions/runs/37652236506)
is queued at inspection. Released Testkit 0.21.2 is now
`2db7b4f6b616b484408695656e26207628d74c5f`; its
[branch](https://github.com/dragginzgame/ic-testkit/actions/runs/37651807716) and
[tag](https://github.com/dragginzgame/ic-testkit/actions/runs/37651807993) runs are
in progress/queued at inspection. Pending compatible notes select 0.21.3 in both
views without changing package metadata or published history. Native acceptance
for these new working bytes remains separate from the focused local checks.

### 2026-10-07: focused validation ownership cleanup

Pending 0.21.3 removes the local generic dangling-symlink and missing-parent
path tests because their owner is `ic-host-fs::path`, including
`dangling_symlink_targets_chains_and_parent_traversal_share_one_identity` and
`missing_parent_traversal_resumes_symlink_resolution`. Published Host 0.4.6 has
exact-source native acceptance recorded above. The caller-base wrapper and all
product cache/retention behavior remain local. The release tag adapter now
leaves hash admission to the selected shared tag checker, while still resolving
the consumer's version and exact HEAD.

Remaining cache-filesystem tests (7), cache input/output confinement (1),
caller-relative exact target integration (1), actual Make publication refusal
fixtures, snapshot and formatting pass. Logs use
`/tmp/ic-testkit-0213-cleanup-*.log`; the initial trailing-blank formatting
failure and corrected result are retained. No dependency or package version
changes accompany this compatible cleanup.

Released Testkit 0.21.2 source `2db7b4f6b616b484408695656e26207628d74c5f` now passes
[branch CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37651807716),
including checks, portable fixtures, MSRV and live concurrency on Linux and both
macOS architectures. Its tag run remains queued at inspection. This establishes
released-source qualification for the Rust-tool fixture and cache composition;
it does not qualify pending 0.21.3 bytes.

### 2026-10-07: Shared Tooling 0.1.22 snapshot refresh

Released Testkit 0.21.3 is `a8e83a1940e5f44927c6df95b5d1269a3ac699bc`; its
[branch](https://github.com/dragginzgame/ic-testkit/actions/runs/37661836622) and
[tag](https://github.com/dragginzgame/ic-testkit/actions/runs/37661836641) CI are
queued at inspection. The only open Testkit proposal concerns changing the
maintainer's pre-1.0 hard-cut policy; no compatibility bridge is introduced.

The canonical exporter refreshes the same 61-file selection to committed Shared
Tooling 0.1.22 `2687f26317952c43c685f7f799ed09288dc10a67`, using a private detached
source clone to preserve the sibling's active unpublished PR-runner changes.
Only selected documentation and the revision/digest records change: the current
release engine, consumer runtime and dependency graph remain unchanged. This
scope does not introduce a numbered release draft for a governance-only refresh.

Snapshot, prepared tools, declaration pins, formatting and whitespace checks
pass. The exact-source upstream LOC-context and failure-retention fixtures pass
on Linux with the consumer's prepared host bin directory explicitly on PATH.
They cover enclosing Git/Cargo configuration, inherited output selection and
physical/trailing-slash/aliased TMPDIR roots. These are upstream qualification
fixtures, not newly selected consumer CI fixtures. The initial attempt lacked
cloc on PATH and is retained separately; no tool was installed implicitly.
Evidence uses `/tmp/ic-testkit-shared0122-*.log`; the private clone is identified
by `/tmp/ic-testkit-shared0122-evidence`. The initial fixture is retained at
`/tmp/cloc-fixture-contexts.GfpVT4`.

The matching
[Shared Tooling run](https://github.com/dragginzgame/shared-tooling/actions/runs/37659875012)
is queued, so native owner acceptance remains outstanding. IC Host's committed
revision remains 0.4.6 `0fb05f9`; its active unpublished child-process and artifact
work is preserved and not consumed. No manifests, versions, cache formats,
release effects or broad gates change in this refresh.

### 2026-10-08: Host 0.5 adoption for pending 0.22.0

The workspace selects the four published IC Host 0.5.0 registry packages,
explicitly updated from 0.4.6 without changing unrelated lock selections.
`cargo fetch --locked --offline` prepared the selected graph successfully.
The [Host release workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37744999108)
for `db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7` passed MSRV and native Linux,
macOS 15 ARM64 and Intel jobs. This source now supplies process-group ownership
through `OwnedChild`; the earlier frozen dirty-candidate rehearsal does not
serve as registry-backed adoption evidence.

Focused working-tree Linux checks with the actual registry selection pass:
startup (16 passed, one live test ignored), server runner (7 passed, two live
checks ignored), artifact host integration (2) and Wasm artifact reading (1).
Strict library/test Clippy, Rust 1.88 library/binary checking, Wasm target
checking, `make fmt-check` and declaration pin checking pass. Cargo checks use
`--locked --offline` and this workspace's `target/`. Logs remain at
`/tmp/ic-testkit-022-*.log`. These tests use synthetic servers and transforms;
they do not qualify live PocketIC or native macOS execution of this changed
consumer source. Full gates and native consumer CI remain maintainer-owned.

The canonical 61-file Shared Tooling snapshot now selects 0.1.23,
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0`. Snapshot verification passes.
The release-runner and PR fixtures pass against a committed-source export at
`/tmp/ic-testkit-shared0123-proof`; the consumer does not vendor those upstream
fixtures. Initial attempts to invoke absent consumer fixture paths failed and
are retained at `/tmp/ic-testkit-022-release-runner.log` and
`/tmp/ic-testkit-022-release-pr.log`. The successful committed-source fixture
logs use the `-proof.log` suffix. The
[Shared Tooling 0.1.23 workflow](https://github.com/dragginzgame/shared-tooling/actions/runs/37746567888)
also completed successfully. These fixture runs do not publish a consumer
release or change its direct delivery policy.

### 2026-10-08: pending 0.22 process ownership audit

Applied `audits/flow-convergence-and-duplication.md` from Shared Tooling
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0`, with the root AGENTS.md overlay,
to the dirty pending 0.22 changes based on `a8e83a1940e5f44927c6df95b5d1269a3ac699bc`.
Source inspection traced managed startup/command ownership, observed Cargo
execution, bounded reads, atomic copies and cache fingerprint/retention owners.
This is a scoped flow review, not a whole-crate performance or safety verdict.

The remaining observed-Cargo child guard duplicated ownership and only killed
its leader on observer unwinding. It now uses published Host 0.5 `OwnedChild`;
a synthetic compiler-descendant test verifies that neither leader nor descendant
remains running after callback panic (zombies count as stopped, not reaped).
Three focused observed-Cargo tests, strict library/test Clippy and formatting
pass on Linux with the prepared locked/offline graph. Logs are retained at
`/tmp/ic-testkit-022-audit-*.log`. The new production flow removes 34 net lines
before documentation; this is a source reduction, not a measured speed-up.

Unobserved command-output execution has no callback unwind lifecycle and remains
separate. Cache retention locks and length-framed digest domains remain local
because they carry Testkit-specific retention and retained v1 identity contracts.
Resource-limit admission and staged executable installation were not added:
this reviewed flow owns neither replica admission nor tool installation.
Native macOS consumer CI and live PocketIC qualification remain outstanding;
these local results do not extend earlier source qualification.

### 2026-10-08: fingerprint exclusion error audit

The scoped `audits/code-hygiene.md` review at Shared Tooling
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0` and the root overlay examined
artifact digest and atomic-publication boundaries on the pending 0.22 tree
based on `a8e83a1940e5f44927c6df95b5d1269a3ac699bc`. Both fingerprint entry
paths previously discarded all exclusion-root canonicalization errors. The
shared local resolver now permits only `NotFound` as a not-yet-created root;
other failures stop acquisition before source-lease digest reuse. Valid input
framing and retained v1 layouts are unchanged. The bounded correctness finding
and local fix are tracked in [#26](https://github.com/dragginzgame/ic-testkit/issues/26).

All 10 focused digest tests and strict library/test Clippy pass against the
prepared locked/offline Host 0.5 registry graph on Linux; formatting passes.
The native symlink-cycle fixture verifies both fingerprint entrypoints and
cached composable reuse, while absent exclusions remain accepted. Logs remain
at `/tmp/ic-testkit-022-audit-digest*.log`. Atomic copying continues to delegate
publication to Host's durable writer; the local wrapper retains source and
destination error context. No performance measurement, whole-crate safety verdict,
native macOS qualification or retained-installation migration is claimed.

### 2026-10-08: scoped complexity review of pending 0.22

Applied `audits/complexity-and-technical-debt.md` from reviewed Shared Tooling
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0`, with the root AGENTS.md overlay,
to pending build-progress, batch-reporting and consumer release-adapter owners
on base `a8e83a1940e5f44927c6df95b5d1269a3ac699bc`. Local and remote Shared
Tooling main still select 0.1.23; no further snapshot update was needed.

The current progress axes are observer presence, optional heartbeat interval,
output forwarding and build/cache phase. `ProgressReporter` owns their execution
and failure-timing projection; batch reports preserve ordered results and partial
failure evidence. A low-severity duplication finding in `record_phase` was fixed:
phase selection now chooses one timing slot before shared saturating accumulation.
Input-resolution phases alone contribute to their subtotal; optional phases still
distinguish not run from zero elapsed. This removes 48 lines without removing
symbols, adding public modes or changing cache storage. It is not measured
performance evidence.

Source rehearsals covered adding a build phase (the enum/phase mapping and timing
slot remain explicit), changing release metadata files (the consumer adapter and
its recovery fixtures remain the owner), and changing observer output policy (the
reporter and stream projection remain separate from process ownership). No new
framework was justified. The release adapter's retained old/new metadata,
validation bindings and exact release files are consumer policy; extracting them
into a generic shared state machine would spread that ownership rather than
remove an evidenced contract duplicate. Batch domain errors and source-lease
invalidation also retain independent responsibilities.

During this review another writer changed Cargo.lock to Host 0.5.1 and newer TOML
packages. That selection was preserved, not resolved by this review. Initial
locked/offline preparation failed on missing Host 0.5.1 packages; ordinary online
preparation then failed to resolve static.crates.io in the sandbox. An explicitly
approved `cargo fetch --locked` with network access downloaded the selected
packages; subsequent locked/offline preparation succeeded. No version upgrade
was performed by the cache-preparation commands. The earlier failed batch log
remains at `/tmp/ic-testkit-022-complexity-batch.log`.

Against the actual prepared selection, 12 focused Wasm batch tests, 3 observed
Cargo tests, 10 digest tests and 16 startup tests pass (one live startup test
ignored). Strict library/test Clippy, formatting and shared snapshot verification
pass. Successful logs use `/tmp/ic-testkit-022-complexity-*.log`, including the
`batch-prepared` suffix. Host 0.5.1 source at
`81f9809861159def2fd0987fcb7961cda4afd969` has no library-source changes from
0.5.0; its [native workflow](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37750135927)
was still running when inspected. Neither that run nor these Linux checks
qualify native macOS execution of the pending consumer source. This is a scoped
complexity review with no unresolved actionable structural finding in the named
owners, not a whole-repository correctness or performance verdict.

### 2026-10-08: pending 0.22.1 failure-evidence archives

On released base `2951fd19e58799580e60ec0f6f5864d296271a62`, the local
workflow fix for [#28](https://github.com/dragginzgame/ic-testkit/issues/28)
archives each job's existing selected failure-evidence roots instead of giving
raw legal-Unix filenames to the artifact uploader. `collect-failure-evidence.sh`
owns only Testkit's checks/portable selections and records the pre-collection
step context. Missing early-failure files are skipped; original roots stay in
place. Successful tar creation precedes publication of `evidence.tar.gz`;
failed attempts retain `evidence.tar.gz.partial`. A second attempt cannot
replace an existing bundle. Intermediate linked parents are rejected, archive
links are not dereferenced, inherited TAR_OPTIONS is cleared, and Git metadata
is excluded. The existing failure artifacts retain their 14-day policy.

Shared Tooling remains committed at 0.1.23
`0ba0ad00ed94848e54ecc82629b6b7873b7284c0`; its archive helper and related
[#59](https://github.com/dragginzgame/shared-tooling/issues/59) work were dirty
and uncommitted at inspection. No sibling file or shared snapshot was edited.
The consumer adapter is explicitly outside the snapshot; adopting the eventual
reviewed helper remains separate from this local fix.

Linux focused native execution with GNU tar 1.35 passes early/late evidence
selection, colon/newline filenames, exact payload and outcome bytes, executable
and 0640 modes, links without outside content, metadata exclusion, prior-attempt
preservation, linked-parent refusal and partial-writer retention. Bash syntax,
ShellCheck 0.11.0, actionlint 1.7.12, YAML parsing and dependency declaration/pin
checking pass. The first mock-writer test failed because its substitute printed
partial bytes instead of writing the requested archive path; that attempt is
retained at `/tmp/ic-testkit-0221-evidence-test.log`. The corrected retry and
final results remain under `/tmp/ic-testkit-0221-evidence-*.log`.

Portable CI now creates a controlled qualification archive, uploads/downloads
it with immutable action selections, and verifies the downloaded bytes, modes,
links and step outcome on all three required native hosts. This adds no live
failing-job dispatch: actual artifact service and native macOS qualification
remain pending a run of this changed workflow. Qualification artifacts retain
one day; failure artifacts retain 14 days. These checks use no Cargo jobs,
resolve no dependencies, and make no release or publication claim.

The earlier 0.22.0 [tag workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/37752946475)
now passes, including full native gates and process/portable/concurrency tests;
its source is the released base above, not this pending archive change. Its
branch run was still queued when inspected.

### 2026-10-08: qualification-failure evidence containment

The pending 0.22.1 archive fixtures and downloaded-archive verifier now use the
retained portable TMPDIR in native CI. Portable collection also explicitly
includes the qualification source/download directories; checks collection
still excludes those portable-only roots. This prevents a failure in the new
qualification itself from leaving its detailed fixtures outside failure uploads.

Focused Linux proof at `/tmp/ic-testkit-0221-retained-verifier.h2FgWt`
creates a valid qualification archive, supplies an incomplete downloaded copy,
and runs the actual verifier with retained TMPDIR. Verification fails as
expected and keeps its extraction fixture. The actual portable collector then
archives that fixture and both qualification inputs; extraction and byte
comparisons pass. The valid qualification archive separately passes the downloaded
verifier. Original source inputs and all failed logs remain in that proof tree.
The maintained selection/round-trip fixture, ShellCheck and actionlint pass;
its local log is `/tmp/ic-testkit-0221-qualification-retention.log`.

These are native Linux archive/selection observations, not an actual artifact
service round trip or native macOS proof. The configured changed-workflow CI
remains the owner of those outstanding checks. Shared Tooling's helper still
has no committed source beyond the recorded 0.1.23 snapshot at inspection.
