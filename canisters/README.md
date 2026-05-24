# Test Canisters

Small canisters used by `ic-testkit` to test its own PocketIC harness behavior.

The layout mirrors the `canisters/test/...` convention used by related repos.
These canisters are fixtures, not application examples.

## Current Fixtures

- `test/perf_probe`: emits compact `ICTK|...` benchmark markers via
  `ic_testkit::performance::Performance::measure`.

Build all current test fixtures with:

```sh
make build-test-canisters
```

Run the PocketIC fixture test with:

```sh
make test-canisters
```
