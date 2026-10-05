#!/usr/bin/env bash
set -euo pipefail

# Qualify the reviewed hook against this consumer's actual formatting targets
# and tracked lockfile. No commits, builds, tool installation or network effects.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-git-hooks.XXXXXX")"
cleanup() {
    local status=$?
    if [[ "$status" -eq 0 ]]; then
        rm -rf -- "$fixture"
    else
        echo "Failed hook qualification retained at $fixture" >&2
    fi
}
trap cleanup EXIT
source_commit="$(git -C "$root" rev-parse HEAD)"
source_objects="$(git -C "$root" rev-parse --git-path objects)"
case "$source_objects" in /*) ;; *) source_objects="$root/$source_objects" ;; esac
mkdir "$fixture/templates"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$fixture/templates"

new_fixture() {
    mkdir "$fixture/$1"
    cd "$fixture/$1"
    git init --quiet
    mkdir -p .git/objects/info .githooks ci scripts/ci scripts/dev
    printf '%s\n' "$source_objects" > .git/objects/info/alternates
    git update-ref HEAD "$source_commit"
    git read-tree HEAD
    git checkout-index --all
    # Project the reviewed adoption, including files not yet committed locally.
    for path in Makefile Cargo.toml crates/ic-testkit/Cargo.toml \
        canisters/test/perf_probe/Cargo.toml ci/tool-versions.env \
        scripts/ci/check-format-tools.sh .githooks/pre-commit \
        scripts/dev/install-git-hooks.sh; do
        cp "$root/$path" "$path"
        git add -- "$path"
    done
    cp Cargo.lock selected-lock
}
expect_failure() {
    if "$@" > rejected.log 2>&1; then
        echo 'hook qualification unexpectedly accepted a failing input' >&2
        exit 1
    fi
}

new_fixture formatting
rust_file=canisters/test/perf_probe/src/lib.rs
printf '\npub fn hook_fixture( ){}\n' >> "$rust_file"
git add -- "$rust_file"
printf '\nUnrelated working edit.\n' >> README.md
cp README.md unrelated-before
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > formatted.log
[[ "$(git show ":$rust_file" | tail -n 1)" == 'pub fn hook_fixture() {}' ]]
git diff --quiet -- "$rust_file"
cmp unrelated-before README.md
[[ "$(git show :README.md)" == "$(git show HEAD:README.md)" ]]
cmp selected-lock Cargo.lock
[[ ! -e target ]]
make --no-print-directory fmt-check > checked.log
tree="$(git write-tree)"
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > repeated.log
[[ "$(git write-tree)" == "$tree" ]]
cmp selected-lock Cargo.lock

new_fixture partial
printf '\npub fn hook_fixture( ){}\n' >> "$rust_file"
git add -- "$rust_file"
printf '\n// Unstaged edit.\n' >> "$rust_file"
tree="$(git write-tree)"
cp "$rust_file" partially-staged-before
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]]
cmp partially-staged-before "$rust_file"
cmp selected-lock Cargo.lock

new_fixture formatter-failure
printf '\nfmt:\n\t@false\n' >> Makefile
git add Makefile
tree="$(git write-tree)"
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]]
git diff --quiet -- Makefile Cargo.toml
cmp selected-lock Cargo.lock

new_fixture installer
bash scripts/dev/install-git-hooks.sh > installed.log
[[ "$(git config --get core.hooksPath)" == .githooks ]]
bash scripts/dev/install-git-hooks.sh >> installed.log
git config --local core.hooksPath custom-hooks
expect_failure bash scripts/dev/install-git-hooks.sh
[[ "$(git config --get core.hooksPath)" == custom-hooks ]]
echo 'Consumer hook formatting, lockfile preservation, rejection and activation checks passed'
