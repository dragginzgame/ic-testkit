# Host support and qualification

## Prepared 0.53.1: private sample statistics

The private `ic-testkit-dev/src/statistics` module takes ownership of finite
sample projections from Metrics 0.13.4 source
`76cca76c8e591d91c9dbed6b276ac2ef57533335`, preparing for Metrics' 0.14 hard cut.
Its typed errors remain available as the driver's original error source;
count/mean/p50/p95 and the worker/report schema are unchanged. Unused minimum
and maximum projections are removed. The development package drops its direct
Metrics dependency; the published Testkit library still uses Metrics integer
benchmark arithmetic. No package version or external lock selection changes.

Five statistics units and three report-boundary units pass on Linux, including
invalid-input immutability, midpoint/nearest-rank distinctions, signed zero,
extreme fractions, non-finite means and original typed error sources. Strict
all-target development-package Clippy and Rust 1.91 compilation pass. This is
functional qualification, without a timing benchmark or a Wasm-size claim.
Full CI, native macOS and live server measurements remain separate. Focused
receipts are retained in `/tmp/metrics-hardcut-0140.7Fcf5q/`.

## Pending 0.53.0: package tests receive prepared PocketIC

The failed archive attempt at `target/package-qualification-GTmwz7/` reaches
live pool tests without `POCKET_IC_BIN` or an explicit server URL. The earlier
package receipt below supplied the environment manually and did not qualify
the actual Make target's configuration. The test target's command-scoped
selection does not persist into a later package target.

`make package` now builds the owner CLI, checks the existing prepared bundle
offline and passes its admitted binary to the package command and archived
tests. The existing consumer guard checks this actual recipe with no inherited
PocketIC configuration, preserved parallel Make jobserver access and retained
server bytes. CLI build, server admission and package failures stop later work
and propagate their status. The previous Makefile fails that same caller check;
the fixed caller passes under modern Bash and genuine Bash 3.2.57 on Linux.

The exact `make package` command passes with both PocketIC environment variables
explicitly unset, including archive tests, doctests and source/lock custody.
Its successful attempt is `target/package-qualification-YOKOxN/`. ShellCheck,
formatting and the isolated release-guard/metadata checks also pass. No dependency
or package-version changes are required. Native macOS and a complete release-gate
rerun remain separate. Source before-images, command evidence and the negative
control are under `target/package-server-env-053-evidence/`.

## Pending 0.53.0: Host 0.27 and Shared Tooling 0.23 adoption

The normal registry graph selects Host 0.27.1 and Build 0.16.0, with one Host
family across the library and development commands. The driver and marker test
use Build's typed binary selector and explicit direct-child `KillAndWait`
spawner. Cargo builds retain no elapsed-time limit. Own package versions remain
0.52.0; the exposed Host type change belongs in the existing 0.53.0 hard cut.

Shared Tooling 0.23.0 is exported from committed revision
`0b351cd2ba2bcd8927b783a5fec89afc8013f708`; all 119 payloads pass snapshot
verification. The selected roots retire no payloads: the upstream publication,
Make-copier, documentation-debt, Blob and semantic-Wasm retirements were outside
this consumer's selection. Their still-open producer ownership PRs supply no
new Testkit qualification claim. The refresh adds the common host observer,
uses validated catalog records and advances the optional ICP catalog to 1.7.0.
Generic IC tools remain an explicit setup selection. CI's four direct uploaders
now match the shared action's exact 7.0.2 source. The GitHub description was
reviewed against current purpose and remains accurate.

[Issue #122](https://github.com/dragginzgame/ic-testkit/issues/122) is repaired
locally by completing every required host digest in the synthetic evidence
catalog. Only the native host's synthetic transport payload is installed.
Modern Bash and genuine Bash 3.2.57 on Linux pass normal and rejected-selection
archive round trips, failure retention, prepared host/Cargo admission and the
actual formatting-hook caller. ShellCheck and workflow lint pass. The isolated
consumer release-guard and metadata fixtures pass with substituted release effects.
The issue stays open pending its required native macOS qualification.

On the selected registry graph, all 22 development library cases and nine
provisioning cases pass. Strict affected all-target Clippy, actual Rust 1.91
all-target compilation and warning-denied documentation pass. The real Wasm
marker test and a two-task fresh/pooled smoke with one worker and 64 state bytes
also pass; the smoke proves integration, without a performance comparison.
Coordinated archive qualification through Build 0.16 passes, including archived
target tests, doctests and source/lock custody, with evidence at
`target/package-qualification-atLsRu/`. The earlier sandbox attempt at
`target/package-qualification-FHPCFT/` retains its local-listener permission failures.

Dependency declarations pass with a disposable index reflecting the already
pending private-package move. The real index, own versions, refs and sibling
checkouts are unchanged. Full CI and native macOS remain unrun. Before-images,
passing and failed logs, and the smoke JSON are under
`target/adopt-host-shared-053-evidence/`.

## Pending 0.53.0: streamed provisioning and fixed reset coverage

The selected Host Artifacts 0.26.1 streams authenticated gzip payloads into the
existing private durable writer and returns the decoded identity for executable
admission. The decoded executable Vec and its separate hash pass are removed.
All nine provisioning cases pass on the normal registry graph, including corrupt
CRC, truncation and trailing-data refusal before execution or bundle publication,
retained attempts, exact version admission and concurrent setup exclusion.

Reset requirements and receipts carry independent fixed `ResetPolicies` fields.
Three reset/timing units pass, including missing versus irrelevant coverage,
all domain mismatches and first-failure ordering. Thirteen multi-canister pool
cases and ten concurrency cases pass with prepared PocketIC 16.1. The new live
probe rejects incomplete/mismatched extra-canister receipts before warm validation,
rebuilds and admits later valid reuse. The two dedicated server-kill cases remain
ignored. The initial sandboxed run could not bind localhost; an earlier version
of the new probe reached its time guard before coverage validation. Both failed
attempts are retained separately from the passing native run.

The private driver derives Display and its three conversions with Derive More
2.1.1. Its small manual Error source projection remains: a complete thiserror
derive exposed boxed types rather than the original causes. That failed attempt
is retained. All 22 driver cases pass with the original direct typed sources and
cancellation behavior. The macro dependency belongs only to the private development
package, not the published Testkit library. A two-task fresh/pooled smoke with one
worker and 64 state bytes passes; its report is
`target/reset-stream-driver-053-evidence/benchmark-smoke.json`. This proves
integration, without claiming a speed or memory benchmark result.

Strict affected-target Clippy, actual Rust 1.91 all-target compilation,
warning-denied native docs and all three Testkit doctests pass.
[Canic draft PR #594](https://github.com/dragginzgame/canic/pull/594)
passes strict Clippy and actual Rust 1.91 for its affected native test
targets with only the Testkit and matching codec source overrides. Its normal
manifests/lock are restored; normal adoption awaits Testkit/Candid 0.53 publication.
Formatting and dependency declarations pass, the latter with a disposable index
reflecting the pending private-package move. Full CI, managed Canic journeys and
native macOS remain separate. The ordinary sibling checkout, own package versions and real Testkit index are unchanged.
Before-images, candidate identities and raw passing/failed logs remain under
`target/reset-stream-driver-053-evidence/`.

## Pending 0.53.0: one reply model and shared qualification owners

Candid calls use only the argument-sequence functions. Existing tests retain
wire/work bounds, exact principals and rejections, transport classification and
the original unrelated panic payload. Six codec-boundary units and seven live
call/teardown parents pass, including the three child probes selected by their
parents. The development package's 21 library cases and two-task fresh/pooled
smoke pass with explicit one-value, one-unit and tuple-valued reply shapes.

Private programs now belong to `ic-testkit-dev`; Make/CI and release fixtures
follow that owner. IC Build 0.15 qualifies the two exact coordinated archives,
including all-target tests and doctests, isolated verification artifacts and
before/after source/lock identities. The successful attempt is retained at
`target/package-qualification-RrerMJ/`; an initial missing archive-directory
attempt remains separately recorded. Publishing dry runs use normal registry
admission. This task performed no publication.

Strict affected-target Clippy, actual Rust 1.91 all-target compilation, native
warning-denied documentation and formatting pass on the normal registry graph.
Dependency declarations pass with a disposable index reflecting the moved
manifest; the real index is unchanged. The ordinary pin check still sees the
tracked old manifest until the maintainer stages its deletion.

[Host PR #184](https://github.com/dragginzgame/ic-host-tooling/pull/184) owns
the shared ordered label validator. An isolated Testkit consumer using only
that Artifacts source override passes six diagnostic units, strict all-target
Clippy and the live exact-sender diagnostics case. Its deletion patch is
`target/three-cuts-053-evidence/consumer-labels.patch`; normal adoption awaits
Host publication. [Build PR #65](https://github.com/dragginzgame/ic-build/pull/65)
prepares the other consumer. These are source-qualified patches, without a
normal-checkout source override. Full CI and native macOS remain separate.
Before-images and focused logs are under `target/three-cuts-053-evidence/`.

Caller preparation selects the already published 0.52 argument-sequence functions.
[Canic PR #592](https://github.com/dragginzgame/canic/pull/592) and
[Toko Miner PR #52](https://github.com/dragginzgame/toko-miner/pull/52) use explicit
qualification-only ceilings: 16 MiB reply, 100 million decoding work, one million
skipping work, 4,096 types and 256 KiB header. Canic's public testing library and
internal harness pass strict Clippy and actual Rust 1.91 compilation; its affected
native test targets compile and pass strict Clippy on the registry graph. Toko Miner's User Hub test
target compiles and passes strict Clippy. These checks do not execute application
journeys. [Toko draft PR #1843](https://github.com/dragginzgame/toko/pull/1843)
passes formatting and its test-support library compiles, but its three project
test targets still require migration from retired artifact imports before their
qualification can run.

## Pending 0.52.0: standalone restore recovery qualification

The standalone deleted-canister test matches the pool-owned typed snapshot
failure directly. Failed restore/teardown timings, slot disposal and the next
acquisition's rebuild assertions remain in place. All ten ordinary concurrency
cases pass with prepared PocketIC 16.1; the dedicated server-death case remains
ignored. Selected-target strict Clippy and formatting pass on the normal registry
graph with Build 0.15.0, Host 0.26.1 and Metrics 0.13.3. Evidence is under
`target/standalone-restore-052-evidence/`. This focused run does not cover full CI
or native macOS qualification.

## Pending 0.52.0: private benchmarks, typed modes and derived errors

Fixture benchmark programs, their shared modules and live marker qualification
now belong to private `ic-testkit-bench`. Make/CI, selected compiler-floor checks
and isolated release fixtures include the package; publishable archives exclude
it. Coordinated archive verification seeds its lock from the packaged Testkit
closure and projects only Candid onto its extracted sibling archive. External
versions stay selected and the verification build still uses `--locked`.

The incoming dependency updates select Build 0.14.0, Metrics 0.13.2 and direct
Host 0.26.0. They are preserved. Build 0.14 exposes Host 0.25 types, so normal
benchmark compilation awaits the aligned release from
[Build PR #57](https://github.com/dragginzgame/ic-build/pull/57). No conversion
shim or normal-checkout source override is installed. Pending 0.52 remains a
minor hard cut, including the exposed Host error family.

On the normal Host 0.26 graph, 26 selected startup cases, seven tick/polling cases
and five live snapshot cases pass, preserving original causes and cleanup
diagnostics after deriving error implementations. Coordinated packaging and its
locked archive build pass; the Testkit archive contains no benchmark programs.
The isolated consumer release guards pass on the settled dependency graph.

An isolated Testkit clone using only the aligned Build source override passes
21 benchmark library cases, strict all-target Clippy, actual Rust 1.91 all-target
compilation and warning-denied documentation. The moved live marker test passes.
Fresh and capacity-one pooled smoke runs also pass with two tasks each, one
worker, 64 state bytes and the prepared PocketIC 16.1 binary. The complete
report is `target/errors-bench-mode-052-evidence/consumer-smoke.json`; it proves
integration correctness, without establishing a performance improvement.
Typed modes preserve valid JSON spelling and independently reject mismatched or
unknown modes. These source-candidate results do not qualify normal registry
adoption. Evidence and before-images are under
`target/errors-bench-mode-052-evidence/`; failed attempts remain there too.
The signal adapter patch is refreshed for the private benchmark path and passes
apply-check; its earlier compilation evidence retains its original source graph.
Full CI and native macOS qualification remain separate.

## Pending 0.52.0: report publication, restore results and diagnostic fields

Report publication uses Host 0.25's existing-parent writer with private replace
options and the same cancellation/phase projection. All 21 driver cases pass,
including retained prior output after producer/publication failures, complete
replacement for existing readers, final-entry symlink preservation, parent alias
resolution and directory-suffix refusal before producing bytes. The first run
failed an old suffix-error assertion: Host returns `InvalidInput` rather than
`NotADirectory`. The current assertion and package migration notes describe that
selected contract; no duplicate pathname check or error adapter remains.

Successful cached-baseline restoration returns unit, without allocating a
copied canister-ID receipt. `PreparedBaseline::Restored { reset }` still carries
the independently checked non-snapshot evidence. Diagnostic output records expose
fields; log bounds, raw-byte/truncation invariants, exact senders, collect-all
ordering and secondary failures retain their existing owners.

Focused Linux qualification passes six diagnostic and three reset/timing units,
13 live pool cases including explicit dedicated-server recovery, one live
diagnostic case, two install-failure cases, five snapshot cases and six controlled
transport/teardown cases (whose three ignored child probes are selected by their
parents). Strict all-target Clippy, actual Rust 1.91 all-target compilation,
three package doctests, warning-denied Rustdoc and formatting pass. Full CI and
native macOS qualification remain separate; no speed-up is measured or claimed.

Caller review finds no additional source changes in Canic's prepared recipes:
they ignore the restored payload and manual restoration already discards its
success value. The previously qualified signal adapter is refreshed for current
imports and passes apply-check; its compilation evidence stays bound to the
original private graph and adoption still awaits Host publication. Current
before-images, the refreshed adapter and driver log are under
`target/report-receipt-diagnostics-052-evidence/`. Incoming manifest/lock selections
and the real index are preserved; no release or sibling checkout was changed.

## Pending 0.52.0: recipe identity removal and Shared Tooling 0.20

Pools own one recipe, so recipe IDs and identity-only validation are removed.
Standalone recipes take funding and the builder; batch installation uses the
ordered single-install operation with `collect::<Result<Vec<_>, _>>()`.
No retained snapshot layout changes. Canic's existing caller PR also removes
root/Fleet/profile identity strings and the standalone payload fixture ID;
its four touched Rust files pass formatting, with native adoption still pending.

Focused Linux checks pass three reset/timing units, two standalone recipe units,
12 live multi-canister pool cases, four live standalone pool cases and the exact
ignored dedicated-server recovery case. Strict all-target Clippy, actual Rust
1.91 all-target checking and three package doctests pass. These results cover
this pending tree; native macOS and full release validation remain separate.

Shared Tooling 0.20.0 is adopted from committed revision
`3f87ddc1a0c1616775b2114c42a4adee7376c9b1`, leaving the sibling's dirty 0.20.1
work untouched. Snapshot integrity (118 files), prepared host/Cargo/PocketIC
checks, dependency pins and consumer formatting hooks pass. The consumer release
fixtures pass after aligning common setup with optional generic IC tools;
failures still stop before product setup. No release runner or real release
adapter was invoked against this checkout. Tool installation remains explicit.
Raw evidence and task before-images are under
`target/identity-shared-052-evidence/`.

## Pending 0.52.0: pool-owned restore and native Cargo selection

Funding is selected once in reset requirements. The pool admits optional recipe
preconditions before performing the complete captured snapshot restore.
Successful restoration returns unit; baseline IDs and selected funding remain
available from their canonical owners. Reset-domain coverage, readiness and invariant checks
retain their ordering. Caller-authored restore constructors and cycle-policy
mirrors are removed. Canonical failures retain the original snapshot cause and
classifiers accept the error trait object; a failed recovery retains both errors.

Three reset/timing units, 12 focused live multi-canister cases, the explicitly
selected dedicated server-death case and four standalone pool cases pass on
Linux. The live cases include pre-restore cycle/time/extra-canister guards,
real canonical snapshot failure plus failed recovery, caller/recipe unwinds,
capacity bounds and 100 consecutive restores. The first dedicated selection was
ignored by libtest; the retained explicit `--ignored --exact` run actually passes.
Strict all-target lint, actual Rust 1.91 all-target compilation, warning-denied
Rustdoc and canonical formatting pass. Package doctests are qualified separately.

The benchmark driver now uses published Build 0.13's exact native executable and
target-directory result. All 21 driver cases pass, including a reported path that
cannot be reconstructed from the profile, prepared tool context, retained Wasm
identity, bounded text failures, cancellation and report publication. Build time
and total streamed diagnostics retain no default ceiling.

[Host PR #108](https://github.com/dragginzgame/ic-host-tooling/pull/108) is refreshed
through a normal merge onto published Host 0.25.1; its signal implementation is
unchanged. Producer signal cases pass on current Rust and actual Rust 1.88, with
strict Process lint and floor Rustdoc. A private Testkit adapter deletes both
local guards, retaining each executable's async-signal-safe callback, signal set,
flags and cleanup policy. It passes 21 driver and 13 CLI cases, strict all-target
lint, actual Rust 1.91 compilation and a real two-task fresh/pooled smoke. Four
explicit Host-family source overrides stay isolated. Normal adoption awaits
Host publication; the apply-checked consumer patch and raw logs remain under
`target/pool-build-signal-052-evidence/`. No full CI, native macOS acceptance or
performance improvement is claimed.

[Canic PR #592](https://github.com/dragginzgame/canic/pull/592) removes both pool
restore forwarders and updates funding/classifier inputs. This is prepared source
against an older pinned Testkit graph, with focused formatting/diff checks only;
publication and the earlier bounded-reply alignment precede normal adoption.
Normal sibling checkouts, Testkit's own versions, manifest, incoming lock, index
and refs are preserved.

## Pending 0.52.0: explicit startup variants and Host publication

Startup uses field-bearing `Spawn` and `Connect` variants; TTL/output options
exist only on `Spawn`. All repository callers, the CLI, examples and current
usage guides use the new contract. The caller trace also identifies startup
construction in Canic, Storage, Blob Storage, Wire and Backup; those consumers
must change their variant construction when selecting Testkit 0.52. Their
current pins and normal checkouts were not changed by this batch.

On Linux, eight asset cases, 26 selected startup cases, 13 server-runner cases
and four explicit live synchronous/asynchronous instance ownership cases pass.
The asset cases preserve substituted staging and occupied destinations, original
archive/version failures, failed attempts and existing bundles. Host owns
no-replace directory publication and phase-aware errors; reuse authenticates the
installed bundle before confirming parent durability. Live cases first failed
inside the sandbox because PocketIC could not bind localhost; the same four pass
with host networking permitted. Strict all-target lint, actual Rust 1.91 native
all-target compilation, warning-denied Rustdoc, package doctests and canonical
formatting pass. Full CI and native macOS remain release qualification.

[Metrics PR #92](https://github.com/dragginzgame/ic-metrics/pull/92) supplies
checked allocation-free wide accumulation. The prepared Testkit adapter passes
19 benchmark cases, a transactional multi-counter case, both example report
cases, strict affected lint and actual Rust 1.91 Wasm compilation through an
isolated source override. Normal adoption awaits publication; current benchmark
arithmetic remains in place. No override, package-version change or dependency
resolution is applied to the normal checkout. The reviewed patch and raw consumer
logs remain in `target/three-cuts-052-evidence/`. The incoming Host 0.25.1 lock
selection is preserved byte-for-byte. This is functional qualification, with no
measured speed, IC instruction, cycle or Wasm size claim.

## Pending 0.51.0: explicit callers, fixture records and example reporting

Installation/retry/snapshot-target inputs are field records; retry counts retain
`NonZeroUsize`. Typed Candid calls require their exact principal under one query
and update name. Existing decoding limits, rejection/transport handling, snapshot
senders, funding and rollback remain unchanged. Native CSV/Markdown directory
output and its bundling record now belong to `examples/benchmark_report/`; the
library retains parsing, span pairing and measurement arithmetic. The example
preserves the nine file layouts and scale-one comparisons. Its two output tests
move from the library integration target; this is relocation, not deleted coverage.
Existing reports are historical evidence and are not converted by this change.

Linux focused qualification passes three retry and six call-codec units, 19
marker/arithmetic cases, two relocated report cases, live typed-call/install/
snapshot cases and both selected managed-server ownership cases. The report CLI
writes its expected current/previous outputs, refuses missing arguments and
preserves existing output on missing input. Strict all-target Clippy, actual
Rust 1.91 native/Wasm compilation, warning-denied Rustdoc, package doctests and
prepared formatting pass. Raw logs and the task's incoming dirty before-images
are retained in `target/explicit-inputs-051-evidence/`.

Caller preparation is delivered through draft PRs in
[Canic #592](https://github.com/dragginzgame/canic/pull/592),
[Jobs #65](https://github.com/dragginzgame/ic-jobs/pull/65),
[Auth #67](https://github.com/dragginzgame/ic-auth/pull/67),
[Blob #79](https://github.com/dragginzgame/ic-blob-storage/pull/79),
[Storage #5](https://github.com/dragginzgame/ic-storage/pull/5),
[Query #74](https://github.com/dragginzgame/ic-query/pull/74) and
[IcyDB #375](https://github.com/dragginzgame/icydb/pull/375).
Jobs/Auth/Blob/Query and Timers pass affected native compilation on Rust 1.91;
Storage's benchmark declares Rust 1.99 and passes there. All six pass strict
focused lint through temporary source overrides, excluded from contributions.
Normal dependency adoption awaits Testkit publication. Canic and IcyDB also need
their earlier bounded-reply alignment; only formatting/patch checks are claimed
for those drafts. IcyDB's source-override compilation stops at five missing decode
limit arguments in its integration helper. Timers' reviewed patch is delivered in
[#66 — Prepare PocketIC probes for Testkit 0.51](https://github.com/dragginzgame/ic-timers/issues/66)
without a commit, respecting its explicit maintainer-owned contribution exception.
Normal sibling checkouts, root index/refs, own versions and normal dependency
selections are unchanged. Full CI and native macOS qualification remain release
checks; these source rehearsals do not establish downstream installed acceptance.

## Pending 0.51.0: diagnostic inputs and shared release metadata

Diagnostic inputs are public-field records; output records retain their existing
accessors and invariants. Six diagnostic units and the live diagnostic, two
installation and six teardown parent cases pass on Linux with PocketIC 16.1.
Exact independent senders, batch label admission, bounded text and subordinate
failure handling remain. Strict affected Clippy, actual Rust 1.91 compilation,
warning-denied Rustdoc and package doctests pass. Caller preparation is delivered
in [Canic PR #592](https://github.com/dragginzgame/canic/pull/592) and
[Jobs PR #65](https://github.com/dragginzgame/ic-jobs/pull/65). Both remain drafts
until Testkit 0.51 publication and coherent normal dependency adoption. Jobs'
recovery target additionally passes actual Rust 1.91 compilation and strict
Clippy through a temporary source override; Canic compilation is not claimed.

The snapshot adds canonical Cargo metadata preparation/qualification from the
already reviewed Shared Tooling 0.15.0 revision, without a baseline upgrade or
local payload edits. Its preparation runs inside Testkit's retained private
workspace. Testkit retains validation receipts, exact old/new recovery, extra
note/installation views and committed-tree checks. The shared helper fixtures,
consumer metadata/recovery fixtures, snapshot verification and shell lint pass
under prepared Linux Bash 5. Bash 3.2 and native macOS remain release checks.

[Build PR #55](https://github.com/dragginzgame/ic-build/pull/55) supplies required
spawn/cancellation callbacks for native Cargo builds. Its focused native cases,
actual Cargo profile/artifact/error case, strict lint, Rust 1.91 and floor
Rustdoc pass. A private Testkit replacement uses Cargo's reported executable and
target root with owned-group cleanup and active cancellation. All 21 driver
cases, strict driver lint, Rust 1.91 and a real two-task fresh/pooled smoke pass.
No default build deadline or total diagnostic limit is added. Normal Testkit
keeps its published graph and current driver until Build publication; retirement
remains [#94](https://github.com/dragginzgame/ic-testkit/issues/94). The apply-checked
consumer patch and raw evidence are in `target/next-three-051-evidence/`; package
versions, the real index/refs and normal sibling checkouts are unchanged.
No full local CI, registry adoption of the new Build API or native macOS
qualification is claimed.

## Pending 0.51.0: caller-owned benchmark provenance

The public benchmark report now contains measurements only. Run identities,
Git/tool versions, command recording, previous-run selection and metadata sidecars
belong to callers; the library metadata record, reader and writer are removed.
A checked-out Rust caller trace finds only their owning Testkit tests. Existing
metadata sidecars remain historical evidence; use fresh directories and an
application-owned provenance record before adopting the current contract. The
fixture-reuse worker and driver keep their separate example-owned JSON schema.

All 21 current benchmark cases pass. The two removed cases exclusively qualified
the retired metadata codec. Strict affected Clippy passes. Native JSON and Serde
derive dependencies are now dev-only; native Candid decoding retains Serde, while
Testkit no longer directly selects Serde for Wasm. Transitive dependencies still
select their own features; this does not claim a measured build-time improvement.

Snapshot restoration selects its funding policy and computes its top-up once.
The focused live snapshot target passes five cases with prepared PocketIC 16.1,
including preserve, needed funding and an already satisfied minimum. Original
rejections, exact senders, rollback and transport classification are retained.
Actual Rust 1.91 native library/all-target checks, Wasm library/probe checks,
warning-denied Rustdoc, formatting and dependency pins pass. The selected lock
and package release versions are unchanged. Task before-images and logs live in
`target/more-cuts-051-evidence/`; earlier
0.51 qualification below remains evidence for its named source, not a claim that
retired metadata APIs remain supported. Full CI and native macOS remain release
checks.

## Pending 0.51.0: pool callbacks and benchmark counter hard cut

Pool readiness and validation hooks return `Result<(), E>` after their existing
checks. Canister/reset receipts still validate snapshot-set and reset coverage;
stage timings, recipe errors and unwind invalidation remain. On Linux with
prepared PocketIC 16.1, all 12 selected live pool cases and six pool unit cases
pass. The dedicated kill-server case retains its explicit isolated selection.

Benchmark producers, parsers and reports now carry three measured counters.
All 23 marker/report cases and the real Wasm perf-probe case pass, including
current CSV shapes, wide arithmetic and overflow/underflow refusal. Before
adopting the cut, retire prior marker logs/reports from active comparison inputs
and preserve them as historical evidence. Rebuild producers and consumers
together, generate fresh baselines and choose fresh report directories. Testkit
selects no retained installation or old-layout reader; the fixture-reuse worker
JSON is unchanged. The benchmark metadata ownership cut above supersedes the
metadata fields qualified in this earlier slice.

Canic's two directly affected recipes are prepared in
[PR #592 — Unit-returning pool callbacks](https://github.com/dragginzgame/canic/pull/592).
It remains a draft pending the existing Canic 0.50 caller alignment and Testkit
0.51 publication; only formatting and patch checks are claimed there. Strict
focused Testkit Clippy, actual Rust 1.91 native/Wasm compilation, warning-denied
Rustdoc and prepared formatting pass. A live two-task fresh/pooled driver smoke
also completes with its existing invariant checks. Raw logs, smoke JSON and task
before-images are retained in `target/hard-cut-051-evidence/`; full CI and native
macOS remain release checks.

Structured PocketIC transport errors remain an upstream prerequisite tracked by
[#118 — Replace transport panic-text recognition](https://github.com/dragginzgame/ic-testkit/issues/118).
The proposal is prepared and verified against current upstream source; posting
outside DragginzGame awaits destination approval. No typed upstream API or
transport-parser retirement is claimed.

## Pending 0.51.0: shared server admission and compact codec diagnostics

[PR #109 — Library server asset admission](https://github.com/dragginzgame/ic-testkit/pull/109)
at `0b165891acb3510b752a428859b7c8d20f4d0f35` and
[PR #110 — Compact Candid diagnostics](https://github.com/dragginzgame/ic-testkit/pull/110)
at `830898538a5cbcf8f5e341316d68bc33fa9af647` are ported into the existing draft.
The authenticated asset implementation now lives at Unix `pic::server_assets`;
the CLI delegates setup/check and supplies cancellation while retaining signal
ownership. It keeps the same pins, bounds, physical-path/byte admission, offline
checks, evidence retention and publication rules. Seven asset cases re-exec
before executable writes, removing the ineffective per-module mutex without
adding production retries. The parallel native library target passes all 87
non-ignored cases together; four explicit real-server cases remain ignored.
Fixture isolation belongs to
[#115 — Isolate server-asset fixtures from parallel process tests](https://github.com/dragginzgame/ic-testkit/issues/115).

The codec selects Candid's compact diagnostics through its shared configuration.
All six codec cases pass, including large-input failures through ordinary,
exact-single-result and argument-list decoding. Original causes and inputs are
retained; compact output is not redaction. On the incoming registry graph (Build
0.12.0, Host 0.24.0, Metrics 0.12.0), Linux also passes the CLI unit case, all 13
non-ignored server-runner cases, six teardown parents, the live rejection case
and all 21 driver cases. The explicitly selected provisioned-CLI case completes
a real PocketIC 16.1 instance without consumer server selection or new setup.

Strict all-target Clippy, actual Rust 1.91 all-target native checks, Wasm codec
checks, MSRV execution of all six codec and seven asset cases, warning-denied
Rustdoc and prepared formatting pass. Before-images and raw failed/passing logs
are retained in `target/pr109-110-evidence/`; the first extra live-call invocation
lacked a server selection and is distinct from the passing admitted-path run.
Source PR heads retain their own hosted evidence; one Intel job per PR was still
running at inspection. The combined draft requires fresh native macOS CI after
the maintainer's push. No full local CI, package-version change, merge or release
was run; incoming dependency selections and earlier draft work are preserved.

## Pending 0.51.0: published Host/Build alignment and native fixtures

The selected registry graph contains Build 0.12.0 and one Host 0.24.0 family.
The incoming manifest/lock edits are retained without another resolution. This
is a minor Testkit batch because startup probe failures and cleanup accessors
expose Host Process error types; direct users select Host Process 0.24. The
library compiler floor remains Rust 1.91 and own package versions remain 0.50.0.

The driver uses published observed-version admission, eliminating its separate
preliminary Git version invocation while retaining executable/output budgets,
explicit environment, cancellation checks and typed causes. Startup diagnostics
use Host's regular-file prefix read, preserving the 16 KiB allowance, lossy text,
omitted-byte rendering and special-file refusal. Published adoption of the Git
operation belongs to
[#112 — Single-probe Host observed-version admission](https://github.com/dragginzgame/ic-testkit/issues/112).

[PR #113 — Native Git-path and command-cleanup fixtures](https://github.com/dragginzgame/ic-testkit/pull/113)
is ported into the current source. Its Git fixture uses portable control/quote
names on every native host and actual invalid UTF-8 directory entries on Linux.
The output-limit producer stays alive until owned capture terminates and reaps
it, with exact output and signal/cleanup evidence. This addresses the two macOS
Git-fixture failures in the
[0.50.0 release run](https://github.com/dragginzgame/ic-testkit/actions/runs/38930567473).
The original PR's Linux/macOS passes qualify that source, not this updated graph.

On this registry graph, all 21 driver cases, 27 startup cases and seven provisioning
cases pass on Linux; four caller-provided-server startup cases remain explicitly
ignored. Strict all-target Clippy, actual Rust 1.91 all-target compilation,
warning-denied Rustdoc and prepared formatting pass. A live PocketIC 16.1 smoke
completes two tasks each in fresh and capacity-one pooled modes with one worker
and 64 state bytes. This is correctness evidence, not a performance comparison.
Logs, the exact graph, source before-images and the smoke report remain under
`target/post050-evidence/`. Current-source native macOS and the full release gate
remain separately qualified after the maintainer's push.

Published Host 0.24 does not export gzip-to-sink copying. Its source contribution
was superseded into [Host PR #172 — Queued Host contributions](https://github.com/dragginzgame/ic-host-tooling/pull/172),
which remains open. The CLI retains its existing authenticated buffering flow;
normal adoption stays with
[#116 — Adopt published Host gzip streaming in server provisioning](https://github.com/dragginzgame/ic-testkit/issues/116).

## Pending 0.50.0: snapshot causes and borrowed aggregation

Snapshot capture, restore and rollback retain one `PocketIcCallFailure` source
with exact sender/canister context. Rejection/panic classification has one owner;
snapshot errors no longer provide cloning or equality traits. Aggregation borrows
suite names until row/error emission, preserving suite/label order, a distinct
global scope and checked overflow semantics. All 23 benchmark cases, five live
snapshot cases, 13 baseline-pool cases and 10 concurrency cases pass on Linux;
two dedicated server-kill cases remain explicitly ignored. Two snapshot cleanup/
funding cases and ten transport-source cases pass, as does the run-count overflow
case. Strict affected-target Clippy, actual Rust 1.91, warning-denied Rustdoc and
prepared formatting pass. This is focused evidence, not full CI or native macOS.

[Host PR #171 — Stream bounded gzip payloads into caller-owned sinks](https://github.com/dragginzgame/ic-host-tooling/pull/171)
adds the sink operation needed to delete the CLI's decoded executable buffer and
separate hash traversal. Its producer at `5c22e3d` passes all 16 gzip cases on
current Rust and actual Rust 1.88, strict Artifacts Clippy and warning-denied docs.
The prepared consumer passes seven provisioning cases, including authenticated
CRC/truncation/trailing-data refusal, strict CLI/test Clippy and actual Rust 1.91.
Its exact selected graph changes only Host Artifacts' source; versions and all
other dependency edges remain selected. This is source-override qualification;
the live checkout retains its published dependency and buffering flow until
publication. Adoption belongs to
[#116 — Adopt published Host gzip streaming in server provisioning](https://github.com/dragginzgame/ic-testkit/issues/116).
Before-images, the apply-checked consumer patch and raw logs remain under
`target/snapshot-streaming-evidence/`. Host's hosted checks are pending; native
macOS and full release qualification remain separate. Package versions, the
real index and Git refs were not changed by this task.

## Pending 0.50.0: caller-bounded Candid replies and pool classification

All typed PocketIC calls now use the independent Candid codec, including its new
bounded top-level argument decoder. Callers supply `DecodeLimits`; fixtures own
small-reply budgets and documentation passes caller budgets through explicitly.
Decode failures retain their typed cause and cannot become transport recovery
merely because decoder evidence quotes a reqwest-looking string. Pool contract
classification is projected directly at its only call site; recipe classification
and fatal recipe-identity mismatch behavior remain unchanged.

On normal registry Build 0.11.0 and Host 0.23.1, Linux passes five codec cases,
six call-codec cases, ten transport cases, six pool unit cases, 13 live pool cases,
the six-variant live rejection case and all six teardown parent cases. Strict
all-target Clippy for both packages, actual Rust 1.91 native/Wasm compilation,
warning-denied Rustdoc, library doctests and prepared formatting pass. A real
PocketIC 16.1 smoke completes fresh and capacity-one pooled workloads with two
tasks each, one worker and 64 state bytes. This is correctness evidence, not a
performance comparison. Native macOS and full CI/release remain separate.

The Candid signatures require downstream updates when adopting 0.50.0. Traced
callers include Auth's qualification runner, Timers' PocketIC harness, Storage's
benchmark, Jobs' consumer recovery tests, Query's Governance testkit, Blob's
PocketIC fixtures, Canic's qualification helpers/tests and IC Crates' managed
fixtures. Use the [package migration guide](../crates/ic-testkit/CHANGELOG.md#0500)
and select budgets for each consumer's actual reply shapes; no global default is
introduced or pushed into those independent application policies.

[Host PR #167 — Single-probe observed-version admission](https://github.com/dragginzgame/ic-host-tooling/pull/167)
removes the need for the benchmark's preliminary Git version capture. Its
producer passes focused version tests on current Rust and actual Rust 1.88,
provenance tests, strict Process Clippy and warning-denied docs. Testkit's
apply-checked consumer patch and source qualification are retained at
`/tmp/testkit-bounded-candid-git.m7WVPu/`; publication gates normal adoption.
The live checkout retains its existing Git flow and registry requirements.
Before-images and failed/passing logs are retained in that same directory.
Own package metadata, the real index and Git refs remain maintainer-owned.

The prepared Git consumer passes all 21 driver cases, strict driver Clippy,
actual Rust 1.91 and the same live two-task fresh/pooled smoke. Only four Host
package sources change in that qualification; all selected package versions and
external dependency edges are retained. Normal adoption is tracked in
[#112](https://github.com/dragginzgame/ic-testkit/issues/112).
Host PR #167's exact-source required CI also passes Linux, Rust 1.88 and native
macOS 15 on both architectures; that does not qualify Testkit's native matrix.

Packaging and publication dry runs verify the exact coordinated archives with
the selected graph and a temporary archived-sibling dependency override. Neither
archive gains a path dependency. This avoids Cargo reusing the older published
Candid source under the current workspace version during pre-bump validation.
The release adapter supplies saved workspace identities to Shared's existing
renderer so the internal version requirement follows minor/major releases too.
Real offline package checks and the registry-backed publication dry run pass;
no upload occurs. Isolated release metadata/recovery and Make jobserver checks,
ShellCheck, Bash 3.2 package/metadata qualification and the four updated README
call examples pass. The full release gate remains maintainer-run.

## Pending 0.50.0: HTTP ownership and direct metadata decoding

Testkit's duplicate HTTP request reader and export are removed. The startup and
simulated-time peers call published Host's opt-in HTTP module directly, retaining
optional Content-Length, deadlines and byte bounds. Query's directly affected
callers are prepared in [PR #72](https://github.com/dragginzgame/ic-query/pull/72),
and Blob's in [PR #78](https://github.com/dragginzgame/ic-blob-storage/pull/78).
Their sibling working checkouts remain unchanged. Query's 19 focused cases and
Blob's three CLI cases pass; Blob's affected standalone/storage targets compile.
Both caller PRs pass strict affected-package Clippy and formatting on their
selected registry Host 0.22.1 graphs; those selections are not upgraded here.
Canic's two additional callers are in local committed `47d3521`, ahead of GitHub
main. Their focused patch passes formatting and apply checks; owning-target
compilation is blocked by the same 13 Host 0.23/Build-alignment errors on its
unchanged baseline. That caller patch must accompany Canic's pending batch.
Query's Intel macOS CI failed during a common-tool download connection reset,
before compilation; GitHub refused a retry while that workflow was still running.

On Testkit's incoming registry Host 0.23.1 graph, 23 benchmark, 27 startup,
seven simulated-time, eight CLI, 13 server-runner and six teardown parent cases
pass on Linux. Metadata remains object-only and now rejects duplicate declared
fields without allocating a generic JSON tree. CLI setup selects the prior
KillAndWait cleanup explicitly, authenticates the observed archive digest and
retains a held descriptor lock. Focused strict Clippy, actual Rust 1.91 checks
and formatting pass. This does not qualify the normal benchmark driver: Build
0.10 still exposes Host 0.22 types, requiring its pending aligned release.

[Host PR #108 — Scoped Unix signal actions](https://github.com/dragginzgame/ic-host-tooling/pull/108)
is refreshed onto published Host 0.23.1; its producer passes current Rust and
Rust 1.88 signal cases, strict Process Clippy, warning-denied docs and formatting.
The prepared Testkit signal patch removes both local guards. It passes 18 driver,
eight CLI and the real CLI interruption case, strict all-target Clippy, actual
Rust 1.91 and a live PocketIC 16.1 two-task fresh/pooled smoke. That qualification
uses the isolated Host PR and a frozen copy of Build's pending dirty 0.11 source
over `95567eb`; only the five selected Build/Host package sources change. It is
source-override evidence, not normal registry adoption or a performance comparison.
Publication gates normal adoption under
[#93 — Adopt scoped Host signal actions](https://github.com/dragginzgame/ic-testkit/issues/93).
The live checkout retains its guards until that API is published. Before-images,
the apply-checked signal patch, exact graph admission and raw failed/passing logs
remain at `/tmp/testkit-http-signals-metadata.IiS7s0/`. Own package metadata,
incoming manifest/lock selections, index and refs remain unchanged by this work.
Native macOS and full CI/release qualification remain maintainer checks.

## Pending 0.49.0: shared hook qualification and benchmark rows

The teardown integration fixture reads bounded requests through Host's HTTP
capture API with optional Content-Length, retaining its timeout and byte bounds.
All six parent cases pass on Linux, including subprocess teardown ordering and
refused/reset transport handling; the three child probes run through those parents.
Focused strict Clippy, actual Rust 1.91 and formatting pass. Evidence is retained
at `/tmp/testkit-teardown-http.IyKSMc/`; native macOS and full CI remain separate.

The normal registry graph now selects Build 0.10.0 and one Host 0.22.1 family,
resolving the driver's output-limit type mismatch at workspace discovery and
metadata observation. The fake Cargo fixture emits a compiler-artifact message
using the probe's actual metadata; Build now discovers Wasm through those messages.
All 18 driver cases pass, including retained-byte and failed-producer behavior.
Strict Clippy and actual Rust 1.91 checks pass for the driver and worker examples,
as do formatting and a live PocketIC 16.1 fresh/pooled smoke with two tasks per
mode, one worker and capacity one. This is Linux correctness evidence, not a
performance comparison or full release qualification. Source before-images and
logs are retained at `/tmp/testkit-build010-boundary.othovz/`; earlier candidate
receipts below remain bound to their source overrides. The Build publication
blocker described there is resolved; Host PR #99's command API remains separate.

Testkit's hook adapter supplies its actual workspace manifest and Rust sources
plus reviewed Make inputs to Shared Tooling's canonical formatting checker.
The snapshot adds that checker and its Rust-source selector from the already
reviewed 0.15.0 revision; no shared payload is edited locally. Missing/wrong
formatter and checkout-local discovery fixtures remain at their canonical owner
in Shared Tooling's `test-git-hooks.sh`. Consumer qualification covers real
Rust/manifest formatting, partial staging, unrelated files and lock preservation,
formatter rejection, repeatability and hook installation/refusal.

Linux qualification passes the consumer adapter under Bash 5 and actual Bash
3.2.57 with inherited CDPATH, snapshot verification, formatting and ShellCheck.
The benchmark API returns row vectors directly, preserving checked arithmetic,
comparison ordering and emitted file layouts. All 23 benchmark integration cases,
the aggregate run-count overflow case and environment URL-selection case pass.
Focused strict Clippy, actual Rust 1.91 native/Wasm checks and warning-denied
Rustdoc pass on the selected normal registry graph. Shared Tooling's pinned hook
fixture suite also passes. Evidence is retained at `/tmp/testkit-049-simplifications-j8neo_lp/`; native macOS and full
release validation remain maintainer-run qualifications.

The next cleanup delegates driver preparation commands to Host 0.21, retaining
inherited Cargo output and no elapsed-time deadline. Short version/platform
stdout capture is bounded to 64 KiB. The built driver exits 130 after SIGINT
and its owned preparation child is reaped; worker interruption still waits for
workload cleanup. Candid transport errors retain the typed operation source,
including through nested IO errors, while unrelated panic payloads resume intact.
Invalid spans retain their two original events; exact CSV evidence preserves the
counter-decrease explanation without an owned per-span reason string.

Five Candid cases, ten transport cases, 23 benchmark cases, 21 driver cases and
two actual-client refused/reset transport cases pass. Focused strict Clippy,
actual Rust 1.91 native/Wasm checks, warning-denied Rustdoc and formatting pass.
The incoming Build 0.9 selection is preserved, with its two benchmark constructor
callers adjusted and checked. This batch's before-images and logs are retained
at `/tmp/testkit-049-next-three-tqi_jl0q/`. It supplies Linux evidence; native
macOS and the full release gate remain separate maintainer qualifications.

Pool outcomes now carry slot and timings once, with Built/Restored/Rebuilt
represented by `BaselinePoolDisposition`. Callers read the common fields directly.
Seven slot-coordinator and six reset/receipt unit cases pass, along with live
PocketIC checks for multi-canister build/restore/rebuild, failed recovery and
standalone slot reuse. Strict affected-target Clippy, actual Rust 1.91 native
checks and warning-denied Rustdoc pass.

Four release/evidence fixtures now source the already reviewed lifecycle helper.
Their actual release-metadata, publication and evidence rejection/preservation
matrices pass under Bash 5 and genuine Bash 3.2.57 with inherited CDPATH;
ShellCheck follows the shared helper. No shared snapshot payload changes here.

[Host PR #155 — Bounded regular-file prefix reads](https://github.com/dragginzgame/ic-host-tooling/pull/155)
contains the producer helper, qualified by five read cases on current Rust and
Rust 1.88, strict Clippy, Rustdoc and formatting. Testkit's prepared consumer patch
passes its sparse-output and FIFO tests plus strict Clippy against the isolated
producer source. Normal Testkit adoption still awaits Host publication; the live
checkout retains its published-dependency implementation until then.
[IcyDB draft PR #367 — SQL pool outcome callers](https://github.com/dragginzgame/icydb/pull/367)
updates the directly affected assertions and compiles against the prepared Testkit
source; merging requires published Testkit 0.49 adoption and lock selection.
Source before-images, the consumer patch and logs remain under
`/tmp/testkit-049-pool-fixtures-prefix-m9_a7q4m/`. These are focused Linux and
source-override receipts, not native macOS, normal dependency adoption or a full
release receipt. The normal sibling checkouts remain unchanged.

Diagnostics and snapshot capture, cleanup and restore now share management-call
rejection/panic translation. Diagnostics expose `PocketIcCallFailure`; snapshot
outer errors retain their context and original operation causes. Focused Linux
qualification passes six diagnostics, two snapshot, ten transport and five Candid
unit cases, plus six live diagnostics/snapshot cases and two actual-client
refused/reset cases. All 23 benchmark parser cases pass after removing the
temporary formatted-string copy. Affected-target strict Clippy, actual Rust 1.91
native checks, warning-denied Rustdoc and formatting pass.

[Host PR #133 — Share shell-style child termination projection](https://github.com/dragginzgame/ic-host-tooling/pull/133)
is refreshed onto Host main `833d278e25649affd00f797647aa0fc0570961e3`, with
topic head `ee36356f2d1d1f4c7c186538261e11c3973d3141`. Its producer passes two
status cases on current Rust and Rust 1.88, strict Clippy, Rustdoc and formatting.
The isolated Testkit consumer patch passes the CLI status/cleanup case, strict
server Clippy and Rust 1.91, retaining fallback 129 and range-conversion fallback
1. Only source paths replace the four selected Host packages in that qualification
copy; normal manifests and lock selections remain unchanged. Publication and
normal registry adoption remain pending under
[#102 — Adopt Host shell-status projection in the server CLI](https://github.com/dragginzgame/ic-testkit/issues/102).
Before-images, the adoption patch and logs are retained under
`/tmp/testkit-049-management-parser-status-7bri_hg8/`; failed intermediate Clippy
attempts are separate from final passing logs. Native macOS and full CI remain
maintainer qualifications. Own package metadata and retained layouts are unchanged.

Startup construction now shares operation panic capture and retains the typed
`PocketIcOperationError` in `BuilderPanicked::source`. On the normal registry
graph, 27 focused startup cases pass, including async timeout/cancellation and
owned-server failure cleanup. Affected strict Clippy, actual Rust 1.91 native
checks, warning-denied Rustdoc and formatting pass.

[Host PR #99 — Share controlled group capture with Testkit and Build](https://github.com/dragginzgame/ic-host-tooling/pull/99)
is refreshed to `70b56d9` on published Host 0.22.1 `b99beea`. It preserves
configured IO through `run_group_command`; capture pipes, cleanup, cancellation
and optional deadlines remain explicit. Seventy focused tool/engine cases pass;
eight group cases also pass on Rust 1.88, with strict Clippy, Rustdoc and formatting.
The earlier `f4ea565` evidence remains bound to its own 0.21 source graph.

Host 0.22.1 published during qualification and includes the HTTP producer. Its
Process archive checksum matches the registry and its VCS source is
`b99beeae7c3115ee5323ea3b48fccad2fe325596`. Published Build 0.9 still exposes Host
0.21 types, so normal Testkit adoption needs the aligned Build family rather than
consumer-side conversions.
[Build PR #48 — Align exposed Host types with published 0.22.1](https://github.com/dragginzgame/ic-build/pull/48)
prepares that 0.10 hard cut without changing own versions. All 168 artifact and
six Cargo-observation cases pass against registry Host 0.22.1, with strict Clippy,
actual Rust 1.91, Rustdoc and formatting. Byte publication retains Replace and
umask-filtered 0o666 choices; no retained-cache reset is introduced.

The isolated HTTP deletion passes seven provisioning, 27 startup, seven time
and two actual-client transport cases against published Host 0.22.1 and the
Build compatibility source, with affected Clippy and Rust 1.91. The combined
command candidate also passes seven provisioning and 18 affected driver cases,
four live startup cases and a prepared PocketIC fresh/pooled smoke with two tasks
per mode, plus affected Clippy and Rust 1.91. The Build probe deletion is qualified
separately. These are source-override consumer receipts, not normal registry
adoption or performance comparisons. Build publication blocks normal HTTP
retirement; PR #99 publication also blocks command retirement. Normal manifests
and implementations remain intact until supplying releases can be adopted.

Evidence, before-images and consumer patches are retained under
`/tmp/testkit-049-http-command-startup-hmcq9x34/`. Query and Blob HTTP caller
patches pass formatting and apply-check only; compiler/native acceptance and
supplying dependency minima remain pending. No normal sibling is edited. Failed
attempts remain separate. Native macOS and full CI remain maintainer checks.

## Pending 0.47.0: benchmark report ownership

The driver retains one complete `WorkerReport` for validation, summaries and
serialization. Startup, cold-preparation and teardown measurements move into the
shared schema with the task/phase fields; the worker-only outer report and the
driver's duplicate raw JSON tree are removed. Unknown root/sample fields now
refuse, as unknown phase fields already did. Driver RSS projections remain
outside the worker schema. Build worker and driver from the same checkout;
existing retained report files are left untouched and are not read through a
second schema.

The public benchmark-history directory record, name/index allocation and
previous-run discovery are removed. A checked-out Rust caller trace finds only
their owning Testkit tests, which retire with those APIs. Report rendering and
metadata IO remain available; applications own their filesystem/history policy.
No retained directory is reset or reinterpreted by this source removal.

The normal registry graph passes 23 benchmark cases, 18 driver cases and strict
affected library/test/example Clippy. Actual Rust 1.91 library/example checks,
warning-denied documentation, formatting and a live two-task fresh/pooled smoke
pass on prepared PocketIC 16.1. The final report borrows the collected typed
runs when encoding output rather than constructing another JSON copy.

### Published Metrics arithmetic adoption

The normal registry graph selects Metrics 0.11.1, and the workspace requirement
now starts at that first supplying version. The maintainer's incoming lock
selection is preserved. The official registry checksum
`a26a294c2207605f600f5859a12752da5097f37c4eb592d24cf350e884c55c2d`
matches the cached crate archive; its VCS source is
`98e5c2f6788d142b6d0fe3fb111f8bb8d1936632`. Its sample-statistics module matches
the earlier prepared producer byte-for-byte. Publication supplies the API even
though [Metrics PR #86](https://github.com/dragginzgame/ic-metrics/pull/86) was
not used as the merge path.

The driver delegates sample sorting, mean, median and observed nearest ranks to
Metrics, keeping only the count/mean/p50/p95 report projection and explicit 95/100
selection. Original typed numeric causes remain in the driver's error chain;
non-finite derived rates refuse before report publication. Midpoint arithmetic
also preserves equal subnormal medians. Workload selection, sample collection,
units, report fields and process cleanup remain Testkit-owned. The old numerical
implementation is removed without a source override or compatibility branch.

Explicit locked cache admission, 19 driver cases, 23 benchmark cases and strict
affected Clippy pass on the registry graph. Actual Rust 1.91 native/Wasm library
and example checks, warning-denied documentation, formatting and dependency-pin
admission also pass. A live two-task fresh/pooled PocketIC 16.1 smoke preserves
required task/lifecycle/sampling fields and count/mean/p50/p95 summary names.
This local normal-graph receipt is separate from configured Testkit native CI;
the pending Testkit source has not yet been delivered for hosted qualification.
Earlier source-only rehearsal and producer checks remain under
`/tmp/testkit-three-report-cuts-0rjddwfr/`; current normal-graph before-images,
archive receipt and focused logs are under `/tmp/testkit-metrics-registry-jj001_ka/`.
[#103 — Adopt published Metrics sample statistics](https://github.com/dragginzgame/ic-testkit/issues/103)
owns Testkit delivery and its current native acceptance. No Testkit version,
index/ref, incoming lock record or sibling checkout is changed by this adoption.

## Pending 0.47.0: typed Cargo observations and Rust 1.91

The benchmark driver uses Build's canonical metadata observation rather than
launching and interpreting a separate untyped Cargo catalog. It retains selected
Cargo/Rustc commands, locked/offline policy, an absolute target-directory check
and interruption checks around the observation. Metadata output is bounded to
16 MiB stdout and 64 KiB stderr with no elapsed-time deadline; original command
and schema errors remain available through the driver's source chain. Synthetic
catalog fixtures now carry the canonical complete Cargo shape.

The maintainer approved aligning both published packages to Rust 1.91 for the
incoming Metrics 0.11 selection. Workspace metadata, Make, hosted MSRV selection
and current package documentation agree; own release versions remain unchanged.
On the normal registry graph, 18 driver cases and all four Candid cases pass,
with actual Rust 1.91 native/Wasm library and probe checks, strict affected
Clippy, warning-denied documentation, formatting and dependency-pin checks.
The live two-task fresh/pooled smoke also passes on prepared PocketIC 16.1.

### Simulated-time operation

`pic::advance_and_tick` implements only the caller-selected time advance and
exact tick count, including zero duration and zero ticks. The existing polling
loop and install cooldown reuse it; predicate budgets, two cooldown ticks and
retry ordering remain at their existing owners. Zero cooldown now omits the
redundant same-time read/write while retaining both ticks.

Seven focused time cases and three retry cases pass on the normal locked graph.
The actual-client transport peer checks zero/nonzero advancement, exact
0/2/4/24 tick batches, polling and install cooldown ordering. Strict affected
Clippy, actual Rust 1.91 checking, warning-denied Rustdoc and formatting pass.
The initial restricted-socket refusal is retained separately from the passing
localhost-enabled run. Evidence is in `/tmp/testkit-more-hardcut-40jftvpu/`;
this pending source has no hosted or native macOS receipt yet.

### Prepared Host owner retirement

[Host PR #108](https://github.com/dragginzgame/ic-host-tooling/pull/108) is refreshed
onto released 0.19.0 at `c6c55a7b107912b536ab3e0c1a42d4c849b9e140`;
[Host PR #125](https://github.com/dragginzgame/ic-host-tooling/pull/125) is refreshed
at `f7d12b8782ed8a39384ea54b21e05603f8ac6d57`. Published history is preserved and
both proposals select additive 0.19.1 notes. The HTTP proposal also carries the
partial-byte/typed-failure engine from Testkit PR #92 at this generic owner.
The two signal and seven HTTP producer cases, strict Process lint/docs and
actual Rust 1.88 producer checks pass on Linux. Both exact heads now pass their required Linux, Intel macOS, Apple Silicon
macOS and MSRV checks; owner merge/publication remain separate.

The earlier disposable Testkit candidate removes both local signal guards and the public
simulator-independent HTTP module, imports Host directly and preserves caller
callbacks, cancellation, framing policy and evidence custody. Its 66 active
focused driver/startup/polling/CLI/interruption/teardown cases pass (seven dedicated
server cases remain ignored), as do strict affected lint/docs, Rust 1.91 native/
Wasm checks and a live fresh/pooled smoke. Those are explicit temporary-source
rehearsals, not normal Cargo adoption. The real checkout retains its current
implementations until a supplying Host version is published and selected.
The prepared patch now incorporates the shared tick operation: seven current time
cases, three retry cases and strict affected Clippy pass against the combined Host
source. `testkit-host-adoption-current.patch` applies to the current normal tree;
the earlier 66-case receipt remains bound to its original prepared source.

The isolated Backup source update keeps signed-request limits, required length
and evidence-file formatting in its Agent fixture while delegating capture to
Host. Backup forbids local sibling Cargo patches, so its actual package checks
must wait for registry publication; no normal sibling checkout was modified.
The prepared Testkit retirement needs a 0.47.0 hard cut, and Backup's caller-only
fixture change is compatible with its published package API.

Before-images, exact isolated sources, source patches and focused logs are
retained at `/tmp/testkit-three-cuts-e5vusihf/`. Host merge/publication remain
maintainer-owned under the contribution rules. No release execution ran.

## Pending 0.46.0: Host 0.19, Build 0.6 and Shared Tooling 0.15

The incoming Host 0.19 selection is adopted normally from the registry. Host
error/cleanup types exposed by Testkit now belong to that minor line, selecting
breaking 0.46.0 notes without changing Testkit's package metadata. Callers must
also respect Host's stricter private-parent admission and supply physical paths
when intentionally selecting aliases.

The driver supplies configurable macOS RSS command budgets (16 MiB stdout,
64 KiB stderr) with no elapsed-time deadline. Provenance records those budgets;
Linux retains 64 KiB complete-record admission. Original sampling causes and
worker cleanup remain intact. The new real native sampling-failure case proves
that the worker finishes before the sampling error returns. This is the local
adoption for [#101 — Explicit Host RSS limits](https://github.com/dragginzgame/ic-testkit/issues/101);
native macOS qualification and publication remain separate.

A concurrent incoming Build 0.6 update removed the old discovery import while
checks were running. Both repository callers now use Cargo observation with
explicit selected commands and 64 KiB discovery stdout/stderr budgets, preserving
relative tool resolution and unlimited build duration. Build 0.6 unifies the
selected Host graph on 0.19. The contributor-only Build floor remains Rust 1.91;
Testkit's native library still passes actual Rust 1.88 checking.

Shared Tooling records committed main `fec7c9b97f6a35a23daf54abe9a6472a5d1829b0`
(version 0.15.0), with 110 verified payloads and unchanged executable pins.
Unmerged/local sibling changes are excluded. The release-metadata fixture copies
the new release-input validator. Formatter evidence uses the canonical shared
selector; its isolated partial-output fixture carries that required companion.
No tool reinstall is needed for the unchanged catalog selections.

On Linux, 18 driver cases, 27 startup cases (four dedicated-server cases ignored),
eight CLI cases and the live canister marker case pass. A live two-task fresh
and pooled smoke emits complete reports with the selected RSS budgets. Strict
affected Clippy, native Rust 1.88 library checking, warning-denied Rustdoc,
formatting, dependency pins and snapshot verification pass. Release metadata/
guards and the shared runner's substituted effects pass in disposable fixtures;
runner and consumer failure-evidence fixtures also pass under genuine Bash
3.2.57 on Linux. These are focused receipts, not native macOS or full-suite proof.

Evidence and before-images are retained at `/tmp/testkit-host-shared-qozq10yu/`.
Earlier pending fixes are carried into the same 0.46.0 notes. The real index,
package versions and refs are preserved; full release validation remains with
the maintainer. The previously published 0.45.0 CI matrix has now completed
successfully on all configured hosts.

## Pending 0.46.0: polling, Candid replies and package licensing

Adapt [PR #96](https://github.com/dragginzgame/ic-testkit/pull/96),
[PR #89](https://github.com/dragginzgame/ic-testkit/pull/89) and
[PR #99](https://github.com/dragginzgame/ic-testkit/pull/99) to released 0.45.0.
Zero-duration polling sends only ticks; nonzero rounds retain get/set/tick order.
Explicit-caller Candid argument helpers decode multiple top-level results while
existing single-result calls preserve tuple-valued semantics and typed failures.
Both published packages include the workspace's MIT license text.

On Linux, five Candid unit cases, five polling cases (including actual bounded
HTTP requests), the live PocketIC 16.1 rejection case and the focused environment
version-probe case pass. Strict library/test Clippy, warning-denied Rustdoc,
actual Rust 1.88 library checking, formatting and package file-list checks pass.
Test-target checking on Rust 1.88 is unavailable with the selected development
Build 0.5 / cargo-platform graph, which requires Rust 1.91; native test evidence
uses the prepared Rust 1.99 contributor toolchain. Library MSRV remains 1.88.

The Intel macOS job on PR #99 timed out a successful version fixture at 200 ms
before it produced output. Successful fixtures now receive five seconds of
scheduling headroom; the deliberately stalled case retains its 200 ms deadline.
This changes test policy only. Native macOS reruns and full release validation
remain maintainer-owned. Source/receipt evidence is retained under
`/tmp/testkit-0451-jzsqt9wn/`; package versions and locked selections are unchanged.

## Pending 0.45.0: package-specific release guard qualification

The release fixture distinguishes standalone Candid tests from PocketIC tests,
requiring server admission only for the latter. Separate failure injections cover
Candid execution, server admission and PocketIC execution; unexpected substituted
recipe failures print their retained log before the guard exits.

The focused Make release-guard target passes on Linux. The updated script also
passes under genuine Bash 3.2.57 with an inherited server selection; explicit
`/bin/bash` children still use Linux system Bash. An injected Candid failure
confirms the recipe and cause reach the outer log. ShellCheck and syntax checks
pass. This does not establish native macOS or a complete release run.
Evidence and before-images are retained under `/tmp/testkit-release-guard.2_zlvsdq/`.

## Pending 0.45.0: portable benchmarks and fallible fixture construction

Native benchmark directory discovery, metadata and CSV/Markdown output now live
under `benchmark::report`. Marker parsing, counters and aggregation remain
portable; the Wasm normal-dependency graph no longer selects Testkit's JSON edge.
Report bytes and directory-selection behavior are unchanged. Candid calls retain
only fallible entry points; fixture callers choose their assertion boundary.
Standalone recipes return `io::Result` and propagate setup failures before capture.

On Linux, 28 report/parser tests, two standalone-recipe tests, three Candid unit
cases, one live rejection case and ten live pool/concurrency cases pass. The
separate ignored dedicated-server recovery case remains unrun. Native and Wasm
strict Clippy, warning-denied Rustdoc, actual Rust 1.88 library checks, formatting
and dependency declarations pass. The new builder regression retains the typed
source and build timings across repeated failed acquisitions without starting a
server. Package versions are unchanged. A concurrent maintainer lock update selects
Host 0.18.6 and Metrics 0.10.1; the final focused root checks use that selection.

[Canic PR #572](https://github.com/dragginzgame/canic/pull/572) and
[Jobs PR #51](https://github.com/dragginzgame/ic-jobs/pull/51) include explicit
fallible Candid callers. A frozen prior pending Canic graph qualifies its internal
library, governed test code, test library and five affected integration targets;
all pass strict package-only Clippy with the exact pending Testkit source.
Canic's additional recipe adaptation is qualified and included in that PR's
handoff for composition after its pending pool migration. Jobs' recovery target
and Timers' native harness also pass strict lint against the pending source.
Timers' maintainer-owned commit exception leaves its update as a prepared patch.
[Auth PR #63](https://github.com/dragginzgame/ic-auth/pull/63) passes strict lint
against its selected registry Testkit 0.37;
full future-version adoption needs its separate pending startup migration.
With explicit maintainer authorisation,
[Toko Miner PR #52](https://github.com/dragginzgame/toko-miner/pull/52) passes
strict native User Hub library/test Clippy on its unchanged registry graph
(Testkit 0.33.8 through Canic). These Auth/Toko Miner receipts establish caller
preparation, not adoption of the complete pending Testkit contract.

Before-images, exact source copies, graphs, logs and caller patches are retained
under `/tmp/testkit-45-more.xkly4l2g/`. Normal sibling checkouts are untouched;
source preparation does not claim current-main consumer adoption, live consumer
journeys, native macOS or full release qualification.

## Pending 0.45.0: Wasm emission and fallible installation boundaries

Performance emission and the direct IC CDK dependency are Wasm-only. Native
marker formatting stays available with caller-supplied counters. Installation
retains the fallible APIs; fixture callers explicitly expect success at their
existing boundary. Retry policies take a non-zero attempt count directly, while
first-attempt counting, cooldowns and rejection classification stay unchanged.
The private probe canister and its dependencies are also Wasm-gated, so native
member checks do not require canister-only APIs.

On Linux, three retry cases, two installation-failure cases, ten concurrency/pool
cases and the live canister marker case pass with prepared PocketIC 16.1; the
ignored dedicated-server recovery case is unrun. Strict all-target Clippy, native
and Wasm warning-denied docs, actual Rust 1.88 native library and Wasm probe checks
pass. Native normal dependency inspection no longer selects IC CDK through Testkit.
An initial Wasm-doc attempt exposed existing native-only intra-doc links; those
links now use target-independent code references. Package metadata stays 0.44.0;
the same 0.45.0 pending notes include all prior accepted changes.

[Draft Canic PR #572](https://github.com/dragginzgame/canic/pull/572) prepares
installation/retry callers and four remaining receipt-fixture clock reads.
Against an isolated copy of the prior pending Canic graph with the current exact
Testkit source selected temporarily, its internal library and six affected test
targets compile. Package-only strict Clippy passes for the internal library and
five test targets; the ingress target retains an existing `type_complexity`
warning on its fixture pool type. Normal adoption also needs the pending clock,
error-field and snapshot integration. [Draft Jobs PR #51](https://github.com/dragginzgame/ic-jobs/pull/51)
prepares the recovery harness; its actual test target compiles and passes strict
package-only Clippy against the same temporary Testkit source. The old registry
constructor rejection is retained. Neither consumer PR changes normal Cargo
selections or its own package versions; integration awaits Testkit publication.

Source before-images and logs are retained under `/tmp/testkit-45-final.apzeuusi/`
against Testkit `5f9ca5cc96dbda54ac4e0879a3521b7e37232113` plus incoming pending
work, including the selected Host 0.18.5 lock. Those external selections are
preserved. A concurrent IC Build update selects 0.4.0; strict all-target Clippy
and the live canister benchmark are rechecked against that normal graph. Canic
qualification reuses a frozen pending graph, not published-main
or live acceptance. Normal sibling checkouts are untouched. Full release gates,
native macOS and governed consumer journeys remain maintainer-owned.

## Pending 0.45.0: direct Host ownership and shared worker arguments

The normal registry graph uses direct Host imports in the provisioner, benchmark
driver and integration tests. Testkit no longer re-exports the complete artifacts,
filesystem or process crates. The driver and worker share mode parsing and a typed
seven-argument invocation; native path bytes survive encoding/decoding. Driver
report/workload checks remain independent. Pending Candid/package notes move with
this public hard cut to 0.45.0; own package metadata stays 0.44.0.

On Linux, 16 driver tests, three worker protocol tests and strict affected-target
Clippy pass against the unchanged incoming Host 0.18.4/Build 0.3.3 selections.
A live fresh/pooled run with prepared PocketIC 16.1 passes and writes raw samples;
this establishes worker exchange and fixture correctness, not a speed-up claim.

[Host PR #125](https://github.com/dragginzgame/ic-host-tooling/pull/125) at
`1ee3f0d` extracts the seven HTTP reader/framing helpers and four behavior tests
into opt-in `ic_host_process::testing::http`. Portable HTTP and Unix executable
fixtures have separate gates. Four HTTP tests, selected existing fixture tests,
strict Process lint/docs, default-feature checking and actual Rust 1.88 pass.
The first sandbox denied loopback binds; the native retry passes.

The isolated Testkit adoption removes its entire testing module and changes the
startup/teardown callers to Host. Four startup and six teardown cases pass; three
ignored cases in each selection remain unrun. Strict affected-target Clippy and
actual Rust 1.88 library checking pass with temporary source selections of the
exact Host PR, not a published registry release. Normal HTTP deletion waits for
Host publication under [#123 — Own bounded synthetic HTTP request fixtures outside Testkit](https://github.com/dragginzgame/ic-host-tooling/issues/123).
Testkit [PR #92](https://github.com/dragginzgame/ic-testkit/pull/92)'s independent
partial-capture changes should follow the Host owner. No compatibility re-export
or temporary dependency override is installed in the normal working tree.

Before-images, exact graphs, live JSON, logs and the apply-checked HTTP consumer
patch remain at `/tmp/testkit-owner-cuts.ZaUG64/`, against Testkit
`5f9ca5cc96dbda54ac4e0879a3521b7e37232113` plus all incoming Candid/Host-lock work.
Native macOS and full release qualification remain separate.

## Candid addition qualified before the 0.45.0 hard cut

The workspace adds `ic-testkit-candid` from [PR #90](https://github.com/dragginzgame/ic-testkit/pull/90)
at `a421f05593720a68a94a20af5c00795dec9216c0`, adapted to the current release
adapter without the separate native allocation package. It has two direct
dependencies, Candid and Serde. Existing PocketIC helper defaults remain unchanged.

Four focused codec tests cover ordinary/exact argument admission, missing optional
and null wire arguments, trailing/truncated bytes, original errors and caller
wire/work/header/type limits. Strict all-target Clippy, warning-denied Rustdoc,
actual Rust 1.88 native/Wasm all-target checks and offline archive verification
pass on Linux. Make checks select both published libraries; portable CI also
selects the codec tests.

Publication admission/retry, metadata recovery and hook fixtures pass under
system Bash and genuine Bash 3.2 with inherited CDPATH. Explicit `/bin/bash`
children still use Linux system Bash. ShellCheck, actionlint, formatting,
dependency declarations and installation requirements pass. Native macOS,
full release gates and publication remain maintainer-owned.

Before-images and focused logs are retained under `/tmp/testkit-candid.nC3OZV/`
against `5f9ca5cc96dbda54ac4e0879a3521b7e37232113` plus the incoming Host 0.18.4
lock update. Only the new local package record is added to that lock; existing
records and dependency edges are unchanged. Own package versions remain 0.44.0;
The compatible addition was initially noted as 0.44.1; the combined pending
contract now targets 0.45.0.

## Pending 0.44.0: explicit snapshots and structural benchmark scope

Snapshot capture takes exact targets; restore takes explicit funding and reuses
captured senders. Rejections retain one sender/response and partial-capture
cleanup remains transactional. Benchmark scope is a public enum field; a probe
through the real report writer produces ten byte-identical files before/after,
including a suite named ALL, comparisons, escaped labels and missing values.

Linux qualification passes 28 benchmark cases, 13 baseline-pool cases, 10
concurrency cases, five snapshot cases, the snapshot funding unit case and the
explicitly selected owned-server/baseline lifetime case. Affected strict Clippy,
library Rust 1.88, warning-denied docs and formatting pass. Examples on Rust 1.88
are refused by the incoming Build 0.3.3 graph's cargo-platform 0.3.3 (Rust 1.91),
tracked in [Build #34 — Published dependency graph exceeds Rust 1.88](https://github.com/dragginzgame/ic-build/issues/34); no
selection was downgraded. Native macOS and full release gates are unrun.

[Draft Canic PR #570](https://github.com/dragginzgame/canic/pull/570) prepares the
caller migration. An isolated copy of its pending 0.43 adoption graph compiles
and passes package-only strict Clippy with this Testkit source and the already
prepared clock/error migrations. A dependency-lint attempt hits the pending
Canic Host input_snapshot length warning (104/100); unrelated source is untouched.
This is not published-main or live Canic acceptance. The PR needs Testkit 0.44
publication and Canic's pending dependency integration.

Source before-images include all incoming edits at Testkit HEAD
`940e0835d00d64f4326ae4ae6b99ac3dd4ea5b92`. Logs, private graph selections, report
outputs and qualified inputs are retained under `/tmp/testkit-44-contracts.U333KL/`.
Own metadata and the incoming normal lock remain unchanged.

## Pending 0.44.0: error access and native operation boundaries

Candid errors and contexts now use their existing public fields without duplicate
getters. Installation, snapshot capture/restore/cleanup and diagnostics share one
internal catch-all panic conversion; Candid calls preserve the original payload
when resuming unrelated panics. Server setup uses the locked registry Host FS
0.18.2 private-directory creator and pathname admission, retaining its descriptor
through synchronization and bundle publication. Bundle layout and retention stay
unchanged; a replaced staging directory is refused before publication.

Canic's two rejection-reader adaptations are delivered in
[PR #569](https://github.com/dragginzgame/canic/pull/569) at
`d8c4e1200030381bc0afd530feb151548293a17e`. Selected locked/offline compilation
and strict Clippy pass against its unchanged Testkit 0.33.1 graph, reusing the
prior isolated native artifact directory; governed Canic live journeys are unrun.

Against clean Testkit `940e0835d00d64f4326ae4ae6b99ac3dd4ea5b92`, Linux
qualification passes 26 selected library cases, 7 provisioning cases and 15
PocketIC integration cases across calls, diagnostics, installation, snapshots and
teardown. Affected strict Clippy, actual Rust 1.88, warning-denied documentation
and formatting pass. The initial sandbox refused PocketIC's localhost bind;
the native retry uses the same prepared 16.1 binary and locked registry graph.
Package metadata remains 0.43.0. Logs and frozen inputs are retained under
`/tmp/testkit-three-cuts.eA9P54/`; native macOS and full release gates are unrun.

## Pending 0.43.0: metadata serialization and prepared owner adoption

Benchmark metadata now serializes the typed record directly. A before/after
probe through the actual report writer preserves decoded fields, escaped values
and the final newline; JSON keys follow record order instead of the intermediate
map's sorted order. All other report files are byte-identical. Three metadata/
report boundary cases and strict selected library/test Clippy pass on the normal
registry graph. Object-only admission and required/optional fields are unchanged.

The fresh isolated consumer integrates both prepared retirements with the current
shared report types and published Host table renderer. It uses Host PR #108
`f9377c547d305e19afa98c66e198e1e15ea99c77` and Build PR #31
`f791954f7dacd32e51e2727ce6199c73cf1aec52` as temporary source selections. The
13 driver cases, CLI descendant cancellation, strict affected lint, actual Rust
1.88 and a fresh/pooled PocketIC smoke pass on Linux. No performance gain is
claimed. Cargo builds retain caller admission and no default elapsed-time or
total-diagnostic-output ceiling. External package records and edges match the
frozen normal lock after restoring incidental Windows edge reselection in the
private qualification graph; locked complete Cargo metadata admits that graph.

Registry inspection still selects Host 0.18.2 and Build 0.3.2 without these APIs.
Normal signal/build callers therefore await producer integration/publication
under [#93](https://github.com/dragginzgame/ic-testkit/issues/93) and
[#94](https://github.com/dragginzgame/ic-testkit/issues/94). The refreshed patch
applies to this pending working tree; no temporary override is installed here.
Before-images, source identities, failed attempts, probes, patch and raw logs
remain under `/tmp/testkit-final-cuts/`. Own metadata remains 0.42.0; native macOS,
full release gates and publication remain maintainer-owned.

## Pending 0.43.0: report ownership and native time calls

The normal working tree shares typed fixture-report fields between worker and
operator driver, retaining raw extra fields and independent workload/duration/
sampling admission. Its terminal table uses published IC Host Text 0.18.2;
the archive checksum is
`fe0d780d03023e8eeaedb5356570f57b32fbdb48c75af58069581929543acc03`
and its VCS receipt names `55a31aa13a42f55fa3985db33e1586722638ad40`.
Only the new text selection and local dependency edge are added to the incoming
lock. Exposed runtime Host selections, existing pins and own 0.42.0 metadata remain.

The redundant time trait is removed; direct PocketIC timestamp conversion replaces
its consumers. All 13 driver cases, four bounded-polling cases, affected strict
Clippy, actual Rust 1.88 and a live fresh/pooled smoke pass on Linux. The smoke
checks worker/driver exchange and fixture correctness, not a performance gain.
An initial sandbox attempt refused PocketIC loopback binding; the native retry
uses the same selected binary and sources. Native macOS and full release gates
remain maintainer-owned. Before-images, exact inputs and raw logs are retained at
`/tmp/testkit-schema-time/` against released Testkit
`5d1cce0a1034c7e7bcb2490544b6f81dda5edc32` plus this pending batch.

[Jobs PR #49](https://github.com/dragginzgame/ic-jobs/pull/49) at
`f0c9ac73cfcbc60ef74f8c89830953274ab74187` passes selected recovery compilation
and strict lint on its unchanged registry Testkit 0.42.0 graph.
[Canic PR #565](https://github.com/dragginzgame/canic/pull/565) at
`fd2257d06dbc05d5fbe834c7fe85934b03f9b1a6` passes selected compilation and strict
lint on published main. Its earlier rehearsal also passes on the unpublished local anchor
`644d4b13012b395d6211a6907e2e6aafdf391db9` and published main
`25b73cd403b20b4e148c6e340f8c760a26d97f6e`; those graphs/evidence remain distinct.
Normal sibling sources are untouched. Consumer delivery and their governed live
journeys remain separate from this compile qualification.

## Pending 0.42.0: canonical release projection and prepared native callers

The working tree adopts Shared Tooling 0.12.7 at
`9364ffe3f770a6d5d94809f0bf82c33202054bf8` through its exporter and recorded
106-file snapshot. Release preparation projects only admitted local lockfile
versions through the canonical rewriter; saved manifests still support interrupted
cache preparation. Snapshot integrity, 19 rewriter assertions with real Cargo,
consumer metadata/recovery, prepared-tool fixtures, formatting and focused shell
lint pass on Linux. Metadata/recovery and host/IC/Rust installer fixtures also
pass under genuine Bash 3.2.57. Catalog pin bytes remain unchanged.

[Host PR #108](https://github.com/dragginzgame/ic-host-tooling/pull/108) at
`ba2e007dc9dc11949c124d21e613614a14617f60` supplies scoped Unix signal actions;
[Build PR #31](https://github.com/dragginzgame/ic-build/pull/31) at
`31658b37b1958886c25342c08ffc54f036cab236` supplies Cargo-reported native executable
and target paths. Their focused native tests, strict lint, docs and actual Rust
1.88 checks pass. The isolated Testkit consumer passes eight driver cases, CLI
cancellation, strict caller lint and Rust 1.88 with temporary source selections.
A fresh/pooled PocketIC smoke passes through that build path, establishing
correctness rather than a performance improvement. Cargo duration and total
diagnostic output remain unlimited by default; decoding bounds apply per message.

Normal Testkit signal/build callers await published APIs under
[#93](https://github.com/dragginzgame/ic-testkit/issues/93) and
[#94](https://github.com/dragginzgame/ic-testkit/issues/94). Exact inputs, failed
attempts, the consumer patch and raw evidence remain in
`/tmp/testkit-next-three/`. Full validation, native macOS acceptance and publication
remain with the maintainer. The combined draft selects 0.42.0 for the concurrent
exposed Host 0.18 filesystem contract change; own package metadata remains 0.41.0.
The earlier prospective 0.41.1 qualification below retains its original identity.

## Pending 0.41.1: phased synthetic HTTP capture

The authorized Blob extraction extends the existing native testing reader with
bounded request-line capture and completion through one caller absolute deadline.
Incomplete connections may stay caller-owned while another request progresses;
response/fault policy and original socket provenance remain local.
[Blob #70](https://github.com/dragginzgame/ic-blob-storage/issues/70) owns consumer
activation after the additive API is published. Existing whole-request capture
and the 0.41.0 startup/teardown callers keep their contracts.

Focused Linux qualification passes 77 library units (four governed cases remain
ignored), four socket/framing cases, strict library lint/docs and Rust 1.88.
The isolated Blob candidate passes complete CLI cases, strict lint, Rust 1.88
and three installed observation/recovery journeys. Published 0.41.0 source is
`5abb2d6cde750386647817741f7ccec403bc7fca`; prospective 0.41.1 metadata/path
selection is confined to the isolated qualification graph. Exact before-images,
inputs, failed sandbox attempts and raw logs remain at
`/tmp/blob-cleanup3.OxzNfJ/`. Native macOS and published phased adoption remain
separate; no full gate, package-version change, Git effect or publication ran
for this extension. The maintainer's concurrent 0.41.0 release is preserved.

## Pending 0.41.0: numerical and synthetic transport convergence

The benchmark API projects checked integer means through published IC Metrics
0.9.4. Exact counter totals, aggregation overflow rejection and platform reads
remain local. Comparison rows retain both means/scales instead of percentage
estimates; write the changed CSV columns into new report directories
([#84](https://github.com/dragginzgame/ic-testkit/issues/84)).

All 28 benchmark cases, one request-framing case, four ordinary async startup
cases and six parent teardown/transport cases pass on Linux. Startup and teardown
now share blocking socket selection, bounded framing and a single read deadline;
the existing teardown barrier supplies socket-mode qualification. Strict
all-target Clippy, docs and actual Rust 1.88 native/Wasm library checks pass.
The incoming CI partition and its distinct real/native selections are retained;
no full local suite or new macOS qualification is claimed.

Host's isolated [capture contribution](https://github.com/dragginzgame/ic-host-tooling/pull/99)
shares its existing execution engine. Its nine group/direct cases, strict
all-target/all-feature process Clippy, docs and Rust 1.88 checks pass. The prepared
Testkit consumer passes six provisioning cases and strict CLI Clippy with a
coherent temporary Host family override. The normal Testkit graph retains
published Host 0.17.13; actual removal of downloader plumbing awaits the API's
publication under [#85](https://github.com/dragginzgame/ic-testkit/issues/85).
Temporary overrides and the concrete consumer correction are confined to
`/tmp/testkit-three-reductions-1791977164924197643/`. Versions, releases and native
acceptance remain with their owners; the normal Host checkout is untouched.

## Pending 0.41.0: CI selection and release-policy qualification

The locally prepared PR #80 policy reference adopts only Shared Tooling 0.12.5's
validation cadence at `4256171b5b7ec4c4cc4078548c1c1617ee150699`; executable
snapshot selections remain unchanged. PR #82's CI partition is reconciled against
released 0.40.0 `2a7f2d48fd666ff42ae9ee9a99e230eaefb8cc31`, retaining the new
real instance lifetime step and using the existing Make gate as the ordinary
package/guard owner ([#81](https://github.com/dragginzgame/ic-testkit/issues/81)).

| Qualification | Native CI owner |
| --- | --- |
| Common guards, formatting and ordinary package tests | `checks`: `make ci` under system Bash, CDPATH and physical TMPDIR |
| Installer, dependency and release-runner fixtures | `portable-hosts`: existing portable commands |
| Benchmark example and hosted archive upload/readback | `portable-hosts`: existing dedicated steps |
| Ignored real sync/async lifetimes, recovery, provisioning and idle evidence | `pocket-ic-concurrency`: existing dedicated steps |

Actionlint, prepared-tool hook qualification, formatting, publication/installation
guards and the complete focused release guard pass on Linux in the consolidated
shell/path context. Actual workflow failure-evidence controls and archive round
trips also pass. The release network-policy fixture initially exposed its caller's
exported offline value; subshell isolation now preserves all four policy cases
and passes the same focused guard with an offline caller
([#83](https://github.com/dragginzgame/ic-testkit/issues/83)). Logs and exact inputs
remain under `/tmp/ic-testkit-pr-work/`. The remote PR #82 head is still conflicted;
the tested correction is applied locally and supplied in its discussion. Native
CI and branch delivery remain outstanding; no full local suite or release ran.

## Published 0.40.0 CLI archive qualification

The cached crates.io archive matches its sparse-index SHA-256
`2627451b0b7bf945b75963dc76218e493ed1ba9d19d6096bb1fec52dea2dfba9` and VCS
source `2a7f2d48fd666ff42ae9ee9a99e230eaefb8cc31`. Its extracted normalized
package installs the CLI locked/offline in debug profile into a private temporary
root, using this workspace's target. That exact installed CLI admits the retained
PocketIC bundle with PATH unavailable and launches a separate real instance worker
through its governed server URL, with no new server setup
([#77](https://github.com/dragginzgame/ic-testkit/issues/77)).

Archive, package-lock and binary identities plus install/admission/launch logs
are retained under `/tmp/ic-testkit-pr-work/published*`. This is Linux package
installation/runtime evidence; both native macOS archive installations remain
unqualified. The released 0.40.0 workflow currently has four Linux passes and
eight queued native jobs; it does not yet establish complete native acceptance.

## Pending 0.40.0: synthetic async peer socket mode

The async startup fixture now configures accepted sockets for blocking reads,
retaining its nonblocking accept loop, bounded read and request/cancellation
barrier. A controlled socket starts nonblocking with no request bytes; before
the fix its reader closes immediately, while the corrected reader remains live
until delivery. The regression observes the actual socket-mode change before
sending bytes, rather than relying on a timing delay
([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

The controlled failure and passing regression, all 28 ordinary startup cases,
strict library/test Clippy, actual Rust 1.88 library/test checking and formatting
pass on Linux. The incoming lock selects Host 0.17.12 and IC Build 0.3.0 and is
unchanged by this repair. Source/lock digests and logs remain under
`/tmp/ic-testkit-068-socket/`. Earlier graph-bound evidence remains unchanged.
The delivered 0.39.0 Intel failure remains unqualified until the repair is
delivered and the supported native checks pass. Full suites and release
execution remain maintainer-owned; this fixture fix changes no production
startup policy, public API or retained installation.

## Pending 0.40.0: retained instance/server ownership

The approved hard cut uses the existing owner-returning constructor and carries
its `PocketIcInstance<PocketIc>` through standalone install, install-error recovery
and cached baselines. Managed drop deletes the synchronous instance before server
teardown; borrowed connections retain no server authority. Host's regular-file
opener preserves descriptor validation while Testkit keeps read bounds, pending
port handling and native I/O causes. Ephemeral test roots use private `TempDir`
ownership; retained provisioning attempts and installations remain unchanged
([#79](https://github.com/dragginzgame/ic-testkit/issues/79)).

Focused Linux checks use released anchor `d10763417b3e3a083c9de7f5b0405be97688b21e`
plus the preserved incoming batch and this unpublished implementation. The incoming
lock selects Host 0.17.10 and IC Build 0.2.7; no resolution or package-version edit
ran. Explicit offline source preparation precedes qualification. Ordinary startup,
real owned/borrowed construction, error/fixture/baseline transfer, failed capture,
synthetic teardown, affected live callers and both dedicated-server recovery cases
pass. Provisioning, CLI runner and benchmark report fixtures pass, including retained
failure evidence. The real canister marker test and a two-task fresh/pooled smoke
pass; this is correctness evidence, not a measured performance improvement.

Actual Rust 1.88 affected-target and Wasm checks, strict Clippy, warning-denied
Rustdoc, doctests, formatting and dependency admission pass. Logs, the private
smoke report and exact source/lock digests remain under `/tmp/ic-testkit-040-cut/`.
[Canic #548](https://github.com/dragginzgame/canic/issues/548) carries the apply-checked
caller migration; [Backup #43](https://github.com/dragginzgame/ic-backup/issues/43)
retains its consumer action. No sibling source or graph was edited. Both native
macOS hosts, full maintainer-owned validation, publication and consumer adoption
remain outstanding. Package metadata stays 0.39.0; pending notes are 0.40.0.

## Published Host 0.17.9 for pending 0.39.1

The normal registry graph selects Host 0.17.9 from published source
`6fd078d9b914fff530b1efe521dc08ce18b97cc0`. Its opt-in Unix test writer replaces
both local implementations; script bodies and domain assertions remain in
Testkit. Only native Unix development enables `test-support`, and no path
override or compatibility layer is added
([#78](https://github.com/dragginzgame/ic-testkit/issues/78),
[Host #87](https://github.com/dragginzgame/ic-host-tooling/issues/87)).

The selected update changes only the three Host package records. All other
lock selections and own metadata at 0.39.0 are preserved. Locked source
preparation precedes offline qualification. Linux passes 27 ordinary startup
cases, three real owned/borrowed startup cases, 13 controlled CLI runner cases,
and both exact real CLI/instance cases. Strict library/test Clippy, actual Rust
1.88 native/Wasm checks, public Rustdoc, formatting and dependency admission pass.
This also requalifies the pending real-instance CI command on the new graph.

Published archive identity, source/input digests and logs are retained under
`/tmp/ic-testkit-host179-adoption/`. Earlier isolated producer overrides and
Host 0.17.8 results remain source-bound below. Full maintainer validation,
delivery and both native macOS consumer executions remain outstanding; no
release, sibling edit, tool installation or retained artifact retirement ran.

## Pending 0.39.1: real instance lifetime coverage

The native concurrency matrix now explicitly runs the three ignored prepared
PocketIC startup tests. Ordinary library tests covered the synthetic startup
failures but skipped real managed construction and owned sync/async/borrowed
instance lifetimes. The new step uses each host's existing admitted server and
the unchanged locked/offline command
([#68](https://github.com/dragginzgame/ic-testkit/issues/68)).

That exact command passes all three cases on Linux at released source
`d10763417b3e3a083c9de7f5b0405be97688b21e` plus the CI/note changes, with the
retained official PocketIC 16.1 binary and unchanged Host 0.17.8 graph. Selected
actionlint 1.7.12, formatting and diff checks pass. Logs are retained under
`/tmp/ic-testkit-0391-native-coverage/`. This is focused Linux evidence; delivery,
full maintainer validation and both native macOS executions remain outstanding.
Own package metadata stays at 0.39.0 and retained installations are unchanged.

## Pending 0.39.0: packaged CLI qualification

Focused offline packaging of the current dirty source retains the published
CLI target and selects all three Host crates at 0.17.8, with no older Host graph.
The normalized package requirements also start at 0.17.8; native `tempfile` is
included. Packaging leaves the workspace lock unchanged. This is a candidate
archive with unchanged own metadata at 0.38.0, not a new registry publication
([#77](https://github.com/dragginzgame/ic-testkit/issues/77)).

The extracted package's CLI builds locked/offline and passes actual Rust 1.88
checking in this workspace's target. That exact CLI admits the retained official
PocketIC 16.1 bundle with `PATH=/nonexistent`, then launches a separate process
that constructs and drops a real application-subnet instance using only its
exported server URL. Neither check downloads or provisions another server.
Archive, manifest, lock, binary/worker identities and logs are retained under
`/tmp/ic-testkit-issues-pass-20261014/`. These are Linux checks; published CLI
installation and matching Intel/Apple Silicon CI remain outstanding. The failed
0.38.0 native receipts remain unchanged.

## Pending 0.39.0: standalone calls and temporary-file ownership

The approved public hard cut calls the existing `CandidCallExt` on the fixture's
retained PocketIC with an explicit canister ID. Installation, typed failures and
`into_parts()` ownership remain. `StartupFiles` and the synthetic server fixture
use private `tempfile::TempDir` ownership instead of local collision/Drop logic.
The 0700 directory, 0600 create-new outputs, port readiness and external-log
retention contracts remain. These directories are ephemeral; no retained format
or installation needs reset ([#78](https://github.com/dragginzgame/ic-testkit/issues/78)).

The normal requirements start at published Host 0.17.8 for its macOS permission
repair; the incoming selected lock already used that coherent Host family and
is unchanged by this selection
([#77](https://github.com/dragginzgame/ic-testkit/issues/77)). Own package metadata
remains at 0.38.0, with the pending notes selected as 0.39.0.

Focused normal-registry Linux qualification passes 27 startup tests, three real
PocketIC startup cases, the actual canister benchmark, strict library/test Clippy,
Rust 1.88 native/Wasm checks and public Rustdoc. An isolated candidate additionally
removes both local executable-script writers and selects
the opt-in Unix test API from frozen Host source. That candidate passes the same
27 cases, three real PocketIC 16.1 startup cases, 13 controlled CLI runner cases,
the real canister benchmark, installation rejection and decomposition ownership,
strict library/test Clippy, actual Rust 1.88 native/Wasm checks and public Rustdoc.
Candidate evidence uses a temporary producer override and does not establish
normal registry adoption. Logs and candidate input digests are retained under
`/tmp/ic-testkit-0390-cut/`.

The shared writer's normal consumer deletion awaits published Host 0.17.9 under
[Host #87](https://github.com/dragginzgame/ic-host-tooling/issues/87). The maintainer
owns its full validation/publication. Both native macOS consumer checks, full
Testkit validation and delivery remain outstanding. Only targeted development
checks ran under the session's explicit validation instruction; historical
evidence below remains bound to its original source and selections.

## Shared 0.12.0 for pending 0.38.1

The canonical export selects pushed Shared `f7fe31bf80a2b445bf37a470527f001210241aea`
from an immutable checkout, excluding dirty sibling documentation. The existing
99-file selection, own package version, selected dependency graph and common
catalog bytes are preserved. This extends the contributor-only 0.38.1 batch
([#73](https://github.com/dragginzgame/ic-testkit/issues/73)).

Release preparation uses Shared's strict finalizer: unnumbered Draft headings
and candidates displaced below history refuse. Numbered top candidates, canonical
previous-version history and existing finalization/recovery behavior remain
covered. New consumer cases exercise both note views and preserve all six metadata
files while withholding the prepared readiness record on refusal
([Shared #162](https://github.com/dragginzgame/shared-tooling/issues/162)).

The actual consumer metadata fixture passes on modern Bash and genuine Bash
3.2.57, alongside the adopted release-runner controls. Sourced ShellCheck,
formatting, dependency declarations, diff checks and snapshot admission pass.
Logs and source comparisons are in `/tmp/ic-testkit-0381-shared012/`. These are
focused development checks under the session's targeted-validation instruction;
no full CI or release operation ran. A concurrent AGENTS overlay change is
preserved and is outside this task's edits.

At review, Shared's exact-source Linux/Apple Silicon and lint jobs pass; Intel
is still running. This does not qualify native consumer CI. Testkit's published
0.38.0 macOS build remains blocked by unpublished Host 0.17.8 under
[Host #84](https://github.com/dragginzgame/ic-host-tooling/issues/84).

## Shared 0.11.4 for pending 0.38.1

The canonical exporter selects pushed Shared `031d214ed044705b989d43e2379f30e6a3d44749`
from an immutable checkout, excluding dirty sibling policy changes. The reviewed
99-file snapshot retains common catalog versions/digests and the selected Cargo
lock. Contributor-only adoption keeps own package metadata at 0.38.0
([#73](https://github.com/dragginzgame/ic-testkit/issues/73)).

Portable CI replaces discovery plus a separate action download with shared
exact-ID, repository/run and uploaded-digest readback. The three qualification
archives travel inside one owned bundle; extraction rejects unexpected members
and links first. Original normal/rejected-selection bytes, modes, statuses and
failure retention are still verified. Release preparation delegates literal
version rendering to Shared while retaining private candidate validation,
source/receipt custody, byte recovery and publication policy
([#59](https://github.com/dragginzgame/ic-testkit/issues/59)).

Local modern Bash and genuine Bash 3.2.57 pass evidence round trips (including
extra-member/link refusal), isolated metadata/recovery checks and substituted
release-runner controls. Shared exact-download and literal-renderer fixtures pass.
Workflow lint, sourced ShellCheck, formatting, dependency declarations, snapshot
integrity and offline prepared-tool admission pass. The initial tool check found
the cleaned local CLI absent; explicit locked offline rebuild restores admission
without resolving dependencies. Logs are `/tmp/ic-testkit-0381-shared/`.

[Released 0.38.0 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38439336526)
passes all four Linux lanes. Both macOS architectures fail before consumer tests
on Host 0.17.4's Darwin permission conversion, tracked by
[Host #84](https://github.com/dragginzgame/ic-host-tooling/issues/84). The prepared
Host 0.17.8 repair is unpublished at inspection; producer/native acceptance and
this new consumer snapshot's hosted readback remain outstanding. No full gate,
real release/staging/ref effects, own version bump or sibling edits ran.

## Shared 0.9.10 for pending 0.35.1

The canonical exporter selects immutable revision
`7e55adecf6f0b196c050f104b37f8c69b6bf434e`: 64 roots and 96 payloads.
The new change-reporting rules are selected explicitly; the Rust-tool fixture
adds the canonical Cargo-tool cache-key helper through its companion closure
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). Uncommitted sibling
changes are excluded. Pending benchmark edits, dependency selections, package
metadata, the real index and refs are preserved.

Checks and portable CI restore only Cargo sources and physical Rust tool
generations under an exact compiler/native/image/pin/policy key. Complete setup
and offline Rust-tool admission still run on cache hits. Only main pushes save
verified generations before validation; PRs only restore. Restore/save have
three-minute timeouts and nonfatal transport errors, while setup/admission
remains fatal. Checkout selections, user credentials and host/IC bundles are
excluded; no broad restore prefixes are used.

Explicit `make install-rust-tools` prepares selections under the changed policy
digest without discarding earlier builds. Actual offline Rust/formatter checks,
snapshot integrity and dependency pins pass. The adopted Rust-tool fixture and
consumer evidence/archive fixture pass on modern Bash and genuine Bash 3.2.57.
These fixtures exercise invalid offline TOML, environment overrides, cache
identity/round trips, receipt refusal, preserved partial staging and forbidden
Git-effect detection. The actual consumer hook qualification, selected
ShellCheck and actionlint pass. Repeated local cache-key reads are stable.

Local logs use `/tmp/ic-testkit-shared0910-*`. This is focused Linux qualification;
hosted cache hits, performance and this consumer source's native macOS acceptance
remain unobserved. No broad gate, release or Git staging/ref operation was run.

## Host 0.15 and Shared 0.9.5 for pending 0.35.0

The approved hard cut retires Testkit's unused IC-tool convenience re-export and
dependency while retaining the three used Host crates at selected 0.15.0
([#67](https://github.com/dragginzgame/ic-testkit/issues/67)). Cargo's explicit
offline workspace update prunes only `ic-host-tools` and its Testkit dependency
edge; all other selected versions, checksums and graph edges are preserved.
`cargo fetch --locked --offline` prepares that graph without resolution. Own
workspace package metadata stays maintainer-owned at 0.34.7.

Focused Linux qualification passes three composed Host/tool/cache tests, ten
cache-filesystem tests, seven CLI unit tests and thirteen controlled CLI runner
tests; four real-server tests remain ignored. Strict library/test Clippy,
Rustdoc warnings and actual Rust 1.88 native library/CLI and both Wasm package
checks pass. These controlled CLI tests use synthetic server/download transport,
not real provisioning or native macOS acceptance.

Canonical preview and Bash 3.2 export select committed Shared 0.9.5
`b1630b89482cc6071fc928a86962d79c9c11fdb7`, excluding newer uncommitted PR work
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[#59](https://github.com/dragginzgame/ic-testkit/issues/59),
[Shared #63](https://github.com/dragginzgame/shared-tooling/issues/63)). The
selection adds only the canonical discovery helper: 63 roots and 94 payloads.
The upstream discovery fixture stays outside the snapshot because it qualifies
Shared's own two-download workflow; the local evidence fixture checks Testkit's
single-download boundary and retains its complete archive/readback controls.

Both fixtures pass on modern Bash and genuine Bash 3.2.57. Common controls cover
pagination, delayed/final visibility, exhaustion, malformed/expired/conflicting
identities and API/authentication refusal with substitute GitHub transport and
sleep. The local workflow requires successful discovery for the same upload ID,
repository and run, retains digest checks and binds its token and two-minute
timeout. Local execution does not establish hosted artifact-service recovery.

Snapshot integrity, stable repeat previews, declaration pins, workflow lint,
selected ShellCheck, prepared formatting, documentation links and diff checks
pass. Logs use `/tmp/ic-testkit-0350-*`; source/input/artifact/runtime/log digests
are recorded in `/tmp/ic-testkit-0350-qualification.json`. Real staged contents,
Git refs, package metadata and published history are preserved. Native acceptance,
full gates and release execution remain maintainer-owned.

## Shared 0.9.4 for pending 0.34.7

Shared 0.9.4 became committed during 0.9.3 qualification and is adopted through
an independently reviewed immutable source,
`4532b66c1b43bab331978523ee97d29fe11a2f50`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
Canonical preview and Bash 3.2 export retain 62 roots and 93 payloads. The
verifier, direct/PR release helpers and direct release fixture change from 0.9.3;
its download/installer inputs are unchanged, retaining the qualification below.

Both-shell private snapshot controls accept full 40- and 64-character identities
and reject lengths 39, 41, 63 and 65 without emitted records or refresh mutation.
Inputs remain at the root recorded by
`/tmp/ic-testkit-shared094-identities-path`. The actual adopted direct release
fixture passes on modern Bash and genuine Bash 3.2.57 using command substitutes
and an inert Git guard. The committed upstream PR fixture passes on both shells
using synthetic histories, private bare destinations and substitute GitHub
transport. Malformed saved source/prepared/merged identities preserve the plans,
emit no release effects and cannot turn into a silent restart. These fixtures
do not touch the real checkout's index, refs or release state.

Actual snapshot verification, stable both-shell repeat previews, selected
ShellCheck, documentation links and diff checks pass. Logs and private control
scripts use `/tmp/ic-testkit-0347-shared094-*`; source/input/runtime/log digests
are bound in `/tmp/ic-testkit-0347-shared094-qualification.json`. Metadata, dirty
selected lock, staged contents, workflow and published history are preserved.
No real release, Cargo compilation, dependency resolution or installation runs.
This is focused Linux qualification; native acceptance and full gates remain
maintainer-owned.

## Shared 0.9.3 for pending 0.34.7

Canonical preview and genuine Bash 3.2 export select Shared Tooling 0.9.3
`3619de9dc36065411b8e6521f8695e9395a0e0ea`, retaining 62 roots and 93 payloads
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #138](https://github.com/dragginzgame/shared-tooling/issues/138)). The
immutable detached source matches GitHub; uncommitted 0.9.4 identity-admission
work stays in the sibling checkout and is excluded. The consumer adds no
upstream-only retry fixture or fleet reporter.

Actual adopted host and IC installer fixtures pass on modern Bash and genuine
Bash 3.2.57. A final HTTP status 22 invokes each installer download once, retains
the failed candidate and prior selection, and releases checkout/shared locks.
Existing storage reuse, corruption, cleanup and interruption checks also pass.
These use private stores and substitute asset transport.

The committed upstream download fixture passes on both shells with real curl
and a fixture-owned IPv4 loopback listener. Transient HTTP 500 and 429 recover
on the second request and authenticate the recovered bytes; repeated 504
exhausts four attempts with status 22. Permanent 404 makes one request; corrupt
bytes make one request and fail checksum admission. The fixture substitutes
HTTP loopback transport while checking production HTTPS/TLS flags; it does not
qualify public-network availability, TLS negotiation or native macOS behavior.

Actual snapshot and prepared host/IC admission, stable both-shell repeat previews,
selected ShellCheck, documentation links and diff checks pass. Logs use
`/tmp/ic-testkit-0347-shared093-*`; source/input/runtime/log digests are recorded
in `/tmp/ic-testkit-0347-shared093-qualification.json`. Own metadata, the
maintainer's dirty selected lock, staged contents, workflow and released cache
test inputs are preserved. No Cargo compilation, cache fetch, dependency
resolution or native tool installation runs. Full gates and matching native
consumer acceptance remain maintainer-owned.

## Cache-lock test isolation for pending 0.34.6

The nonblocking cache-lock regression now executes in an exact-test child with
one test thread ([#66](https://github.com/dragginzgame/ic-testkit/issues/66)),
following the existing staging-pruning test's isolation pattern. It retains all
contention, immediate release/reacquisition, content and redirected-file checks.
The original finding is a source review and Linux kernel inheritance control,
not an observed failure of the Rust test. Production lock semantics and selected
dependencies are unchanged.

Explicit `cargo fetch --locked --offline` prepares the selected cache without
resolution. All ten `artifacts::cache_fs::tests` pass against the locked Host
0.14.7 graph. A focused native process trace confirms an actual second test
executable invocation with the exact test selector and `--test-threads=1`; the
child executes one passing test. Strict library/test Clippy, prepared formatting,
documentation links and diff checks pass. Logs use
`/tmp/ic-testkit-0346-issue66-*`; source/input/log/artifact digests are recorded
in `/tmp/ic-testkit-0346-issue66-qualification.json`.

These are focused Linux tests, not matching native macOS acceptance of the
uncommitted fix. Own metadata, selected lock, Shared snapshot, workflow and
staged contents are preserved. Full validation and release effects remain
maintainer-owned.

## Shared 0.9.2 for pending 0.34.6

Canonical preview and genuine Bash 3.2 export select Shared Tooling 0.9.2
`c118062a070a2e238f1eb3989b6fd700f369ce1e`, retaining 62 roots and 93 payloads
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). Only the three
installers and evidence checksum verifier change from 0.9.1. The immutable
detached source matches GitHub; no upstream-only fixtures are added to the
consumer snapshot. Previous cleanup qualification remains bound to unchanged
shared-store and fixture inputs.

The committed upstream evidence-checksum and script-path fixtures pass on modern
Bash and genuine Bash 3.2.57. NUL in digests, filenames, suffixes and trailing
records refuses before backend execution, including with a substitute probe
backend; manifests and payloads remain unchanged. Valid GNU/Perl hashing and
final lines without newlines pass. The path fixture rejects supplied CR/LF,
invalid ancestors and physical symlink targets for host/IC/Rust install, check
and supported preflight modes before configuration or store creation.

A separate private control confirms the 0.9.1 checksum verifier accepts an
embedded NUL digest while 0.9.2 rejects it with status 1 and no success output.
Its retained input root is recorded in `/tmp/ic-testkit-shared092-nul-path`.
Actual adopted IC installer fixtures also pass on both shells, retaining their
storage, corruption, cleanup and interruption checks with substitute transport.

Actual snapshot and prepared host/IC/Rust admission, stable both-shell repeat
previews, selected ShellCheck, documentation links and diff checks pass. Logs
use `/tmp/ic-testkit-0346-shared092-*`; source/input/runtime/log digests are in
`/tmp/ic-testkit-0346-shared092-qualification.json`. Own metadata, selected lock,
staged contents, workflow and published history are preserved. No tool install,
Cargo compilation or dependency resolution runs. This is focused Linux execution;
native consumer acceptance and full gates remain maintainer-owned.

## Shared 0.9.1 for pending 0.34.6

Canonical preview and genuine Bash 3.2 export select committed Shared Tooling
0.9.1 `e8a9ea6b660f216166eb00d0af6a32462bdbda4a`, retaining 62 roots and 93
payloads ([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). The
reviewed GitHub source consolidates host/IC installation lifecycle handling and
fixes shared-lock release and error preservation on checkout cleanup failure.
The immutable detached clone excludes newer uncommitted 0.9.2 checksum work.

Actual adopted host and IC installer fixtures pass on modern Bash and genuine
Bash 3.2.57. The shared fixture injects checkout `rmdir` status 31 after cached
activation and confirms nonzero completion, no download and released shared
lock. A simultaneous curl status 23 retains that original failure and candidate,
releases the shared lock and preserves the prior active bundle. An explicit
retry succeeds without deleting failed evidence. Existing storage reuse,
corruption, locking, interrupted installation and offline-refusal controls pass.
These use private stores and substitute transport; they do not reinstall native
tools or prove macOS behavior.

The previous temporary Bash 3.2 executable was unavailable on the first attempt,
so that export did not run. Initial modern fixture logs describe unchanged 0.9.0
inputs and are retained as `*-before-adoption.log`; they are not 0.9.1 evidence.
A separately prepared Bash 3.2.57 binary is copied into a private qualification
directory, recorded by `/tmp/ic-testkit-shared091-bash32-path`, and used for the
successful export and both updated fixtures. No tool download or build runs.

Actual snapshot, prepared host/IC admission, stable both-shell repeat previews,
selected ShellCheck, documentation links and diff checks pass. Logs use
`/tmp/ic-testkit-0346-shared091-*`; source/input/log/runtime digests are recorded
in `/tmp/ic-testkit-0346-shared091-qualification.json`. Own package metadata,
selected lock, staged contents, workflow and published history are preserved.
No Cargo compilation or dependency resolution runs; full gates and matching
native consumer acceptance remain maintainer-owned.

## Shared 0.9.0 for pending 0.34.5

Canonical preview and genuine Bash 3.2 export select committed Shared Tooling
0.9.0 `aec7a149f581a42b9531fc747b46024fc0314fcf`, retaining 62 roots and 93
payloads ([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)). Only
the verifier and adoption guide change from 0.8.4. The immutable detached source
matches the published GitHub commit. Fleet reporters remain centrally owned and
are not added to this consumer. Existing v1 snapshots retain their layout.

Focused private controls pass on modern Bash and genuine Bash 3.2.57. The old
0.8.4 verifier accepts an embedded NUL; the new verifier rejects it. Syntax reads
preserve record order and literal spaces without a checksum backend or payloads.
Unknown, duplicate, missing, extra-field, noncanonical-path and NUL records emit
no output. Unreadable input refuses even with an inherited capture value; mixed
modes and symlinked manifests refuse. Ordinary integrity checks still reject
changed payload bytes, executable modes and symlinks, and never execute the
inspected helpers. These are private failure controls rather than the complete
upstream distribution suite.

Both-shell private refresh controls reject malformed manifests and false
retirement ownership without modifying payloads or the input manifest. Restoring
the valid manifest allows an unchanged same-source retry. Initial qualification
attempts are retained separately: the mixed-mode usage check needed a normal
PATH for its diagnostic, and the ownership control needed a retirement request
rather than an already-identical payload refresh.

Actual snapshot integrity, both-shell record reads, stable repeat previews,
selected ShellCheck, documentation links, exact canonical bytes and diff checks
pass. Inputs and logs are bound in
`/tmp/ic-testkit-0345-shared090-qualification.json`; logs use
`/tmp/ic-testkit-0345-shared090-*`, and the source and private control roots are
recorded in `/tmp/ic-testkit-shared090-{source,reader,refresh}-path`. Prior
qualification remains bound to unchanged inputs. Own metadata, selected Host
lock, staged contents, workflow and published history are preserved. No tool
installation, Cargo compilation or dependency resolution runs. These are focused
Linux checks; native consumer CI and full gates remain maintainer-owned.

## Shared 0.8.4 for pending 0.34.5

Canonical preview and Bash 3.2 export select Shared Tooling 0.8.4
`dca2bdd018afa6be598f03bb3cce1bab79c66973`, retaining 62 roots and 93 payloads
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). Only the workspace
LOC helper and snapshot-adoption guide change from 0.8.3. An immutable detached
clone supplies committed bytes; newer uncommitted sibling exporter, verifier and
fleet-reporter work remains intact and is excluded from adoption. Upstream's
changelog archival does not change this repository's published history.

The committed upstream LOC fixture passes on modern Bash and genuine Bash 3.2.57
using private workspaces, real offline Cargo metadata and prepared LOC tools.
It checks JSON totals against the human report, independent workspace selection,
and a substituted late count failure returning status 23 with no partial JSON.
The fixture is executed from the immutable source and is not added to this
consumer snapshot. The actual consumer helper has identical committed bytes.
Actual `make cloc` and both-shell JSON reports agree for both workspace packages:
21,048 runtime lines, 10,765 test lines, 347 test functions and 122 inline test
functions. This is a source-count observation, not a runtime performance result.

Snapshot integrity, unchanged repeat previews, selected ShellCheck, documentation
links and diff checks pass. Logs use `/tmp/ic-testkit-0345-shared084-*`; the
source path is recorded in `/tmp/ic-testkit-shared084-source-path`. Prior 0.8.2
and 0.8.3 qualification remains bound to its unchanged inputs. These are focused
Linux and Bash 3.2 checks; matching native consumer CI remains separate. Own
package metadata, the selected Host lock, staged index contents and pending
portable workflow change are preserved. No tool installation, dependency
resolution or compilation runs; full gates remain maintainer-owned.

## Shared 0.8.3 for pending 0.34.5

Canonical preview/export advances the existing 62 roots and 93 payloads to
Shared Tooling 0.8.3 `8cca7e67b03ce6e701b6e7dcec2e9753eb98337e`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #135](https://github.com/dragginzgame/shared-tooling/issues/135),
[Shared #136](https://github.com/dragginzgame/shared-tooling/issues/136),
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)). The
committed `VERSION` identifies 0.8.3 despite the commit message's 0.8.2 label.
An immutable detached source exports exact committed bytes; no files are added
or retired by this refresh. Interrupted peer payload files are ignored at any
depth by `.shared-tooling-payload.*`.

Modern Bash and genuine Bash 3.2.57 pass the updated Rust/host installer fixtures.
Redirected observation leaves refuse before writing, preserve outside bytes and
the prior selection, and archive the links without following their targets.
Successful PCRE2 probes remain quiet while failed probes retain diagnostics.
The focused canonical Make fixture on both shells checks split host/Rust catalogs
and rejects mismatched formatter selections before formatter execution; its
release effects are replaced with private stubs.

A focused private export control interrupts copying into the peer temporary
file with status 27. Both shells leave the original payload and manifest intact,
remove partial scratch, then complete a same-source retry. Inputs remain at
`/tmp/ic-testkit-shared083-atomic.yk_n_k1e/`; its log is
`/tmp/ic-testkit-0345-shared083-atomic.log`. This qualifies the selected atomic
replacement behavior, not the complete upstream distribution suite.

Actual prepared host/Rust checks and formatting pass. The consumer's actual
isolated hook qualification passes on both shells, as do snapshot integrity,
stable repeat previews, selected ShellCheck, documentation links, ignore-pattern
admission and diff checks. Logs use `/tmp/ic-testkit-0345-shared083-*.log`; source
scratch is recorded in `/tmp/ic-testkit-shared083-source-path`. These are focused
Linux checks with private/synthetic installations, not native consumer acceptance.
Existing freshness-checker evidence remains applicable to its unchanged inputs.
Own package metadata, selected Host lock, staged index contents and pending portable
workflow change are preserved. Read-only Git inspection refreshes index stat-cache
bytes without staging changes. No tool installation, dependency resolution or Cargo
compilation runs in this adoption; full gates remain maintainer-owned.

## Shared 0.8.2 for pending 0.34.5

Canonical preview/export selects committed Shared Tooling 0.8.2
`aaf75a37717357dd62f4b0b15001eaa3ba1629cf`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60)). The sibling checkout
has ongoing installer, formatter and exporter edits; an immutable detached clone
exports only the selected committed bytes. The snapshot adds the canonical Cargo
freshness checker, task and fixture, retaining every prior payload: 62 roots and
93 files. The task/governance index changes are also adopted.

The optional consumer command is
`perl scripts/dev/check-cargo-tool-versions.pl --consumer "$PWD"`. The committed
0.8.2 Make freshness target belongs to Shared Tooling's root Makefile; consumer
adoption uses the supported checker CLI. Registry inspection is explicit and
online, separate from offline prepared-tool admission. No live query, installer,
pin change or schedule is invoked by this qualification.

The maintained fixture passes on modern Bash and genuine Bash 3.2.57 using
private receipts and substitute registry transport. It covers non-yanked stable
selection, local/global/pinned differences, missing tools, failed HTTP/transport,
malformed or mismatched registry data, unsupported receipts, minimum curl and
nonliteral pins without execution. Portable CI now invokes that same offline
fixture. Canonical Bash 3.2 apply, snapshot integrity, stable repeat previews,
Perl syntax/help, selected ShellCheck, workflow lint, documentation links and diff
checks pass. Logs use `/tmp/ic-testkit-0345-shared082-*.log`; source scratch is
recorded in `/tmp/ic-testkit-shared082-source-path`. Native consumer acceptance
remains separate from these focused Linux checks.

The maintainer's pending Host 0.14.7 lock selection is preserved; source review
finds no crate implementation delta from 0.14.6. This Perl/tooling adoption runs
no Cargo compilation or dependency resolution and supplies no new native graph
qualification. Package metadata, real index and sibling dirty work are preserved;
full CI and release gates remain maintainer-owned.

## Shared 0.8.1 and Host 0.14.6 for pending 0.34.4

Canonical preview/export selects Shared Tooling 0.8.1
`c97f9f9c7b7c706fffdcfc5925cf58759b58093f`, preserving 60 roots and 90 payloads
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133),
[Shared #134](https://github.com/dragginzgame/shared-tooling/issues/134)). The
reviewed delta updates snapshot field admission, Rust-tool identity retention,
its fixture and two guides. No tools are installed or retired.

The prior verifier demonstrably accepts a malformed trailing empty field. The
new verifier rejects extra empty fields in version, format, source, revision and
file records on modern Bash and genuine Bash 3.2.57, while accepting literal
spaces in filenames. Private inputs remain at
`/tmp/ic-testkit-shared081-records.a0xg82w0/`; results are in
`/tmp/ic-testkit-0344-shared081-records.log`.

Both shells pass the changed canonical Rust-tool fixture: compiler, native
inventory and build-policy changes refuse activation, preserve the previous
selection, retain both observations without unmodelled secrets, and round-trip
their bytes through the standard selector/archive path. Actual canonical apply
on Bash 3.2, snapshot integrity, prepared Rust checks, formatting and selected
ShellCheck pass. Logs use `/tmp/ic-testkit-0344-shared081-*.log`; source scratch is
recorded in `/tmp/ic-testkit-shared081-source-path`. These use substitute Cargo
effects and private stores; native acceptance remains separate.

The maintainer's selected lock contains all four Host crates at 0.14.6. Cached
archives match their locked checksums and release VCS source
`11f8c31b599ea81f6106225c4068d78d22fed249`. The crate implementation is unchanged
from 0.14.5; this patch adopts contributor tooling. The first explicit offline
cache probe failed because the new archives were absent; that log is preserved.
`cargo fetch --locked` prepares the selected graph without resolution, and the
separate subsequent offline probe passes. All twenty controlled CLI tests and
three producer/cache composition tests pass against this graph, with four real
server tests ignored. Strict selected Clippy and actual Rust 1.88 library/CLI
compilation pass. Logs use `/tmp/ic-testkit-0344-host0146-*.log`. Package metadata,
maintainer lock selection, real index and pending setup-interruption inputs are
preserved. The full gate and native acceptance remain maintainer-owned.

## Build-observation readback for pending 0.34.4

The new Shared installer preserves both build observations, but the consumer's
normal artifact verifier initially ignored them
([#59](https://github.com/dragginzgame/ic-testkit/issues/59)). Removing the observed
build identity still passes the actual standalone readback; the contradiction is
retained in `/tmp/ic-testkit-0344-build-observation-before.log` and
`/tmp/ic-testkit-evidence-test.2EqZxB/verify-missing-build-observation.log`.

The existing three-archive qualification now compares the original and observed
identity/native bytes in the referenced failed shared generation. Compact
collection preserves them; missing build identity and changed native-observation
controls both refuse while retaining extracted evidence. The complete fixture
passes on modern Bash and genuine Bash 3.2.57, including existing linked-parent,
selector failure and corrupt-evidence controls. Proof sets remain at
`/tmp/ic-testkit-0344-observation-proof/` and
`/tmp/ic-testkit-0344-observation-proof-bash32/`; logs use
`/tmp/ic-testkit-0344-build-observation-*.log`. This is local collector/readback
execution with synthetic evidence and substitute transport. Native hosted
upload/download acceptance of the pending proof remains separate; the workflow's
artifact interface and canonical selection/archive helpers are unchanged.

## Setup interruption for pending 0.34.4

Released 0.34.3 `c78b02be422018067b48f6aaa4c20fb0d6f95ad2` leaves the downloader
and its descendant running when SIGINT terminates the setup CLI
([#65](https://github.com/dragginzgame/ic-testkit/issues/65)). An actual owner-CLI
build and private substitute-curl reproduction retain the observation at
`/tmp/ic-testkit-0344-setup-interrupt-before.log` and inputs at
`/tmp/ic-testkit-0344-setup-before.ml_p0fmb/`. Fixture cleanup stops the surviving
group. The maintained regression independently fails before repair because the
CLI dies by signal instead of completing cleanup and returning 130; its log is
`/tmp/ic-testkit-0344-setup-test-before.log`.

Setup now installs the CLI signal policy and passes its interruption predicate
to Host's owned-child communication. Host owns bounded capture, cancellation and
group cleanup; Testkit retains setup evidence and the signal exit-code policy.
Spawn errors retain their original typed operation, and stderr-recording failures
remain secondary to the process failure. The selected stream bounds, curl
limits, explicit download authority and installation admission are unchanged.

All six signal/recording controls pass: SIGINT, SIGTERM and SIGHUP, each with
ordinary or obstructed stderr recording. The downloader and descendant stop;
the partial archive and typed cancellation error survive without publishing an
executable. All seven CLI unit tests and thirteen controlled runner integration
tests pass, including original download-error, path admission, offline check/run,
managed interruption and command-status behavior. Four real-server tests remain
ignored. Strict selected Clippy and actual Rust 1.88 CLI compilation pass, as do
prepared formatting, documentation links and diff checks. Logs use
`/tmp/ic-testkit-0344-setup-*.log`; selected-lock offline cache preparation is in
`/tmp/ic-testkit-0344-cache.log`. These are focused Linux executions with synthetic
server/download transport, not native macOS or hosted acceptance of pending code.
Package metadata, lock selection, reviewed Shared snapshot and real index remain
unchanged; the full gate remains maintainer-owned.

Reported-live 0.34.3's
[matching CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38231005016)
passes all four Linux jobs and Intel macOS MSRV at this inspection, with seven
native jobs queued. Superseded 0.34.2's unfinished run was cancelled by the
newest-ref policy; its completed jobs remain bound to that older source. The
delivered repairs in #59, #60, #62, #63 and #64 still await matching native
acceptance. #61 and #38 retain their independently verified original acceptance
and downstream-owner obligations.

## Shared 0.8.0 snapshots for pending 0.34.3

Canonical preview/export advances the recorded 60 entrypoints and unchanged
90-file closure to Shared Tooling 0.8.0
`045a6f22a9de497bfb3af90cb2571957384806f2`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)). The
committed `VERSION` and generated annotation both identify 0.8.0, resolving the
earlier source-label mismatch. Preview updates only the consuming-snapshot and
Make adoption guides, with no additions or removals. No local command uses the
removed `--retire-file` option. Source and installed tool selections remain
otherwise unchanged.

Actual canonical apply passes on genuine Bash 3.2.57. Snapshot verification and
recorded-root repeat previews pass on modern Bash and Bash 3.2; each repeat
contains only 90 KEEP rows. The focused canonical retirement fixture passes on
both shells using private Git fixtures. It checks automatic removal and repeat
refreshes, preservation of directly selected and unowned files, refusal of
edited/staged/redirected/forged ownership, concurrency, and interrupted-deletion
recovery. This executes the upstream fixture from immutable source scratch; it
does not import that test or exporter into the consumer.

`make shared-tooling-check`, selected ShellCheck, documentation links and diff
checks pass. Logs use `/tmp/ic-testkit-0343-shared080-*.log`; immutable source
scratch is recorded in `/tmp/ic-testkit-shared080-source-path`. These are focused
Linux checks, not the complete upstream delivery suite or native macOS
acceptance. Existing prepared-tool, Host and evidence-repair qualification
remains applicable to unchanged inputs. Package metadata, selected Host lock,
real index and pending evidence-repair inputs are preserved.

## Earlier Shared entrypoint qualification for pending 0.34.3

Canonical preview/export adopts reviewed Shared commit
`8e8cc21265272e0234326c53a8c7bf9e7d8d77df`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #131](https://github.com/dragginzgame/shared-tooling/issues/131)). The
commit is labelled 0.7.0 and introduces that pending entrypoint contract, but its
committed `VERSION` is still 0.6.6 and there is no tag at that source. The generated
annotation retains 0.6.6; the exact revision identifies the reviewed code.

The initial selection explicitly records 60 roots covering the existing shared
commands, integration fixtures and governance documents. Their committed
companion closure, including mandatory integrity helpers, retains the same 90
payloads. No files are added or retired. The only payload changes are the
snapshot verifier, consuming-snapshot guide, Make adoption guide and snapshot
drift task. Consumer Make/CI/script callers remain covered, and no local command
uses the replaced exporter flags. Future refreshes reuse recorded roots;
conditional local callers and other manifests still require ownership review.

Actual canonical apply passes on genuine Bash 3.2.57. Both modern Bash and Bash
3.2 accept the current snapshot, reject duplicate, malformed, unsafe and
unselected roots, and produce a repeat preview containing only 90 KEEP rows
without resupplying roots. `make shared-tooling-check`, selected ShellCheck,
documentation links and diff checks pass. Logs use
`/tmp/ic-testkit-0343-shared070-*.log`; immutable source scratch is recorded in
`/tmp/ic-testkit-shared070-source-path`. These are Linux consumer adoption checks,
not the complete upstream distribution/retirement suite or native macOS
acceptance. Tool pins and executable installers are unchanged, so existing
prepared-tool and Host 0.14.5 qualification remains applicable. Package metadata,
selected lock, real index and the pending failure-evidence repair are preserved.

## Linked checkout evidence for pending 0.34.3

On released 0.34.2 `67ea968831d45e49e7eb6f28b54cab7bcc07f4b7`, the collector
refused a symlinked `target` or `.tools` before creating an archive, losing
unrelated valid logs ([#59](https://github.com/dragginzgame/ic-testkit/issues/59)).
The actual pre-fix fixture fails in
`/tmp/ic-testkit-0343-linked-evidence-before.log`; its retained inputs remain at
`/tmp/ic-testkit-evidence-test.kv2Yn2/`.

The consumer collector now archives the parent link itself, unrelated evidence
and `collection.log` without traversing either redirected tree. It emits the
completed archive path before propagating failure; a simultaneous selector
failure retains its original status and diagnostic-only partial selections.
Canonical Shared selection and archiving helpers are unchanged.

The complete focused fixture passes on modern Bash and genuine Bash 3.2.57.
For both parents and both collector modes, controls check archive member names,
retained link bytes, installer logs and outcomes. A selector emitting a partial
pair before failing with status 23 still returns 23 and excludes its proposed
payload. Existing normal/rejected-selection round trips, corruption controls,
overwrite refusal and partial-archive retention also pass. Logs use
`/tmp/ic-testkit-0343-linked-evidence-*.log`. These are Linux fixtures with
synthetic tools, not native macOS or hosted artifact-service acceptance of this
pending source. Full validation remains maintainer-owned.

## Locked Host 0.14.5 qualification for pending 0.34.3

The maintainer's selected lock contains all four Host packages at 0.14.5. Cached
archives match their locked checksums and release VCS source
`bb8f1e71f3d4599618d0f4084d27fa637408f1fb`. This is a release-metadata-only
patch from 0.14.4; no new API or local code removal is justified. Shared remains
on reviewed 0.6.6 `a15eb9d406d287697df9bc8fa96db8cda6721a71`.

Explicit selected-lock offline cache preparation, all three producer/cache
composition tests, strict library/CLI Clippy and actual Rust 1.88 library/CLI
compilation pass. Logs use `/tmp/ic-testkit-host0145-*.log`. Qualification does
not resolve dependencies or change package metadata, the selected lock, Shared
snapshot or real index. This Linux qualification of the pending graph is
separate from released-source CI and earlier Host 0.14.4 evidence below.

## Earlier Host 0.14.4 qualification for pending 0.34.3

The preceding maintainer lock update selected all four Host packages at 0.14.4. Cached
archives match their selected checksums and release VCS source
`f93cd8854d500616a73eda713d215a20ff0617c7`. The crate implementation tree is
unchanged from 0.14.3; this upstream patch adopts contributor tooling. Explicit
selected-lock offline preparation, the three shared-tool/producer/cache
composition tests, strict library/CLI Clippy and actual Rust 1.88 library/CLI
compilation pass. Logs use `/tmp/ic-testkit-host0144-*.log`. Qualification does
not resolve upgrades or modify package metadata, lock selection or the real
index. Native acceptance of this new graph remains separate.

The reported-live 0.34.2 source is
`67ea968831d45e49e7eb6f28b54cab7bcc07f4b7`; its
[matching workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/38223153251)
passes all four Linux jobs and Intel macOS concurrency at this inspection;
seven native jobs remain queued. The unfinished 0.34.1 workflow was cancelled by the
newest-ref policy, preserving its individual completed results without accepting
its uncompleted native lanes. Delivery notes and older complete 0.34.0 native
evidence remain distinct from this pending graph.

## Shared 0.6.6 derive formatter for pending 0.34.2

Canonical preview/export selects committed Shared 0.6.6
`a15eb9d406d287697df9bc8fa96db8cda6721a71` with the same 90-file roster
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113)).
Only the selected pin catalog, setup guidance and hook rules change from the
0.6.5 adoption. The actual `make install-rust-tools` command prepares the new
`cargo-sort-derives` 0.14.1 selection; offline `make rust-tools-check` then passes.
Previous tool installations remain retained.

Read-only inventory finds no affected Testkit macros or workaround markers.
The prepared formatter processes the committed Shared macro fixture extracted
from its frontend qualification: direct/conditional literal `$crate` paths
survive, ordinary derives sort and repeat output is stable. The formatted macro
library and its separate consuming crate compile and run with Rust 1.88. This
is direct tool/compiler qualification, not execution of Shared's complete mixed
frontend hook suite. Source scratch remains in
`/tmp/ic-testkit-shared066-derives.llufw1wc/`; compiler artifacts remain in
`target/shared-tooling-qualification/shared066/`. Logs use
`/tmp/ic-testkit-0342-shared066-*.log`. Consumer hook qualification passes on
modern Bash and genuine Bash 3.2.57; the full prepared tool/pin checks, formatting,
snapshot integrity, ShellCheck with the pin catalog's Bash context and doc links
pass. Native consumer acceptance remains separate from these Linux results and
prior source-bound qualification.

## Locked Host 0.14.3 qualification for pending 0.34.2

The maintainer's dirty lock update selects all four published Host crates at
0.14.3. Each cached archive matches its locked SHA-256 and release VCS source
`543b137458feba0f6dfd3e458b680fb2be3d9e6d`. Read-only comparison from Host 0.14.1
through 0.14.3 finds no crate implementation change; the later patches update
contributor tooling and documentation. No new Host API or local code removal is
justified. The released Testkit 0.34.1 graph and prior qualification remain
separate from this selection.

Explicit selected-lock offline cache preparation passes without resolution.
Twenty-three focused tests pass: three producer/cache composition tests, ten
cache-filesystem tests, the complete-probe status/spawn/overflow test, six
provisioning fixtures, two actual download-failure CLI tests and the line-break
directory test. Strict selected Clippy and actual Rust 1.88 library/CLI
compilation pass. Logs use `/tmp/ic-testkit-host0143-*.log`. These execute on
Linux with synthetic producers/servers/download transport; native acceptance of
the new selected graph remains separate. No package version, selected lock,
reviewed Shared snapshot, real index or sibling file was changed by qualification.

## Shared 0.6.5 adoption for pending 0.34.2

Canonical preview/export selects committed Shared 0.6.5
`42f2091768b5f71c98a684d3dab563f408ca2dd9`, preserving the 90-file selection
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129),
[Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).
The local gates declare all Cargo manifests, the lockfile, toolchain, pins,
Makefile and Shared snapshot as prepared inputs. Consumer fixtures copy that
declaration and execute the actual gate recipes with substitute stages. Changing
or removing the lock after a stage stops the next stage and retains the cause.
The checksum companion is included in isolated runner exports.

Consumer release guards, canonical nested/input/status runner tests, selected
Rust-tool receipt/staged-export tests and actual hook qualification pass on
modern Bash and genuine Bash 3.2.57. Actual prepared Rust checks, formatting,
snapshot integrity, declaration pins, ShellCheck and documentation checks pass.
Successful fixture logs have no recipe-override warnings. Logs use
`/tmp/ic-testkit-0342-shared065-*.log`. Compilers, installers, complete gates and
release effects in the fixture suites are substitutes; no real tool install,
full gate or release ran. Package metadata, selected dependencies, the real
index and pending runtime work remain intact. Native acceptance still requires
matching delivered-source CI; previous release evidence remains source-bound.

## Store directory admission for pending 0.34.2

On reported-live 0.34.1 `4c24ae1762e1c5c1035563fd972ce75f2a70a423`, an actual CLI
regression with substitute curl showed that a line-break store directory reached
the downloader and created setup files
([#64](https://github.com/dragginzgame/ic-testkit/issues/64)). The executed
pre-fix failure remains in `/tmp/ic-testkit-0342-path-before-executed.log`; the
earlier invocation selecting zero tests is separately retained and supplies no
reproduction evidence.

The path owner now checks complete absolute bytes before normalization and
filesystem admission. The regression passes all LF/CR, explicit-store/relative
ancestor and setup/check/run combinations without tool execution or store
creation. The existing missing-store offline check and all six provisioning
fixtures pass, as do selected strict Clippy and actual Rust 1.88 CLI compilation.
Logs use `/tmp/ic-testkit-0342-*.log`. Initial lint and formatting rejections are
retained separately from their final passing checks. Selected-lock offline cache
preparation passed without dependency resolution. Package metadata, index, lock selection,
reviewed Shared snapshot and retained installations are preserved. This enforces
the existing directory policy and changes no valid path or library API; matching
delivered native qualification and the full gate remain separate.

## Latest released-source acceptance

[Released 0.34.0 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38150408955),
source `ccfd0dbc47a527e330cd55d91b1d02f83cb7a9f2`, now passes all 12 checks,
MSRV, portable and real concurrency jobs across Linux and both macOS 15
architectures. Both native portable logs explicitly pass release metadata,
publication, hook and downloaded evidence qualification. This accepts the
retained implementation on that source; earlier cancelled releases are not
retroactively qualified. Reported-live 0.34.1 has its own
[workflow](https://github.com/dragginzgame/ic-testkit/actions/runs/38216618519):
all four Linux jobs and Intel macOS MSRV pass at final inspection. Intel checks
are running; six native jobs remain queued. Its newer fixes still require the
remaining native acceptance, separate from dirty pending 0.34.2 work.

## Rejected-selection artifact proof for pending 0.34.1

On released 0.34.0 `ccfd0dbc47a527e330cd55d91b1d02f83cb7a9f2`, the remaining
hosted readback boundary from [#59](https://github.com/dragginzgame/ic-testkit/issues/59)
now has maintained CI qualification. The existing fixture exports normal evidence
and both archives produced by actual checks/portable collector blocks after
Shared rejects a symlinked store. The workflow's existing pinned actions upload
these three archives together and read them back by exact artifact ID with digest
admission. The standalone verifier consumes the downloaded proof directory;
there is one current interface and no archive-file fallback.

Modern Bash and genuine Bash 3.2.57 pass the complete fixture and standalone local
readback: retained local build/receipt bytes, executable modes, non-followed links,
original selector status, incomplete collection and outcomes. Shared payloads and
unrelated files remain excluded. A control changing rejected selection to status
zero fails verification and retains extracted evidence; existing corrupt-payload,
corrupt-outcome and failed-archive controls still pass. ShellCheck and actionlint
pass. Fixture logs use `/tmp/ic-testkit-0341-final-*.log`; all three exported
archives remain in `/tmp/ic-testkit-0341-final-proof{,-bash32}/`.

A further byte-preservation check demonstrated that appending a newline to the
normal payload still passed the earlier readback comparison. Exact comparisons
now preserve newline significance for expected payloads, logs, receipts, statuses
and outcomes. The same altered archive now fails and retains its extracted
evidence (`/tmp/ic-testkit-0341-newline-before.w97GpH/`). Added controls append
newlines to the normal payload and portable rejected-selection build log; both
reject without losing their extracted bytes. The complete fixture passes on
modern Bash and genuine Bash 3.2.57, with logs
`/tmp/ic-testkit-0341-bytes-{modern,bash32}.log` and exported archives
`/tmp/ic-testkit-0341-bytes-proof{,-bash32}/`. ShellCheck passes.

Scope qualification then demonstrated that removing validation failure diagnostics
from a rejected-selection archive still passed
(`/tmp/ic-testkit-0341-missing-diagnostic-before.IMZnF2/`). A common readback flow
now checks local tools, formatter/validation diagnostics and outcomes for all
three archives, with checks/portable scope and shared-selection status supplied
by the proof caller. Both workflow fixtures offer the same logs, portable data
and prior readback files so retained and excluded content can be checked together.
Controls removing checks validation diagnostics or a portable payload now fail
without losing extracted evidence. A separate missing-diagnostic attempt retains
its proof and surviving build log under
`/tmp/ic-testkit-0341-missing-diagnostic-after.cxxSC4/`.

The complete fixture and standalone verifier pass on modern Bash and genuine
Bash 3.2.57. Final logs use `/tmp/ic-testkit-0341-scope-final-*.log`; exported
proof sets remain in `/tmp/ic-testkit-0341-scope-final-proof{,-bash32}/`.
ShellCheck passes. This extends consumer-owned qualification without changing
the collector or reviewed Shared helpers.

These runs execute the real collector/workflow shell blocks with synthetic tools
and substitute download transport. Local copies/readback do not establish hosted
artifact-service acceptance or native macOS behavior; those need the matching
next CI run. Contributor qualification is compatible 0.34.1 work. These archive
changes preserve library/CLI behavior, package versions, real index/refs and
release state. Their earlier Shared 0.6.3 and maintainer-selected Host 0.14.1,
cc 1.8.0 and serde_json 1.0.153 inputs remain bound to those logs; no Cargo job or
dependency resolution ran for that archive change. Separate runtime and Shared
0.6.4 work is qualified below.

At inspection, [released 0.34.0 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38150408955)
passes all four Linux jobs, Apple macOS checks, and Intel macOS portable/MSRV.
Intel macOS checks are running; four macOS jobs remain queued. The newest run
cancelled unfinished 0.33.8 jobs; their earlier individual successes remain bound
to that older source. This run qualifies released 0.34.0, not pending 0.34.1.

## Shared 0.6.4 Make interface for pending 0.34.1

On released 0.34.0 `ccfd0dbc47a527e330cd55d91b1d02f83cb7a9f2`, canonical
preview/export selects committed Shared 0.6.4
`6fc8eb9a94f83fb0b9d0a26377d4232508cdbe2d` with 90 files
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).
The root Makefile uses its common entrypoint and removes duplicate local
hook/snapshot-check recipes. Formatter admission still precedes hook setup;
ordered owner PocketIC extensions, product validation, publication and release
metadata adapters remain local. The default command still shows help. Isolated
hook and release exports include the new entrypoint.

Snapshot integrity and actual formatting pass. Consumer hook qualification passes
on modern Bash and genuine Bash 3.2.57 with the prepared formatter: staged-only
formatting, lock/unrelated-byte preservation, failed prerequisites, and isolated
activation/refusal. Consumer release guards also pass on both shells, exercising
actual Make routing with substitute runner, compiler, installer and complete-gate
effects, plus isolated metadata/retry preservation. These fixtures never execute
the real release flow or install tools. Logs use
`/tmp/ic-testkit-0341-shared064-*.log`. Bare Make, ShellCheck and diff checks pass.
No tool reinstall is needed; previous tools and release evidence remain intact.
Matching delivered-source native CI and the maintainer's full gate remain separate.

## Local Make job controls for pending 0.34.1

A disposable copy of the actual consumer Makefile showed `make -j2 check`
advertising pipe jobserver descriptors but closing them before the Cargo
substitute could inspect them
([#63](https://github.com/dragginzgame/ic-testkit/issues/63)). The standard Make
`+` handoff now applies to local compiling Cargo recipes, test shell chains and
the publication adapter. The same descriptor check passes, including through a
substitute publication adapter. Setup, checks and tests share one absolute owner
CLI path. Original and repaired proofs remain in
`/tmp/ic-testkit-0341-make-jobserver-{before,after}.log` and
`/tmp/ic-testkit-0341-make-publish-jobserver.log`.

Consumer release guards pass on modern Bash and genuine Bash 3.2.57. Their
extended isolated routing fixture checks jobserver access for product Cargo
recipes, the PocketIC environment, both test invocations, stopping after owner
admission or Cargo failure, and existing ordered setup/offline checks. Logs use
`/tmp/ic-testkit-0341-make-guards-{modern,bash32}.log`. Publication guards pass
with substituted registry transport, Cargo and Git. Compiler, installer and
complete-gate effects are substitutes; no real full suite, release, publication
or tool setup ran. Matching delivered native qualification remains separate.
This compatible contributor repair preserves package versions and lock selection.

## PocketIC download error precedence for pending 0.34.1

On released 0.34.0 `ccfd0dbc47a527e330cd55d91b1d02f83cb7a9f2`, an actual CLI
regression substitutes curl to write a partial archive, obstruct `download.stderr`
with a directory, then exit 23. Before the fix, `failure.txt` reports the recording
error instead of Host's independently obtained process failure
([#62](https://github.com/dragginzgame/ic-testkit/issues/62)). The failed attempt
log remains at `/tmp/ic-testkit-0341-download-before.log`.

Setup now reports failed stderr persistence separately when downloading already
failed, returning the original typed Host cause to the CLI and `failure.txt`.
Successful curl still stops setup on a recording failure before authentication
or publication. No new wrapper, format, limit, timeout or dependency is introduced.
Focused actual-CLI checks cover exit 23 and SIGTERM, successful/obstructed recording,
two failed retries retaining partial archives and original causes, and successful
curl with failed recording refusing installation. Six provisioning unit fixtures
also pass; these use authenticated synthetic gzip/executables, not a real download.

The maintainer lock update was preserved. Explicit locked fetch prepares the
selected graph without upgrades; current qualification selects Host 0.14.1,
cc 1.8.0, serde_json 1.0.154 and syn 3.0.8. Focused CLI/provisioning tests and
strict selected Clippy and actual Rust 1.88 CLI compilation pass. Logs use
`/tmp/ic-testkit-0341-download-current-*.log`; earlier attempts retain their
original log identities. Native macOS evidence and
the full maintainer gate remain separate from these Linux checks.

## Host 0.14 completed-result adoption for pending 0.34.0

On released Testkit 0.33.8 `f17892332f4b100ffea4b28528927bfb0b4e1c5c`, the
pending source adopts published Host 0.14.0
`4723befe8d65c760c00e05c42652759d691fbd5e`
([#61](https://github.com/dragginzgame/ic-testkit/issues/61)). All four root Host
requirements and selected lock entries advance together; unrelated lock selections
are preserved. The public re-exports require a Testkit minor release. Both pending
changelog views use 0.34.0, carrying the compatible Shared 0.6.3 work below;
the maintainer-owned package version remains 0.33.8.

Cargo/probe and workspace discovery paths use required completed status, removing
nonzero-exit error unpacking and completed-status assertions. Progress observation
returns success through Host's fallible callback. Strict producer success admission
is explicit in PocketIC downloads, server-version probes, the transactional cache
example, its module documentation and composition fixtures. No cache layout,
installed bundle policy, Cargo deadline or observer output limit changes.

Explicit `cargo fetch --locked` prepared the selected graph. Linux all-target
compilation and 20 focused regressions pass: observed Cargo success/failure,
parallel output, descendant cleanup and panic; diagnostic truncation and complete
probe limits; Cargo workspace discovery/membership; Host/cache composition;
PocketIC version admission and cleanup; provisioning; and the actual CLI's failed
download retry/evidence retention. The CLI download test substitutes only curl,
uses the real setup/evidence path and compares its retained primary failure with
Host's typed nonzero/signalled process result. No real download was attempted.
Selected strict Clippy, warning-denied Rustdoc, artifact composition doctest and
actual Rust 1.88 library/CLI compilation also pass. Logs use
`/tmp/ic-testkit-host014-*.log`.

The migrated real `transactional_artifact_cache` executable separately refuses
publication and reuse after a producer writes staged output then exits 7 or is
terminated by SIGTERM, with each failure retried. A successful control publishes
and repeats without rerunning the producer. Six example commands produce five
producer invocations. Source/input/binary digests, results, streams and trace are
retained in `/tmp/ic-testkit-host014-publication.z0s5i1qr/proof.json`. This is Linux
execution of the example, distinct from the curl substitute and from native macOS
acceptance. The real index, release metadata, installations and sibling source
remain untouched.

[Exact-source Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38146341421)
passes Linux native and MSRV; both macOS jobs are queued at inspection. That run
does not qualify pending Testkit 0.34.0. Full consumer gates, native CI, publication
and downstream adoption remain separate acceptance obligations.

A focused #61 follow-up carries Host's `CompletedExecution` through the complete
probe/success-projection path instead of reconstructing `std::process::Output`.
The added native composition regression executes authenticated producers through
Host, checks their actual staged Wasm bytes, then requires success before commit.
Nonzero and SIGTERM cases each run twice, retain the original typed exit failure,
preserve the prior public artifact and cannot reuse failed output. Existing
successful composition/reuse controls also pass. This module is already selected
in portable CI on all three supported hosts; Linux execution does not establish
macOS acceptance.

All three composition tests, the complete probe regression, selected strict Clippy
and actual Rust 1.88 library compilation pass. Initial qualification above remains
bound to its earlier inputs; focused follow-up logs use
`/tmp/ic-testkit-host014-followup{2,3}-*.log`, with final input digests recorded in
`/tmp/ic-testkit-host014-followup-inputs.json`. No additional function/type owner or
public API was added for this cleanup; the pending version stays 0.34.0.

## Shared 0.6.3 committed preparation restart for pending 0.34.0

On released 0.33.8 `f17892332f4b100ffea4b28528927bfb0b4e1c5c`, clean canonical
preview/export advances the existing 88-file snapshot to pushed Shared 0.6.3
`4fd979d13c32adb3cb2f51f213ebfc4f98339b3d`
([Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126)).
Normal direct releases can archive pre-staging preparation after a clean descendant
commit, then select the requested increment from current metadata and revalidate.
Explicit resume, staging, destination and source-conflict refusals retain their
separate contracts. Consumer transaction checks still own retained metadata.

Canonical simulations pass on modern Bash and genuine Bash 3.2.57 for unchanged
or bumped committed metadata, all increments, changed requested kind, preserved
intent/evidence, failed fresh validation and rejected conflicting state. Consumer
release/metadata fixtures pass on both shells. The new consumer case rejects the
old source's receipt, preserves receipts/logs/build evidence after failed
revalidation, and binds successful validation/preparation to the new source.
Shared simulations own descendant admission; consumer fixtures substitute Git
and the full gate while exercising the actual metadata callbacks. Snapshot,
formatting and ShellCheck pass. Logs use `/tmp/ic-testkit-shared063-*`.

The unchanged Cargo build policy needs no tool reinstall. The Shared adoption
left package metadata, lock selection, real index, release plans and refs untouched.
It is compatible contributor work carried into the pending 0.34.0 Host migration;
full gates and release remain maintainer-owned.
At inspection, [0.33.8 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38144433459)
passes all four Linux jobs and Apple Silicon checks, with seven macOS jobs queued.
[Shared 0.6.3 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38145666246)
passes lint; portable/native acceptance is unfinished. These runs do not qualify
the pending consumer source's complete native matrix.

## Shared 0.6.2 release diagnostics and Host 0.13.5 for pending 0.33.8

The unchanged 88-file selection now binds pushed Shared Tooling 0.6.2
`7f1ca16b0dfa98021d2671017ecb7de767409b0e` through clean canonical preview/export.
Release source conflicts report the saved plan, phase, versions and both commits;
prepare conflicts point to guarded maintainer recovery
([Shared #126](https://github.com/dragginzgame/shared-tooling/issues/126)).
No real release plan, validation receipt or ref was changed. The Cargo setup fix
and build policy are unchanged from the qualified 0.6.1 adoption below. The new
Audit/Machete guidance requires no local adapter: Testkit selects neither tool.

Canonical release simulations and consumer release/metadata adapter fixtures
pass on Linux modern Bash and genuine Bash 3.2.57, including preservation of
source-conflict evidence and fresh validation after simulated retirement. All
release effects remain isolated or substituted. Snapshot, dependency-pin and
ShellCheck checks pass. Logs use `/tmp/ic-testkit-shared062-*`.

The maintainer's lockfile already selects all four published Host 0.13.5 crates.
Explicit locked cache preparation, offline all-target compilation and all ten
focused cache-filesystem tests pass (`/tmp/ic-testkit-host0135-{check,cache}.log`).
Review of the upstream Rust delta finds the additive private sidecar opener;
Testkit's existing cache callers require durable creation with missing parents,
so they keep their established Host descriptor APIs. The selected lockfile,
package manifest and real index remain byte-for-byte unchanged during this work.

At inspection, Host 0.13.5 Linux native and MSRV jobs pass; its macOS jobs are
queued. Shared 0.6.2 lint passes, Linux and Apple Silicon portable jobs are running,
and Intel macOS is queued. These observations do not establish complete native
acceptance of this pending compatible 0.33.8 batch.

## Shared 0.6.1 setup repair for pending 0.33.8

The canonical 88-file snapshot selects committed Shared Tooling 0.6.1
`731909525038c58d6f80b498e80f8630b11c685d`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120)).
The export uses a clean isolated source checkout; the sibling's unfinished
release-runner work is excluded. Cargo setup explicitly propagates failures and
cleans up only acquired locks. Activation requires both installation locks.

Explicit native `make install-rust-tools` and offline Rust/host/IC checks pass.
A second checkout with the exact prepared compiler path reuses all three shared
Rust executables under explicit offline policy, without local payload copies.
The first reuse attempt selected the distinct `stable` compiler directory; it
was stopped with its unsuccessful build evidence retained, and is not reuse
proof. No sibling build or foreign owner lock was interrupted.

Canonical Rust and host/IC fixtures, including lock/activation failure propagation,
and consumer failure-archive/readback fixtures pass on Linux modern Bash and
genuine Bash 3.2.57. The store companion runs through its host/IC parent fixtures;
it has no standalone invocation. Snapshot, formatting and ShellCheck checks pass.
All 2,373 recorded physical local tool files and maintainer inputs are preserved.
Logs and digests use `/tmp/ic-testkit-shared061-*`; the successful native reuse
log is `reuse-pinned.log`.

Correction to the historical report below: the linked Shared 0.6.0 adoption
section was present in the committed guide. Its absence was incorrectly reported.
At inspection, all four Linux jobs of [0.33.7 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38141969782)
and both Linux jobs of [Shared 0.6.1 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38141614488)
pass; native macOS jobs remain queued. This qualifies focused local adoption,
not the pending source's complete hosted/native matrix. The compatible 0.33.8
notes leave package metadata and Host 0.13.3 dependency selection unchanged.

## Shared 0.6.0 Cargo storage adoption for pending 0.33.7

The canonical 88-file snapshot selects pushed Shared Tooling 0.6.0
`75180980a0c4e4252d8a915aaecfaefd92bf9a4e`
([#60](https://github.com/dragginzgame/ic-testkit/issues/60),
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120)).
Preview identified `cargo-tool-build.sh` as a required companion; it was reviewed
and added explicitly before export. Cargo tools use the standard shared build
environment and exact source/lock, compiler and native identities. Paths remain
unchanged. The shared selector now owns local Rust and referenced shared Cargo
evidence; the consumer removes its duplicate Rust selection. Archive/readback
qualification includes failed shared Cargo bytes and excludes unrelated Rust
generations, while preserving the incomplete-collection contract from #59.

Snapshot verification, canonical Rust fixtures and final consumer archive
fixtures pass on Linux modern Bash and genuine Bash 3.2.57. Updated host/IC
fixtures pass on both shells too. Formatting, ShellCheck, actionlint and local
documentation links pass. Digests preserve all 15 recorded prior local Rust
files and the real index, package manifest and maintainer's Host 0.13.2 lockfile
selection. Evidence uses `/tmp/ic-testkit-shared060-*`.

Actual Rust setup is **not qualified**. It reported the Host checkout's held
shared installation lock and an attempted `/selected` activation, but returned
zero. Aggregate and same-native-host Rust admission then refused candid-extractor.
The upstream conditional common-setup failure propagation defect is independently
reproduced on both shells using unchanged canonical helpers with substitute
Cargo: held-lock setup reports activation failure yet returns zero. The extra
assertion correctly fails and retains `/tmp/rust-tools-test.53xa12` and
`/tmp/rust-tools-test.CyS3Ig`; the actual attempt and refusal logs remain separate.
No owner lock or sibling process was removed or interrupted. A scoped stop was
attempted only for our setup tree, but it had already completed.

The defect and missing linked 0.6 adoption section are reported at
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120#issuecomment-6108732532).
Adopt a pushed canonical correction before qualifying setup/reuse and exact-source
native/hosted acceptance. No local installer fork, alternate store or old-path
fallback is introduced. Library/product contracts and release metadata stay
unchanged; contributor-only adoption extends pending 0.33.7. Full gates and
release execution remain maintainer-owned.

## Local evidence after failed shared selection for pending 0.33.7

On released base `5a6cd1fbf819d5a07652deea46276cb2763b6e7e`, the consumer
collector now archives local evidence even when shared selection fails
([#59](https://github.com/dragginzgame/ic-testkit/issues/59),
[Shared #121](https://github.com/dragginzgame/shared-tooling/issues/121)).
It captures the original selector status, stderr and partial output; only a
successful selection authorizes store reads. Failure retains the physical local
tool tree without following links, records incomplete shared collection, then
returns the original failure status after emitting the completed archive path.
Both workflow collector steps publish that path before failing, allowing their
existing failure-only upload steps to retain the archive. No admission rule or
vendored helper is changed.

The actual checks/portable workflow collector blocks pass focused Linux
qualification on modern Bash and genuine Bash 3.2.57 with a symlinked store.
Extracted local log/build/receipt bytes, references and outcomes are retained;
shared payloads and outside link targets are excluded. An injected selector's
partial ROOT/PATH output remains diagnostic data and its status 23 propagates.
Existing valid compaction, corrupt/failed generation retention, standalone
readback, and failed verification/archiver evidence still pass. ShellCheck and
actionlint pass. Logs use `/tmp/ic-testkit-0337-{evidence,evidence-bash32,
shellcheck,actionlint}.log`. This contributor-only repair uses patch notes;
package metadata, lock selection, real index and sibling source remain unchanged.

All four Linux jobs of [released 0.33.6 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38134933464)
pass at inspection; all eight native macOS jobs are queued. This does not qualify
the pending repair's native execution or hosted upload/readback. Those remain
outstanding; no broad local gate or release ran.

## Shared 0.5.0 storage adoption for pending 0.33.6

On released base `12959b1ecb110a94584e9c0dde9dfd2cbc55d187`, the reviewed
snapshot selects 87 files from Shared Tooling 0.5.0
`a306180ce534227fcb98b882fba9a526d5d747e0`
([#58](https://github.com/dragginzgame/ic-testkit/issues/58),
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120)).
Preview identified the store engine and sourced store fixture as required
companions; both were added explicitly before canonical export. The interrupted
0.4.2 policy-only refresh was preserved and advanced from its committed source.

Explicit host and IC setup now selects verified bundles in the default per-user
store through the existing command paths. IC setup first refused a held shared
installation lock; retry succeeded after the lock cleared, without reclaiming it.
A disposable second checkout, with curl replaced by a refusing recorder, reused
the exact physical host/IC bundles with zero downloads and no local payload copies.
Checks passed in both checkouts. Digests confirm that all 110 recorded files in
the earlier physical local bundles are unchanged. Rust slots and the prepared
PocketIC 16.1.0 retain their existing paths.

Shared host/IC fixtures pass on Linux Bash 5 and genuine Bash 3.2.57, covering
store reuse, independent selections, corruption, locks, explicit repair and
offline refusal. The consumer archive fixture now delegates active selection
creation to the canonical installer with substitute transport. Its actual
collector, standalone readback and failed-verification retention pass on both
shells. The upload artifact includes referenced shared failed-generation bytes;
unrelated generations are excluded, corrupt selections remain full, and Rust
evidence survives host compaction. Actual hook checks pass on both shells.
Aggregate offline tools, snapshot, declaration pins, formatting and selected
ShellCheck pass. The missing owner CLI was rebuilt offline with `--locked` after
explicit fetch. Index, package metadata, lock selection and published notes remain
unchanged. Contributor-only adoption uses pending 0.33.6 patch notes.

Evidence uses `/tmp/ic-testkit-shared050-*`: preview/refresh, setup and refused
attempts, real reuse, installer fixtures, final consumer archive checks, hooks,
ShellCheck, cache preparation, CLI build and final checks. Old local payloads and
owning inputs are separately digest-recorded. No broad local gate, sibling edit
or release ran. [Exact Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38132790945)
passes Linux portable and lint/security at inspection; both native macOS jobs are
queued. Native acceptance of this pending consumer batch remains outstanding.

## Shared 0.4.0 adoption for pending 0.33.5

On released base `e32ef7e9dcf023cfacf53a0a7a5bbdfb4e69d226`, the reviewed
85-file selection now records Shared Tooling 0.4.0
`b2a77f4b0454bc8b00003a2053f9d9ec2cbcc286`
([#57](https://github.com/dragginzgame/ic-testkit/issues/57),
[Shared #118](https://github.com/dragginzgame/shared-tooling/issues/118)).
Both exports used a clean isolated checkout with the canonical source remote.
Preview required no companion additions; the final preview preserved the real
index and manifest digests before explicit refresh. The final source adds the
package-impact clarification to the initial `e0acfdb` installation batch;
installer, Make, hook, fixture and CI bytes are identical between those sources.

Explicit `make install-rust-tools` prepares the three existing pins in verified
slots and selects `.tools/rust/common/bin`. An offline setup retry preserves the
same complete set and slot receipts. Digests confirm that the retired in-place
executables and Cargo receipts remain unchanged. Missing common-set admission
was observed before setup; no old-path fallback or automatic retirement runs.
CI PATHs, interactive guidance and the actual consumer hook fixture use the new
path. The consumer failure collector now retains the complete physical Rust
tree independently of compact host/IC evidence.

Focused Linux checks pass on Bash 5 and genuine Bash 3.2.57: the shared Rust
fixture exercises receipt/byte corruption, partial upgrades, held locks,
redirected paths and failure retention with substitute Cargo; actual consumer
hooks exercise formatting, lockfile/index preservation and rejected staging.
Consumer archive qualification retains failed attempts, admitted receipts and
common links, preserves compact host evidence, and skips redirected Rust roots
without traversing external targets. Snapshot, prepared tools, declaration pins,
formatting, selected ShellCheck and actionlint checks pass. The missing owner CLI
was rebuilt offline with `--locked` after explicit cache preparation; its check
reuses the prepared PocketIC 16.1.0. The maintainer's existing Host 0.13.1 lock
selection and all package metadata remain unchanged by this adoption.

Evidence uses `/tmp/ic-testkit-shared040-{preview-new-rules,refresh-new-rules,
install-rust,reuse,rust-fixture,rust-fixture-bash32,hooks,hooks-bash32,evidence,
evidence-bash32,checks,tools,pins-format,shellcheck,actionlint,fetch,build-cli}.log`;
old executables/receipts and current owning inputs are separately digest-recorded.
Pending 0.33.5 follows the updated package-impact rules: contributor setup changes
do not change the library's published contract. Native Intel/Apple Silicon macOS
and complete hosted acceptance remain outstanding; Linux Bash 3.2 evidence is
not native qualification. No broad gate, sibling edit or release ran.

## Hosted acceptance readback for released 0.33.3

Exact-source [Shared 0.3.12 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38123325760)
(`52e846369c39a1da371eac47ff97d94da925217e`) and
[Host 0.13.0 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38122069573)
(`7c58c92c72789139ec460475c429f9090581008f`) now pass Linux and both native
macOS architectures. Reviewed Shared portable logs pass physical/aliased tool
roots, nested release-command admission, hook preservation and the strengthened
diagnostic runner regression. This is upstream acceptance, distinct from pending
consumer bytes.

On Testkit `464eb3847d8443511ed3e95fda6ac39d20695c45`,
[Linux portable tooling](https://github.com/dragginzgame/ic-testkit/actions/runs/38124211113/job/114425061330)
and [Apple Silicon portable tooling](https://github.com/dragginzgame/ic-testkit/actions/runs/38124211113/job/114425061162)
pass actual hook checks and hosted archive download/readback. Their downloaded
artifacts are 11696398408 and 11696912429; verification completes successfully
after checking bytes, modes, links and outcomes. All four Linux jobs pass. Seven
other macOS jobs remain queued without assigned runners at inspection, including
Intel portable tooling. The workflow is not yet fully accepted, and these logs
do not qualify the pending 0.33.4 release/publication fixture changes below.

## Consumer release temporary paths for pending 0.33.4

On released base `464eb3847d8443511ed3e95fda6ac39d20695c45`, with the pending
Host guidance corrections present, the unmodified consumer release guard fails
from a caller-relative symlinked TMPDIR. Its first valid installation check
changes directory, then the version reader cannot create
`alias/workspace-version.XXXXXX`. The failed guard and direct child reproduction
are retained at `/tmp/ic-testkit-0334-before-{guards,guard-child}.log`; original
scratch remains at `/tmp/ic-testkit-0334-temp-paths/physical/tmp.u3rg9AREEw/`.
Source review identifies the same relative scratch/PATH/trace ownership in the
metadata and publication fixtures; no live publication reproduction was attempted
([#56](https://github.com/dragginzgame/ic-testkit/issues/56)).

All three consumer fixtures now physically anchor their scratch roots and export
that parent as TMPDIR before child execution. Actual release-guard suites pass
from the relative alias on Bash 5 and genuine Bash 3.2.57. The guard's explicit
`/bin/bash` metadata child uses this Linux host's Bash 5; a separate actual
metadata invocation passes on Bash 3.2 with a caller-relative physical TMPDIR.
Nested env-selected Bash calls use the matching prepared Bash 3.2 shim. Actual
publication suites pass with an absolute parent on Bash 5 and the relative alias
on Bash 3.2. Git, registry, installation and release effects remain substitutes;
real Cargo metadata is offline and no product compilation or registry resolution
is performed. Successful fixtures clean their own scratch.

Twelve controlled copies inject premature success or status 23 after path
selection, then change directory. Both shells preserve the required nonzero
status and original sentinel bytes in physically identified failure directories.
These injections qualify cleanup boundaries, separately from normal full fixture
execution. Logs use `/tmp/ic-testkit-0334-{guards-alias,guards-bash32-alias,
metadata-bash32-relative,publish-absolute,publish-bash32-alias,retention,
shellcheck}.log`. Selected ShellCheck and diff checks pass. Shared 0.3.12 and
Host 0.13 selections, owning manifests/lock, tool receipts and server installations
are unchanged. The compatible pending release remains 0.33.4. Local Linux Bash
3.2 is not native macOS acceptance; no broad gate, sibling edit or release ran.

## Host 0.13 qualification on released 0.33.3

Released Testkit source `464eb3847d8443511ed3e95fda6ac39d20695c45` already
selects all four Host crates at 0.13.0. Reviewed clean upstream source is
`7c58c92c72789139ec460475c429f9090581008f`. Its Rust delta from 0.12.10 removes
only the pathname progress-lock wrapper; Testkit already composes descriptor
admission and waiting with its own reporting policy. The root public re-exports
expose the removed Host API and distinct dependency identities. This was a
breaking change shipped in a patch release, rather than the required minor
boundary. The corrective upgrade guidance documents the existing cut without
rewriting published history or introducing a compatibility alias
([#54](https://github.com/dragginzgame/ic-testkit/issues/54)).

Explicit locked cache preparation, offline all-target compilation and actual
Rust 1.88 library/CLI compilation pass. The selected dependency tree contains
only 0.13.0 for all four Host packages. Focused Linux execution passes ten cache
filesystem tests, two Host adapter tests, the observed cache-lock heartbeat,
cross-process exact-artifact exclusion, six CLI provisioning/recovery tests and
nine synthetic command-runner tests. The rebuilt owner CLI admits the retained
official PocketIC 16.1.0 bundle with PATH unavailable, then its real command-scoped
subprocess test and all nine live concurrency cases pass. The first live CLI
attempt is retained separately: sandbox loopback binding was denied; the same
test passes with authorized socket access. Existing server bytes are reused;
no new server installation or download is performed.

Logs are `/tmp/ic-testkit-host013-{fetch,tree,check,msrv,locks,host,heartbeat,
process-locks,provisioning,runner,cli-build,cli-check,live-cli,
live-cli-authorized,live-concurrency}.log`. Manifest and lock SHA-256 values
remain `5566911f4331a32809337fd8b1bc87f0fc648fc277fb81a5885100ee5d8be940`
and `8b98e5624394363f9c3c7414f7093ecac0ad553d26da2cd5cfbcbbee89e1bf0c`.
These Linux results do not establish native macOS acceptance. At inspection,
[Testkit's matching run](https://github.com/dragginzgame/ic-testkit/actions/runs/38124211113)
and [Host's matching run](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38122069573)
are queued; earlier Host 0.12 results do not qualify 0.13. Shared 0.3.12 remains
the latest reviewed snapshot, with no selected-file changes in this batch.
This Host qualification slice corrects documentation only; no source API,
retained format, dependency selection, package version, sibling file or release
effect changes.

## Shared 0.3.12 and consumer hook temp paths for pending 0.33.3

Reviewed clean committed Shared source is
`52e846369c39a1da371eac47ff97d94da925217e` (0.3.12). The canonical isolated
export retains the same 85-file selection; selected shared file bytes are
unchanged and the snapshot records the new revision/version. The changed
producer formatting-adoption checker and hook regression are not selected in
this consumer's roster, so no unused duplicate fixture/helper is added.

The consumer-owned `check-git-hooks.sh` reproduces the same inherited relative
TMPDIR defect: after entering its private checkout, the real hook cannot create
`alias/pre-commit-format.XXXXXX`. The original attempt and scratch remain at
`/tmp/ic-testkit-shared0312-before-hooks.log` and
`/tmp/ic-testkit-shared0312-paths/physical/ic-testkit-git-hooks.ylYzXd`.
The local correction exports the physical fixture parent before child hooks run,
without changing the caller's selected location, formatting targets or checks
([#55](https://github.com/dragginzgame/ic-testkit/issues/55),
[Shared #116](https://github.com/dragginzgame/shared-tooling/issues/116)).

Focused Linux checks pass against Testkit's actual formatter targets with
absolute and relative TMPDIR on Bash 5 and a relative symlink alias on genuine
Bash 3.2. Existing partial-staging, missing/wrong formatter, failed formatter,
lock/source/index preservation and activation checks remain intact. The unmodified
committed upstream hook suite also passes under both Bash profiles, including
absolute/relative/alias adoption and failed prerequisite retention under the
physical parent. Its initial formatting effects are substitutes; its adoption
checks use real prepared formatting tools. No product Cargo build or dependency
resolution is performed. Snapshot, declaration pins, formatting and selected
ShellCheck pass. Logs use `/tmp/ic-testkit-shared0312-{export,hooks,hooks-bash32,
hooks-absolute,upstream-hooks,upstream-hooks-bash32,checks,shellcheck}.log`.

[Matching Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38123325760)
is queued at inspection. Local Bash 3.2 is not native macOS acceptance of this
pending consumer source. Both changelogs retain compatible 0.33.3. Incoming Host
manifest/lock changes retain the exact hashes recorded in the preceding 0.3.11
proof below and remain separate, uncompiled work. No tool installation, broad
gate, package-version bump, sibling edit, release or CI dispatch ran here.

## Shared 0.3.11 identity refresh for pending 0.33.3

The clean committed source `db143ae9ae140eb66826c413360723d23c078f66`
corrects Shared's canonical VERSION to 0.3.11
([Shared #114](https://github.com/dragginzgame/shared-tooling/issues/114)).
An isolated canonical export preserves the same 85-file selection. All selected
file bytes and modes are unchanged from released Testkit 0.33.2's prior Shared
revision; only the snapshot revision/version annotation changes. The existing
portable path fixes and derive-macro guidance remain intact. Dirty upstream
hook/checker edits are excluded from this committed-source adoption.

The strengthened, unmodified producer validation-runner fixture passes on Linux
under Bash 5 and genuine Bash 3.2, including nested interpreter selection. It
separates each diagnostic beyond neighboring context windows and exercises rg
and grep summaries against byte-identical selected runner code. It is not added
to the consumer roster. Snapshot, declaration-pin and non-mutating formatting
checks pass. Logs are `/tmp/ic-testkit-shared0311-{export,runner,runner-bash32,
checks}.log`. [Matching upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38121526696)
is in progress at inspection; local Bash 3.2 is not native macOS proof.

Incoming Host 0.13 manifest/lock changes remain untouched and are not qualified
by this Shared-only refresh. Their SHA-256 values are respectively
`bc2bc804c933d2cc751c87218db67d4c43b889a78652db734d6bff6ae5bd2e33`
and `701b8f248f69b16ce19146225081a51c6fb3fcb6c0a6634419bf208fe1ba6375`.
This Shared identity update is compatible and requires no API/format/reset change;
both changelogs select pending 0.33.3. No Cargo compilation, dependency fetch,
broad gate, tool installation, source-version bump, sibling edit or release ran.

## Shared temporary-path fixes for pending 0.33.2

The reviewed committed pending 0.3.10 batch is
`37af29b2f8fb4532744b600c5f85ce716a8b57cf`. Its committed `VERSION` remains
0.3.9, so the canonical export preserves that version annotation and binds the
exact new revision in the unchanged 85-file snapshot. This is committed-source
adoption, not a claim that 0.3.10 has been released. An isolated clean source
export updates the selected host fixture, release-command checker and shared
formatter guidance. Tool pins and production formatter behavior are unchanged.
Testkit has no dollar-prefixed derive/cfg_attr paths or macro definitions needing
the documented scoped protection; the producer-only frontend fixture is not
added or run in this consumer.

Focused Linux qualification passes under Bash 5 and genuine Bash 3.2, including
nested interpreter selection. The actual selected host fixture runs with an
aliased TMPDIR, covering physical diagnostic paths, substituted installation,
negative admission and the actual evidence collector. The actual release-command
checker runs against Testkit's real Make inputs with relative TMPDIR on current
Bash and a relative alias on Bash 3.2. The unmodified committed producer command
fixture also passes under both profiles, exercising aliased/relative roots,
nested Make and broken-adapter refusal with retained failure logs against
byte-identical selected checker code. All downloads and release effects in
these checks are substitutes; no actual setup, commit, tag or push occurs.
Snapshot, declaration pins, formatting and selected ShellCheck pass. Logs use
`/tmp/ic-testkit-0332-shared0310-{export,host,host-bash32,commands,
commands-bash32,command-fixtures,command-fixtures-bash32,checks,shellcheck}.log`.

At inspection [exact-source Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38120869060)
is queued. Bash 3.2 on Linux and simulated Darwin selections are not native
macOS proof. Existing Host selection and the pending archive-verification repair
remain byte-identical to the earlier proof below; their checks are not rerun
merely for this snapshot update. Both changelogs keep compatible pending 0.33.2.
No Cargo compilation, broad gate, package-version change, sibling edit or release
ran in this adoption slice. Feedback remains with
[Shared #111](https://github.com/dragginzgame/shared-tooling/issues/111) and
[Shared #112](https://github.com/dragginzgame/shared-tooling/issues/112).

## Shared 0.3.9, Host 0.12.9 and evidence verification for pending 0.33.2

Reviewed Shared source is `1ed2ca523186641e0e0cdf77e3cb10814c22a430` (0.3.9),
adopted through an isolated clean canonical export with the unchanged 85-file
roster. The selected Rust installer and fixture use the consumer working directory
for preflight, installation and offline probes, while retaining explicit toolchain
overrides and documented catalog/lock path resolution. The validation runner
shares its live/retained diagnostic predicate. The unmodified committed upstream
runner fixture exercises identical adopted runner bytes under both Bash profiles,
including rg and grep selection, early diagnostic families and neutral context.
No unused producer regression suite or coordinated-archive adapter was added;
Testkit has one published package and does not need that multi-crate archive route.

The incoming maintainer lock selects all four Host crates at 0.12.9 and
serde_json 1.0.152. Reviewed Host source is
`8cc85c53cdd59701a7352f566425b3faf195a37b`; crate Rust source is unchanged
from the previous selected 0.12.7. Dirty sibling adoption work is excluded.
Lock SHA-256 remains `8782ca01e0af8c6e92dd7b943ec100245e172589d9d04fadc136764ba7cfae23`.
Explicit locked fetch, offline all-target compilation and actual Rust 1.88
library/CLI compilation pass without dependency resolution or metadata changes.
Logs are `/tmp/ic-testkit-0332-host0129-{fetch,check,msrv}.log`.

The actual released archive verifier reproduces [#53](https://github.com/dragginzgame/ic-testkit/issues/53):
it prints success, then returns 1 because its verify branch lacks completion.
Pre-fix proof/archive/logs remain at `/tmp/ic-testkit-0332-before-*`.
The local repair marks successful readback complete and explicitly fails archive
comparisons on Bash 3.2. Normal and separate readback entry points now pass under
Bash 5 and genuine Bash 3.2. Corrupted payload and outcome archives fail with
status 1 and retain extracted original bytes. Successful readback removes only
its scratch; no-overwrite, links, permissions, selection and failed partial
archive retention remain covered. An inherited CDPATH and aliased TMPDIR case
also passes on Bash 3.2. These are local round trips, not hosted artifact readback.

Focused Linux Rust-tool fixtures pass under both Bash profiles with substitute
Cargo effects. The committed runner fixture also passes under both profiles;
selected ShellCheck, snapshot, dependency pins, formatting and prepared offline
Rust-tool admission pass. Logs use `/tmp/ic-testkit-0332-{shared039-export,
archive,archive-bash32,archive-alias,verify,verify-bash32,rust,rust-bash32,runner,
runner-bash32,shellcheck,tooling}.log`. No real tool installation, sibling edit,
broad gate, release, CI dispatch or package-version bump ran. Both changelogs
select compatible 0.33.2; APIs and retained formats require no migration/reset.

Matching [Shared 0.3.9 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38075978465)
and [Host 0.12.9 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38073002651)
pass Linux and both native macOS architectures. Those upstream results do not
qualify this pending consumer source. The delivered
[Testkit 0.33.1 run](https://github.com/dragginzgame/ic-testkit/actions/runs/38066303813)
passes all native complete checks, MSRV and real concurrency jobs, plus portable
setup/operator and archive-creation qualification. All three portable jobs then
fail the separate readback completion branch repaired here. Complete native-check
logs explicitly pass transport/diagnostic units and all six synthetic teardown
parent cases with subprocesses, including actual OS ConnectionReset query and
creation requests. This qualifies the retained transport fix on 0.33.1; it does
not relabel the entire failed workflow or earlier revisions as passing.

## Shared 0.3.7 and Host 0.12.6 for pending 0.33.1

The unchanged 85-file canonical roster selects clean committed Shared
`34e5ad7aac3599306c9572bb547f2239d09df1a3` (0.3.7), reviewed in an isolated
export source. Mandatory portable assertions explicitly fail on Bash 3.2;
selected Cargo-tool admission rejects absent host identities and non-executable
candidates before activation. The incoming consumer workflow already uses
workflow/ref concurrency and cancels older queued or running checks. Actionlint
and a structural comparison confirm its existing jobs, hosts, triggers and
permissions are preserved.

Actual selected Rust, host, IC and release-runner fixtures pass under Bash 5 and
genuine Bash 3.2, including nested interpreter selection. Host/IC suites also
exercise the actual evidence collector. Cargo/release effects, downloads and host
selections are substitutes. A disposable selected Rust-fixture copy contradicts
the successful three-install count: both interpreters return 1 before later
reuse checks and retain the successful-install evidence. Selected ShellCheck,
snapshot, declaration pins, formatting and offline prepared Rust-tool admission
pass. Logs use `/tmp/ic-testkit-0331-shared037-{export,rust,rust-bash32,host,
host-bash32,ic,ic-bash32,runner,runner-bash32,assertion-current,assertion-bash32,
workflow,workflow-structure,shellcheck,tooling}.log`.

All four incoming Host selections are 0.12.6, reviewed against release source
`5f356effea97fbc31dfcca5b9f1b2834f35325a9`. Rust source is unchanged from
0.12.5; this release changes CI tooling only. The incoming lock remains unchanged
with SHA-256 `05a94c890ae6d2e75afbac50923ba31f3489c671dc042fd18409327b6f27e26b`.
Explicit locked fetch, locked offline all-target compilation, Rust 1.88 library/CLI
compilation and the invalid-destination atomic-publication test pass. Logs use
`/tmp/ic-testkit-0331-host0126-{fetch,check,msrv,invalid-path}.log`.
Earlier proof below retains its original input identity.

Exact-source [Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/38064027998),
[Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38063794538)
and [Testkit 0.33.0 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38060751752)
remain queued at inspection. Linux Bash 3.2 is not native macOS qualification;
cancelled earlier checks do not qualify unfinished hosts. Both changelogs retain
compatible pending 0.33.1. No public API or retained-format change, version bump,
broad gate, sibling edit, installation, release or CI rerun was performed here.
Relevant feedback is under [Shared #107](https://github.com/dragginzgame/shared-tooling/issues/107)
and [Shared #108](https://github.com/dragginzgame/shared-tooling/issues/108).

## Host 0.12.5 for pending 0.33.1

On released Testkit 0.33.0 `22d07d2c0a186ffe7dfa48d51ddc243a56c07dda`,
the maintainer's incoming lock selects all four Host 0.12.5 crates. Reviewed
Host source is `85f051c60b2a6f37717c4274b1e31caf5b9d3453`. Its runtime delta
rejects NUL publication paths before creating parents; the other three crates'
Rust source and public APIs are unchanged. Testkit already delegates durable
publication to Host, so no local path-admission shim is needed. The incoming lock
is preserved byte-for-byte with SHA-256
`c510fef5d316a76759c92c938610ec97ec32a364e42fb95d54f1c222bca3af2e`;
explicit locked fetch prepares it without resolution changes. Package metadata
remains 0.33.0.

Focused Linux qualification passes: all twelve digest/copy cases and 28 artifact
transaction cases, then the expanded atomic-copy rejection case covering NUL in
a filename and parent component alongside directory-required suffixes. It
verifies retained source/destination context, Host's before-publication
`InvalidInput` cause without cleanup errors, unchanged original output and no
new parent/staging entries. Locked all-target compilation, Rust 1.88 library/CLI
compilation and strict Clippy including test callers pass. The final expanded
case also compiles on the current toolchain after the earlier MSRV check;
fresh MSRV execution of that test is not claimed. Formatting and declaration
pin checks pass. Logs are
`/tmp/ic-testkit-0331-host0125-{fetch,digest,transaction,check,msrv,invalid-path,
clippy,tooling}.log`.

[Host 0.12.5 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38061390080)
remains queued at inspection. No broad consumer gate, sibling edit, tool setup,
release or CI rerun occurred. Native macOS and delivered-source acceptance are
separate; consumer feedback is recorded under
[host #54](https://github.com/dragginzgame/ic-host-tooling/issues/54).

## Shared 0.3.6 for pending 0.33.1

Based on released Testkit 0.33.0 `22d07d2c0a186ffe7dfa48d51ddc243a56c07dda`,
an isolated clean canonical export adopts Shared 0.3.6
`0604bfd730ec7ec288cd2cfdad217a0d42bf256b` with the unchanged 85-file roster.
The selected hook admits failed Git observations before comparing tree IDs;
the selected Rust installer and its fixture add explicit lockfile selection.
Producer-only hook/checker/runner fixtures are not added to Testkit's roster.
The adopted baseline remains unchanged. Package metadata, lock and executable
pins stay unchanged; lock SHA-256 is
`dd1913627a5d0428337eef61da6cc8783febd6f6b4d7c289e5a3904034789963`.

Focused Linux checks pass under Bash 5 and genuine Bash 3.2, including nested
Bash selection through a PATH shim: the selected Rust installer fixture, the
committed upstream hook suite against identical selected hook bytes, and the
actual consumer hook qualifier against Testkit's formatting targets. Producer
hook cases inject failures at all three real Git tree observations, printing
the matching ID then returning 23; status, staging and working bytes remain
preserved, with later dispatch refused. Consumer cases verify actual formatter
discovery, repeated formatting, partial-staging refusal, missing/wrong formatter
admission, installation selection and lock preservation. Cargo installation in
the Rust fixture is substituted; lock ambiguity/source/version rejection,
offline reuse, changed selection and retained failures exercise the real helper.
No actual executable setup or Cargo compilation is performed. ShellCheck,
snapshot, declaration pins, formatting and offline Rust-tool admission pass.

Logs are `/tmp/ic-testkit-0331-{export,rust,rust-bash32,shared-hooks,
shared-hooks-bash32,hooks,hooks-bash32,shellcheck,tooling}.log`.
No broad consumer gate, sibling edit, installation, release or CI rerun occurred.
At inspection exact-source
[released Testkit 0.33.0](https://github.com/dragginzgame/ic-testkit/actions/runs/38060751752)
and [Shared 0.3.6](https://github.com/dragginzgame/shared-tooling/actions/runs/38061078001)
CI remain queued. Linux Bash 3.2 is not native macOS qualification. Adoption
feedback remains under
[shared #106](https://github.com/dragginzgame/shared-tooling/issues/106) and
[shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).

## Incoming Host 0.12.4 selection for pending 0.33.0

The maintainer's incoming lock selects all four Host 0.12.4 crates, reviewed
against `5400f159474cebac1ec7ae7c8763abfd258bde03`. Their Rust source is unchanged
from 0.12.3; the upstream release adopts Binaryen 133 development tooling, already
selected by Testkit. The incoming lock is preserved byte-for-byte with SHA-256
`d130c63519487e6ad35364983127599e9975d36d93dcd3852162e4fa203cc3d9`.
Explicit locked fetch prepares that graph without dependency resolution changes.
The earlier 0.33.0 proof below retains its original 0.12.3 graph identity.

Pending #52 is now freshly qualified against 0.12.4: ten transport unit cases,
the diagnostic-capture case, all six synthetic socket parent cases and their
explicit child probes pass on Linux. Strict scoped Clippy and Rust 1.88
library/integration-fixture compilation pass. Logs are
`/tmp/ic-testkit-0330-host0124-{fetch,transport,diagnostics,teardown,clippy,msrv}.log`.
Manifest versions remain unchanged. No broad gate, sibling edit, setup, release
or CI rerun occurred. Native delivered-source acceptance stays in
[#52](https://github.com/dragginzgame/ic-testkit/issues/52).

## Connection-reset recovery for pending 0.33.0

Based on released 0.32.2 `9e697b6363441e3fa96477adae5b07820461c3c5`,
[#52](https://github.com/dragginzgame/ic-testkit/issues/52) recognizes the scoped
reqwest/hyper OS `ConnectionReset` shape observed in
[0.30.0 Apple Silicon portable CI](https://github.com/dragginzgame/ic-testkit/actions/runs/38043841501/job/114189268109).
This changes public dead-transport recovery semantics, so both pending release
views select 0.33.0. Package metadata and dependency selections remain unchanged;
lock SHA-256 is
`3e56b63d86c526370cfe836ffd8a151ac5d8c5b528431c8803d022bfb6850f00`.
No retained-format or installation reset is required.

Focused Linux qualification passes: all ten transport unit cases and all five
synthetic transport/teardown parent cases (the three subprocess-only probes stay
ignored in the parent list and are invoked explicitly). The new actual reset
case fully reads an accepted canister-creation request, selects zero-time
`SO_LINGER`, then closes the socket before any response. The observed reqwest
error contains Linux `ConnectionReset` code 104; standalone ownership and the
original transport cause survive failed installation. Exact observed macOS code
54 and Linux code 104 messages qualify through maintained panic contexts,
installation/snapshot causes and contextual I/O wrappers. Bare I/O, quoted
application errors, non-instance URLs, invalid OS codes and other error kinds
remain excluded. Strict scoped Clippy and Rust 1.88 library/integration-fixture
compilation pass, as do formatting and diff checks.

Logs are `/tmp/ic-testkit-0330-{transport-final,teardown-native,clippy,msrv,
format}.log`. The initial unit suffix-matching failure remains in
`transport.log`; the corrected cases pass in `transport-final.log`. Initial
sandbox TCP-listener denial remains in `teardown.log`; the actual loopback suite
passes in `teardown-native.log` with loopback access enabled. No broad local gate,
sibling edit, installation, release or CI rerun occurred. Linux socket execution
and captured macOS-message tests do not replace native macOS execution against
the delivered source; #52 retains that acceptance obligation.

Further pending 0.33.0 qualification extends the same reset fixture to a real
query request. Linux logs contain actual `ConnectionReset` errors for both query
and installation; all six parent cases pass with their subprocess probes.
The focused diagnostic-capture case also passes for the macOS reset shape,
preserving `InstanceUnavailable` through I/O wrapping. Strict scoped Clippy,
Rust 1.88 library/fixture compilation, formatting and diff checks pass again for
the changed callers. Logs are
`/tmp/ic-testkit-0330-reset-{call,diagnostics,clippy,msrv,format}.log`.
The earlier five-parent-case record remains the original installation-only proof;
no additional production recovery path or dependency selection is introduced.
Native macOS execution and delivered-source acceptance remain required under #52.

## Shared 0.3.4 and Host 0.12.3 for pending 0.32.2

The pending batch now selects canonical Shared 0.3.4
`169d77b8440568c5200eede971625126181f7bb2`, retaining the 85-file roster and
unchanged Shared 0.3.3 runner repairs qualified below. The new selected changes
are the Binaryen 133 common pins and their setup/host guidance. No producer-only
Node qualification infrastructure is added to Testkit. All four incoming Host
lock selections are 0.12.3, reviewed against
`aec863191b3c96ef879e59af701cbe1451595f25`; Host Rust source is unchanged from
0.12.2. The incoming lock remains byte-for-byte unchanged with SHA-256
`231311b75e16093315dc2bc09ddd995f376140faaa76a81e078b0332aac7be32`.
Locked fetch explicitly prepares that graph; locked offline all-target compilation
passes in this workspace's `target/`. Package metadata remains 0.32.1.

Actual Linux `make install-ic-tools` prepares the authenticated five-tool bundle
with Binaryen 133. Offline admission and repeated setup pass without reselection.
The earlier Binaryen 132 bundle `ic-set.xJTbQZ` remains retained: its payload
receipt verifies and its wasm-opt SHA-256 remains the pre-setup value
`1014958e6f20d412f1542320b43970214b0fb1ed780595e8f7c0d8761ed53725`.
Selected installer fixtures pass under Bash 5 and genuine Bash 3.2 with nested
Bash invocations using the same shell. Snapshot, pins and formatting checks pass.

The unmodified committed upstream Wasm qualifier also passes on Linux in the
isolated Shared clone, using a private copy of the prepared bundle with matching
wasm-opt SHA-256
`8f25e9fd5db0fc5f210003aaa432922feb2e52d309e430def2f929e34da9466b`.
Node 24.21.0 executes reference and `-O3`, `-Os`, `-Oz` outputs, checking integer
boundaries, imports/exports and an IC-style reply. This synthetic common-tool
smoke does not qualify a product canister deployment. Testkit owns no production
optimizer policy; downstream projects must qualify their own optimized Wasm.

Logs are `/tmp/ic-testkit-0322-shared034-{export,install,tools-check,reuse,
retained,ic,ic-bash32,wasm,tooling}.log` and
`/tmp/ic-testkit-0322-host0123-{fetch,check}.log`. Synthetic Wasm inputs, outputs
and source/tool identity remain in
`/tmp/ic-testkit-0322-shared034-wasm-evidence/`. No sibling edit, broad consumer
gate, release or CI rerun occurred. At inspection exact-source
[Shared 0.3.4](https://github.com/dragginzgame/shared-tooling/actions/runs/38054347275),
[Host 0.12.3](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38054067633)
and [released Testkit 0.32.1](https://github.com/dragginzgame/ic-testkit/actions/runs/38053227144)
CI remain queued. Native macOS and full consumer acceptance remain outstanding;
[shared #102](https://github.com/dragginzgame/shared-tooling/issues/102) owns the
common Binaryen adoption feedback.

## Shared 0.3.3 for pending 0.32.2

Based on released Testkit 0.32.1 `ff5da12c27bf0835586892ada8e443117229290b`,
the isolated canonical export selects Shared 0.3.3
`d63f0cfaba8ab2961d6012064adbf051c1898bc1`, retaining the 85-file roster.
Only the selected validation runner and snapshot record change. Later dirty
upstream wasm-opt/tool-pin work is excluded. Package version and dependency
graph remain unchanged; incoming lock SHA-256 is
`67a8ed3416cc3c5a186a85cecb127b51a81e42fa9cef54633a0545e2406f262f`.
The runner refuses malformed nesting depth before target logs/dispatch and
requires explicit source-wrapper/body completion, preserving available evidence
after premature exits and the original status after completed target failures.

Focused Linux qualification passes: the committed upstream runner fixture
projected onto exact selected consumer runner/guard bytes under Bash 5 and genuine
Bash 3.2. A PATH shim selects Bash 3.2 for nested Bash invocations too. Cases cover
canonical depth/increment controls, malformed/overflow/arithmetic-shaped values,
24 fault-injected exits across wrapper/body/summary boundaries and both shells,
unsafe Make modes, nested context/jobserver routing, source mutation, fail-fast,
timings and retained combined failure evidence. Fixture targets use controlled
effects; no complete consumer validation gate is dispatched. The actual owning
runner also dispatches harmless `help` and refuses depth `1/0` with status 2
before creating target logs under both shells. ShellCheck, snapshot, pins,
formatting and diff checks pass.

Logs use `/tmp/ic-testkit-0322-{export,runner,runner-bash32,consumer,
consumer-invalid,consumer-bash32,consumer-invalid-bash32,shellcheck,tooling}.log`.
Actual help-run logs are retained beneath `/tmp/ic-testkit-0322-help-logs` and
`/tmp/ic-testkit-0322-help-bash32-logs`. No Cargo compilation, broad local gate,
tool/server setup, sibling edit, release or CI rerun occurred. At inspection
[released Testkit 0.32.1](https://github.com/dragginzgame/ic-testkit/actions/runs/38053227144)
and [Shared 0.3.3](https://github.com/dragginzgame/shared-tooling/actions/runs/38052409053)
CI remained queued. These focused passes do not qualify native/full delivery
of this pending consumer source.

## Shared 0.3.2 for pending 0.32.1

Based on released Testkit 0.32.0 `1502f667bc6b7f54067dd57d1841cd0a150110a7`,
the isolated canonical export selects Shared 0.3.2
`c16444bf006f17c5bb4dda5ad070a0f345da9623`. The roster grows from 84 to 85
only to include the advisory README task linked by the updated catalog; no
schedule or gate is activated. Later dirty upstream runner work is excluded.
Released Testkit already selects all four Host 0.12.2 crates, reviewed against
`e1ef99e6a4c6d05f0b0d8364f8586c6cc358dadc`; their Rust source remains unchanged.
The manifest version and dependency graph remain unchanged. Lock SHA-256 is
`b0712fdc72397ff49e948fa55cd3f9c4a4acfc5946e8999b1863d165a3bf8be6`.
A scoped update produced unrelated Windows dependency-edge churn, which was
reverted to the exact incoming lock before qualification; locked fetch prepared
that graph without changing selection.

Shared jobserver and explicit-completion repairs are propagated to selected
snapshot files. Five consumer-owned release, metadata, publication, hook and
evidence fixtures also require completion before cleanup/success. A disposable
copy of released metadata qualification, injected with an unset-variable read
immediately after its trap, printed an expansion error but returned zero under
Bash 3.2. Original source/log remain in `/tmp/ic-testkit-0321-before/` and
`/tmp/ic-testkit-0321-before.log`. Sixty injected cases now pass across Bash 5
and genuine Bash 3.2: five actual fixture copies, six termination/completion
cases, preserved failure inputs and cleanup only on genuine success. Injection
stops before fixture bodies, without installation/release effects.

Normal consumer release guards/metadata, hook, publication and evidence checks
pass under both shells, as do selected host/IC/Rust installer fixtures. Committed
upstream command, formatting and retention fixtures projected onto selected
consumer bytes pass under both shells. Installer/Cargo/release effects in those
fixtures are substituted; retained failure/archive cases exercise their actual
owners. Locked all-target compilation passes. Actual parallel common Rust and
formatting checks pass without jobserver warnings. The consumer preflight recipe
also forwards jobserver descriptors; final release guards pass under both shells
without closed-descriptor warnings. Cargo still reports that its separately
configured job count is ignored when an external jobserver is present; this is
distinct from invalid descriptor inheritance. ShellCheck, snapshot integrity,
dependency pins, formatting and diff checks pass. Logs use
`/tmp/ic-testkit-0321-{export-final,fetch,cargo,commands,commands-bash32,format,
format-bash32,retention,retention-bash32,injection,injection-bash32,guards-final,
guards-bash32-final,hooks,hooks-bash32,publish,publish-bash32,evidence,evidence-bash32,
host,host-bash32,ic,ic-bash32,rust,rust-bash32,shellcheck,tooling}.log`.
Initial passing release-guard logs with the old preflight descriptor warnings
remain in `guards.log` and `guards-bash32.log` with the same prefix.
No broad local gate, real tool/server setup, sibling edit, release or CI rerun
occurred. At inspection exact-source CI remained queued for
[Testkit 0.32](https://github.com/dragginzgame/ic-testkit/actions/runs/38051695584),
[Shared 0.3.2](https://github.com/dragginzgame/shared-tooling/actions/runs/38051446835)
and [Host 0.12.2](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38051203749).
Native/full acceptance of this consumer batch remains under
[#51](https://github.com/dragginzgame/ic-testkit/issues/51).

## Shared 0.3.1 and Host 0.12.1 for pending 0.32.0

Based on released Testkit 0.31.0 `f1ae9e6d3b0f3f20ec1e1f1b49c8b1dea3155e0a`,
the isolated canonical export selects Shared 0.3.1
`fa452afaa5012866eb1c20820dfa8038c106e7ec`, preserving the 84-file roster and
consumer pin catalogs. The incoming manifest requirements already select Host
0.12, and its lock already selects all four published 0.12.1 crates; both are
preserved. Reviewed Host source is `e5ecfa06c14d144cfeb85ea89d65906b1bf81636`.
Its Rust source is unchanged from 0.11; the public reexport dependency identity
change requires the 0.32 minor boundary. Manifest package version stays 0.31.0.
Lock SHA-256 is
`56e6a6e9a69fc1ca67ad2f2d0d9c1f18b5c567003baacf5743509bdfb8a88c7d`.
Later uncommitted sibling work is excluded.

Focused Linux qualification passes: locked all-target compilation, Rust 1.88
library/CLI compilation, and all seven CLI provisioning tests. Shared host, IC
and Rust installer fixtures, projected committed aggregate/preflight fixtures,
and actual consumer release guards pass under Bash 5 and genuine Bash 3.2.
The consumer's parallel-Make fixture now covers both preflight failure stops
before setup/product dispatch and unchanged retained server bytes. Projected
upstream fixtures use the exact selected installers/include/catalogs and prove
cold and retained-state preflight, missing/unavailable toolchain refusal and
unsupported-platform refusal. Installer effects remain substituted; these are
not real native installation or multi-hour Cargo build measurements.

Logs use `/tmp/ic-testkit-0320-{export,fetch,cargo-final,msrv,cli,host,host-bash32,
ic,ic-bash32,rust,rust-bash32,commands,commands-bash32,guards-final,guards-bash32,
shellcheck,tooling}.log`. Initial Cargo network-sandbox failure remains in
`cargo.log`; the consequent uncached offline metadata refusal remains in
`guards.log` and its retained fixture. Explicit locked fetch prepared the selected
cache before rerunning. Existing jobserver warnings remain a separate upstream
concern under [Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99).
No real tool/server setup, broad gate, sibling edit, release or CI rerun occurred.
At inspection, exact released-source runs were queued for
[Shared](https://github.com/dragginzgame/shared-tooling/actions/runs/38049622600),
[Host](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38050076519)
and [Testkit 0.31](https://github.com/dragginzgame/ic-testkit/actions/runs/38045250725).
Native/full qualification of this pending consumer source remains outstanding.

## Shared Tooling 0.3.0 for pending 0.31.0

Based on released Testkit 0.30.0 `6ac161b8ed689012bf8b0ce926946f94ea9f407d`,
the isolated canonical export selects committed Shared Tooling 0.3.0
`88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`, retaining the 84-file roster.
Uncommitted sibling changes are excluded. The complete common toolset contract
requires a minor release; both pending changelog views select 0.31.0 while the
maintainer-owned manifest version remains 0.30.0. Dependency selections and the
lockfile remain unchanged (SHA-256
`45dc20ba66d7188fa0552c29de229ee75cd318702ccc932c568be3476d73a919`).

The actual root Makefile registers PocketIC setup/check through Shared's ordered
product target lists. Explicit setup prepares host, IC and Cargo tools before
building the owning CLI and preparing PocketIC. Offline admission checks the same
order and refuses a missing CLI without compiling it. CI and release preflight
use that aggregate; source/candidate admission still precedes preflight setup.
Rust toolchain preparation remains explicit. There is no published Testkit CLI
self-installer, retained installation reset or stored-format change.

Focused Linux qualification passes under Bash 5 and genuine Bash 3.2: host
installer fixtures, the committed upstream aggregate fixture projected onto the
selected consumer include, actual consumer Make routing under `-j4`, release
guards/metadata qualification, and failure-evidence archive/retention fixtures.
Consumer fixtures substitute installers, Cargo and CLI effects; they prove
ordering, failure stops, setup-free admission and preservation of fixture server
bytes, rather than native installation behavior. ShellCheck, actionlint,
snapshot integrity, dependency pins, formatting and diff checks pass.

Logs use `/tmp/ic-testkit-0310-{host-tools,host-bash32,commands,commands-bash32,
guards-current,guards-bash32,evidence,evidence-bash32,shellcheck-final,
actionlint-final,checks}.log`. Initial fixture projection failures remain in
`guards.log` and `guards-final.log`; the initial workflow style failure remains
in `actionlint.log`, with the same prefix. No broad local gate, actual tool/server
installation, Cargo compilation, sibling edit, release or CI rerun was performed.
At inspection the released Testkit 0.30.0 run
[38043841501](https://github.com/dragginzgame/ic-testkit/actions/runs/38043841501)
and Shared 0.3.0 run
[38044218125](https://github.com/dragginzgame/shared-tooling/actions/runs/38044218125)
were queued. Native setup and full delivery qualification for this pending
consumer source remain outstanding; upstream and released-source results do not
qualify these local edits.

## Bounded diagnostics and Host probe capture for pending 0.30.0

The maintainer authorized the minor boundary for
[#49](https://github.com/dragginzgame/ic-testkit/issues/49), based on released
0.29.0 `e15cc2acfd9324f6877f854415005f91d169a031`. The reviewed proposal from
Host 0.11 was applied and qualified here, with local strict-Clippy corrections.
The pending Shared 0.2.14 work below is carried into the same 0.30.0 notes.
Manifest version, dependency selections and lock remain unchanged (SHA-256
`47cf2d3922416b3ad78233f68f82087e9d63527dc127a8bd011e8c20bf79463c`).

Host owns pipe capture and cleanup for Cargo builds, version/identity probes,
metadata and workspace discovery. Testkit keeps its command/environment/schema
policy and error projections. Cargo retains a 1 MiB raw prefix per stream;
every chunk is forwarded when requested, and failure strings mark truncation.
Lossy UTF-8 rendering can expand the retained raw prefix. Diagnostic volume
never terminates a build. All these paths have no elapsed-time deadline.
Complete probes are hard-bounded: identity/discovery streams and probe stderr
at 64 KiB, metadata stdout at 16 MiB. Current owning locked/offline metadata is
1,373,890 bytes (`/tmp/ic-testkit-0300-metadata.json`); this single graph is not
a universal workspace-size guarantee. Oversized responses cannot be parsed or
hashed from a prefix. Nonzero status, original spawn I/O causes and whole typed
Host failures remain distinguishable. Successful bounded identities and stored
v1 framing are unchanged; retained entries/installations need no reset.

Focused Linux qualification passes: 60 Wasm-cache tests, one workspace capture
unit test, four real Cargo workspace integration tests, and the benchmark
driver's selected tool/context test. Larger-than-cap successful/failing output
reaches observers intact, silent/observed failure prefixes are bounded/marked,
exact-limit identity/metadata/discovery responses pass, and over-limit complete
JSON/identity output is refused. Existing observer unwind, descendant cleanup,
cache publication and command selection cases still pass. Typed cleanup retention
follows whole-error projection; no cleanup syscall fault was injected locally.
No multi-hour build was dispatched; the no-deadline policy is explicit in the
invocation limits. Strict Clippy including test callers, Rust 1.88 native
library/CLI and Wasm library compilation, strict rustdoc, fmt/snapshot/pins and
diff checks pass. The final complete-probe and workspace unit runs cover the
small method-reference correction after initial strict Clippy failure.

Logs use `/tmp/ic-testkit-0300-{cache,wasm,workspace-unit,workspace-integration,
workspace-final,probe-final,driver,clippy-final,msrv,msrv-wasm,docs,tooling}.log`.
The initial Clippy failure remains in `clippy.log` with the same prefix.
No broad local gate, tool/server installation, sibling edit, package-version
change, release or CI rerun was performed. Released 0.29.0
[38041559012](https://github.com/dragginzgame/ic-testkit/actions/runs/38041559012)
has passing Linux checks/MSRV/portable/concurrency and ARM MSRV/concurrency,
with other native jobs queued/running at inspection. Those results do not
qualify this pending 0.30.0 source; native/full delivery checks remain outstanding.

## Shared Tooling 0.2.14 for pending 0.29.1

Based on released Testkit 0.29.0 `e15cc2acfd9324f6877f854415005f91d169a031`,
the clean isolated canonical export selects committed Shared 0.2.14
`fd11692f31e7dfd44dcc2ca56634eaeab3569825`, retaining 84 files. New dirty
upstream tools/evidence changes are excluded. CI inspection preserves partial
failed-step logs and fetch errors, propagates unsuccessful observations, and
refuses empty failure evidence unless a completed non-failing run permits it.
This changes tooling diagnostics, not a library API or stored format.

The committed upstream fixture is projected onto the exact selected consumer
gh-ci.sh bytes without adding it to the snapshot roster. Bash 5 and genuine
Bash 3.2 checks pass for command routing, failed/partial log retention,
unavailable and whitespace-only failure output, failed state observation and
completed success without failed-step logs. These use substituted Git/GitHub
commands, not live log downloads. Actual snapshot/pins checks, selected
ShellCheck and diff checks pass. Logs use
`/tmp/ic-testkit-0291-{export,gh-ci,gh-ci-bash32,checks,shellcheck}.log`.
The selected lock is unchanged (SHA-256
`47cf2d3922416b3ad78233f68f82087e9d63527dc127a8bd011e8c20bf79463c`).
No Cargo job, tool installation, sibling edit, full gate, package-version change,
release or CI rerun was performed. Released 0.29.0 CI
[38041559012](https://github.com/dragginzgame/ic-testkit/actions/runs/38041559012)
and Shared 0.2.14 CI
[38041453237](https://github.com/dragginzgame/shared-tooling/actions/runs/38041453237)
were queued at inspection; native results for this pending consumer source
remain outstanding. Earlier evidence below belongs to its stated inputs.

## IC Host 0.11 and Shared Tooling 0.2.13 for pending 0.29.0

The maintainer authorized the 0.29.0 hard cut on released Testkit 0.28.1
`ebbea97c90161eb03834b01a9fb0159aa0d99d89`. Reviewed clean Host source is
`1d768c80a5bb87e3330a6b7bacfdfc543063968f` (published 0.11.0); Shared is
`5864f468d39f8f9d1bd26fca1afe0e20f25f1b5e` (0.2.13). The canonical export
retains 84 files. Incoming Host manifest/lock changes are preserved; the final
lock selects all four Host crates at 0.11.0, with no unrelated package changes
against HEAD (SHA-256
`d23fc74319824fe86e5256cd663197d958496cb6f75ad33283eb8e6fa37e2468`).
An attempted registry update failed DNS resolution and is retained in
`/tmp/ic-testkit-0290-host-update.log`; subsequent locked cache preparation
succeeded against the selected graph without unlocking it.

Process callers and the transactional example use the single OutputLimits
contract with per-stream policies and optional deadlines. Finite probe/download
bounds remain unchanged. Cargo retains its existing output policy and no
deadline; this adoption imposes no quota on multi-hour builds. Any execution
cleanup failure now prevents flattening the typed error into a plain failed
Cargo status. Public re-exports require Host 0.11 source migration; both note
views describe it. Stored v1 formats and retained installations need no reset.

Release preflight admits source and reconciles the selected saved workspace,
fetches its locked graph, then invokes existing install-tools and
install-format-tools, followed by tools-check and format-tools-check.
PocketIC provisioning remains the explicit owning setup operation. Standalone
verification and the sequential full-gate roster perform offline tool admission
before dependent validation; checks never install. Actual Make callback fixtures
under parallel Make prove cache-before-setup ordering, repeated preparation,
source refusal without setup, setup failure propagation and standalone early
admission refusal without gate dispatch. Existing offline/network policy,
metadata interruption and saved-release checks remain covered. Setup/gate/Git
effects are substituted; these fixtures do not prove live tool installation or
native macOS behavior. Shared's installer fixtures separately cover reuse,
invalid selections and retained failed installation evidence.

Focused Linux checks pass: 58 Wasm-cache tests, two Host adapter tests, 23 startup
tests (one separately ignored live case), seven CLI tests, strict Clippy including
test callers and the transactional example, strict rustdoc, and actual Rust 1.88
library/CLI plus Wasm library compilation. Snapshot distribution and Rust-tool fixtures pass under Bash 5 and
genuine Bash 3.2; consumer release guards pass under both. Actual consumer
snapshot verification refuses LF/CR roots and resolved aliases beside a valid
trimmed-name neighbor, accepts an ordinary alias with CDPATH set, and preserves
the real manifest. Snapshot/pins/fmt-check, selected ShellCheck and diff checks
pass. Logs use `/tmp/ic-testkit-0290-{shared-export,cache,wasm,host-callers,startup,
cli,clippy,example,msrv,wasm-msrv,docs,snapshot-fixture,snapshot-bash32,rust-tools,
rust-tools-bash32,guards-current,guards-bash32,paths,shellcheck,tooling-final}.log`.
Earlier guard attempts remain in `guards.log` and `guards-final.log`: the first
missed tracking the new fixture trace; the second was superseded by the final
source-bound runs. Their retained temporary evidence is not erased.

Exact-source upstream CI remains separate: Host
[38040092044](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38040092044)
is queued and Shared
[38039035514](https://github.com/dragginzgame/shared-tooling/actions/runs/38039035514)
is in progress at inspection. This pending consumer source has no hosted/native
result. Full local gates remain maintainer-owned under the explicit overlay;
none ran. No package version, sibling source, release, tag, publication or CI
dispatch/rerun was changed by this work.

## IC Host 0.10.2 selected-graph qualification for pending 0.28.2

The maintainer's incoming lock selects all four published Host crates at
0.10.2, reviewed clean source `6b755763aca71b7ea8c3de40edf032093dfdc62c`.
No dependency resolution or manifest change ran. Host now rejects pathname
publication targets ending in `/` or `/.` before normalization, parent creation
or staging. Testkit needs no production adapter change. A real-I/O atomic-copy
regression verifies existing and missing-parent targets are refused, original
destination bytes survive, no parent/staging entries appear, and the wrapper
retains both path context and Host's typed `BeforePublication`/`InvalidInput`
error without a cleanup failure. Owner-level callback and named/validated
writer coverage remains upstream rather than being duplicated here.

Explicit `cargo fetch --locked` prepares the selected cache. Focused Linux
checks pass: 12 digest/copy tests, 10 cache-filesystem tests, 28 transaction
tests, all seven CLI tests and strict Clippy including test callers. Logs use
`/tmp/ic-testkit-0282-host-{cache,digest,cache-fs,transaction,provisioning,
clippy,tooling}.log`. The incoming lock remains byte-for-byte unchanged
(SHA-256 `ebe24416c52e7a2b4a82b88f465eafe0299f83cff14dde7290b209a36173cddd`).
Earlier 0.28.2 evidence below used the preceding incoming 0.10.1 selection and
is not relabelled as qualification of this graph.

Host's exact-release CI [38037252544](https://github.com/dragginzgame/ic-host-tooling/actions/runs/38037252544)
has passing Linux and MSRV jobs; ARM/Intel macOS jobs are queued at inspection.
The pending Testkit source still has no exact-source hosted/native result.
Public APIs, v1 layouts and retained installations need no migration or reset.
No function, method or type was removed, package version changed, sibling
file edited, broad gate run, release performed or CI rerun requested.

## Shared admission and provisioning fixture fixes for pending 0.28.2

This batch starts at released 0.28.1
`ebbea97c90161eb03834b01a9fb0159aa0d99d89`. The clean canonical export selects
Shared Tooling 0.2.11 `83efac446348dea024798a331d77933b24b429dc`, preserving
the 84-file selection. Admission now rejects unsafe outer Make modes when
MAKEFLAGS is cleared or replaced; MFLAGS supplies independent evidence.
The actual consumer Makefile fixture covers six formatter/release targets,
eleven unsafe mode spellings and five flag contexts (330 cases), requiring
status 2 without runner or formatter effects. Ordinary parallel help remains
admitted, and a safe formatter failure propagates through Make.

Released Linux CI [38035257345](https://github.com/dragginzgame/ic-testkit/actions/runs/38035257345)
failed spawning a provisioning test executable with ETXTBSY. Mutable fixture
writes can overlap another test's subprocess fork, which can briefly retain
the writable descriptor. Host 0.10.1's private test support already coordinates
this lifetime. Testkit now holds a test-only mutex through each provisioning
fixture's cleanup; its concurrent setup test still runs two callers against
one root and checks one download. Production setup, retries and cache formats
are unchanged. The initial focused run passed before the fix, so this is an
intermittent failure analysis rather than a deterministic local reproduction.

Focused Linux qualification passes: all seven CLI unit tests, strict Clippy
including test callers, and 100 repeated six-test provisioning runs with six
test threads (600 test executions). Committed upstream formatting/release
fixtures pass under Bash 5 and genuine Bash 3.2. The final consumer release
guard run under Bash 3.2 includes the 330 cases, safe formatter failure and
controlled-effect release/metadata fixtures. Snapshot, dependency pins,
fmt-check and selected ShellCheck pass. No broad gate was run.

Logs use `/tmp/ic-testkit-0282-{shared-export,cache,provisioning-final,
provisioning-clippy-tests,format,format-bash32,release,release-bash32,
guards-bash32,tooling,shellcheck}.log`; repeated runs use
`/tmp/ic-testkit-0282-provisioning-stress-{1..100}.log`. The original hosted
failure excerpt remains in `original-ci-failure.log` with the same prefix.
The pre-fix passing run remains in `provisioning.log`; the first guard-field
attempt's dead-code warning remains in `provisioning-fixed.log`. The final
named guard field passes strict Clippy without a lint suppression.

The incoming Cargo.lock remains unchanged (SHA-256
`56e4b00ee4c11704935761aa0e3867e1350209abbcef0cb1225ce37e458f4d96`).
No function, method or type was removed, package version changed, sibling
file edited, retained installation reset, release performed or CI rerun
requested. These local results do not qualify the pending source on native
macOS or hosted CI; released 0.28.1's results belong to its own source.

## IC Host 0.10.1 selected-graph qualification for pending 0.28.1

The maintainer's selected lock already uses all four Host crates at 0.10.1;
no dependency resolution or manifest change ran. Reviewed published source is
`c7bdc3d4e1c658957202eebd76bff2c51e22f645`. Its filesystem owner now completes
both syncs for a formerly missing parent even when another writer wins mkdir.
Existing APIs, publication phases, modes, permissions and v1 formats remain
unchanged. Testkit's adapters need no further source changes.

With explicitly prepared locked dependencies, focused Linux qualification
passes: 11 digest/copy tests, 10 cache-filesystem tests, 28 transaction tests
and 6 provisioning tests; actual Rust 1.88 library/CLI compilation; fresh
owner CLI build; and offline admission of the retained official Linux 16.1.0
bundle with PATH=/nonexistent. These are consumer integration checks; the
deterministic competing-parent/sync-failure tests remain with Host's owner.
Logs use `/tmp/ic-testkit-0281-host-{cache,digest,cache-fs,transaction,
provisioning,msrv,cli,server-check}.log`.

The incoming lock remains unchanged (SHA-256
`80649a8da3e0fec49e325a1651cf69b07b86d0dfc767b055e59b63f97f19ac9f`).
No server setup/download, tool installation, retained-data reset, function,
method or type deletion, sibling edit, package-version change, broad gate or
release ran. Host's exact-source CI
[37964131536](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37964131536)
passes; this pending consumer graph still needs its own hosted native results.

## Shared Tooling 0.2.10 adoption for pending 0.28.1

The reviewed snapshot selects committed Shared 0.2.10
`43a0dc46cdc3c77e70a68e192561642ed50a3e0f`, exported from a clean isolated
checkout. Its 84-file selection adds only the declared formatter reporter
companion. Snapshot v1 now records the source package version alongside its
revision and file digests. Fleet dashboards remain upstream-owned and are not
added to Testkit's CI or source selection.

The existing shared Rust formatter recipes emit one success line or a short
failure summary with a retained log. Both consumer fixture projections carry
the new reporter. Testkit's failure collector archives formatter logs for
checks and portable jobs, preserving the existing evidence roots, modes and
non-followed links. The workflow's exact-ID artifact readback now declares
read-only Actions permission, uses the workflow token, and binds repository
and run explicitly. Existing payload/digest verification stays in place.

Focused Linux qualification passes: snapshot/pins/actual fmt-check; committed
upstream snapshot distribution and registry observation fixtures; formatting,
actual consumer hooks and consumer evidence archive fixtures under Bash 5 and
genuine Bash 3.2; selected ShellCheck; pinned actionlint; and controlled-effect
release guards with offline real Cargo metadata. The registry fixture uses
substitute curl, including its new metadata contract; Testkit's publish policy
continues using presence-only observation. No live registry upload or
authenticated artifact download was triggered by these local checks.

Logs use `/tmp/ic-testkit-0281-{shared-export,snapshot,registry,registry-bash32,
format,format-bash32,hooks,hooks-bash32,evidence,evidence-bash32,checks,
workflow-prepared,shellcheck,cache,release-guards}.log`. The initial workflow
lint used a nonexistent local path; that failed attempt remains in
`/tmp/ic-testkit-0281-workflow.log`, separate from the passing prepared-tool run.
The maintainer's incoming Cargo.lock selection remains untouched by this
tooling adoption; locked cache preparation does not resolve upgrades.
No function, method or type was removed, package version changed, tool installed,
sibling file edited, broad gate run, or release performed. The pending source
has no native macOS/hosted readback result; released 0.28.0's successful full CI
belongs to its own source and does not qualify these changes.

## IC Host 0.10 adoption for pending 0.28.0

The maintainer-selected manifest and lockfile now use published IC Host 0.10.0,
reviewed source `98562bea26a98993d93b80ed908bea4876c32a91`. The selected lock
is preserved byte-for-byte (SHA-256
`e635def66d8c809f068e23ea8cdbf61d2c4a78e64e37e176c4743ba6fb3572ba`).
Uncommitted upstream 0.10.1 parent-sync changes are excluded. Public Host
re-exports make this a breaking 0.28.0 batch; both changelogs carry the existing
Shared 0.2.8 work forward, without changing Testkit's package version.

Atomic copying uses the single explicit-options pathname writer with the
previous replacement mode and permissions; provisioning uses the same writer
with its previous create-only executable options. Cache tags, maintenance/use
markers, transaction manifests and build/input stamps preserve complete Host
publication errors inside their existing I/O boundaries. Producer and
before-publication cleanup evidence is retained, as is the distinct visible
after-publication state. No caller strips the error down to its source or adds
an automatic retry. Formats stay v1; existing installations and cache bytes
need no reset. No Testkit function, method or type was removed.

Focused Linux qualification passes with explicitly prepared locked dependencies:
11 digest/copy tests, 10 cache-filesystem tests, 28 transaction tests, 3 watched
input stamp tests, 58 Wasm cache tests and 6 provisioning tests. New real-I/O
regressions verify a producer read failure preserves the original destination
and staging cleanup, and a refused cache-tag replacement retains its typed
before-publication error. They do not inject actual failed cleanup or final
sync syscalls; complete error retention for those phases follows the unchanged
whole-error wrapping, separately from Host's owner fault tests.

Library/CLI compilation, strict Clippy including test callers, actual Rust 1.88
library/CLI compilation, fmt-check, snapshot and dependency pins pass. Logs use
`/tmp/ic-testkit-0280-host-{cache,check,digest,cache-fs,transaction,icp,wasm,
provisioning,clippy,msrv,tooling}.log`. This adopts no implicit Cargo deadline,
quota or tool installation. No broad local gate, sibling edit, commit or
release ran. Host 0.10.0 Linux/MSRV CI passes while its native macOS jobs are
queued; this pending consumer source has no hosted native result of its own.

## Shared Tooling 0.2.8 adoption for pending 0.27.3

The existing 83-file selection now records committed Shared 0.2.8
`b2646cde9abbc8861857a4379c683a0c19eba43e`, exported from a clean isolated
checkout. Make admission resolves its probe beside the selected include and
uses `MAKE_COMMAND` rather than treating recursive `MAKE` arguments as an
executable path. Runtime release/formatting policy remains consumer-owned.

Focused Linux checks pass: snapshot, dependency pins and non-mutating formatting;
the exact upstream release and formatting fixtures under Bash 5 and genuine
Bash 3.2; actual consumer release forwarding with an inherited nonexistent
tooling root; 60 direct/inherited unsafe-mode cases on the copied consumer
Makefile with no runner/formatter effects; and recursive parallel help using
extra Makefiles, plus help with environment/command-line tooling roots.
The latter help cases qualify parse-time admission, not runtime recipe-root
selection. Logs are retained at `/tmp/ic-testkit-0273-{shared-export,checks,
format,format-bash32,release,release-bash32,consumer-release,consumer-modes,
consumer-recursive}.log`.

The pre-existing maintainer-selected Host 0.9.5 lockfile is preserved (SHA-256
`de9fe2beb8beb8ad22d0288ae001c39580ba4f3c0cd86d129b87d8705da6c390`).
No Host compilation or runtime qualification is claimed by these Make checks.
No function, method or type was removed. No tools were installed, sibling files
edited, broad gates run, or release performed. Native macOS acceptance of this
pending batch remains outstanding; the released 0.27.2 Linux CI result belongs
to its earlier source, not these edits.

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
- `cargo-sort` 2.1.4, prepared explicitly under `.tools/rust/common/bin` with
  `make install-rust-tools`, then verified offline with `make rust-tools-check`, for
  hooks, manifest sorting, and independent CI/release formatting checks.
  Hook setup and formatting checks also require an already prepared `rustfmt`
  component; use `rustup component add rustfmt` during explicit setup.
- `curl`, gzip and xz-capable tar for explicit tool setup. Run `make install-tools`
  to prepare pinned jq/yq, ripgrep with PCRE2, cloc and IC executables locally,
  then `make tools-check` for
  offline byte/version verification. Versions and native digests have one owner
  in the reviewed snapshot's `ci/tool-versions.env` and `ci/ic-tools.tsv`.
  Host, Rust and IC exceptions are selected independently under the shared
  [catalog contract](local-setup.md#catalog-selection). Compiler and PocketIC
  policy remain local. See the shared
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

## IC Build extraction source qualification

The maintainer authorized moving generic Testkit build/cache mechanics and
Canic feature-equivalent Cargo grouping to IC Build
([IC Build #1](https://github.com/dragginzgame/ic-build/issues/1)).
Testkit's native development dependency selects published IC Build 0.1 through
crates.io and the tracked lockfile. Formatting and validation need no sibling
checkout or external-path pin exception.
The engine preserves v1 cache/stamp/lock bytes and namespaces, returned artifact
lifetime, typed failures, input-change rejection and explicit maintenance.
PocketIC lifecycle stays in Testkit; Canic's complete-release admission, recipes,
credentials, generated inputs and release identities remain consumer policy.
The separate complete-release reuse/finalization path is not replaced by a
generic cache-hit predicate in this batch. IcyDB adoption and native macOS
qualification remain separately scoped. No package versions, refs, tags or
release artifacts are changed by the extraction.

### 0.35.1 retained benchmark artifact handoff

The focused canister benchmark test uses registry IC Build 0.1.1 and Host 0.15.1,
with locked Cargo inputs and one build acquisition. It reads the retained exact
artifact and holds its record through real PocketIC 16.1 installation, the
benchmark update and fetched marker parsing. It releases the fixture and record
before deleting its own temporary output. Generic cache semantics are qualified
by IC Build rather than repeated in this consumer test.

Locked offline cache preparation, selected strict Clippy and formatting pass.
The actual Linux test passes with the prepared server: use `POCKET_IC_BIN` from
`target/debug/ic-testkit-server check`, then `CARGO_NET_OFFLINE=true cargo test
-p ic-testkit --locked --offline --test canister_benchmark
perf_probe_canister_emits_parseable_benchmark_markers -- --exact --nocapture`.
The first sandboxed run failed at PocketIC's local-port bind; the unrestricted
focused run passes. Both observations remain in
`/tmp/ic-testkit-0351-retained-handoff{,-native}.log`. Native macOS acceptance for
this source remains outstanding; no broad gate or release was run.

### 0.35.2 retained sync/async instance construction

Source qualification compares the pending working tree with released 0.35.1
`245bbb9361722c7d562d5c83666850f4ddbb4364`. The new construction entrypoints retain
an optional exact managed server alongside the upstream instance, sharing process
spawn, port/readiness admission, failure projection and cleanup. Connect mode
claims no server. Async readiness/HTTP waits yield on the caller's Tokio runtime;
process spawn, bounded file reads and cleanup remain synchronous. The configured
deadline cannot preempt synchronous work within an upstream future poll.

Linux qualification passes 30 focused startup tests, including three tests with
the prepared PocketIC 16.1.0 executable. Both new APIs preserve application/NNS
topology, allow overlapping borrowed instances, and retain the owned child through
use before exact teardown. Synthetic fixtures verify typed configuration, URL,
server-exit, readiness-timeout, instance-timeout and panic failures. Cancellation
during readiness and the HTTP instance wait reaps the exact owned PID; explicitly
selected raw logs survive. Existing one-shot reaper behavior remains qualified.
The lifecycle fixture now observes its exact port path instead of guessing a
concurrently allocated directory. The first native run exposed that pre-existing
fixture race; its failure remains in `/tmp/ic-testkit-0352-startup-native.log`.
The final native run is `/tmp/ic-testkit-0352-startup-complete.log`:

```bash
POCKET_IC_BIN="$PWD/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic" cargo test -p ic-testkit --locked --offline --lib pic::startup::tests -- --include-ignored --nocapture
```

The benchmark driver passes 17 tests; canonical protocol admission passes with
leading-zero and numeric-overflow rejection. Both construction Rustdoc examples
compile. Strict selected Clippy, Rust 1.88 library compilation, the Wasm library
check, formatting and dependency pins pass. Logs are
`/tmp/ic-testkit-0352-{driver,protocol,doctests,clippy-complete,msrv,wasm,format,pins}.log`.
Locked offline caches were explicitly prepared without changing the lockfile.
These targeted results are not full CI, downstream harness migration or native
macOS proof. Timers, Blob and Backup migration and both native macOS hosts remain
outstanding under [#68](https://github.com/dragginzgame/ic-testkit/issues/68) and
[#69](https://github.com/dragginzgame/ic-testkit/issues/69). Upstream synchronous
instance deletion remains unbounded; nonblocking instances need explicit async
deletion while their server is retained. Cancelled borrowed construction can leave
an instance created before timeout/lost reply for caller-owned reconciliation.

### Pending 0.36.0: IC Host 0.16 and Shared Tooling 0.9.13 integration

Reviewed upstream heads are Host `046751faafecf8fb9e924e7c813efcbb69d36d10`
and Shared `35cad65677ff989b5fb24a88d947fda37d2366e8`, confirmed against GitHub.
The incoming Testkit manifests and lockfile already select Host 0.16.0; their
bytes are preserved. Host's three used subcrates have unchanged source APIs from
0.15.2, but their public re-exports and typed probe/cleanup diagnostics now carry
0.16 crate identities. The pending notes therefore move to 0.36.0; direct Host
types crossing this boundary must use 0.16. Own package versions remain unchanged.
IC Build's existing development-only Host 0.15 graph stays separate. The new
aggregate `ic-host-tools` default-feature change has no Testkit dependency caller.

Shared's canonical exporter adopts the unchanged 96-file selection. Active
sibling CI-helper edits remain untouched: a clean temporary clone of the reviewed
commit supplies the export and CI-inspection fixture. Snapshot verification
passes. Explicit Rust-tool setup prepares the changed build-helper identity,
retaining earlier generations; offline Host/IC/Rust/PocketIC admission passes.
Locked dependency sources were prepared with `cargo fetch --locked` before checks,
without resolution or an implicit fetch in offline validation targets.

Native Bash and actual Bash 3.2 pass the Host, IC and Rust installer fixtures and
CI-selection fixture, covering lock acquisition/status fidelity, source-download
failure propagation, retained selections and failed/detached branch observations.
These are substituted tool/transport tests, not native macOS proof. The initial
direct invocation of the sourced tool-store support file was invalid; its retained
log is `/tmp/ic-testkit-upstream-tool-store.log`. The owning Host/IC harnesses pass.

The locked Host 0.16 graph passes 30 startup tests (including three prepared-server
tests), six CLI provisioning tests, 13 command-runner fixtures, and separate real
runner and provisioned-CLI instance tests against PocketIC 16.1.0. The 17 benchmark
driver tests, both construction doctests, strict selected Clippy, actual Rust 1.88
library/CLI checks, Wasm library check, dependency pins and formatting pass.
Evidence is `/tmp/ic-testkit-upstream-{host,ic,rust}-fixtures.log`, corresponding
`*-bash32.log`, `gh-ci{,-bash32}.log`, `rust-setup.log`, `startup.log`,
`provisioning.log`, `runner{,-native}.log`, `cli-native.log`, `driver.log`,
`clippy.log`, `doctests.log`, `msrv.log`, `wasm.log` and `admission.log`.
No broad gate or release was run. Both native macOS hosts and downstream fixture
migration remain outstanding for this consumer source.

### Fixture-pool flow simplification, initially prepared for 0.36.1

Source comparison is released 0.36.0 `a31cce9efd711ef2a5aee68dbe5b5b443dae6505`
to the pending working tree. The requested complexity review uses the adopted
Shared 0.9.13 complexity/flow-convergence methods and local AGENTS overlay,
scoped to standalone and recipe-baseline acquisition. Cold and recovery build
dispatches converge from four sites to two, one per owning pool. One forwarding
boundary disappears. Public behavior axes and the bounded-slot coordination
owner stay unchanged; the distinct restore/reset/readiness policies remain local
([#71](https://github.com/dragginzgame/ic-testkit/issues/71)).

With prepared PocketIC 16.1, focused Linux qualification passes 12 pool unit tests,
13 baseline tests (including 100 consecutive restores), nine ordinary ownership/
standalone tests and both exact dedicated-server recovery probes. A command-owned
`ic-testkit-server run -- cargo test ...` server contains each live suite. The
new probe preserves both failed-restore/replacement-capture causes, completed
phase timings and the single build attempt, then proves a new acquisition can
build the empty slot. The baseline probe replaces upstream manual process control
with the managed handle. CI runs both exact probes across the three native hosts.

Strict selected Clippy, actual Rust 1.88 library checking, the standalone pool
Rustdoc example, prepared actionlint 1.7.12 and `make fmt-check` pass. The first
workflow lint invocation used a nonexistent local path; its log remains separate
from the passing prepared-tool run. Evidence logs are
`/tmp/ic-testkit-0361-{pools-unit,baseline-live,standalone-live,baseline-recovery,
standalone-recovery,pools-clippy,pools-msrv,pools-doctest,pools-actionlint,
pools-actionlint-final,pools-format}.log`.
No dependency, package version, index/ref or stored format changed, and no broad
gate or release ran. Native macOS qualification remains outstanding for this
source; no latency or throughput improvement is claimed from source reduction.

### Pending 0.37.0: Host 0.17, Shared 0.9.15 and startup ownership

Reviewed released heads are IC Host `93a0ecc2afb7d4b8d8a252a21ed10d7e58b35901`
and Shared Tooling `a9a8ea5ddea0db279d8c139ef5a951474c50f742`. Host's dirty
governance work is separate and untouched. Incoming manifests and lockfile already
select Host 0.17.0 and IC Build 0.1.3; their bytes remain unchanged. Exposed Host
crate identities move the complete pending batch to 0.37.0. Own versions remain
maintainer-owned; IC Build's development-only Host 0.16 graph stays separate.

Managed-server child ownership is now nonoptional. Startup files own port reading
and bounded capture, removing childless server fixtures. Explicit termination
and child Drop still complete before final capture; background reaping retains
the child and files together. No startup bounds, API, stored layout or pool policy
changes. The existing recovery/cancellation/reader fixtures cover these contracts.

The canonical exporter preserves the 96-file Shared selection. Snapshot admission,
CI-log recovery and maintenance-path fixtures pass on native Bash and actual
Bash 3.2. The two fixture harnesses run from a clean temporary clone with adopted
script bytes compared exactly. These substitute transports/CLI, not native macOS.
The executable-tool build policy is unchanged; no tool reinstall is required.

Locked source preparation precedes offline checks. Linux passes 27 startup tests,
three prepared PocketIC 16.1 ownership tests, six provisioning tests and 13 runner
tests. Selected strict Clippy, actual Rust 1.88 library/CLI checks and formatting
pass. Logs are `/tmp/ic-testkit-host17-{fetch,snapshot,gh-ci,gh-ci-bash32,
maintenance,maintenance-bash32,startup,startup-native,provisioning,runner,clippy,
msrv,format}.log`. No broad gate, release or sibling edit ran. Native macOS and
remote CI for this uncommitted source remain outstanding.

### Registry IC Build 0.2 adoption — 2026-10-13

The authorized continuation selects published IC Build 0.2.0 in the native
workspace development catalog and lock. Archive SHA-256 is
`fb055d1b5649636c09a52f23fdc091417c732cbb87a2125d5a24f93c043c8df8`;
its VCS receipt names `9ddcd4ee0b2ac85f0608b8aaf49493cf015f295a`.
Only Build advances; the three now-unused Host 0.16 packages retire from the
lock. All other registry versions/checksums, own package versions, incoming
Shared 0.10/helper/driver work and retained artifacts are preserved. No path
patch or runtime Build dependency is introduced.

Strict library/driver Clippy, 17 driver tests, six diagnostics tests, six pure
baseline-pool tests and actual Rust 1.88 library/driver checking pass on the
real registry graph. Receipts are `/tmp/ic-build-020-registry-adoption/testkit-*`.
The prior real PocketIC smoke remains evidence for its earlier graph; this
adoption runs no server lifecycle or benchmark. Native macOS and broad suite
qualification remain separate. Pending notes stay compatible 0.37.1; delivery
and publication remain maintainer-owned
([IC Build #9](https://github.com/dragginzgame/ic-build/issues/9)).

### Earlier pending 0.37.1: IC Build benchmark handoff and helper cleanup

The incoming lockfile selects published IC Build 0.1.4, reviewed against
`848ae20548527f5595ea68e7edf9cfec4f3188ce`; its bytes and own package metadata
remain unchanged. Source comparison starts at Testkit 0.37.0
`6407a42844def1c38128522825f306b977365517`. IC Build stays development-only.
The driver holds the retained probe record through all workers and report writes,
hashing and consuming its exact artifact. It preserves tool selections, offline
arguments, foreground build handling and no elapsed-time deadline. Diagnostic
admission and phase accumulation keep their public contracts with local owners.

Focused Linux checks pass six diagnostics tests, six baseline-pool unit tests
and 17 driver tests. The tool fixture covers bare/absolute/relative multicall
proxies, missing tools, original producer status/streams, and repeated retained
reads after replacing/removing public Cargo outputs. A prepared PocketIC 16.1
smoke run passes fresh and capacity-one pooled workers with two tasks each, using
one retained probe artifact and the unchanged report schema. This is correctness
evidence, not a performance comparison or native macOS qualification.

Strict selected Clippy, actual Rust 1.88 library/example checking, declaration
pins and formatting pass. Logs use `/tmp/ic-testkit-0371-{fetch,driver-final,
pool-unit,diagnostics,benchmark,clippy,msrv,pins,format}.log`; the smoke report is
`/tmp/ic-testkit-0371-benchmark.json`. Earlier fixture failures remain in their
separate `driver-before-fixture.log` and `driver-fixture.log` files. No broad gate,
release, dependency resolution or sibling edit ran. Delivery and native macOS
acceptance remain outstanding for this consumer source.

### Pending 0.37.1: Shared Tooling 0.10 catalog adoption

The canonical exporter adopts `ed9c26f015691859f67b50b512a94709dcdc0c6c`,
preserving the 96-file selection. Common catalog assignments, versions and native
digests match the prior snapshot; only catalog ownership comments change. The
consumer's redundant host/IC Make defaults are removed. Direct CI cache reads,
formatter/release adapters and evidence selection already name the same reviewed
root-layout catalog, with no local exception or duplicate default list
([#73](https://github.com/dragginzgame/ic-testkit/issues/73)).

Explicit `make install-tools` reuses the prepared common bundles and PocketIC
selection, building the local owner CLI. Offline tool checks, formatting, pins
and snapshot verification pass. Native Bash and actual Bash 3.2 pass host/IC
installer, independent Make catalog/formatter, Cargo freshness, release-runner,
LOC path-admission and consumer evidence/archive fixtures. The affected extra
harnesses run from a clean clone of the exact source, with adopted script bytes
compared. These use synthetic tool/registry/release inputs, not macOS proof.

The initial LOC runs lacked the explicitly prepared host PATH; reruns with that
PATH pass. The first ShellCheck omitted sourced companions, and the first workflow
lint named an absent publish workflow. Correct source-following and actual-workflow
checks pass; failed attempts remain separate. Logs use `/tmp/ic-testkit-more-*`.
The incoming Rust fixes are unchanged. An independent maintainer dependency
selection advanced to IC Build 0.2 during qualification and is preserved below.
No broad gate, release, dependency resolution, installation retirement or sibling
edit ran. Testkit's matching native macOS qualification remains outstanding.

### Pending 0.37.1: selected IC Build 0.2 graph convergence

The maintainer selection now uses registry IC Build 0.2.0, reviewed against
`9ddcd4ee0b2ac85f0608b8aaf49493cf015f295a`; this session does not resolve or
rewrite its manifest/lock selection. Build's engine Rust sources are unchanged
from 0.1.4. Its Host dependency boundary now shares Testkit's 0.17 family, removing
the three older Host 0.16 packages. Build remains development-only; Testkit's
public library boundary and pending compatible 0.37.1 classification are unchanged
([IC Build #9](https://github.com/dragginzgame/ic-build/issues/9)).

Locked cache preparation, all 17 driver fixtures, selected strict Clippy and
actual Rust 1.88 library/example checks pass on the new graph. A prepared
PocketIC 16.1 capacity-one pooled smoke also passes
with one warm task and the retained probe/report contract. Logs use
`/tmp/ic-testkit-more-build02-{fetch,driver,clippy,msrv,benchmark}.log`; the new smoke
report is `/tmp/ic-testkit-more-build02-benchmark.json`. Earlier 0.1.4 results
remain bound to their own graph. Native macOS consumer acceptance stays pending.

### Pending 0.37.1: published Host 0.17.4 mechanics

The normal registry graph selects Host 0.17.4 and development-only IC Build 0.2.3.
The incoming lockfile already selected these versions and is preserved exactly.
Testkit's Host requirements now start at 0.17.4; no path override is installed.
PocketIC admission shares Host's no-follow identity/version boundary. Native RSS
observations and checked report staging use their Host owners; Testkit retains
workload, sampling cadence, output schema and interruption policy
([#75](https://github.com/dragginzgame/ic-testkit/issues/75),
[Host #79](https://github.com/dragginzgame/ic-host-tooling/issues/79)).

Locked source preparation precedes focused Linux qualification: 13 benchmark
driver tests, six provisioning tests, selected strict Clippy, actual Rust 1.88,
formatting and dependency declarations pass. A live prepared PocketIC 16.1 smoke
passes fresh and capacity-one pooled modes with two tasks each, one worker,
64 state bytes and a complete private report. This proves integration correctness,
not a performance comparison. Receipts are under
`/tmp/ic-testkit-host174-adoption/`, including `benchmark.json`.

Earlier path-candidate evidence remains bound to its unpublished source graph.
Both native macOS hosts and full maintainer-owned consumer validation remain
outstanding for this source. Pending notes stay compatible 0.37.1; own versions,
retained installations and release effects remain unchanged.

### Pending 0.38.0: one fixture-pool lifecycle

The accepted hard cut replaces the standalone pool family with
`StandaloneCanisterRecipe` and the existing recipe baseline pool. Static suites
use `LazyLock`; builders own their `Send + Sync + 'static` captures. Snapshot-only
funding and restore policy stay explicit. A failed non-transport restore discards
the slot before returning, while dead transport retains one recovery build and
both causes. Process-local pools require test-process restart, without a retained
cache/schema change ([#76](https://github.com/dragginzgame/ic-testkit/issues/76)).

Focused Linux qualification passes 13 driver tests, three Candid unit tests,
one lazy-construction case, six bounded-slot cases, six baseline-contract cases,
11 live Candid/concurrency/funding cases and the exact dedicated-server recovery
case. Selected Rust 1.88, all local callers under strict Clippy, public Rustdoc,
Wasm library compilation, formatting and dependency checks pass. A prepared
PocketIC 16.1 smoke passes fresh and capacity-one pooled workers with two tasks
each and a complete report. Logs and `benchmark.json` remain under
`/tmp/ic-testkit-0380-cut/`; the initial compile failure is retained separately.
These observations establish correctness, not a performance improvement.

The Canic payload-limit migration patch passes apply-check and is tracked under
[Canic #540](https://github.com/dragginzgame/canic/issues/540). Its normal adoption
requires published Testkit 0.38; no Canic source or lock was changed. The consumer
checkout has active validation, so no competing consumer build was launched.
Both native macOS hosts, full maintainer-owned validation and release remain
outstanding. Own package metadata stays at 0.37.0; pending notes move to 0.38.0.

### Pending 0.47 restore receipts, Candid variants and Git observations

Linux qualification of the current working tree passes five selected live
baseline cases (including empty admission, full-set/funding receipts, mismatch
recovery and mutation guards), the live Candid rejection case and six teardown
cases. Five call-codec and ten transport unit tests pass. All 20 driver tests
pass, including byte-preserving Git status and retained typed failures. A live
fresh/pooled two-task smoke passes with the prepared PocketIC 16.1 server.
Strict affected-target Clippy, Rust 1.91 library/three-example checks, formatting
and warning-denied native documentation pass. Evidence is retained at
`/tmp/testkit-three-simplifications-nzoXH8/`; the first live attempt was refused
by sandbox loopback restrictions and is retained separately. Native macOS and
full CI remain maintainer qualification; Git observations remain separate reads,
not an atomic source snapshot. No persistent layout or format identity changes.
