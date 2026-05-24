MSRV := 1.88.0

.PHONY: test test-canisters build-test-canisters fmt fmt-check check check-canister check-wasm-canister clippy clippy-canister msrv package publish-dry-run release-check

test:
	cargo test

test-canisters:
	cargo test --test canister_benchmark -- --nocapture

build-test-canisters:
	CARGO_TARGET_DIR=target/pic-wasm cargo build --target wasm32-unknown-unknown -p ic_testkit_perf_probe

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check

check-canister:
	cargo check --features canister

check-wasm-canister:
	cargo check --target wasm32-unknown-unknown --features canister

clippy:
	cargo clippy --all-targets -- -D warnings

clippy-canister:
	cargo clippy --all-targets --features canister -- -D warnings

msrv:
	cargo +$(MSRV) check

package:
	cargo package --allow-dirty

publish-dry-run:
	cargo publish --dry-run --allow-dirty

release-check: fmt-check check check-canister check-wasm-canister clippy clippy-canister test test-canisters build-test-canisters msrv package publish-dry-run
