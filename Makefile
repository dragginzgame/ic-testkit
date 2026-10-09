.PHONY: \
	build-test-canisters check check-wasm ci clean \
	clippy docs-check ensure-clean git-hooks-check help \
	install-format-tools install-hooks installation-check msrv package publish \
	publish-dry-run publish-guards-check release-check \
	release-guards-check \
	release-version release-preflight release-verify release-prepare-version \
	release-prepared-check release-files release-commit-check release-committed-check \
	release-tagged-check release-push-check \
	release-tag-check shared-tooling-check tags test test-canisters version

REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
IC_TOOL_PINS ?= $(REPO_ROOT)ci/ic-tools.tsv
HOST_TOOL_VERSIONS ?= $(REPO_ROOT)ci/tool-versions.env

SHARED_TOOLING_ROOT := $(REPO_ROOT)
include $(REPO_ROOT)make/tools.mk
include $(REPO_ROOT)make/release.mk
include $(REPO_ROOT)make/rust-format.mk

.PHONY: dependency-pins-check
.PHONY: build-server-cli install-server server-check

build-server-cli:
	cargo build -p ic-testkit --locked --bin ic-testkit-server --target-dir "$(REPO_ROOT)target"

install-server: build-server-cli
	target/debug/ic-testkit-server setup

server-check: build-server-cli
	target/debug/ic-testkit-server check

MSRV ?= 1.88.0
.DEFAULT_GOAL := help
CI_TARGETS := shared-tooling-check tools-check dependency-pins-check installation-check publish-guards-check \
	release-guards-check git-hooks-check fmt-check check check-wasm clippy docs-check test \
	package publish-dry-run

RELEASE_CHECK_TARGETS := $(CI_TARGETS) msrv

help:
	@echo "Available commands:"
	@echo ""
	@echo "  install-tools   Explicitly install pinned jq/yq, ripgrep, cloc and IC executables"
	@echo "  install-server  Build the owner CLI and explicitly prepare PocketIC"
	@echo "  server-check    Build the owner CLI and verify the prepared bundle offline"
	@echo "  tools-check     Verify installed host and IC tools offline"
	@echo "  cloc            Count this workspace's Rust code"
	@echo "  dependency-pins-check Check dependency declarations and tracked lockfiles offline"
	@echo "  install-format-tools Install the pinned checkout-local Rust tool bundle"
	@echo "  install-hooks   Enable the repository-local formatting hook"
	@echo "  fmt             Sort Cargo manifests and format all workspace Rust code"
	@echo "  fmt-check       Check manifest sorting and Rust formatting without mutation"
	@echo "  git-hooks-check Check hook preservation and installation in isolated fixtures"
	@echo "  check           Check the host crate with locked dependencies"
	@echo "  check-wasm      Check the crate for wasm32"
	@echo "  clippy          Run Clippy with warnings denied"
	@echo "  docs-check      Build public API documentation with warnings denied"
	@echo "  installation-check Check current README dependency requirements"
	@echo "  shared-tooling-check Verify the reviewed shared snapshot offline"
	@echo "  test            Run the ic-testkit test suite"
	@echo "  test-canisters  Run the PocketIC canister integration test"
	@echo "  msrv            Check the crate with the declared MSRV"
	@echo "  package         Build and verify the publishable crate"
	@echo "  ci              Run the local push gate"
	@echo "  release-check   Run the complete release gate, including MSRV"
	@echo "  version         Show the current workspace package version"
	@echo "  tags            List recent version tags"
	@echo "  release-patch   Verify, then bump, stage, commit, tag, and push a patch release"
	@echo "  release-minor   Verify, then bump, stage, commit, tag, and push a minor release"
	@echo "  release-major   Verify, then bump, stage, commit, tag, and push a major release"
	@echo "  release-resume VERSION=X.Y.Z Resume the exact saved release"
	@echo "  publish         Publish the tagged release to crates.io"

ensure-clean:
	@untracked="$$(git ls-files --others --exclude-standard)" || exit $$?; \
	if ! git diff-index --quiet HEAD -- || test -n "$$untracked"; then \
		echo "error: working directory is not clean; commit or stash changes first" >&2; \
		exit 1; \
	fi

version:
	@bash "$(REPO_ROOT)scripts/ci/read-cargo-workspace-version.sh" Cargo.toml

tags:
	@git tag --sort=-version:refname | head -10

test: build-server-cli
	@server="$$(target/debug/ic-testkit-server check)" || exit $$?; \
		POCKET_IC_BIN="$$server" cargo test -p ic-testkit --locked && \
		POCKET_IC_BIN="$$server" cargo test -p ic-testkit --locked --test server_runner real_server_runs_a_separate_process_using_environment_startup -- --ignored --exact

build-test-canisters:
	CARGO_TARGET_DIR=target/pic-wasm cargo build --locked --target wasm32-unknown-unknown -p ic_testkit_perf_probe

test-canisters: build-server-cli
	@server="$$(target/debug/ic-testkit-server check)" || exit $$?; \
		POCKET_IC_BIN="$$server" cargo test -p ic-testkit --locked --test canister_benchmark -- --nocapture

install-format-tools: install-rust-tools

dependency-pins-check:
	bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

install-hooks: format-tools-check
	bash scripts/dev/install-git-hooks.sh

git-hooks-check: format-tools-check
	bash scripts/ci/check-git-hooks.sh

check:
	cargo check -p ic-testkit --locked

check-wasm:
	cargo check -p ic-testkit --locked --target wasm32-unknown-unknown

clippy:
	cargo clippy -p ic-testkit --all-targets --locked -- -D warnings

docs-check:
	RUSTDOCFLAGS="-D warnings" cargo doc -p ic-testkit --locked --no-deps

shared-tooling-check:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

msrv:
	rustc +$(MSRV) --version
	cargo +$(MSRV) --version
	cargo +$(MSRV) check -p ic-testkit --locked
	cargo +$(MSRV) check -p ic-testkit --lib --target wasm32-unknown-unknown --locked
	cargo +$(MSRV) check -p ic_testkit_perf_probe --lib --target wasm32-unknown-unknown --locked

installation-check:
	bash scripts/ci/check-installation-version.sh

publish-guards-check:
	bash scripts/ci/check-publish-guards.sh

release-guards-check:
	bash scripts/ci/check-release-guards.sh

package:
	cargo package -p ic-testkit --locked --allow-dirty

publish-dry-run:
	cargo publish -p ic-testkit --locked --dry-run --allow-dirty

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(CI_TARGETS)

release-check:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(RELEASE_CHECK_TARGETS)

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

release-version:
	@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

release-preflight:
	@bash scripts/release/metadata.sh preflight

release-verify:
	+@bash scripts/release/metadata.sh verify

release-prepare-version:
	@bash scripts/release/metadata.sh prepare

release-prepared-check:
	@bash scripts/release/metadata.sh prepared

release-files:
	@bash scripts/release/metadata.sh files

release-commit-check:
	@bash scripts/release/metadata.sh commit

release-committed-check:
	@bash scripts/release/metadata.sh committed

release-tagged-check:
	@bash scripts/release/metadata.sh tagged

release-push-check:
	@bash scripts/release/metadata.sh push

release-tag-check:
	bash "$(REPO_ROOT)scripts/release/check-tag-at-head.sh"

clean:
	cargo clean
