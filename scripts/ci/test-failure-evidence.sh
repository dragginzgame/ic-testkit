#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
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
mkdir -p "$fixture/workspace/.tools/host-set.fixture/bin" "$fixture/temp"
export GITHUB_WORKSPACE="$fixture/workspace" RUNNER_TEMP="$fixture/temp"
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
# A failed archiver retains its partial output and original evidence.
rm "$GITHUB_WORKSPACE/target"
mkdir "$fixture/bin" "$fixture/failed-temp"
# The generated tar substitute expands its own argument, not this shell's.
# shellcheck disable=SC2016
printf '#!%s\nprintf partial > "$2"\nexit 23\n' "$BASH" > "$fixture/bin/tar"
chmod +x "$fixture/bin/tar"
if PATH="$fixture/bin:$PATH" RUNNER_TEMP="$fixture/failed-temp" bash "$collector" checks; then exit 1; fi
[[ "$(cat "$fixture/failed-temp/ic-testkit-checks-failure/evidence.tar.gz.partial")" == partial ]]
[[ -f "$payload" && ! -e "$fixture/failed-temp/ic-testkit-checks-failure/evidence.tar.gz" ]]
if [[ -n "${IC_TESTKIT_ARCHIVE_PROOF_DIR:-}" ]]; then
    mkdir "$IC_TESTKIT_ARCHIVE_PROOF_DIR"
    cp "$archive" "$IC_TESTKIT_ARCHIVE_PROOF_DIR/evidence.tar.gz"
fi
echo 'Failure evidence selections, archive round trips and failure retention passed'
