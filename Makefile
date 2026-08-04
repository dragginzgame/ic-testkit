.PHONY: \
	actions-check build-test-canisters changelog-check check check-wasm ci clean \
	clippy ensure-clean fmt fmt-check help msrv package patch publish \
	minor publish-dry-run publish-guards-check release-check release-commit \
	release-guards-check release-minor release-patch release-push release-stage \
	release-tag-check tags test test-canisters version

REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

MSRV ?= 1.88.0
CHANGELOG_VERSION ?=

CI_TARGETS := changelog-check actions-check publish-guards-check \
	release-guards-check fmt-check check check-wasm clippy test \
	build-test-canisters test-canisters package publish-dry-run

RELEASE_CHECK_TARGETS := $(CI_TARGETS) msrv

help:
	@echo "Available commands:"
	@echo ""
	@echo "  fmt             Format Rust code"
	@echo "  fmt-check       Check Rust formatting"
	@echo "  check           Check the host crate with locked dependencies"
	@echo "  check-wasm      Check the crate for wasm32"
	@echo "  clippy          Run Clippy with warnings denied"
	@echo "  test            Run the ic-testkit test suite"
	@echo "  test-canisters  Run the PocketIC canister integration test"
	@echo "  msrv            Check the crate with the declared MSRV"
	@echo "  package         Build and verify the publishable crate"
	@echo "  ci              Run the local push gate"
	@echo "  release-check   Run the complete release gate, including MSRV"
	@echo "  version         Show the current workspace package version"
	@echo "  tags            List recent version tags"
	@echo "  patch           Run CI, then bump patch-version files"
	@echo "  minor           Run CI, then bump minor-version files"
	@echo "  release-patch   Bump, stage, commit, tag, verify, and push a patch release"
	@echo "  release-minor   Bump, stage, commit, tag, verify, and push a minor release"
	@echo "  publish         Publish the tagged release to crates.io"

ensure-clean:
	@if ! git diff-index --quiet HEAD -- || test -n "$$(git ls-files --others --exclude-standard)"; then \
		echo "error: working directory is not clean; commit or stash changes first" >&2; \
		exit 1; \
	fi

version:
	@sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1

tags:
	@git tag --sort=-version:refname | head -10

test:
	cargo test -p ic-testkit --locked

build-test-canisters:
	CARGO_TARGET_DIR=target/pic-wasm cargo build --locked --target wasm32-unknown-unknown -p ic_testkit_perf_probe

test-canisters: build-test-canisters
	cargo test -p ic-testkit --locked --test canister_benchmark -- --nocapture

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check -p ic-testkit --locked

check-wasm:
	cargo check -p ic-testkit --locked --target wasm32-unknown-unknown

clippy:
	cargo clippy -p ic-testkit --all-targets --locked -- -D warnings

msrv:
	cargo +$(MSRV) check -p ic-testkit --locked

actions-check:
	bash scripts/ci/check-github-actions-pinned.sh

changelog-check:
	bash scripts/ci/check-changelog-version.sh $(CHANGELOG_VERSION)

publish-guards-check:
	bash scripts/ci/check-publish-guards.sh

release-guards-check:
	bash scripts/ci/check-release-guards.sh

package:
	cargo package -p ic-testkit --locked --allow-dirty

publish-dry-run:
	cargo publish -p ic-testkit --locked --dry-run --allow-dirty

ci:
	+@set -e; for target in $(CI_TARGETS); do \
		$(MAKE) --no-print-directory "$$target"; \
	done

release-check:
	+@set -e; for target in $(RELEASE_CHECK_TARGETS); do \
		$(MAKE) --no-print-directory "$$target"; \
	done

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

patch:
	bash scripts/release/bump-version.sh patch

minor:
	bash scripts/release/bump-version.sh minor

release-patch:
	+$(MAKE) --no-print-directory patch
	+$(MAKE) --no-print-directory release-stage
	+$(MAKE) --no-print-directory release-commit
	+$(MAKE) --no-print-directory release-push

release-minor:
	+$(MAKE) --no-print-directory minor
	+$(MAKE) --no-print-directory release-stage
	+$(MAKE) --no-print-directory release-commit
	+$(MAKE) --no-print-directory release-push

release-stage:
	git add Cargo.toml Cargo.lock

release-commit:
	@set -eu; \
	version="$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"; \
	if [ -z "$$version" ]; then \
		echo "error: failed to read package version from Cargo.toml" >&2; \
		exit 1; \
	fi; \
	if git rev-parse "v$$version" >/dev/null 2>&1; then \
		echo "error: tag v$$version already exists; aborting" >&2; \
		exit 1; \
	fi; \
	git commit -m "Release $$version"; \
	git tag -a "v$$version" -m "Release $$version"

release-tag-check:
	bash "$(REPO_ROOT)scripts/release/check-tag-at-head.sh"

release-push: ensure-clean release-tag-check
	+$(MAKE) --no-print-directory ci
	git push --follow-tags

clean:
	cargo clean
