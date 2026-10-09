# Host support and qualification

Initial 0.27.2 hook qualification used committed Shared 0.2.5
`04e07b4bf54e7aeb03eb7804a845cee27b7305df` through a clean isolated export;
the 80-file selection is unchanged. Hook setup now preserves literal existing
hook selections ending in newlines. The consumer fixture verifies refusal and
unchanged bytes for both custom paths and `.githooks` with a trailing newline.
The upstream focused suite also covers hooks in newline-named checkouts using
its substitute formatter; this does not qualify all Testkit commands in such
directories. Both upstream and actual consumer hook fixtures pass on Linux
under Bash 5 and genuine Bash 3.2; consumer Bash 3.2 also inherits CDPATH.
Partial staging, unrelated working edits, missing/wrong formatters and lockfile
preservation remain covered. ShellCheck, snapshot, pin and formatting checks
pass. Logs use `/tmp/ic-testkit-0272-{shared-export,consumer-hooks,consumer-hooks-bash32,shared-hooks,shared-hooks-bash32,shellcheck,checks}.log`.

That qualification preserved the maintainer-selected Host 0.9.3 lockfile (SHA-256
`5ab545cbacd036e17fcdde38046007219833caee17aa9fe161f4fd7378f0d11f`). Reviewed
Host source is `545e7236b91d84e190c80931b784f72cc4fafb11`; library sources
are unchanged from 0.9.2. Explicit locked offline cache preparation, actual
Rust 1.88 library/CLI compilation, and a fresh local CLI build plus offline
admission of the retained official Linux bundle pass. Logs use
`/tmp/ic-testkit-0272-host-{cache,msrv,server-check}.log`. No tools or server
were installed, and no dependency resolution or package-version change ran.
No function, method or type was removed. Native macOS acceptance remains
outstanding; the dirty upstream Shared 0.2.6 fixture change was excluded.
No broad CI/release gate, commit or publication ran.

The recipe consolidation initially adopted committed Shared 0.2.6
`ce13a5314916891fd239d9b199b4a91b04775054` from a clean isolated export,
adding only `make/release.mk` and `make/rust-format.mk` to the selection (82
files). Standard release goals, remote/branch defaults, conflicting-goal
rejection and simple root-workspace formatting have moved from the local
Makefile into those canonical includes. Consumer validation gates, metadata
admission, delivery policy, tool pins, explicit installation and `help` default
remain local. No function, method or type was removed; the seven Make targets
`release-patch`, `release-minor`, `release-major`, `release-resume`,
`format-tools-check`, `fmt` and `fmt-check` retain their names and move owners.
Isolated consumer fixtures explicitly copy the new includes before exercising
the actual Makefile.

Focused Linux snapshot/pin/format checks, ShellCheck, substitute release
forwarding/failure/conflict cases, default-goal admission, the actual consumer
hooks and the shared formatting fixture pass. Both formatting/hook controllers
also pass under genuine Bash 3.2; consumer hooks inherit CDPATH. The focused
release guards pass with controlled Git/registry/release effects and offline
real Cargo metadata. Logs use
`/tmp/ic-testkit-0272-shared-026-{export,checks,release-commands,default-goal,consumer-hooks,consumer-hooks-bash32,format,format-bash32,shellcheck,release-guards-retry}.log`.
The first guard run incorrectly forced offline mode across fixtures that test
other policies; its failed log remains at `release-guards.log` under the same
prefix. No broad gate or actual release ran.

The subsequently maintainer-selected Host 0.9.4 lock remains unchanged (SHA-256
`08b29067b556c1f84278d2b4999495c45c8b5a422c7ba2d37662721fcc3492b4`).
Host source `4e3daebd5df07c6449279535668436024a45c02b` has unchanged library
sources from 0.9.3. Its focused Rust 1.88 library/CLI check passes
(`/tmp/ic-testkit-host-094-{cache,msrv}.log`). Native macOS acceptance of this
consumer adoption remains outstanding. Later dirty upstream execution-guard
work is excluded rather than silently added to the committed snapshot.

The pending batch now selects committed Shared 0.2.7
`47d6ae6488b8007323fa7c2e22a6efa11d77ae63`, adding the declared
`make/execution.mk` companion (83 files). The new guard refuses ignore-errors,
dry-run, touch and question modes before recipes can run or be skipped. Both
consumer fixture projections include the guard and its behavioral probe.
The release checker explicitly binds its tooling root to its disposable
snapshot, including when the caller exports a different root. The generic
newline-checkout formatting checker is not selected here; no unused suite was
added to consume that separate upstream fix.

Sixty direct/inherited unsafe-mode cases using a copy of the actual consumer
Makefile pass with substitute Cargo/runner effects: all reject before either
tool runs. Ordinary parallel help with a quoted Make variable remains valid.
The exact committed shared release and formatting fixtures pass under Bash 5
and genuine Bash 3.2; the actual consumer hook fixture also passes under both,
with inherited CDPATH for Bash 3.2. Release isolation passes with an exported
nonexistent tooling root. Snapshot/pins/format checks, selected ShellCheck and
the focused controlled-effect release guards pass. Logs use
`/tmp/ic-testkit-0272-shared-027-{export,checks,release-isolation,consumer-modes,consumer-hooks,consumer-hooks-bash32,format,format-bash32,release,release-bash32,shellcheck-selected,release-guards}.log`.
The initial lint selection incorrectly named the unselected generic formatting
checker; that failed attempt remains in `shellcheck.log` under the same prefix.
No source fix or extra vendoring was required for that selection correction.
Host 0.9.4 remains selected with the unchanged lock digest above. No function,
method or type was removed. No broad gate or actual release ran; matching native
macOS acceptance of this source is still outstanding.

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

CI source qualification runs on main pushes and PRs. Standard releases push
main and its tag atomically; the main run qualifies that exact release SHA,
without a duplicate tag matrix. All four gate families retain all three hosts.
Pushed revisions use separate concurrency groups so consecutive releases keep
their evidence; superseded PR revisions can still be cancelled. No job depends
on tag-specific metadata, and failure evidence collection remains unchanged.
A tag-only push does not launch CI. Its source is qualified only by successful
matching main jobs at the exact tag commit; a PR merge-test SHA or an earlier
release is insufficient. Tags without that evidence remain unqualified. An
alternate release branch needs an explicit workflow/support-policy change.

The pending 0.27.1 selection change is locally checked with actionlint and
structural event cases for PR, main, atomic main/tag, tag-only and topic pushes.
All job bodies, matrices and concurrency settings match released 0.27.0 after
removing redundant job-level conditions. Logs are retained at
`/tmp/ic-testkit-0271-{actionlint,events,gates,tooling}.log`; matching remote
native acceptance remains required after delivery.

The pending 0.27.1 snapshot now selects Shared 0.2.3
`ac4549c5ebde497f7db0da5d05d32835112e51de`, exported from an isolated clean
checkout with the unchanged 80-file selection. Its optional installer-test
companion guard does not affect this consumer, which does not select that suite;
the changed selected files are snapshot/host documentation and CI-health task
guidance. No installer, runtime code, format or tool pin changed. The initial
export refused the sibling's dirty release metadata without changing consumer
files; that log remains at `/tmp/ic-testkit-0271-shared-023-export.log`. The clean
export and focused checks are retained at
`/tmp/ic-testkit-0271-shared-023-{export-clean,checks}.log`. Native upstream
acceptance remains outstanding; this documentation adoption does not relabel
earlier native results or install tools. The previously selected Host 0.9.2
lock remains intact; its library sources are unchanged from 0.9.1 and the
focused Rust 1.88 library/CLI check passed at
`/tmp/ic-testkit-host-092-msrv.log`.

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
- `cargo-sort` 2.1.4, prepared explicitly under `.tools/rust/bin` with
  `make install-format-tools` (the shared pinned Rust tool bundle), for
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
  `POCKET_IC_BIN` to the path returned by Testkit's offline check to avoid
  upstream automatic download. Explicit `make install-server` prepares the
  owner CLI and bundle. `make server-check` and test targets build the local CLI
  if needed, then verify the retained bundle offline; they never provision it.
  The CLI build uses this checkout's `target/` even if an enclosing consumer
  selects another Cargo target directory.
  The 0.27.1 missing-CLI repair is qualified with eight substitute-Cargo/CLI
  Make cases (ordering, shared prerequisites and failure refusal), plus actual
  locked offline CLI compilation and admission of the retained Linux bundle.
  Logs are `/tmp/ic-testkit-0271-server-prereq-{fixtures,real}.log`; substitute
  tests do not constitute a full test gate or native macOS proof. The original
  failed release-validation logs remain under `target/validation-failures/`.
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
CDPATH="$PWD" /bin/bash scripts/ci/test-release-runner.sh
/bin/bash scripts/ci/check-installation-version.sh
/bin/bash scripts/ci/check-publish-guards.sh
cargo test -p ic-testkit --locked --offline --lib pic::startup::tests
cargo test -p ic-testkit --locked --offline --test server_runner
server="$(target/debug/ic-testkit-server check)" || exit
POCKET_IC_BIN="$server" cargo test -p ic-testkit --locked --offline --test server_runner real_server_runs_a_separate_process_using_environment_startup -- --ignored --exact
cargo test -p ic-testkit --locked --offline --lib artifacts::host_tests
cargo test -p ic-testkit --locked --offline --test artifact_helpers
cargo test -p ic-testkit --locked --offline --test pocket_ic_teardown
cargo test -p ic-testkit --locked --offline --example fixture_reuse_benchmark_driver
cargo test -p ic-testkit --locked --offline --test pocket_ic_concurrency
```

Fetching is an explicit preparation step and preserves the selected lockfile.
For minimum-compiler qualification, prepare Rust 1.88.0 and its
`wasm32-unknown-unknown` target explicitly, then run
`CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 make msrv`. This records Cargo/rustc
versions and checks the native public package, independent Wasm library path,
and internal Wasm probe. CI prepares the same minimum compiler/target explicitly.
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

### 2026-10-08: Shared Tooling 0.1.24 adoption

The compatible 0.22.2 draft refreshes the existing 61-file selection from exact
committed Shared Tooling 0.1.24
`e9bfdc54c0daefc3dbbdfe091e5665dca5468eb3`. Export used a clean isolated checkout;
ongoing sibling edits were preserved. Companion admission accepts the existing
selection. No archive helper was added: [shared #59](https://github.com/dragginzgame/shared-tooling/issues/59)
records occupied-output, leading-option root and newline-parent defects in the
committed helper, with a corrected 0.1.25 implementation still uncommitted.
Testkit's released collector remains unchanged pending that correction.

Focused Linux checks pass for snapshot integrity, dependency declarations/pins,
ShellCheck, tooling LOC, snapshot distribution and release-runner regressions.
The runner fixture includes 15 real-Git tracking cases covering completed resume,
custom mappings, unrelated upstreams, divergent/raced observations and failed
updates. Logs are retained at `/tmp/ic-testkit-0222-{release,cloc,export}.log`.
These fixtures use an isolated committed source and inert product gate targets;
no product release, Cargo build, dependency update or broad validation ran.

At inspection, released Testkit 0.22.1 `fef41481a930a5f65615507cc8039ea69fb5ae87`
[tag CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37757385152)
has passed Linux checks, portable artifact qualification and concurrency;
native macOS jobs remain queued. Exact-source
[Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37756978585)
has passed Linux and lint/security, but Apple Silicon failed artifact download
and its consequent verification; Intel qualification is still running. Those
observations do not qualify this pending consumer snapshot on native macOS or
close either repository's archive issue.

### 2026-10-08: caller-owned PocketIC output and shared archive adoption

The compatible 0.22.3 draft adds
`PocketIcStartupConfig::with_server_output_files(stdout, stderr)` and paired
runner flags `--server-stdout` / `--server-stderr` for two new caller-selected
files. Parent custody, retention, presentation and disk budget stay with the
caller. Creation refuses occupied final entries and requests owner-only Unix
permissions. Raw streams survive partial preparation, startup failure, server
exit, command completion and cancellation; process-group ownership and bounded
public excerpts stay unchanged. Private port state is still removed. Connect
mode rejects output custody because it owns no server process.

Locked offline cache preparation passed without changing the selected lockfile.
Focused Linux startup tests pass (18; one prepared-server test ignored), including
markers after 20 KiB of raw bytes in both streams, private file permissions,
occupied file/symlink/FIFO refusal, partial preparation and original error/status
propagation. After final test cleanup, the two output-custody tests pass again.
Runner tests pass (8; two live tests ignored), including raw stream retention
following nonzero command completion and real SIGTERM cleanup of the synthetic
server, command and descendants. Package Clippy with library/tests/runner targets
and `-D warnings` passes. Initial new-test lint failures and corrected results
remain in `/tmp/ic-testkit-0223-{clippy,clippy-final}.log`; runtime logs are
`/tmp/ic-testkit-0223-{startup,startup-final,output-final,runner,runner-final}.log`.
The final startup and runner checks include absolute path anchoring at startup
and CLI relative-path selection; the final Clippy log covers that source.

The 62-file snapshot now selects committed Shared Tooling 0.1.25
`eeb72e741199bd8574280eacb3542d8379b912f6`, including its canonical archive helper.
The consumer collector retains explicit roots/outcomes, cleared TAR_OPTIONS,
partial-to-complete publication and retention policy. Shared and consumer archive
fixtures, ShellCheck, snapshot verification and dependency pins pass on Linux;
archive logs remain at `/tmp/ic-testkit-0223-{evidence,shared-archive}.log`.
The actual exported-archive verifier also passes a local copied-file round trip
at `/tmp/ic-testkit-0223-roundtrip.nphbOZ`; this is not hosted artifact proof.
Sibling dirty release-tracking edits were inspected and preserved, not adopted.

Released 0.22.2 `2da9f92fbe7fc31c37136e99cac46179e08911ad`
[tag CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37762453183)
passed all applicable checks on Linux and both native macOS hosts, including
portable artifact upload/download and process qualification. Exact-source
[Shared 0.1.25 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37762726615)
also passed. Those runs qualify their released sources, not this pending consumer
batch. Native macOS/process/artifact qualification for these consumer changes
remains with the configured CI; no broad gate, release or workflow dispatch ran.

### 2026-10-08: IC Host 0.6 and startup cleanup diagnostics

The pending batch selects 0.23.0 because the maintained local dependency update
from Host 0.5 to 0.6 changes publicly re-exported `ExecutionError` fields, and
invalid-port/builder startup error variants now retain output/cleanup evidence.
The initial maintainer-owned Cargo.toml/Cargo.lock edits were preserved without
another resolution or Testkit package-version change. All four selected registry
crates are locked at 0.6.0. Explicit locked offline cache preparation passed.
The consumed process capture sources match committed Host
`6f066e727c977e0b7ec8d3d77821df8508b95c64` byte-for-byte.

PocketIC version qualification now calls Host's `capture_group_command` instead
of direct-child capture. The existing startup deadline and output budgets remain
consumer-owned; wrapper descendants in the newly owned group are stopped on
natural exit or timeout. No background handoff, discovery, retry or download was
added. Managed invalid-port and builder failures retain the bounded server
streams and actual termination error. Display appends secondary cleanup evidence
without replacing the original startup cause. Borrowed-server failures capture
no owned-server output or cleanup. This does not add a bounded teardown or
process-tree confinement guarantee.

Focused Linux qualification passes 21 startup tests (one live test ignored),
eight runner subprocess tests (two live tests ignored), and the shared-tool
admission/publication/invalidation test. New cases exercise wrapper descendants
on exit and timeout, a real synthetic-server builder panic followed by reaping,
and deterministic error projections from captured output plus simulated cleanup
failures. Projection tests use the same constructors and Display as production;
they do not inject a real signalling/reaping syscall failure. Existing process,
retained-output, invalid-port and cancellation fixtures remain valid. Package
Clippy for library/tests/runner targets with `-D warnings`, Rust formatting,
dependency pins and diff checks pass. Initial new-test/formatter lint failures
remain in `/tmp/ic-testkit-0230-clippy.log`; corrected results are retained in
`/tmp/ic-testkit-0230-clippy-final.log`. Runtime evidence remains in
`/tmp/ic-testkit-0230-{startup,runner,host-admission,projections-final}.log`.

At inspection, released Testkit 0.22.3
`b2a04648c54842c0f9c69c9d6112d4a71944103f` has queued
[tag](https://github.com/dragginzgame/ic-testkit/actions/runs/37770038013) and
[branch](https://github.com/dragginzgame/ic-testkit/actions/runs/37770038453) CI.
[Host 0.6.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37769906817)
is also queued. Neither those pending runs nor the earlier green Host 0.5.2 and
Testkit 0.22.2 runs qualify these new paths on native macOS. Native consumer and
upstream qualification remain outstanding; no full gate or release ran locally.

### 2026-10-08: Shared Tooling 0.1.26 and literal evidence paths

The compatible 0.23.1 draft refreshes the existing 62-file snapshot to committed
Shared Tooling `75a8a60f49cec11d3f6aecab5c977029c42cc549` (0.1.26), using a clean
isolated checkout. Sibling dirty documentation and fixture work was preserved.
This includes the locked tracking-ref type check from later 0.1.25 source and
reviewed uncommitted-snapshot refresh from 0.1.26. The repository description
remains consistent with the current toolkit scope. Snapshot integrity and
focused canonical release-runner/exporter fixtures pass on Linux; the runner
checks all 19 real-Git tracking scenarios. Logs are retained at
`/tmp/ic-testkit-0231-{release,snapshot}.log`.

Consumer-owned evidence wrappers now anchor relative operands before physical
cd and preserve trailing newline bytes with a non-newline PWD suffix. The fixture
creates workspace/temp roots ending in newlines; CI invokes both creation and
verification with inherited CDPATH. Evidence selection, outcomes, completed and
partial archive disposition, link/metadata policy and retention remain unchanged.
The original CDPATH failure remains at `/tmp/ic-testkit-0230-cdpath-review.log`
and `/tmp/ic-testkit-evidence-test.DXS3iA`.

Focused Linux archive fixtures pass with Bash 5 and genuine Bash 3.2.57 selected
for parent and nested shells, both under inherited CDPATH. Logs are retained at
`/tmp/ic-testkit-0231-{evidence,bash32}.log`. The copied actual collector, archiver
and verifier also pass from a physical checkout whose name ends in a newline,
with Bash 3.2 and inherited CDPATH: proof, source digests, logs and exported archive
are retained under `/tmp/ic-testkit-0231-path-proof.HR8vn8`. This is a local
export/verification round trip, not a hosted upload/download result. ShellCheck,
actionlint, snapshot verification, dependency pins and diff checks pass. No Cargo
job, dependency update, package-version change or broad gate ran.

[Host 0.6.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37769906817)
is now successful at exact released source
`6f066e727c977e0b7ec8d3d77821df8508b95c64`. The selected Host graph is unchanged.
Released Testkit 0.23.0
[tag CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37772507706)
and [Shared 0.1.26 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37770856593)
remain incomplete at inspection. Those runs do not qualify this pending consumer
batch; native macOS/artifact qualification remains with the configured workflow.

### 2026-10-08: IC Host 0.7 and the 0.24 hard cut

The maintainer selected Host 0.7 and authorized moving the complete pending batch
from 0.23.1 to **0.24.0**. Both changelog views now carry the evidence-path and
Shared Tooling changes together with the Host upgrade and API removal. Package
version metadata remains 0.23.0 for maintainer-owned release preparation. The
selected lockfile has all four registry Host crates at 0.7.0; explicit
`cargo fetch --locked --offline` cache preparation passed without resolution.

The scoped surface/duplication review used `audits/module-surface-hardening.md`,
`audits/flow-convergence-and-duplication.md` and `audits/module-cleanup.md` from
Shared Tooling `75a8a60f49cec11d3f6aecab5c977029c42cc549`, with this repository's
AGENTS overlay (SHA-256
`9bb76e1cb57bb736f95edfc213b207d9fb70ce95b07237344b3b244b679e7fd8`).
Consumer source is HEAD `59b1c1de924116752282eac48c6531dce159ccc9` plus the
pending working-tree changes. Scope was the Host re-export boundary, Wasm file
helpers and owned process execution; retained cache schemas and canister runtime
were not changed or requalified.

`artifacts::read_wasm` duplicated the composition of Cargo's artifact path and
Host's bounded file reader. It and its re-export were removed. Benchmark callers
now compose `wasm_path` with `ic_host_fs::read::read_file` using the same limits.
The duplicate `compiled_artifact_reads_use_selected_profile_and_explicit_limit`
test was removed from `artifacts/wasm.rs`; Host owns its file-reader rejection
tests and the existing artifact-helper tests retain Cargo profile/path coverage.
No function was moved or renamed and no compatibility alias was added.

The remaining layout helpers retain Testkit's Cargo target/profile policy.
`PocketIcStartupConfig::run_command` retains arbitrary caller IO, nonzero status
propagation, no command deadline and owned-server monitoring. Host's
`communicate_child` requires a deadline and treats unsuccessful exits as execution
failures. Observed Cargo builds additionally require live byte callbacks and
heartbeat events, which that helper does not expose. Neither loop is an
equivalent offload; their existing `OwnedChild` lifecycle owner remains shared.
No runtime speed-up is claimed from the removed facade.

Focused Linux checks passed with Rust 1.99 and the locked registry graph:
21 startup tests (one ignored), two shared-tool/cache boundary tests, four
artifact-helper tests, the binary-only shared-target benchmark rejection test,
the artifact module doctest and strict all-target Testkit Clippy. Rust 1.88
library compilation and `make fmt-check` also passed. Logs are retained at
`/tmp/ic-testkit-0240-{cache,startup,host,helpers,benchmark-read,doc,clippy,msrv,fmt}.log`.
The benchmark check built a disposable Wasm workspace; the live PocketIC
performance probe was not run. Full tests and release gates remain maintainer-owned.

[Host 0.7.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37773664766)
passed MSRV, Linux and native macOS Intel/ARM at exact source
`491fc0e231b9650526f5f57b9ab7b1f62f02218c`. This qualifies the upstream release;
this pending consumer hard cut still requires its own matching native CI.

### 2026-10-08: consumer adapter paths for 0.24.1

Released source is `e7a9c6cbad29c4cfb8f5c9a5a0e64f82e9d9c7c1` (0.24.0).
The next compatible draft is 0.24.1; package metadata, the Host 0.7.0 lock
selection and the 62-file Shared 0.1.26 snapshot remain unchanged. The scoped
code-hygiene follow-up reviewed consumer shell entrypoints, nested Make calls,
and their helper paths against `audits/code-hygiene.md` from Shared
`75a8a60f49cec11d3f6aecab5c977029c42cc549` and the existing AGENTS overlay.

The publication fixture failed under inherited CDPATH before reaching its
shared version reader. Its log and inputs remain at
`/tmp/ic-testkit-0241-cdpath-before.log` and
`/tmp/ic-testkit-publish-guards.YjFyR2`. Correcting only the fixture exposed the
same defect in the relative release adapter invoked by Make; that attempt is
retained at `/tmp/ic-testkit-0241-publish.log` and
`/tmp/ic-testkit-publish-guards.fycv1K`.

Eight consumer-owned scripts now anchor their invocation before physical cd,
and preserve captured PWD bytes with a non-newline suffix: release guards,
metadata qualification, publication guards, hook qualification, installation
version checking, and the metadata/publication/tag release adapters. Their
existing admission, recovery, registry and cleanup policies are unchanged.
Native portable jobs now set CDPATH for release, installation and publication
checks. No function, method or type was removed; canonical exports were not edited.

Linux release guards (including nested metadata qualification and shared command checks)
and publication guards pass under CDPATH with Bash 5 and a genuine Bash 3.2.57
parent. Installation version checking also passes under Bash 3.2/CDPATH.
Explicit `/bin/bash` children in this Linux rehearsal still use Bash 5; this is
not native macOS qualification. Ordinary hook qualification passes with empty
CDPATH. ShellCheck, actionlint and diff checks pass. Logs are retained at
`/tmp/ic-testkit-0241-{release,release-bash32,publish-final,publish-bash32,install-bash32,hooks-normal,shellcheck,actionlint}.log`.
No source compilation, full gate, release or remote workflow dispatch was run.

A cloned checkout ending in a newline retains the source patch and successful
direct installation-version, tag-admission and metadata-files probes under
Bash 3.2/CDPATH at `/tmp/ic-testkit-0241-path-proof.SvDh6t`. Its direct mocked
publication checks also pass, but the subsequent Make qualification fails:
GNU Make's root derivation strips the trailing newline and cannot locate
`make/tools.mk`. The failed Make log remains in
`/tmp/ic-testkit-publish-guards.fBzt9U/annotated-tag.log`. These probes qualify
the adapters' bootstrap, not full Make support for newline checkout names.

Hook qualification under CDPATH separately reaches an unchanged canonical
installer defect. Inputs/log remain at `/tmp/ic-testkit-git-hooks.9rdl0p` and
`/tmp/ic-testkit-0241-hooks.log`; reproduction is reported on
[Shared #67](https://github.com/dragginzgame/shared-tooling/issues/67#issuecomment-6059936638).
The dirty sibling installer contains a proposed fix, which was not adopted.
The native hook check retains its existing environment until a reviewed shared
revision supplies that fix.

[Shared 0.1.26 Intel CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37770856593/job/113289949054)
hit the 15-minute regression-step timeout after its 19 real-Git tracking cases
and subsequent PR/metadata checks passed. Failure archive upload succeeded;
remaining native checks did not finish. This is tracked in
[Shared #71](https://github.com/dragginzgame/shared-tooling/issues/71).
Host's newer `81f47841aa5a75c36a0191869a81851b43cebef4` commit changes tooling
and reports, without Rust/dependency changes or a finalized 0.7.1 release.
No upstream dirty files or new dependency versions were selected. Consumer
0.24.0 CI was still incomplete at inspection; this draft needs matching native CI.

### 2026-10-08: Shared 0.1.27 tool-evidence adoption for 0.24.1

The pending 0.24.1 batch now adopts exact committed Shared source
`db039347d2372b877c1c46dcdd2b5c3aa9412009` (0.1.27), including the later
relative installer-path corrections. Export used a clean isolated checkout.
The snapshot grows from 62 to 65 files: the canonical tool selector, its test
and the companion action required by that test. The exporter initially rejected
the incomplete companion selection without replacing files; both attempts and
snapshot verification remain under `/tmp/ic-testkit-shared-0127-adoption.tDx2FC`.
Canonical files retain upstream bytes/modes. The baseline rules and repository
description are unchanged. The maintainer's pending Host 0.7.1 lock selection
was preserved; this batch did not resolve dependencies or change package metadata.

The consumer collector replaces its host/IC wildcard loop with the shared
selector's NUL-delimited root/path pairs. It explicitly selects compact mode
with this repository's pins. Active bundles are freshly verified offline before
their payloads are omitted; caller pins, check logs and IC receipts remain.
Failed, changed, unmanaged and unselected bundles retain full evidence.
Testkit still owns its log/fixture roots, outcomes, exclusive attempt directory,
partial archive and retention policy. No function, method or type was removed.
Consumer fixtures exercise verified active-set omission, full changed-set
retention, unselected candidates, early failure, parent-link rejection and
partial-output preservation through the actual collector. Synthetic tools are
used; no archive-size or collection-time saving is claimed from these fixtures.

The previously blocked hook fixture now passes under CDPATH, and native CI
enables that environment for the hook check. Shared host/IC installer fixtures
and their tool-evidence/action checks pass with Bash 5 and genuine Bash 3.2
selected for parent and nested `bash` commands on Linux. Rust installer fixtures,
prepared tool checks, dependency declarations/fixtures, consumer evidence
fixtures, hook qualification, release/metadata guards, ShellCheck, actionlint
and 65-file snapshot verification also pass. Explicit `/bin/bash` children in
the hook and metadata rehearsals remain Linux Bash 5. Logs are under
`/tmp/ic-testkit-0241-*new*.log`, `/tmp/ic-testkit-0241-compact*.log`,
`/tmp/ic-testkit-0241-evidence-*.log` and the adoption directory above.
The copied actual collector, selector, installers and archive verifier pass
from a newline checkout under Bash 3.2/CDPATH, including export and read-back.
Source patch/digests and artifacts remain at
`/tmp/ic-testkit-0241-compact-path-proof.Jz6Ihf`. This is local archive proof;
native hosted upload/download remains pending. No full gate or release ran.

[Exact-source Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37787910279)
has successful Linux and lint/security jobs; both macOS jobs remain queued at
inspection. The earlier `b866d410` run was cancelled. Shared #71's Intel timeout
budget is unchanged. Consumer qualification still requires matching native CI
for this pending source; earlier passes do not qualify the new collector.

### 2026-10-08: qualification identity and live test selection for 0.24.1

The pending workflow now downloads the archive by the numeric ID returned by
its own upload step. The actual Bash guard rejects missing/malformed IDs and
requires a new destination directory before dispatch; existing payloads remain
untouched. Download uses the pinned v8 action with merged extraction and strict
digest failure, followed by the existing payload verifier. The pinned action's
ID path still lists artifacts; this change establishes exact selection and
fail-closed inputs, not a proven fix for the observed hosted discovery failure.
The root cause and native round-trip acceptance remain with
[#33](https://github.com/dragginzgame/ic-testkit/issues/33). No retries, new upload,
workflow rerun or transport substitute were introduced.

Native CI also replaces the removed `artifacts::wasm::tests` selection with
`--test artifact_helpers`. All four maintained Cargo path/readiness/membership
checks execute and pass. The maintainer-selected lock now includes PocketIC
16.1.0; the first explicit offline cache preparation failed because that payload
was missing. Authorized `cargo fetch --locked` preparation then passed, followed
by the focused locked/offline test. Lock selection and package metadata were not
changed by this work. PocketIC 16.1's source accepts the pinned 16.0 server;
this artifact-helper test is not managed-server runtime qualification.

Actionlint passes. A disposable probe executes the actual workflow guard under
genuine Bash 3.2, proving valid identity admission, missing/malformed identity
rejection and occupied-destination preservation. Logs are retained at
`/tmp/ic-testkit-0241-{ci-cache,ci-cache-online,ci-helpers,ci-actionlint,artifact-id-guard}.log`;
the extracted guard is `/tmp/ic-testkit-0241-artifact-id-guard.sh`.
No production function, method or type was removed. Host 0.7.2 contains tooling
changes rather than new Rust runtime behavior; its dirty 0.8 work is a separate
breaking boundary and was not adopted for 0.24.1. New shared dirty fixtures
were likewise left unselected. Native consumer artifact transport remains pending.

### PocketIC 16.1 server selection

`ci/ic-tools.tsv` is the single shared-owned IC tool matrix in the reviewed
0.1.29 snapshot `1a54fb625d6e47efa64c4384808ecbc87be84e7e`, adopted for
pending 0.25.3. Its PocketIC 16.1 versions and hashes match the consumer selection
qualified in 0.25.0. The temporary consumer pin exception used in 0.25.0–0.25.2
is retired; future refreshes use the canonical catalog and common installers.

The Linux x86-64, macOS Intel and macOS Apple Silicon archive hashes were
reviewed against the official
[PocketIC 16.1.0 release](https://github.com/dfinity/pocketic/releases/tag/16.1.0)
on 2026-10-08. The selected Rust library is also 16.1.0. Explicitly run
`make install-ic-tools` before offline checks; the shared installer verifies
archive hashes, exact executable versions and the complete bundle before
switching `.tools/ic`. Prior sets and failed attempts remain available.
Native macOS runtime qualification remains with the configured CI matrix;
asset availability and local Linux checks do not qualify macOS execution.

Local qualification used the actual Linux x86-64 16.1.0 server installed by the
unchanged shared installer, the locked PocketIC 16.1.0 client, and the maintainer's
selected IC Host 0.8.0 crates. Startup (21 checks), synthetic server-runner (8),
real environment-selected server execution (1), live instance/pool concurrency
(9), and the benchmark driver's standalone tool-context fixture (1) pass.
Version-sensitive success fixtures now print upstream `LATEST_SERVER_VERSION`
instead of a copied 16.0.0 constant. Parser tests for accepting/rejecting the
supported major retain their deliberate fixed input examples.

Strict focused Clippy, selected-file Rust formatting, snapshot integrity,
dependency declarations and prepared IC bundle checking pass. Logs are retained
under `/tmp/ic-testkit-0241-pocket161-*.log` (the evidence prefix predates the
minor-target correction). The initial fixture-version failures and Clippy
borrow failures remain alongside passing final logs. The first installation
attempt could not resolve GitHub in the sandbox; authorized network setup
succeeded and retained both candidate sets. The first live runner attempt could
not bind localhost in the sandbox; the explicitly authorized socket-enabled
run passed. macOS runtime and full maintainer gates were not run locally.

The manifest already selected IC Host 0.8.0 when this qualification began.
Its public `child::CleanupError` and `tool::ExecutionError` add `term_error`,
which is breaking through this crate's public re-exports. The pending batch is
therefore numbered 0.25.0 in both changelogs; package versions remain maintainer
owned. No new cleanup timing policy is selected here. Shared default pin
adoption is tracked in
[shared #76](https://github.com/dragginzgame/shared-tooling/issues/76).

### Pending 0.25.1 live server CLI coverage

At released source `6b6d2cfe7f4c3e7a204a0c18beb6edb8895007b4`, ordinary
`server_runner` tests run through the full test gate, but the real-server case
is explicitly ignored and was absent from native workflow selections. The
pending workflow now selects
`real_server_runs_a_separate_process_using_environment_startup` in the existing
PocketIC concurrency job on Linux x86-64, macOS Intel and macOS Apple Silicon.
It uses the already prepared, receipt-checked `.tools/ic/bin/pocket-ic` and the
locked dependencies. This tests the actual CLI-owned server, URL propagation to
a separate worker, instance construction and command completion. It does not
replace synthetic failure/cancellation/diagnostic checks or concurrency checks.

The exact selected command passes locally on Linux with PocketIC client/server
16.1.0 and IC Host 0.8.0, with authorized localhost socket access. Offline locked
cache preparation, prepared IC tool checking, actionlint and whitespace checks
also pass. Logs: `/tmp/ic-testkit-0251-server-{cache,tools,live,actionlint}.log`.
This compatible coverage change selects pending 0.25.1 in both changelogs and
leaves package versions and published notes intact. Native CI has not run on
these working changes; no macOS result or complete release gate is claimed.

### Pending 0.25.2 Shared Tooling adoption

Shared Tooling revision `1872ed2c20f6c70689bb2249050b1d673c60bfa0` (0.1.28)
was reviewed and exported through its canonical helper from a separate clean
checkout. The 79-file snapshot includes the complete maintenance catalog and
its declared companions, plus the simulation-only release-runner fixture.
No timer or agent schedule is activated. `ci/ic-tools.tsv` remains outside the
snapshot as the single consumer-owned PocketIC 16.1 selection; its bytes and
other executable selections are unchanged. The export's initial temporary-clone
remote mismatch and initial reintroduction of default pin ownership are retained
in `/tmp/ic-testkit-shared-0128.*/export*.log`; both were corrected before tool
installation or runtime/tool validation, with no installed bundle switch or sibling edits.

The selected IC Host 0.8.1 crates declare Rust 1.88, so lowering the common
native/Wasm package floor below 1.88 is not supported by this selected graph.
The development compiler remains separately selected at 1.99. `make msrv` now
prints actual Cargo/rustc versions, checks the native public package, checks the
public library's Wasm path independently of the probe, then checks the probe's
Wasm path. The packages declare no optional feature axes. The standard-library
Wasm memory intrinsic replaces its equivalent `core` path, following the new
std-based code rule without a retained compatibility path.

Focused Linux qualification uses locked, explicitly prepared dependencies,
including the maintainer's pre-existing Host 0.8.1 lock selection. Native and
Wasm checks pass on actual Rust/Cargo 1.88.0. Host/IC installer fixtures and the
simulation-only release fixture pass with genuine Bash 3.2 (including nested
PATH-selected Bash), and the simulation uses its own real-Git refusal guard.
Malformed active links are rejected before execution/downloads and retain their
literal bytes. Native CI now runs that same release simulation; real-Git shared
tracking qualification stays with the upstream owner suite.

Snapshot verification, prepared offline host/IC tools, dependency declarations,
consumer release guards under inherited CDPATH, actionlint, formatting and
whitespace checks pass. Logs are retained under
`/tmp/ic-testkit-0252-*.log`. This is compatible work for pending 0.25.2;
package versions and finalized release history remain unchanged. Linux results
and source review do not qualify native macOS or a full maintainer gate.

### Pending 0.25.2 nonblocking cache lock adoption

`artifacts::cache_fs::try_lock_cache_file` now projects
`ic_host_fs::durable::try_lock_regular_file_with_parents` instead of opening and
trying an exclusive lock locally. The declaration requires ic-host-fs 0.8.1,
where the API was introduced; the maintainer-selected lock remains 0.8.2.
The shared `io::Error::from` projection preserves native WouldBlock and typed
admission causes. Busy-entry skipping remains local to retention pruning,
abandoned-staging pruning and replacement of unretained entries. Returned file
descriptors retain their exclusive lock through each deletion operation.

The new admission refuses redirected or non-regular final lock entries. Trusted
parents and namespace-lock ordering remain consumer responsibilities. Existing
v1 lock names and regular lock-file contents remain valid, without a layout
reset or another retained-format reader. Host performs durable setup before
nonblocking contention acquisition; this is not a filesystem-latency bound or
measured pruning speed-up.

Shared retention acquisition and final-owner explicit unlock remain unchanged,
as do observer cadence and post-open acquisition-duration measurement. Their
local opener and fs2 dependency remain necessary until the separate admission
composition gap in [Host #27](https://github.com/dragginzgame/ic-host-tooling/issues/27)
is resolved; no new consumer opener or try/unlock/reacquire bridge was added.

Linux focused checks pass: 8 cache filesystem tests (including contention,
reacquisition, redirected-file rejection, preservation of existing lock bytes
and final retention-owner release), 2 transaction pruning tests, 3 Wasm pruning
checks and 1 retained-corruption replacement check. Strict library Clippy,
actual Rust 1.88 public-library compilation, formatting, dependency declarations
and whitespace checks pass. Evidence uses
`/tmp/ic-testkit-0252-{cache-lock-*,transaction-pruning,wasm-*-pruning,retained-corruption}.log`.
These tests use actual native filesystem locks, not a mocked acquisition result.
Native macOS qualification remains with the configured CI jobs; no full local
gate was run. No functions, methods or types were deleted in this adapter change.

### Pending 0.25.3 Shared Tooling adoption

Shared Tooling 0.1.29 at `1a54fb625d6e47efa64c4384808ecbc87be84e7e`
was exported through its canonical helper from a clean, pinned checkout. All
81 snapshot files verify. The canonical IC catalog replaces the temporary
consumer selection; comparison of all non-comment records confirms unchanged
tool versions, hosts and hashes. Explicit `make install-ic-tools` verified the
canonical bundle before offline checks; prior installed sets were preserved.
The comment-only catalog change required another download, reported in
[Shared #79](https://github.com/dragginzgame/shared-tooling/issues/79).

Consumer release metadata now delegates source admission to the shared helper,
retaining its tracked regular-file checks and existing release recovery order.
The consumer fixture substitutes Git status and release effects; its untracked
file case uses real Git enumeration. The upstream owner's separate real-Git
fixture verifies source/index preservation and observation failures. Both pass
on Linux with genuine Bash 3.2 selected for Bash invocations, including inherited
CDPATH coverage; explicitly selected system-shell children remain system shells.

Prepared offline tools, dependency declarations, formatting, snapshot checks,
dependency-pin fixtures, release-runner simulations and ShellCheck pass. Logs
are retained at `/tmp/ic-testkit-0253-*.log`. No Rust source, manifest version or
lock selection changed, and no functions, methods or types were removed.
No schedule was activated. Native macOS and full maintainer gates remain
unqualified by these local checks. Further lock-opening reuse waits for the
committed and published Host API tracked in
[Host #27](https://github.com/dragginzgame/ic-host-tooling/issues/27).

### Pending 0.25.4 PocketIC provisioning owner

The published `ic-testkit-server` binary now provides explicit `setup` and
offline `check` alongside managed `run`. This release selects PocketIC 16.1.0
using the official asset digests reviewed on 2026-10-09:
[upstream release](https://github.com/dfinity/pocketic/releases/tag/16.1.0).
Linux x86-64, macOS Intel and macOS Apple Silicon assets are selected locally;
unsupported hosts fail before provisioning. Testkit's stable 16.x protocol
admission is independent of the client's latest downloadable server constant.
The exact provisioned version and override protocol policy are distinct.

Setup uses system curl's HTTPS-only redirect policy and finite download budgets,
Host's bounded digest/gzip admission, regular-file locking, durable file
publication and executable/version admission. There is no Cargo build deadline.
Whole bundles are admitted under the canonical `pocket-ic` filename in private
candidates before directory publication. Root namespaces must remain caller-owned
and free of concurrent untrusted writers; symlinked directory components, final
archives/binaries and redirected lock files are refused. Cooperating setup calls
serialize and recheck. No active pointer, new receipt format, migration reader or
automatic repair is introduced. Retained authenticated archives are the offline
authority for installed executable bytes; checks create no installation files.

Actual Linux execution downloaded and authenticated the official archive, passed
offline checking with `PATH=/nonexistent`, and ran a separate worker creating a
real application-subnet instance through `run` without a caller-selected binary
or URL. The successful native run is `/tmp/ic-testkit-38-live-run-native.log`.
The sandboxed run could not bind a loopback socket and remains separate failed
evidence. An earlier real setup exposed PocketIC's basename requirement; its
failed candidate and logs are retained. The corrected setup/check logs use
`/tmp/ic-testkit-38-live-{setup-final,check-final}.*`. Existing shared bundles and
the maintainer's dependency/CI-timeout edits remain intact.

Focused fixtures use authentic gzip/digest admission and real native file locks
with substituted downloads and executable scripts. They cover verified reuse,
changed-byte refusal before execution, failed/interrupted candidates, retained
previous bundles, redirected paths and concurrent setup. Existing runner
status/signal/diagnostic tests remain maintained. Native owner provisioning and
real launch are wired into the Linux and two macOS CI jobs; local Linux results
do not qualify either Mac host. No full local CI/release gate was run.

The current immutable Shared snapshot still requires PocketIC in its complete
catalog and receipts. Retirement remains coordinated through
[Shared #76](https://github.com/dragginzgame/shared-tooling/issues/76) and
[Testkit #38](https://github.com/dragginzgame/ic-testkit/issues/38); no shared
snapshot or sibling was patched to simulate that cut. Published-owner consumer
adoption and native qualification remain required. The separate operation-idle
lifetime work stays in [#37](https://github.com/dragginzgame/ic-testkit/issues/37).
No functions, methods or types were removed in this batch, and manifest versions
remain maintainer-owned.

Focused qualification passes: 5 provisioning fixtures, 9 maintained runner
fixtures, the protocol-policy fixture, bounded environment-probe admission,
strict selected Clippy, actual Rust 1.88 binary compilation, warning-denied public
Rustdoc, formatting, declaration pins, snapshot verification and actionlint.
Logs use `/tmp/ic-testkit-38-*.log`; initial compile/lint and runtime failures
remain distinct from final passes. No offline dependency resolution upgrade ran.

### Pending 0.25.4 shared refresh and cache admission

Reviewed Shared commit `635a39a9dd5f8d021fa9c9196b591e00521a7e02`
(the 0.1.32 batch; its committed VERSION remains 0.1.31) was exported from an
isolated clean checkout with the canonical helper. All 81 selected files verify;
the export log is retained under `/tmp/ic-testkit-shared-0132.*/export.log`.
Dirty sibling dashboard work was excluded. No fleet dashboard, Cargo-install
qualification workflow or schedule was added to consumer CI.

This evidence describes that intermediate selection. The later 0.1.33 adoption
and removal of the optional fleet reporter are qualified separately below.

The installer now compares validated tool-selection records while retaining
the original installed catalog as provenance. Genuine Bash 3.2 fixtures pass
under inherited CDPATH, including comment/reorder reuse and retained-selection
validation. Actual prepared Linux tools also pass an offline check with a
comment-only catalog copy; the active link and installed catalog digest remain
unchanged. A first copy included a disallowed blank row and was correctly
refused; that failed attempt remains separate evidence. No download or bundle
activation occurred during these checks. Logs use
`/tmp/ic-testkit-0254-{ic-fixtures,real-pin-reuse-final}.log`.

Native CI now groups pushed commits by SHA and cancels superseded runs only for
PR updates, following the shared source-owner policy. This prevents a newer
push from cancelling earlier native qualification; it does not turn cancelled
historical runs into passes. Actionlint passes for the changed consumer workflow.

The cache lock opener is now only a consumer error projection of
`ic_host_fs::durable::open_regular_lock_file_with_parents`. The declaration
requires Host fs 0.8.4 so existing locks avoid staging/sync and can be opened
under non-writable parents; the maintainer-selected lock currently uses 0.8.5.
Shared retention acquisition, record-clone lifetime, explicit final-owner unlock,
25 ms observer polling cap and acquisition timing after opening remain local.
The fs2 dependency remains necessary. Existing v1 lock names/bytes and cache
layout are unchanged, without a retained-data reset or compatibility reader.
Trusted parents and namespace stability remain consumer obligations; redirected
or non-regular final lock entries fail through Host's typed admission.

Linux focused checks pass: 10 cache filesystem tests, the exact-cache heartbeat
wait, 6 pruning cases, warm retained-Wasm publication and retained-corruption
refusal/recovery. Selected strict Clippy and actual Rust 1.88 library compilation
pass with the locked Host 0.8.5 graph. Offline tools, declaration pins, formatting,
snapshot verification and the isolated metadata fixture pass; logs use
`/tmp/ic-testkit-0254-*.log`. Two initial overly narrow test filters selected zero
tests and remain inconclusive logs; the named behavior checks above were then
executed successfully. Upstream Shared and Host native CI passed their reviewed
commits, but these local tests do not qualify native consumer macOS. Full local
CI/release gates remain maintainer-owned. No functions, methods or types were
removed; the private opener name remains as the narrow error adapter.

### Pending 0.25.4 optional fleet selection and benchmark precision

Shared Tooling `ddd3e1c01ba8aab13a56277e05679e43a8a9d88a` (0.1.33)
was exported from an isolated clean checkout through the canonical helper.
The consumer removed the unused `scripts/dev/cloc-tooling.pl` and its manifest
record under the published optional-selection procedure. The 80 retained files
verify; local setup, checks, cloc and checksum helpers remain selected. Consumer
help and README now direct fleet reporting to Shared Tooling. The canonical
optional target refuses the absent selection with an owner-directed diagnostic,
without invoking a sibling. Central audit task instructions remain applicable.
The deleted reporter's private functions were `usage`, `capture`, `read_file`,
`write_file`, `safe_path`, `is_linked`, `in_scope` and `load_snapshot`; all remain
owned by Shared Tooling's reporter rather than maintained here.

Offline snapshot/tools/declaration/format checks pass. The clean canonical
source's tool-command regression passes with substitute installers/reports;
its selected `make/tools.mk` bytes match this consumer. The local cloc
prerequisite check passes. Evidence uses
`/tmp/ic-testkit-0254-{shared-0133-checks,optional-tools,cloc-tools,no-fleet}.log`;
export evidence is `/tmp/ic-testkit-0254-shared-0133-export.log`.

All 28 benchmark integration tests and strict selected Clippy pass with the
maintainer-selected locked Host 0.8.6 graph. New native Rust cases cover every
counter near 2^53 and u128::MAX: distinct exact integer totals can share a finite
f64 average and yield zero percentage change. Public docs specify this existing
approximation and text rounding; public fields and report formats are unchanged.
Logs are `/tmp/ic-testkit-0254-{wide-benchmark,wide-clippy}.log`.

Committed Host 0.8.6 `9f3d9a83def91030056c78e44c9efaa489be7d12`
adds optional communication deadlines; its CI run 37896909262 passed MSRV,
Linux and native macOS Intel/ARM. Dirty sibling streaming no-follow hashing
work was inspected separately and was not adopted. The Cargo communication
refactor remains held under the maintainer's earlier instruction; no compilation
elapsed-time or retained-output cap was introduced. Shared 0.1.33 CI run
37898351268 had passed lint/security and Linux when inspected, with Intel
running and ARM queued. Local consumer evidence is Linux only; matching native
consumer macOS and full pre-push gates remain maintainer-owned.

### Pending 0.25.4 streaming executable verification

The snapshot now selects committed Shared Tooling 0.1.34
`3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822`, exported through an isolated
clean canonical checkout; all 80 files verify. The newly fixed optional
PocketIC alignment script is not selected here. No fleet reporter was restored.

PocketIC executable admission now uses Host fs 0.8.7's
`read::hash_file_no_follow` rather than buffering the executable to compute its
checksum. The existing 512 MiB bound, checksum authority, rejection before
version execution, trusted-parent custody and subsequent tool admission remain
unchanged. Archive authentication/decompression still requires its separate
buffer. No function or type was removed and no retained format changed.
The dependency minimum is 0.8.7; the maintainer-selected lock already contained
that version. Committed Host source is `8edce53c43bb872cd4aaa15659e37f0236adc203`;
its dirty direct-child ownership work was excluded.

Five provisioning fixtures, strict selected Clippy and actual Rust 1.88 binary
compilation pass on Linux. A freshly built CLI verifies the existing official
16.1.0 bundle with PATH=/nonexistent, without downloading or activating tools.
Logs use `/tmp/ic-testkit-0254-host-087-*`; snapshot/tools/pins/format evidence is
`/tmp/ic-testkit-0254-shared-0134-checks.log`. Native upstream CI for the reviewed
Host/Shared commits was queued when inspected; consumer macOS qualification
remains pending. Full gates were not run. The Cargo communication refactor stays
paused. Shared #84's compiled release-adapter startup finding does not match
this consumer's Bash release-version/preflight entry points.

### Pending 0.25.5 managed operation-idle lifetime

For #37, the CLI's `--idle-ttl SECONDS` and
`PocketIcStartupConfig::with_server_idle_ttl` select PocketIC's operation-idle
lifetime separately from its launch-relative hard TTL. Omitted idle selection
retains the server default; positive whole seconds and spawn mode are required.
The selected server's actual `--help` confirms a 60-second idle default and
independent `--hard-ttl`. No Cargo duration/output limit, dependency selection,
package version or retained format changed. No functions or types were removed.

On Linux, the final CLI ran a worker that waited 65 seconds before constructing
an application-subnet instance, with idle TTL 120 and hard TTL 180 seconds.
The worker and runner succeeded; complete streams/status remain under
`target/idle-lifetime-301766-0/`, and the invocation log is
`/tmp/ic-testkit-0255-idle-live-final.log`. An earlier successful run remains in
`target/idle-lifetime-123070-0/`; it predates the path-parser factoring.
Nine ordinary runner fixtures pass, including independent flag forwarding,
invalid/duplicate idle values, external-server refusal, exit status, signal and
cleanup behavior. Focused library checks verify positive bounds and opt-in
configuration. Strict selected Clippy, actual Rust 1.88 binary compilation,
format/declaration checks and actionlint pass. Logs use
`/tmp/ic-testkit-0255-*`; the initial parser-length Clippy failure is retained
separately from `idle-clippy-clean.log`.

The real idle test and evidence upload are added to native Linux/macOS Intel/ARM
CI; no native macOS result exists for these uncommitted changes. The released
0.25.4 source `085756dd0e0e2304de7e4a0b6b887201918646b6` passed its Linux
checks/portable/concurrency/MSRV jobs when inspected, with macOS queued/running
(runs 37901828926 and 37901828971). Shared's adopted revision is unchanged;
new committed Host 0.8.8 was reviewed but not adopted for this separate fix.
Full local CI and release gates remain maintainer-owned.

### Pending 0.25.5 Shared Tooling 0.1.35

Reviewed `be550afa57fe9e16872e5110b5cd69c24b4fa9e8` was exported through
a clean isolated canonical checkout; all 80 selected files verify. The snapshot
adds shared registry binary/example installation and the clarified distinction
between authorized dependency preparation and offline validation. Our Bash
release preflight already fetches the selected coherent manifest with `--locked`,
honours explicit offline settings, and selects saved metadata before fetching;
no release adapter change or release operation was needed.

The adopted Rust installer fixtures pass with substitute Cargo under Bash 5
and genuine Bash 3.2 with inherited CDPATH, covering both the established tool
bundle and the new selected-target mode. They qualify consumer snapshot
integration, not real registry installation or native macOS execution. Offline
snapshot/tools/declaration/format checks pass. Logs are
`/tmp/ic-testkit-0255-{rust-tools-0135,rust-tools-0135-bash32,shared-0135-checks}.log`;
export evidence is `/tmp/ic-testkit-0255-shared-0135-export.log`. No tools were
installed, product dependencies upgraded or functions/types removed. Upstream
0.1.35 CI run 37904190217 was queued when inspected; native acceptance remains
separate from these local Linux checks.

### Pending 0.25.6 streaming bundle checks

The maintainer explicitly authorized this scoped sibling edit from Host Tooling.
On Testkit base `311c39b9a7f9cc04fec050797c3324842e338328`, bundle checks now
authenticate the bounded compressed archive and use Host `hash_gzip` to derive
the executable identity without retaining its decoded payload. Installation and
checks share one archive-authentication helper; installation still decodes bytes
for publication. Existing limits, gzip integrity, executable admission and bundle
policy remain unchanged. No public contract, function or type was removed.
The compatible pending version is 0.25.6; package metadata is unchanged.

Focused locked/offline Linux checks pass: six provisioning tests, strict CLI
Clippy, binary build, selected formatting and documentation links. New regression
coverage proves authentication precedes decoding and rejects malformed,
truncated, CRC/length-invalid, trailing-data and concatenated-member archives
before executable admission. The freshly built CLI also checks the existing
official Linux 16.1.0 bundle with `PATH=/nonexistent`. Test downloads are
substituted; that final check uses the actual retained official archive/binary.
No installer, network download, full gate or managed-server launch was run.

The pre-existing dirty lock selects Host 0.8.9 and is byte-for-byte preserved
(SHA-256 `72c100b25547b7b307244ba93f89989b27fc83740ea935969d4163434195631c`).
The tested provisioning source digest is
`cd2286b3808ed0c836d278f7463388d04b53e058d1c153239fd2d08522414491`.
Logs are retained under `/tmp/testkit-gzip-0256/`. This establishes Linux behavior,
not a measured RSS/throughput improvement or native macOS qualification.
Delivery and the wider ownership handoff remain in
[#38](https://github.com/dragginzgame/ic-testkit/issues/38). No commit or release ran.

### Pending 0.25.6 Cargo communication convergence

For [#36](https://github.com/dragginzgame/ic-testkit/issues/36), observed and
silent Cargo builds now use Host's communication engine. Observed builds retain
their owned compiler group; leader completion can trigger descendant cleanup
before inherited output pipes reach EOF. Silent builds retain inherited
foreground group membership and noninteractive stdin through `spawn_direct`.
Testkit still owns event projection, heartbeat cadence and failure rendering.
Typed communication failures retain Host output and cleanup evidence and are
classified as Cargo-build failures.

The process dependency minimum is 0.8.8, using the already selected Host 0.8.9
lockfile without resolving dependencies. Retained stream limits remain
`usize::MAX` and the deadline is `None`; no compilation time limit or practical
output quota was added. Package metadata and retained formats are unchanged.

Five focused Linux fixtures pass: descendant-held pipes, concurrent raw-byte
streams, quiet heartbeats/live output, observer-panic cleanup and failure
diagnostics with observed output enabled/disabled and silent execution. The
descendant fixture gates pipe closure on a release file and requires completion
before releasing it; its watchdog is not a timing benchmark. Strict selected
Clippy and actual Rust 1.88 library/CLI checks pass. Logs are
`/tmp/ic-testkit-0256-{cargo-final,clippy-complete,msrv-final}.log`.
Six provisioning fixtures and a freshly built offline check of the official
Linux bundle also pass (`provision.log`, `provision-real.log` under the same
prefix). Native macOS consumer results and delivery remain outstanding; no
full gate, source commit, release or long production Cargo build was run.

The removed private communication implementation is replaced by Host and
`communicate_cargo_build`: `run_observed_cargo_build`,
`capture_observed_cargo_output`, `read_process_output`, `join_output_reader`,
`CapturedProcessOutput` and `ProcessOutputChunk` in `artifacts/wasm_cache.rs`.
Their implementation-specific reader tests were removed from
`artifacts/wasm_cache/tests.rs`: `InterruptedReader`, `InterruptedReader::read`,
`FailedReader`, `FailedReader::read`,
`observed_output_reader_retries_interrupted_reads_without_losing_bytes`,
`observed_output_drains_both_streams_through_a_bounded_queue`,
`observed_output_reader_stops_when_the_consumer_disconnects` and
`observed_output_reader_propagates_permanent_errors`. Host owns those reader
contracts; the consumer fixtures above verify real process/event boundaries.

### Pending 0.25.6 checkout-local formatter adoption

For [#41](https://github.com/dragginzgame/ic-testkit/issues/41), reviewed Shared
Tooling 0.1.38 `926a20606591214ab29faa236b0b584e4857439e` was exported from a
clean isolated checkout. All 80 selected files verify. This includes the
original-checkout tool PATH correction and stricter single-document JSON
admission in shared installers and the dependency checker. No vendored bytes
were patched locally.

`make install-format-tools` now delegates to the existing shared Rust bundle
installer, preparing the three already-pinned tools under `.tools/rust/bin`.
The global cargo-sort installation recipe is removed. Actual explicit registry
installation and offline `rust-tools-check` pass. The initial sandbox DNS
failure is retained in `/tmp/ic-testkit-0256-format-install.log`; the authorized
network-enabled retry succeeded (`format-install-network.log` at the same
prefix). Formatting and hooks never install tools. Existing local hook
activation remains `.githooks`; snapshot adoption did not change Git settings.

Focused Linux checks pass: actual formatting/hook preservation with no checkout
tool directories inherited in PATH, a trace proving the local formatter was
used, missing/wrong selected-version refusal before formatting, partial staging,
unrelated edits, lockfile preservation and installer conflict refusal. These
consumer fixtures and substitute-Cargo installer fixtures also pass with genuine
Bash 3.2.57 and inherited CDPATH. Shared hook and dependency admission fixtures,
ShellCheck, offline formatting, pins, tool and snapshot checks pass. Logs use
`/tmp/ic-testkit-0256-{formatter-local,hooks-bash32,rust-tools-0138,rust-tools-bash32,pins-0138,canonical-hook,hook-shellcheck}.log`.

Upstream 0.1.37 run 37909074809 has passing Linux/lint jobs but queued native
macOS Intel/ARM jobs when inspected. The maintainer requested local adoption
despite that outstanding acceptance; Linux Bash 3.2 is not native macOS
qualification. Configured consumer portable CI exercises these same fixtures on
both macOS architectures after delivery. No package version, product source,
dependency selection, function or type changed in this formatter batch. Full
gates, commits and release effects remain maintainer-owned.

### Pending 0.26.0 PocketIC ownership hard cut

Shared 0.2.0 `8140e3dd1b44409d682c721889ab702f438c6a17` is now the reviewed
80-file snapshot. Its documented handoff records passing three-host published
Testkit 0.25.4 provisioning acceptance (run 37901828971); that evidence is not
relabeled as qualification of these uncommitted consumer changes.

The shared matrix and installer now own five generic IC tools, without PocketIC.
The old Make server-path default and all three CI shared-server exports are
removed. Explicit `make install-server` builds the local CLI and invokes its
setup; `make server-check` performs offline admission. Make test targets and CI
use the admitted check-returned path. A missing/invalid owner installation
refuses before Cargo tests rather than requesting an implicit download. CI
prepares the owner before any live tests; the later duplicate setup step is
removed. No fallback installer, server catalog, format reader or compatibility
bridge was added. This changes maintained setup/test selection, so the complete
pending batch is 0.26.0; manifest versions remain maintainer-owned.

Explicit Linux setup activated `.tools/ic-set.xJTbQZ` with the five-tool pins.
Previous `.tools/ic-set.*` directories, pin receipts and failed evidence remain
intact. The installer publishes a separately admitted bundle rather than
reinterpreting the old receipt. The retained Testkit PocketIC bundle is unchanged.
Fresh CLI compilation, explicit owner setup/reuse, offline owner/shared checks,
six provisioning fixtures, actual managed launch and application-instance
creation without external server selection all pass. Shared installer fixtures
pass with Bash 5 and genuine Bash 3.2/CDPATH; formatting, snapshot, declaration
pins, hook preservation and actionlint pass. Logs use
`/tmp/ic-testkit-0260-{cache,cli-build,owner-setup,server-check,ic-install,provisioning,real-runner,ic-fixtures,ic-bash32,tooling,actionlint}.log`.

No functions, methods or types were removed in this hard-cut slice; the retired
Shared PocketIC checker files were not selected in this consumer snapshot.
No broad gate, commit, publication or sibling source edit ran. Native consumer
CI must qualify this exact adoption after delivery. Remaining downstream
coordination stays in Testkit #38 and Shared #76, rather than another local list.

### Pending 0.26.1 complete IC pin-row consumption

Reviewed Shared 0.2.1 `06b2e22f6bd213f1a590eb2a8797aee34c42dd69` is exported
from a clean isolated checkout on released Testkit 0.26.0
`9ad0d57a5c5c9ece9f03d2edcde1f8b4e84a3dc7`. All 80 selected files verify.
The installer and verifier now process a populated final TSV record at EOF,
matching the existing matrix validator. Pin bytes, tool selections and bundle
receipt semantics remain unchanged; no function, method or type was removed.

Focused substitute-download/archive installer fixtures pass on Linux under
Bash 5 and genuine Bash 3.2 with inherited CDPATH, covering all three host
selections without a final newline and refusal of wrong final-tool versions.
Prepared real tools, declaration pins, formatting and snapshot checks pass
without installation or dependency resolution. Logs are
`/tmp/ic-testkit-0261-{shared-export,ic-fixtures,ic-bash32,checks}.log`.
Upstream 0.2.1 CI run 37918240103 and Testkit's released 0.26.0 runs
37917989870/37917989313 were queued when inspected. Native macOS acceptance is
outstanding; substitute host selections and Linux Bash 3.2 are not native proof.
Both changelogs select compatible 0.26.1; package metadata remains unchanged.
No full gate, commit, publication, cleanup or sibling source edit ran.

### Pending 0.26.1 Shared 0.2.2 and selected Host 0.9.1

Shared `ee48bb37c98c771e77b92fd891f0757d8c1c8b99` is the clean committed
80-file export. It retains the IC final-row fix and documents exact-path CI-tool
publication; the CI installer is not selected or called here, so this adoption
adds no unused installer or claim of exercising that publication path. Generic
IC versions/checksums are unchanged. Snapshot, prepared-tool, pin and formatting
checks pass (`/tmp/ic-testkit-0261-shared-022{,-checks}.log`).

The maintainer's pre-existing dirty lock now selects all four Host crates at
0.9.1; its SHA-256 `3e5b695d614ba5e051b2dc6327bf4cb9dcf3a96a4626c502aba77ec5417f3f3a`
is preserved. Released Testkit 0.26.0 already declares Host 0.9 requirements, so
this introduces no new crate-identity boundary. Reviewed Host release source is
`4a016053525fa710bc13f3aedbe85a471b78f6ed`. Public APIs are unchanged; Host
simplifies durable publication and closes captured output descriptors at EOF.
No Testkit duplication becomes newly removable from these internal changes.

Explicit locked offline cache preparation succeeds. Five Cargo communication,
21 startup and six provisioning fixtures pass with that graph; one opt-in
startup test is ignored. Strict selected Clippy and actual Rust 1.88 library/CLI
checks pass. Logs use `/tmp/ic-testkit-0261-host-{cache,cargo,startup,provision,clippy,msrv}.log`.
The CLI freshly rebuilt with Host 0.9.1 also verifies the retained official Linux
PocketIC bundle with `PATH=/nonexistent`, without installation or download
(`host-cli-build.log` and `host-real-check.log` at the same prefix).
These are focused Linux consumer checks, not new native Host or Testkit CI proof.
No function, method or type was removed, package version changed, dependency
resolved or installation performed in this review/adoption slice.

### Pending 0.27.0 common startup-error hard cut

The maintainer selected 0.27.0 for [#30](https://github.com/dragginzgame/ic-testkit/issues/30).
The historical common-record candidate was rebased onto released 0.26.0
`9ad0d57a5c5c9ece9f03d2edcde1f8b4e84a3dc7`, preserving the pending Shared
0.2.2 and Host 0.9.1 changes above. Earlier 0.26.1 headings describe their
original qualification; both current changelogs now select 0.27.0. Manifest
versions and the existing lockfile selection remain unchanged.

`PocketIcStartupError` is now a common record; its cause variants move to
`PocketIcStartupFailure`. Bounded output and typed command/server cleanup
reports are projected once, keeping the original cause and error source.
The CLI preserves a failed command's status when server teardown fails, fails
a successful command on teardown failure, and prints cancellation diagnostics
before returning the signal status. Stable 16.x admission, setup/check, idle
TTL, borrowed-server ownership and retained raw files remain covered.

Focused Linux qualification passes: 23 startup unit tests (one opt-in test
ignored), seven CLI tests, nine ordinary runner tests (four opt-in tests
ignored), real managed PocketIC launch/application-instance creation, and all
nine live concurrency tests. Strict selected Clippy, warning-denied library
rustdoc and actual Rust 1.88 library/CLI checks pass. Logs use
`/tmp/ic-testkit-0270-{startup-final,cli-final,runner-final,real-runner,concurrency,clippy-final,rustdoc,msrv}.log`.
Constructed typed cleanup failures prove projection of both reports, including
native error codes; they do not simulate actual failing OS kill/wait calls.
Live tests exercise real ownership and ordinary teardown. Native macOS CI for
this dirty source remains outstanding; no broad gate or release ran.

Removed from `crates/ic-testkit/src/pic/startup.rs`: private `CapturedServer`,
its `From<CapturedServer> for PocketIcManagedServerOutput::from` conversion,
and `CapturedServer::{invalid_port_error,builder_thread_error,builder_panic_error,builder_disconnected_error}`.
Common output plus `finalize_failure` replaces those duplicate projections.
Private `PocketIcStartupError::termination_error` is replaced by separate typed
cleanup accessors. Test `startup_failure_projections_preserve_cleanup_and_bounded_output`
is replaced by `original_failure_output_and_both_typed_cleanup_reports_survive_projection`.
The public error enum representation is replaced by the same-named record;
its cause variants move rather than being silently discarded.
