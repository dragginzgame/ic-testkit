<p align="center">
  <img src="../images/ic-testkit-readme-header.svg" alt="IC Testkit — Internet Computer helper library" width="100%">
</p>

# Fixture reuse benchmark

This opt-in Linux benchmark measures PocketIC 16 fixture construction and warm
baseline reuse. It does not run in `cargo test` or the ordinary CI gate, download
a server, or change dependency versions.

```sh
python3 scripts/dev/benchmark-fixture-reuse.py \
  --server /absolute/path/to/pocket-ic \
  --iterations 100 --workers 2 --repeats 3 \
  --state-bytes 1048576 --output /tmp/fixture-reuse.json
```

The driver builds the example in release mode and the existing `perf_probe`
canister in dev mode, using locked dependencies. The Rust Wasm target must
already be installed. Use `--profile dev` for a quick development smoke check;
compare performance only within the same host and canister profiles. The result
records the binary version/path, Git revision and dirty state, Rust compiler,
Wasm SHA-256, workload, raw samples, and mean/p50/p95 summaries. Run from a quiet
host and retain the JSON when comparing changes.

## Workload and correctness

Every fixture has two canisters with configurable deterministic heap state.
Each task checks the seeded counter, byte length, and first byte, mutates both
canisters through update calls, and verifies the mutation through query calls.
Warm acquisitions must restore the seed and report `Restored`; unexpected builds,
recovery, state mismatches, or failed calls stop the measurement.

Fresh fixtures create, install, seed, validate, mutate, and drop an independent
PocketIC instance for every task. They do **not** capture unused snapshots.
Pooled fixtures capture and validate each slot once, then restore it for every
task. All slots are prepared and mutated before timing begins, so the first
measured restore must also prove it resets state.

Every mode runs the same total task count with the same number of worker
threads. Capacity one serializes leases and measures waiting; larger capacities
permit overlap up to the worker count. Each case/repeat has a separate worker
process and one explicit managed PocketIC server. Instances share that server
within the run. Repeat order rotates over all selected cases.

On Ctrl-C, the driver waits for the current worker to finish so its owned server
can be torn down normally; it does not start another mode.

This workload changes only snapshot-contained state. Cycle policy is
`PreserveCurrent`; time and cycle balances are not asserted to rewind. Reset and
readiness hooks are no-ops because calls are synchronous. Their timings expose
pool overhead, not realistic application reset/readiness cost. The probe is a
controlled workload, not a throughput prediction for a downstream suite.

## Measurement boundaries

- Raw per-task samples report acquisition, capacity wait, build, restore,
  non-snapshot reset, readiness, validation, body, release, and total latency.
  Absent phases are `null`. Fresh release includes instance deletion; pooled
  release returns a lease. Cold pooled preparation samples are retained
  separately and include snapshot capture.
- `wall_ms` and tasks/second cover the concurrent task loop. `fixture_wall_ms`
  also includes pooled preparation and final instance teardown, allowing an
  amortized comparison. Server startup and teardown are separate measurements.
  Build-tool time and Wasm loading are excluded.
- Peak RSS is the largest **sampled sum** of the worker and its live descendants,
  including the owned PocketIC server and any sandbox processes. Sampling is
  every 10 ms plus sampling overhead, across setup, work, and teardown. It excludes
  Cargo and the driver. Threads are not counted twice; shared pages across
  processes can be counted more than once. This is not unique physical memory
  or a guarantee of capturing short peaks.

Lower wall time at capacity two can require more memory. Report both, and avoid
adding timing thresholds to correctness tests. The existing 100-restore test
continues to guard correctness independently of this benchmark.

## Initial sample, 2026-10-04

PocketIC 16.0.0 on WSL2 Linux (64 logical CPUs), Rust 1.99.0, host release
profile and probe dev profile: two workers, 12 tasks per mode, three rotating
repeats, and 1 MiB of seeded state in each of two canisters. These are medians
across repeats; peak memory is the sampled process-tree RSS sum described above.

| Mode | Warm/task-loop wall | Including preparation and instance teardown | Tasks/s | Sampled peak MiB |
| --- | ---: | ---: | ---: | ---: |
| Fresh | 8.24 s | 8.24 s | 1.46 | 1090 |
| Pooled, capacity 1 | 1.13 s | 2.62 s | 10.62 | 566 |
| Pooled, capacity 2 | 0.59 s | 3.39 s | 20.47 | 765 |

Capacity two improves steady-state throughput here, while capacity one has the
lower total cost for this short batch. Warm validation averaged 34–37 ms and
snapshot restoration 16–17 ms. These results describe this controlled probe
workload, not downstream suite performance.

Reproduce the sample by replacing `--iterations 100` with `--iterations 12` in
the command above. Inspect the saved per-task phases and cold preparation before
comparing changes; a different profile, Wasm, host, or workload is a new baseline.

## Longer pooled batches

With the same profiles, Wasm, host, state size, and two workers, three alternating
repeats of 100 tasks per pooled mode gave the following medians. All 600 measured
tasks validated restored state and their subsequent mutations successfully.

| Mode | Task-loop wall | Including preparation and instance teardown | Tasks/s | Sampled peak MiB |
| --- | ---: | ---: | ---: | ---: |
| Pooled, capacity 1 | 9.42 s | 10.93 s | 10.61 | 559 |
| Pooled, capacity 2 | 4.62 s | 7.38 s | 21.66 | 770 |

Capacity two amortizes its extra preparation cost in this longer batch: total
fixture time is about 32% lower, with greater sampled memory use. These results
support using capacity two for this probe's long batches when the memory budget
allows it; they do not establish an optimal capacity for other workloads.

Repeat this comparison without rebuilding fresh fixtures for every task:

```sh
python3 scripts/dev/benchmark-fixture-reuse.py \
  --server /absolute/path/to/pocket-ic \
  --modes pooled --capacities 1 2 --iterations 100 --workers 2 --repeats 3 \
  --state-bytes 1048576 --output /tmp/fixture-reuse-long.json
```

The default measures fresh fixtures and pooled capacities one and two. Selected
flows and pool capacities are recorded in the JSON, with raw samples grouped by
case (`fresh`, `pooled-1`, `pooled-2`, and so on).

## Capacity sweeps

Compare any positive pool capacities with one shared worker budget:

```sh
python3 scripts/dev/benchmark-fixture-reuse.py \
  --server /absolute/path/to/pocket-ic \
  --modes pooled --capacities 1 2 4 8 \
  --iterations 100 --workers 8 --repeats 3 \
  --state-bytes 1048576 --output /tmp/fixture-reuse-capacities.json
```

Use enough workers to exercise the largest capacity. A capacity above the worker
count still prepares every slot, but cannot increase simultaneous task execution.
Compare capacity wait, throughput, preparation-inclusive time, and sampled RSS
together. The driver does not choose a capacity automatically. Every warm
acquisition still validates restored state before running the task. The earlier
tables retain their original two-worker measurements; they do not predict the
results of this eight-worker sweep.

## Query latency investigation

A focused release-profile probe used the same Wasm and server, one instance,
two canisters, and 12 mutation/restore cycles. After each restore it queried each
canister four times. Raw `PocketIc::query_call` and testkit's `query_candid`
alternated, including which path made the first query. Raw calls timed Candid
encoding, the upstream call, and decoding separately. Every reply had to match
the restored counter, length, and first byte.

With 1 MiB per canister, the first post-restore query averaged 16.48 ms through
the typed helper and 16.78 ms through the raw API. Subsequent queries averaged
1.67 and 1.68 ms respectively. Raw-call encoding plus decoding averaged
0.024 ms. Repeating with zero seeded payload gave the same pattern: first
queries averaged 16.56–16.57 ms and subsequent queries 1.72–1.81 ms.

The observed first-query cost is in PocketIC's call path, rather than testkit's
Candid processing or pool bookkeeping. This probe does not distinguish upstream
client polling from server query preparation; zero seeded payload also does not
mean zero Wasm memory. It provides no evidence that changing the typed helper
would improve long runs. Retain validation after every restore: adding a warmup
query would only move the measured cost and add another call.

Focused sampler checks:

```sh
python3 scripts/dev/test-fixture-reuse-benchmark.py
```

## Released 0.14.7 baseline

Revision `87d4dd282804bc4a74554507ea7a3e344161a3cc` was measured with a
clean working tree on 2026-10-04: PocketIC 16.0.0, WSL2 Linux, 64 logical CPUs,
Rust 1.99.0, host release profile, and probe dev profile. The probe Wasm SHA-256
was `210fa3962038089637dbaa4f4763ac90647451190879b2e8ef2e2bebed4f6259`.
The longer-batch command above ran 100 tasks per mode, two workers, three
alternating repeats, and 1 MiB of state per canister. All 600 measured tasks
validated their restored state and subsequent mutations.

| Mode | Median task-loop wall | Median including preparation and instance teardown | Median tasks/s | Median sampled peak MiB |
| --- | ---: | ---: | ---: | ---: |
| Pooled, capacity 1 | 10.58 s | 12.17 s | 9.45 | 564 |
| Pooled, capacity 2 | 5.04 s | 7.87 s | 19.83 | 774 |

Capacity two reduced task-loop wall time by about 52% and total fixture time by
35%, with about 37% more sampled process-tree RSS. This compares capacities on
the same release; it does not measure a release-to-release speedup. The earlier
samples have a different Wasm identity and are separate baselines.

Across the 300 tasks at each capacity, mean capacity wait was 103.61 ms at
capacity one and 0.00049 ms at capacity two. Mean restore time was 18.65/17.22 ms,
validation 36.66/36.32 ms, and task body 49.27/47.79 ms respectively. Reset and
readiness hooks remain no-ops in this probe. These results support allowing two
leases for this workload when memory permits. They do not justify weakening
restore validation or predict another suite's optimal capacity.

## Warm artifact setup on 0.14.7

A separate exploratory release-profile probe measured six existing public
acquisition paths at the same revision and compiler. Its dependency versions
and checksums matched the workspace lockfile. Each artifact size used 30
acquisitions per mode and three rotating repeats, for 540 measured acquisitions.
All 1,080 acquisitions were cache hits, retained the expected fingerprint where
applicable, and returned bytes identical to the seeded artifact. Byte checks
ran after timing stopped.

The private, dependency-free Cargo workspace contained one dev-profile Wasm
crate and 64 additional source files containing 1 MiB of comments in total.
The extra files exercise conservative source hashing, even though they are not
compiled modules. Exported data arrays of 1 MiB and 16 MiB produced artifacts of
2,722,789 and 18,451,433 bytes, including debug information. Measurements were
serial, with warm filesystem caches and no lock contention. Initial build,
session seeding, snapshot preparation, and transaction seeding were excluded.
Sources had no concurrent writer; tool executables, configuration, and relevant
environment values were held constant throughout the experiment.

| Warm path | Mean wall ms, 2.60 MiB Wasm | Mean wall ms, 17.60 MiB Wasm |
| --- | ---: | ---: |
| Ordinary `build_wasm_canisters_cached` | 190.75 | 211.19 |
| Seeded `WasmBuildSession::build_batch` | 5.68 | 25.94 |
| Prepared `WasmBuildInputSnapshot::build_batch` | 5.72 | 25.51 |
| Seeded session with public Wasm removed before each acquisition | 14.05 | 53.51 |
| `prepare_artifact_cache`, retained Wasm input | 16.89 | 79.39 |
| Same transaction with `with_cargo_build_inputs` | 206.16 | 271.24 |

The transaction recipe copied one retained Wasm into one public output; it did
not run an optimizer. The seed build record stayed alive throughout processing.
Removing the public Wasm before each rematerialization sample was outside the
timed region. Sessions and snapshots used their full immutability contract;
holding an unrelated mutex alone does not establish that contract.

Ordinary Wasm input resolution averaged 184.79/184.45 ms: tool identity
88.69/88.54 ms, Cargo metadata 91.90/91.62 ms, discovery 0.44/0.44 ms, and hashing
3.74/3.83 ms. Warm leased paths reported zero input-resolution time because
their immutable input snapshots were already prepared. Uncontended exact-lock
wait averaged at most 0.003 ms; this does not include every lock-related filesystem
operation or predict contended behavior.

For the transaction with only a retained Wasm input, mean input capture was
5.25/36.05 ms, cache lookup 1.90/11.58 ms, and materialization 7.45/29.21 ms.
Adding the Cargo guard raised input capture to 193.01/226.35 ms. These guards
check the complete Cargo identity as well as source content; they are needed
when the external operation depends on those inputs. The remaining Wasm record
time includes artifact verification, publication, retention, and metadata writes;
the available successful-build timings do not isolate those costs. The installed
`perf` launcher lacked support for this WSL kernel, so no CPU samples were taken.

The measured choices are:

- Reuse a session for sequential acquisition or a prepared snapshot for shared
  readers only while the caller prevents changes to every covered executable,
  source, manifest, configuration, additional input, and relevant environment
  value. The ordinary path retains full revalidation when that guarantee is
  unavailable. This experiment did not measure concurrent prepared readers.
- For a transformation that consumes only retained Wasm, declare that Wasm,
  transformation tools, arguments, environment, and other actual inputs. Keep
  the build record alive. Add a Cargo-input guard when the transformation also
  depends on the Cargo workspace or resolved build identity.
- Preserve content checks and atomic publication. Public outputs are mutable,
  cache contents can be corrupted, and input changes can race acquisition.
  Larger artifacts make those costs visible; the measurements do not establish
  a safe reason to remove them.

These are controlled setup measurements, not downstream test-suite speedups.
Savings only affect suites that repeat the measured setup operations; they do
not reduce PocketIC query or restore latency.

## 0.14.8 output-reuse comparison

The output-reuse change retains the lengths and digests already
computed while validating a transactional cache entry. On a hit, it checks
each public output against that verified information before deciding whether
to copy. On Unix, a matching regular file owned by the effective user, with one
link, owner read/write permissions, and no executable or special permission bits
remains in place. Links, foreign-owned files, restricted files, missing files,
and changed contents still take the atomic replacement path. Cold commits always publish, and non-Unix hosts keep
the existing replacement behavior. Cache schema and content validation, input
revalidation, locking, and retention are unchanged. Matching files retain their
inode, modification time, and permissions; acquisitions do not promise a fresh
modification time on every hit.

A focused probe compared release-profile binaries built from released revision
`87d4dd2` and that revision plus the ownership-checked output fast path and
fixed 64 KiB heap hashing buffer, before the small-file buffer-sizing and label
ordering follow-ups below. Both used the same compiler
and locked dependencies described above. One fixed private Wasm input and one
public output exercised ordinary transactional acquisition without a Cargo
guard. Both binaries reused the same private cache and input bytes. Three paired
process runs alternated before/after, after/before, then before/after; the size
order also alternated. Each process measured 30 acquisitions per condition, for
90 samples per size, condition, and implementation. Cold preparation, deliberate
destination deletion or corruption, and byte verification were outside timing.
All 1,440 acquisitions reused the expected key and returned identical public and
retained bytes.

| Destination before acquisition | 2.60 MiB: baseline mean ms | 2.60 MiB: updated mean ms | 17.60 MiB: baseline mean ms | 17.60 MiB: updated mean ms |
| --- | ---: | ---: | ---: | ---: |
| Matching | 15.70 | 10.48 | 77.55 | 59.09 |
| Missing | 15.02 | 14.76 | 73.60 | 72.99 |
| Changed length | 15.53 | 15.04 | 75.76 | 74.39 |
| Changed contents, same length | 15.72 | 17.32 | 77.43 | 87.56 |

Matching-output acquisition was about 33%/24% faster in this probe. Missing and
different-size repairs stayed within about 3% of baseline; these small changes
do not establish an improvement or regression. A same-size mismatch needs an
extra digest read before copying and was about 10%/13% slower. The fast path
therefore benefits repeated acquisition of stable public outputs, with a measured
cost when their contents change without changing length. No downstream suite
speedup or concurrent throughput improvement was measured.

Focused checks cover unchanged inode/time, same-length corruption, missing and
explicitly valid empty outputs, detachment of matching symlinks and hard links,
restricted and executable permissions, and unchanged cold publication. Existing
checks continue to cover corrupt retained entries, Cargo input guards, process
coordination, batch reuse, and consumption during pruning.

## Streamed hashing buffer follow-up

A direct hashing probe compiled the actual before/after digest modules into one
release-profile executable and alternated their order over five pairs per file
size. The baseline uses a 16 KiB stack buffer; the initial implementation uses a
fixed 64 KiB heap buffer. Each small-file batch hashed 10,000 times; each artifact
batch hashed 100 times. The final digest in every batch matched the baseline,
and all calls completed. There were 50 timed batches and 302,000 hashing
operations. Inputs were private, immutable during timing, and resident in warm
filesystem caches. Both modules used the same SHA-256 implementation and framing.

| File size | Mean per hash, 16 KiB buffer | Mean per hash, 64 KiB heap buffer |
| --- | ---: | ---: |
| Empty | 4.79 microseconds | 5.19 microseconds |
| 1 KiB | 5.99 microseconds | 6.38 microseconds |
| 16 KiB | 13.13 microseconds | 13.60 microseconds |
| 2.60 MiB | 1.43 ms | 1.35 ms |
| 17.60 MiB | 11.68 ms | 11.17 ms |

The fixed larger buffer showed roughly 4–5% lower large-file hashing cost.
Small-file hashing was 4–8% slower, an additional 0.4–0.5 microseconds per call.
The buffer is on the heap; thread stack usage is not increased by a larger local
array. There is no new buffer setting or digest format. These measurements cover
Linux on this host, not cold storage,
other platforms, or complete downstream test runs.

The final implementation sizes the buffer to the opened file's declared length,
with a one-byte minimum and a 64 KiB maximum. This avoids a full-size allocation
for small source files while retaining large-file read sizes. The nonempty
minimum allows empty-file growth to be detected. Reading beyond the declared
length stops immediately and returns the existing size-mismatch error; reaching
EOF before that length still fails. The maintained streaming regression checks
empty and small files, the buffer boundary, and a partial final chunk. A local
read-only probe of `/proc/self/cmdline`, which declares zero length but returns
bytes, confirmed rejection; it is not retained as a platform-specific host-file
test.

A follow-up compiled the released, fixed-buffer, and final length-sized-buffer
modules together and alternated them over five pairs per size. All 75 batches
completed 453,000 hash operations and every final digest matched the released
implementation. Timing results were highly variable: host load exceeded 98,
and identical fixed-buffer 2.60 MiB batches varied from about 2 to 89 ms per
hash. Those timings are excluded from performance claims. The tables above
describe the earlier fixed-buffer experiments, not measurements of this final
small-file refinement. The follow-up sources and raw results are retained in
the local review bundle.

Artifact-spec builders also store opaque identity and Cargo input labels in
canonical order, as they already do for output names. Key calculation consumes
that order directly instead of allocating and sorting temporary reference lists
on every source check. Duplicate labels are still rejected before acquisition;
existing key framing and Cargo source guards are preserved. Reordered identity
and Cargo declarations now compare equal as specifications. This ownership
cleanup has no measured performance claim.

A read-only probe using a foreign-owned regular file also confirmed that
permission bits alone allowed a false destination match. The same probe
rejected reuse after the effective-user ownership check was added. The maintained
tests use private fixtures; this local probe is not retained as a test that
depends on host filesystem layout.

## 0.14.9 Wasm publication follow-up

Wasm acquisition now gives a verified retained entry authority over its public
outputs. Independently stamped public bytes cannot override a valid exact entry
for the same fingerprint. Verified lengths and digests from exact entries are
retained through publication, so public stamp generation does not reread copied
Wasm files. Recovery from mutable public files instead hashes the newly copied
private files before stamping them and rejects empty copies, removing the
incomplete entry on failure. Public files and stamps are repaired independently;
a missing stamp alone need not cause a
Wasm copy. Matching independent writable Unix files owned by the effective user
retain their inode and modification time. Cold and non-Unix publication use
atomic replacement, and links or restricted files are replaced.

These changes have correctness and filesystem-behavior verification, including
a regression reproduced on separately compiled released `0.14.8`. They have no
new runtime measurements. The earlier tables describe their recorded versions;
they are not measurements of `0.14.9` or downstream test-suite speedups.
