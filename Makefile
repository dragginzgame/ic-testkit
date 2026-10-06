.PHONY: \
	build-test-canisters check check-wasm ci clean \
	clippy docs-check ensure-clean fmt fmt-check format-tools-check git-hooks-check help \
	install-format-tools install-hooks installation-check msrv package publish \
	publish-dry-run publish-guards-check release-check \
	release-guards-check release-minor release-patch release-major release-resume \
	release-version release-preflight release-verify release-prepare-version \
	release-prepared-check release-files release-commit-check release-committed-check \
	release-tagged-check release-push-check \
	release-tag-check shared-tooling-check tags test test-canisters version

REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
IC_TOOL_PINS ?= $(REPO_ROOT)ci/ic-tools.tsv
HOST_TOOL_VERSIONS ?= $(REPO_ROOT)ci/tool-versions.env
export PATH := $(REPO_ROOT).tools/host/bin:$(REPO_ROOT).tools/ic/bin:$(PATH)
export POCKET_IC_BIN ?= $(REPO_ROOT).tools/ic/bin/pocket-ic

.PHONY: install-tools tools-check install-host-tools host-tools-check install-ic-tools ic-tools-check dependency-pins-check

MSRV ?= 1.88.0
.DEFAULT_GOAL := help
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

CI_TARGETS := shared-tooling-check tools-check dependency-pins-check installation-check publish-guards-check \
	release-guards-check git-hooks-check fmt-check check check-wasm clippy docs-check test \
	package publish-dry-run

RELEASE_CHECK_TARGETS := $(CI_TARGETS) msrv

help:
	@echo "Available commands:"
	@echo ""
	@echo "  install-tools   Explicitly install pinned local jq/yq and IC executables"
	@echo "  tools-check     Verify installed host and IC tools offline"
	@echo "  dependency-pins-check Check dependency declarations and tracked lockfiles offline"
	@echo "  install-format-tools Install the pinned manifest formatter during setup"
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
	@bash "$(REPO_ROOT)scripts/release/read-workspace-version.sh" Cargo.toml

tags:
	@git tag --sort=-version:refname | head -10

test:
	cargo test -p ic-testkit --locked

build-test-canisters:
	CARGO_TARGET_DIR=target/pic-wasm cargo build --locked --target wasm32-unknown-unknown -p ic_testkit_perf_probe

test-canisters:
	cargo test -p ic-testkit --locked --test canister_benchmark -- --nocapture

format-tools-check:
	bash scripts/ci/check-format-tools.sh

install-format-tools:
	@. "$(REPO_ROOT)ci/tool-versions.env"; \
		cargo install cargo-sort --version "$$SHARED_TOOLING_CARGO_SORT_VERSION" --locked

install-tools:
	+$(MAKE) --no-print-directory install-host-tools
	+$(MAKE) --no-print-directory install-ic-tools

tools-check:
	+$(MAKE) --no-print-directory host-tools-check
	+$(MAKE) --no-print-directory ic-tools-check

install-host-tools:
	bash scripts/dev/install-host-tools.sh --versions "$(HOST_TOOL_VERSIONS)"

host-tools-check:
	bash scripts/dev/install-host-tools.sh --versions "$(HOST_TOOL_VERSIONS)" --check

install-ic-tools:
	bash scripts/dev/install-ic-tools.sh --pins "$(IC_TOOL_PINS)"

ic-tools-check:
	bash scripts/dev/install-ic-tools.sh --pins "$(IC_TOOL_PINS)" --check

dependency-pins-check:
	bash scripts/ci/check-dependency-pins.sh

install-hooks: format-tools-check
	bash scripts/dev/install-git-hooks.sh

fmt: format-tools-check
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo sort --workspace
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo fmt --all

fmt-check: format-tools-check
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo sort --workspace --check
	CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check

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
	cargo +$(MSRV) check -p ic-testkit --locked

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
	+@set -e; for target in $(CI_TARGETS); do \
		$(MAKE) --no-print-directory "$$target"; \
	done

release-check:
	+@set -e; for target in $(RELEASE_CHECK_TARGETS); do \
		$(MAKE) --no-print-directory "$$target"; \
	done

publish: ensure-clean release-tag-check
	bash scripts/release/publish-workspace.sh

release-patch release-minor release-major:
	+@bash scripts/ci/run-release.sh "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	+@bash scripts/ci/run-release.sh resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-version:
	@bash scripts/release/read-workspace-version.sh --stable Cargo.toml

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
