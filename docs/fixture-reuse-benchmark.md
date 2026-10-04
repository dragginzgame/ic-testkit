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
threads. Capacity one serializes leases and measures waiting; capacity two
permits overlap. Each mode/repeat has a separate worker process and one explicit
managed PocketIC server. Instances share that server within the run. Repeat
order rotates between fresh, pooled-one, and pooled-two.

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
  --modes pooled-1 pooled-2 --iterations 100 --workers 2 --repeats 3 \
  --state-bytes 1048576 --output /tmp/fixture-reuse-long.json
```

The default still measures all three modes. Selected modes and their raw samples
are recorded in the JSON; repeat ordering rotates over the selected modes.

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
