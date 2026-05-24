MSRV := 1.88.0

.PHONY: test fmt fmt-check check clippy msrv package publish-dry-run release-check

test:
	cargo test

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check

clippy:
	cargo clippy --all-targets -- -D warnings

msrv:
	cargo +$(MSRV) check

package:
	cargo package --allow-dirty

publish-dry-run:
	cargo publish --dry-run --allow-dirty

release-check: fmt-check check clippy test msrv package publish-dry-run
