MSRV := 1.88.0

.PHONY: test test-canisters build-test-canisters fmt fmt-check check check-wasm clippy msrv package publish-dry-run release-check

test:
	cargo test -p ic-testkit

test-canisters:
	cargo test -p ic-testkit --test canister_benchmark -- --nocapture

build-test-canisters:
	CARGO_TARGET_DIR=target/pic-wasm cargo build --target wasm32-unknown-unknown -p ic_testkit_perf_probe

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check -p ic-testkit

check-wasm:
	cargo check -p ic-testkit --target wasm32-unknown-unknown

clippy:
	cargo clippy -p ic-testkit --all-targets -- -D warnings

msrv:
	cargo +$(MSRV) check -p ic-testkit

package:
	cargo package -p ic-testkit --allow-dirty

publish-dry-run:
	cargo publish -p ic-testkit --dry-run --allow-dirty

release-check: fmt-check check check-wasm clippy test test-canisters build-test-canisters msrv package publish-dry-run
