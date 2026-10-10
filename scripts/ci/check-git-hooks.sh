#!/usr/bin/env bash
set -euo pipefail

# Qualify the reviewed hook against this consumer's actual formatting targets
# and tracked lockfile. No commits, builds, tool installation or network effects.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# Exercise checkout-local discovery without inheriting Make's prepared PATH.
for tool_directory in host ic rust; do
    PATH="${PATH//"$root/.tools/$tool_directory/bin:"/}"
done
export PATH
export IC_TESTKIT_HOOK_CARGO_SORT="$root/.tools/rust/bin/cargo-sort"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-git-hooks.XXXXXX")"
fixture_complete=false
cleanup() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" -eq 0 ]]; then
        rm -rf -- "$fixture"
    else
        echo "Failed hook qualification retained at $fixture" >&2
    fi
    exit "$status"
}
trap cleanup EXIT
fail() { echo "hook qualification failed: $*" >&2; exit 1; }
# System Bash 3.2 needs explicit rejection for standalone conditional commands.
# macOS temporary paths and caller-supplied TMPDIR may be logical aliases.
# Keep private fixture identities physical across subsequent Git operations.
fixture="$(cd "$fixture" && pwd -P)"
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
    for path in Makefile make/tools.mk make/release.mk make/rust-format.mk make/execution.mk Cargo.toml crates/ic-testkit/Cargo.toml \
        crates/ic_testkit_perf_probe/Cargo.toml crates/ic_testkit_perf_probe/src/lib.rs ci/tool-versions.env \
        scripts/ci/check-format-tools.sh scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh .githooks/pre-commit \
        scripts/dev/install-git-hooks.sh; do
        mkdir -p "$(dirname "$path")"
        cp "$root/$path" "$path"
        git add -- "$path"
    done
    # Prepared tools are not part of the staged export. The hook must discover
    # this original fixture's local executable even when the shell omits it.
    mkdir -p .tools/rust/bin
    cat > .tools/rust/bin/cargo-sort <<'FORMATTER'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${BASH_SOURCE[0]%/*}/../../../cargo-sort-used"
exec "$IC_TESTKIT_HOOK_CARGO_SORT" "$@"
FORMATTER
    chmod +x .tools/rust/bin/cargo-sort
    cp Cargo.lock selected-lock
}
expect_failure() {
    if "$@" > rejected.log 2>&1; then
        echo 'hook qualification unexpectedly accepted a failing input' >&2
        exit 1
    fi
}

new_fixture formatting
rust_file=crates/ic_testkit_perf_probe/src/lib.rs
printf '\npub fn hook_fixture( ){}\n' >> "$rust_file"
git add -- "$rust_file"
printf '\nUnrelated working edit.\n' >> README.md
cp README.md unrelated-before
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > formatted.log
[[ -s cargo-sort-used ]] || fail 'hook did not discover the checkout-local formatter'
[[ "$(git show ":$rust_file" | tail -n 1)" == 'pub fn hook_fixture() {}' ]] || fail "fixture invariant in $PWD at line $LINENO"
git diff --quiet -- "$rust_file"
cmp unrelated-before README.md
[[ "$(git show :README.md)" == "$(git show HEAD:README.md)" ]] || fail "fixture invariant in $PWD at line $LINENO"
cmp selected-lock Cargo.lock
[[ ! -e target ]] || fail "fixture invariant in $PWD at line $LINENO"
make --no-print-directory fmt-check > checked.log
tree="$(git write-tree)"
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > repeated.log
[[ "$(git write-tree)" == "$tree" ]] || fail "fixture invariant in $PWD at line $LINENO"
cmp selected-lock Cargo.lock

new_fixture partial
printf '\npub fn hook_fixture( ){}\n' >> "$rust_file"
git add -- "$rust_file"
printf '\n// Unstaged edit.\n' >> "$rust_file"
tree="$(git write-tree)"
cp "$rust_file" partially-staged-before
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]] || fail "fixture invariant in $PWD at line $LINENO"
cmp partially-staged-before "$rust_file"
cmp selected-lock Cargo.lock

# Substitute only Cargo's admission probe. Missing and wrong selected versions
# must refuse before formatter execution, preserving both staged and live bytes.
for rejected_version in missing wrong; do
    new_fixture "cargo-sort-$rejected_version"
    printf '\npub fn hook_fixture( ){}\n' >> "$rust_file"
    git add -- "$rust_file"
    mkdir tool-bin
    cat > tool-bin/cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 97
printf '%s\n' "$*" >> "$FORMATTER_TRACE"
[[ "$*" == 'sort --version' ]] || exit 97
[[ "$REJECTED_VERSION" == wrong ]] || exit 1
printf 'cargo-sort 0.0.0\n'
STUB
    chmod +x tool-bin/cargo
    tree="$(git write-tree)"
    cp "$rust_file" rejected-before
    expect_failure env PATH="$PWD/tool-bin:$PATH" \
        FORMATTER_TRACE="$PWD/formatter-trace" REJECTED_VERSION="$rejected_version" \
        bash .githooks/pre-commit
    [[ "$(cat formatter-trace)" == 'sort --version' ]] || fail 'unexpected formatter effect'
    [[ "$(git write-tree)" == "$tree" ]] || fail 'rejected formatter changed index'
    cmp rejected-before "$rust_file"
    cmp selected-lock Cargo.lock
done

new_fixture formatter-failure
printf '\nfmt:\n\t@false\n' >> Makefile
git add Makefile
tree="$(git write-tree)"
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]] || fail "fixture invariant in $PWD at line $LINENO"
git diff --quiet -- Makefile Cargo.toml
cmp selected-lock Cargo.lock

new_fixture installer
ln -s "$PWD" "$fixture/installer-alias"
(
    cd "$fixture/installer-alias" || fail "cannot enter installer alias"
    make --no-print-directory install-hooks || fail "installer rejected its directory alias"
) > installed.log || fail "aliased installer failed"
[[ "$(git config --get core.hooksPath)" == .githooks ]] || fail "fixture invariant in $PWD at line $LINENO"
make --no-print-directory install-hooks >> installed.log
for selected_hooks in custom-hooks $'custom-hooks\n' $'.githooks\n'; do
    git config --local core.hooksPath "$selected_hooks"
    expect_failure make --no-print-directory install-hooks
    # Preserve path bytes rather than stripping newlines during verification.
    actual_hooks="$(git config --get core.hooksPath && printf '.')"
    actual_hooks="${actual_hooks%$'\n.'}"
    [[ "$actual_hooks" == "$selected_hooks" ]] || fail 'installer changed existing hook selection'
done

# Setup must not activate a hook whose Rust formatter is unavailable. Substitute
# Cargo only at the availability boundary, without removing any installed tools.
new_fixture missing-rustfmt
mkdir tool-bin
cat > tool-bin/cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 97
printf '%s\n' "$*" >> formatter-trace
case "$*" in
    'sort --version')
        # shellcheck source=/dev/null
        source ci/tool-versions.env
        printf 'cargo-sort %s\n' "$SHARED_TOOLING_CARGO_SORT_VERSION"
        ;;
    'fmt --version') exit 1 ;;
    *) echo "unexpected formatter effect: $*" >&2; exit 97 ;;
esac
STUB
chmod +x tool-bin/cargo
tree="$(git write-tree)"
expect_failure env PATH="$PWD/tool-bin:$PATH" make --no-print-directory install-hooks
[[ -z "$(git config --local --get core.hooksPath || true)" ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(cat formatter-trace)" == $'sort --version\nfmt --version' ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(git write-tree)" == "$tree" ]] || fail "fixture invariant in $PWD at line $LINENO"
git diff --quiet -- Makefile Cargo.toml
cmp selected-lock Cargo.lock
echo 'Consumer hook formatting, lockfile preservation, rejection and activation checks passed'
fixture_complete=true
