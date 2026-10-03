# Test Canisters

Small canisters used by `ic-testkit` to test its own PocketIC harness behavior.

The layout mirrors the `canisters/test/...` convention used by related repos.
These canisters are fixtures, not application examples.

## Current Fixtures

- `test/perf_probe`: emits compact `ICTK|...` benchmark markers via
  `ic_testkit::performance::Performance::measure`. It exposes `ping`,
  `benchmark_once`, and `benchmark_start_then_trap` to exercise successful
  spans and unmatched start markers after a trap.

Install the Wasm target for the repository toolchain first:

```sh
rustup target add wasm32-unknown-unknown
```

Build all current test fixtures with:

```sh
make build-test-canisters
```

This builds `ic_testkit_perf_probe.wasm` under
`target/pic-wasm/wasm32-unknown-unknown/debug/`.

Run only the live marker fixture test with:

```sh
cargo test -p ic-testkit --locked --test canister_benchmark \
  perf_probe_canister_emits_parseable_benchmark_markers -- --exact --nocapture
```

The test builds its own content-addressed Wasm in a temporary target directory;
it does not consume the standalone build above. It uses upstream PocketIC
startup, which may download the server. Set `POCKET_IC_BIN=/path/to/pocket-ic`
to use an existing PocketIC 16 binary.

Run the complete `canister_benchmark` integration target, including
artifact-cache and orchestration coverage, with:

```sh
make test-canisters
```

These tests build their own artifacts. The main CI test command already
includes this integration target; the standalone fixture build is a manual
command, not a prerequisite for testing.
