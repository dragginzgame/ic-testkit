.PHONY: \
	build-test-canisters check check-wasm ci clean \
	clippy docs-check ensure-clean git-hooks-check help \
	installation-check msrv package publish \
	publish-dry-run publish-guards-check release-check \
	release-guards-check \
	release-version release-preflight release-verify release-prepare-version \
	release-prepared-check release-files release-commit-check release-committed-check \
	release-tagged-check release-push-check \
	release-tag-check tags test test-canisters version

REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
SERVER_CLI := $(REPO_ROOT)target/debug/ic-testkit-server
.DEFAULT_GOAL := help
override SHARED_TOOLING_ROOT := $(REPO_ROOT)
LOCAL_TOOL_INSTALL_TARGETS := install-server
LOCAL_TOOL_CHECK_TARGETS := server-check
include $(REPO_ROOT)make/consumer.mk

.PHONY: dependency-pins-check
.PHONY: build-server-cli install-server server-check

build-server-cli:
	+cargo build -p ic-testkit --locked --bin ic-testkit-server --target-dir "$(REPO_ROOT)target"

install-server: build-server-cli
	"$(SERVER_CLI)" setup

server-check:
	@test -x "$(SERVER_CLI)" || { \
		echo 'Owner CLI missing; run make install-tools during explicit setup.' >&2; exit 1; }
	"$(SERVER_CLI)" check

MSRV ?= 1.91.0
CI_TARGETS := shared-tooling-check tools-check format-tools-check dependency-pins-check installation-check publish-guards-check \
	release-guards-check git-hooks-check fmt-check check check-wasm clippy docs-check test \
	package publish-dry-run

RELEASE_CHECK_TARGETS := $(CI_TARGETS) msrv
VALIDATION_INPUTS := Makefile Cargo.toml Cargo.lock rust-toolchain.toml \
	crates/ic-testkit/Cargo.toml crates/ic-testkit-candid/Cargo.toml crates/ic_testkit_perf_probe/Cargo.toml \
	ci/tool-versions.env ci/ic-tools.tsv .shared-tooling.snapshot

help:
	@echo "Available commands:"
	@echo ""
	@echo "  install-tools   Prepare common host, IC and Cargo tools, then PocketIC"
	@echo "  install-server  Build the owner CLI and explicitly prepare PocketIC"
	@echo "  server-check    Verify the prepared owner CLI and PocketIC bundle offline"
	@echo "  tools-check     Verify common and product tools offline"
	@echo "  cloc            Count this workspace's Rust code"
	@echo "  dependency-pins-check Check dependency declarations and tracked lockfiles offline"
	@echo "  install-hooks   Enable the repository-local formatting hook"
	@echo "  fmt             Sort Cargo manifests and format all workspace Rust code"
	@echo "  fmt-check       Check manifest sorting and Rust formatting without mutation"
	@echo "  git-hooks-check Check hook preservation and installation in isolated fixtures"
	@echo "  check           Check the libraries with locked dependencies"
	@echo "  check-wasm      Check the libraries for wasm32"
	@echo "  clippy          Run Clippy with warnings denied"
	@echo "  docs-check      Build public API documentation with warnings denied"
	@echo "  installation-check Check current README dependency requirements"
	@echo "  shared-tooling-check Verify the reviewed shared snapshot offline"
	@echo "  test            Run the published packages' test suites"
	@echo "  test-canisters  Run the PocketIC canister integration test"
	@echo "  msrv            Check the workspace with the declared MSRV"
	@echo "  package         Build and verify the publishable crates"
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
	+cargo test -p ic-testkit-candid --locked
	+@server="$$("$(SERVER_CLI)" check)" || exit $$?; \
		POCKET_IC_BIN="$$server" cargo test -p ic-testkit --locked && \
		POCKET_IC_BIN="$$server" cargo test -p ic-testkit --locked --test server_runner real_server_runs_a_separate_process_using_environment_startup -- --ignored --exact

build-test-canisters:
	+CARGO_TARGET_DIR=target/pic-wasm cargo build --locked --target wasm32-unknown-unknown -p ic_testkit_perf_probe

test-canisters: build-server-cli
	+@server="$$("$(SERVER_CLI)" check)" || exit $$?; \
		POCKET_IC_BIN="$$server" cargo test -p ic-testkit --locked --test canister_benchmark -- --nocapture

dependency-pins-check:
	bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

install-hooks: format-tools-check

git-hooks-check: format-tools-check
	bash scripts/ci/check-git-hooks.sh

check:
	+cargo check -p ic-testkit -p ic-testkit-candid --locked

check-wasm:
	+cargo check -p ic-testkit -p ic-testkit-candid --locked --target wasm32-unknown-unknown

clippy:
	+cargo clippy -p ic-testkit -p ic-testkit-candid --all-targets --locked -- -D warnings

docs-check:
	+RUSTDOCFLAGS="-D warnings" cargo doc -p ic-testkit -p ic-testkit-candid --locked --no-deps

msrv:
	rustc +$(MSRV) --version
	cargo +$(MSRV) --version
	+cargo +$(MSRV) check -p ic-testkit-candid --all-targets --locked
	+cargo +$(MSRV) check -p ic-testkit-candid --all-targets --locked --target wasm32-unknown-unknown
	+cargo +$(MSRV) check -p ic-testkit --locked
	+cargo +$(MSRV) check -p ic-testkit --lib --target wasm32-unknown-unknown --locked
	+cargo +$(MSRV) check -p ic_testkit_perf_probe --lib --target wasm32-unknown-unknown --locked

installation-check:
	bash scripts/ci/check-installation-version.sh

publish-guards-check:
	bash scripts/ci/check-publish-guards.sh

release-guards-check:
	perl scripts/ci/test-local-lock-versions.pl
	bash scripts/ci/check-release-guards.sh

package:
	+bash scripts/ci/check-packages.sh package

publish-dry-run:
	+bash scripts/ci/check-packages.sh publish-dry-run

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(addprefix --input ,$(VALIDATION_INPUTS)) $(CI_TARGETS)

release-check:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(addprefix --input ,$(VALIDATION_INPUTS)) $(RELEASE_CHECK_TARGETS)

publish: ensure-clean release-tag-check
	+bash scripts/release/publish-workspace.sh

release-version:
	@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

release-preflight:
	+@bash scripts/release/metadata.sh preflight

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
