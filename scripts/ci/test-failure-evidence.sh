#!/usr/bin/env bash
set -euo pipefail
root="$0"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-evidence-test.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else echo "Evidence fixture retained: $fixture" >&2; fi' EXIT

verify_archive() {
    mkdir "$fixture/unpacked"
    # Only extract this workflow's own qualification artifact, never arbitrary input.
    tar -xzf "$1" -C "$fixture/unpacked"
    local payload="$fixture/unpacked/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
    [[ "$(cat "$payload")" == retained ]]
    [[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2] & 0777' "$payload")" == 640 ]]
    [[ -x "$fixture/unpacked/.tools/host-set.fixture/bin/probe" ]]
    [[ -L "$fixture/unpacked/ic-testkit-portable-fixtures/link" ]]
    [[ "$(readlink "$fixture/unpacked/ic-testkit-portable-fixtures/link")" == ../outside ]]
    [[ ! -e "$fixture/unpacked/ic-testkit-portable-fixtures/link" ]]
    [[ ! -e "$fixture/unpacked/ic-testkit-portable-fixtures/.git" ]]
    [[ ! -e "$fixture/unpacked/outside" && ! -e "$fixture/unpacked/unrelated" ]]
    [[ "$(cat "$fixture/unpacked/ic-testkit-tools-install.log")" == failed ]]
    [[ "$(cat "$fixture/unpacked/formatting.fixture")" == 'formatter diagnostic' ]]
    [[ "$(cat "$fixture/unpacked/target/validation-failures/latest.log")" == diagnostic ]]
    [[ "$(cat "$fixture/unpacked/ic-testkit-archive-proof/evidence.tar.gz")" == 'qualification source' ]]
    [[ "$(cat "$fixture/unpacked/ic-testkit-archive-downloaded/evidence.tar.gz")" == 'qualification download' ]]
    [[ "$(cat "$fixture/unpacked/step-outcomes.json")" == '{"prepare":{"outcome":"failure","status":23}}' ]]
    rm -rf "$fixture/unpacked"
}
if [[ $# == 2 && "$1" == verify ]]; then
    verify_archive "$2"
    echo 'Downloaded evidence archive bytes, modes, links and outcomes passed'
    exit 0
fi
[[ $# == 0 ]] || exit 2
# Exercise physical roots ending in newlines, not only unusual archive members.
export GITHUB_WORKSPACE="$fixture/"$'workspace\n' RUNNER_TEMP="$fixture/"$'temp\n'
mkdir -p "$GITHUB_WORKSPACE/.tools/host-set.fixture/bin" "$RUNNER_TEMP"
export IC_TESTKIT_FAILURE_STEPS='{"prepare":{"outcome":"failure","status":23}}'
collector="$root/scripts/ci/collect-failure-evidence.sh"
# An early failure has no logs or tool sets yet; retain its step outcomes.
mkdir "$fixture/early"
early="$(RUNNER_TEMP="$fixture/early" bash "$collector" checks)"
mkdir "$fixture/early-unpacked"
tar -xzf "$early" -C "$fixture/early-unpacked"
[[ "$(cat "$fixture/early-unpacked/step-outcomes.json")" == "$IC_TESTKIT_FAILURE_STEPS" ]]
mkdir -p "$RUNNER_TEMP/ic-testkit-portable-fixtures/Linux:x86_64" \
    "$RUNNER_TEMP/ic-testkit-portable-fixtures/.git" "$GITHUB_WORKSPACE/target/validation-failures"
payload="$RUNNER_TEMP/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
printf retained > "$payload"
chmod 640 "$payload"
printf '#!/bin/sh\nexit 0\n' > "$GITHUB_WORKSPACE/.tools/host-set.fixture/bin/probe"
chmod 755 "$GITHUB_WORKSPACE/.tools/host-set.fixture/bin/probe"
printf secret > "$RUNNER_TEMP/ic-testkit-portable-fixtures/.git/config"
printf outside > "$RUNNER_TEMP/outside"
printf unrelated > "$GITHUB_WORKSPACE/unrelated"
ln -s ../outside "$RUNNER_TEMP/ic-testkit-portable-fixtures/link"
printf failed > "$RUNNER_TEMP/ic-testkit-tools-install.log"
printf 'formatter diagnostic' > "$RUNNER_TEMP/formatting.fixture"
printf validation > "$RUNNER_TEMP/ic-testkit-validation.log"
printf diagnostic > "$GITHUB_WORKSPACE/target/validation-failures/latest.log"
mkdir "$RUNNER_TEMP/ic-testkit-archive-proof" "$RUNNER_TEMP/ic-testkit-archive-downloaded"
printf 'qualification source' > "$RUNNER_TEMP/ic-testkit-archive-proof/evidence.tar.gz"
printf 'qualification download' > "$RUNNER_TEMP/ic-testkit-archive-downloaded/evidence.tar.gz"
archive="$(TAR_OPTIONS=--dereference bash "$collector" portable)"
verify_archive "$archive"
checks="$(bash "$collector" checks)"
mkdir "$fixture/checks"
tar -xzf "$checks" -C "$fixture/checks"
[[ "$(cat "$fixture/checks/ic-testkit-validation.log")" == validation ]]
[[ "$(cat "$fixture/checks/formatting.fixture")" == 'formatter diagnostic' ]]
[[ ! -e "$fixture/checks/ic-testkit-portable-fixtures" ]]
[[ ! -e "$fixture/checks/ic-testkit-archive-proof" && ! -e "$fixture/checks/ic-testkit-archive-downloaded" ]]
# Do not overwrite a previous attempt or follow intermediate directory links.
cp "$archive" "$fixture/before.tar.gz"
if bash "$collector" portable; then exit 1; fi
cmp "$fixture/before.tar.gz" "$archive"
mkdir "$fixture/linked-temp"
mv "$GITHUB_WORKSPACE/target" "$GITHUB_WORKSPACE/real-target"
ln -s real-target "$GITHUB_WORKSPACE/target"
if RUNNER_TEMP="$fixture/linked-temp" bash "$collector" checks; then exit 1; fi
[[ ! -e "$fixture/linked-temp/ic-testkit-checks-failure/evidence.tar.gz" ]]
rm "$GITHUB_WORKSPACE/target"
# Exercise the real shared selector through this consumer's collector. Synthetic
# authenticated tools keep the fixture offline and independent of installed sets.
case "$(uname -s):$(uname -m)" in
    Linux:x86_64|Linux:amd64) host=LINUX_AMD64; target=x86_64-unknown-linux-musl ;;
    Linux:aarch64|Linux:arm64) host=LINUX_ARM64; target=aarch64-unknown-linux-gnu ;;
    Darwin:x86_64) host=DARWIN_AMD64; target=x86_64-apple-darwin ;;
    Darwin:arm64|Darwin:aarch64) host=DARWIN_ARM64; target=aarch64-apple-darwin ;;
    *) echo 'unsupported evidence fixture host' >&2; exit 1 ;;
esac
verified="$GITHUB_WORKSPACE/.tools/host-set.verified"
mkdir -p "$verified/bin" "$GITHUB_WORKSPACE/ci" "$fixture/ripgrep-1.0.0-$target"
for tool in jq yq rg cloc; do
    case "$tool" in
        jq) version='jq-1.0.0' ;;
        yq) version='yq (https://github.com/mikefarah/yq/) version v1.0.0' ;;
        rg) version='ripgrep 1.0.0' ;;
        cloc) version='1.0.0' ;;
    esac
    printf '#!/bin/sh\necho "%s"\n' "$version" > "$verified/bin/$tool"
    chmod 755 "$verified/bin/$tool"
done
cp "$verified/bin/rg" "$fixture/ripgrep-1.0.0-$target/rg"
tar -czf "$verified/ripgrep.tar.gz" -C "$fixture" "ripgrep-1.0.0-$target/rg"
pins="$GITHUB_WORKSPACE/ci/tool-versions.env"
for tool in JQ YQ RIPGREP CLOC; do
    printf 'export SHARED_TOOLING_%s_VERSION=1.0.0\n' "$tool" >> "$pins"
    case "$tool" in
        JQ) input="$verified/bin/jq" ;;
        YQ) input="$verified/bin/yq" ;;
        RIPGREP) input="$verified/ripgrep.tar.gz" ;;
        CLOC) input="$verified/bin/cloc" ;;
    esac
    digest="$(bash "$root/scripts/ci/verify-file-checksum.sh" --print sha256 "$input")"
    suffix="_$host"; [[ "$tool" != CLOC ]] || suffix=''
    printf 'export SHARED_TOOLING_%s_SHA256%s=%s\n' "$tool" "$suffix" "$digest" >> "$pins"
done
ln -s host-set.verified "$GITHUB_WORKSPACE/.tools/host"
mkdir "$fixture/compact-temp" "$fixture/compact-unpacked"
compact="$(RUNNER_TEMP="$fixture/compact-temp" bash "$collector" checks)"
tar -xzf "$compact" -C "$fixture/compact-unpacked"
[[ ! -e "$fixture/compact-unpacked/.tools/host-set.verified" ]]
[[ -x "$fixture/compact-unpacked/.tools/host-set.fixture/bin/probe" ]]
[[ -s "$fixture/compact-unpacked/tool-evidence/host/check.log" ]]
cmp "$pins" "$fixture/compact-unpacked/tool-evidence/host/caller-pins"
[[ "$(cat "$fixture/compact-unpacked/tool-evidence/host/selection.txt")" == "$verified" ]]
# A selected installation that now fails admission must retain its full bytes.
printf '\nchanged\n' >> "$verified/bin/jq"
mkdir "$fixture/changed-temp" "$fixture/changed-unpacked"
changed="$(RUNNER_TEMP="$fixture/changed-temp" bash "$collector" checks)"
tar -xzf "$changed" -C "$fixture/changed-unpacked"
cmp "$verified/bin/jq" "$fixture/changed-unpacked/.tools/host-set.verified/bin/jq"
[[ -s "$fixture/changed-unpacked/tool-evidence/host/check.log" ]]
# A failed archiver retains its partial output and original evidence.
mkdir "$fixture/bin" "$fixture/failed-temp"
printf '#!%s\nprintf partial\nexit 23\n' "$BASH" > "$fixture/bin/tar"
chmod +x "$fixture/bin/tar"
if PATH="$fixture/bin:$PATH" RUNNER_TEMP="$fixture/failed-temp" bash "$collector" checks; then exit 1; fi
[[ "$(cat "$fixture/failed-temp/ic-testkit-checks-failure/evidence.tar.gz.partial")" == partial ]]
[[ -f "$payload" && ! -e "$fixture/failed-temp/ic-testkit-checks-failure/evidence.tar.gz" ]]
if [[ -n "${IC_TESTKIT_ARCHIVE_PROOF_DIR:-}" ]]; then
    mkdir "$IC_TESTKIT_ARCHIVE_PROOF_DIR"
    cp "$archive" "$IC_TESTKIT_ARCHIVE_PROOF_DIR/evidence.tar.gz"
fi
echo 'Failure evidence selections, archive round trips and failure retention passed'
