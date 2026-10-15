#!/usr/bin/env bash
set -euo pipefail
root="$0"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-evidence-test.XXXXXX")"
fixture_complete=false
cleanup() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else echo "Evidence fixture retained: $fixture" >&2; fi
    exit "$status"
}
trap cleanup EXIT
[[ "$fixture" == /* ]] || fixture="$PWD/$fixture"
fixture="$(cd -P "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"

# The consumer's own upload/download boundary must retain the exact identity and
# verify the uploaded digest and exact-ID readback. Common retry behavior is qualified
# by Shared Tooling's fixture; this checks the actual local workflow wiring.
yq -o=json '.jobs' "$root/.github/workflows/ci.yml" > "$fixture/workflow-jobs.json"
jq '.["portable-hosts"].steps' "$fixture/workflow-jobs.json" > "$fixture/workflow-steps.json"
# Restored tool bytes are untrusted until ordinary setup/admission succeeds.
# Save only verified main-push generations, before later validation can fail.
jq -e '
    ["checks", "portable-hosts"] | all(.[]; . as $job
        | $jobs[0][$job].steps | to_entries as $steps
        | [$steps[] | select(.value.id == "cargo-tool-cache-key")] as $keys
        | [$steps[] | select((.value.uses // "") | startswith("actions/cache/restore@"))] as $restores
        | [$steps[] | select((.value.run // "") | contains("make install-tools"))] as $setups
        | [$steps[] | select((.value.uses // "") | startswith("actions/cache/save@"))] as $saves
        | ($keys | length) == 1 and ($restores | length) == 1
          and ($setups | length) == 1 and ($saves | length) == 1
          and ($keys[0].value.run | contains("scripts/ci/cargo-tool-cache-key.sh"))
          and $keys[0].key < $restores[0].key and $restores[0].key < $setups[0].key
          and $setups[0].key < $saves[0].key
          and (($setups[0].value.if // "") == "")
          and ($setups[0].value.run | contains("make rust-tools-check"))
          and ($restores[0].value.with["restore-keys"] // "") == ""
          and $saves[0].value.if == "${{ github.event_name == '\''push'\'' && github.ref == '\''refs/heads/main'\'' && steps.cargo-tool-cache.outputs.cache-hit != '\''true'\'' }}"
          and $saves[0].value.with == $restores[0].value.with
          and all([$restores[0], $saves[0]][];
              .value["timeout-minutes"] == 3 and .value["continue-on-error"] == true
              and .value.with.key == "cargo-tools-${{ runner.os }}-${{ runner.arch }}-${{ steps.cargo-tool-cache-key.outputs.key }}"
              and (.value.with.path | split("\n") | map(select(length > 0))) == [
                  "${{ steps.cargo-tool-cache-key.outputs.root }}/cargo-cache",
                  "${{ steps.cargo-tool-cache-key.outputs.root }}/rust-*"
              ])
    )
' --null-input --slurpfile jobs "$fixture/workflow-jobs.json" >/dev/null
jq -e '
    all(.[]; ((.uses // "") | startswith("actions/download-artifact@")) | not)
' "$fixture/workflow-steps.json" >/dev/null
jq -e '
    .[] | select(.id == "archive-discovery")
    | .["timeout-minutes"] == 2 and (."continue-on-error" // false) == false
      and .env.EVIDENCE_REPOSITORY == "${{ github.repository }}"
      and .env.EVIDENCE_RUN == "${{ github.run_id }}"
      and .env.EVIDENCE_ARTIFACT == "${{ steps.archive-upload.outputs.artifact-id }}"
      and .env.EVIDENCE_DIGEST == "${{ steps.archive-upload.outputs.artifact-digest }}"
      and .env.GH_TOKEN == "${{ github.token }}"
      and .run == "bash scripts/ci/download-evidence.sh \"$EVIDENCE_REPOSITORY\" \"$EVIDENCE_RUN\" \"$EVIDENCE_ARTIFACT\" \"$EVIDENCE_DIGEST\" \"$RUNNER_TEMP/ic-testkit-archive-downloaded\"\n"
' "$fixture/workflow-steps.json" >/dev/null

verify_archive() {
    local mode="$2" selector_status="$3"
    mkdir "$fixture/unpacked"
    # Only extract this workflow's own qualification artifact, never arbitrary input.
    tar -xzf "$1" -C "$fixture/unpacked"
    cmp -s "$fixture/unpacked/tool-evidence/selection-status" <(printf '%s\n' "$selector_status") || exit 1
    if [[ "$selector_status" == 1 ]]; then
        cmp -s "$fixture/unpacked/tool-evidence/collection-status" <(printf 'shared_collection=incomplete\n') || exit 1
        [[ -s "$fixture/unpacked/tool-evidence/selection.log" ]] || exit 1
    else
        [[ ! -e "$fixture/unpacked/tool-evidence/collection-status" ]] || exit 1
    fi
    if [[ "$mode" == portable ]]; then
        local payload="$fixture/unpacked/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
        cmp -s "$payload" <(printf retained) || exit 1
        [[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2] & 0777' "$payload")" == 640 ]] || exit 1
        [[ -L "$fixture/unpacked/ic-testkit-portable-fixtures/link" ]] || exit 1
        [[ "$(readlink "$fixture/unpacked/ic-testkit-portable-fixtures/link")" == ../outside ]] || exit 1
        [[ ! -e "$fixture/unpacked/ic-testkit-portable-fixtures/link" ]] || exit 1
        [[ ! -e "$fixture/unpacked/ic-testkit-portable-fixtures/.git" ]] || exit 1
        cmp -s "$fixture/unpacked/ic-testkit-archive-proof/evidence.tar.gz" <(printf 'qualification source') || exit 1
        cmp -s "$fixture/unpacked/ic-testkit-archive-downloaded/evidence.tar.gz" <(printf 'qualification download') || exit 1
        [[ ! -e "$fixture/unpacked/ic-testkit-validation.log" ]] || exit 1
    else
        cmp -s "$fixture/unpacked/ic-testkit-validation.log" <(printf validation) || exit 1
        [[ ! -e "$fixture/unpacked/ic-testkit-portable-fixtures" ]] || exit 1
        [[ ! -e "$fixture/unpacked/ic-testkit-archive-proof" && ! -e "$fixture/unpacked/ic-testkit-archive-downloaded" ]] || exit 1
    fi
    [[ -x "$fixture/unpacked/.tools/host-set.fixture/bin/probe" ]] || exit 1
    local shared_reference stored_slot
    shared_reference="$(readlink "$fixture/unpacked/.tools/host-set.failed")"
    stored_slot="${shared_reference%/*}"; stored_slot="${stored_slot##*/}"
    [[ "$stored_slot" =~ ^host-[0-9a-f]{64}$ && "${shared_reference##*/}" == set.failed ]] || exit 1
    [[ ! -e "$fixture/unpacked/$stored_slot/set.unrelated" ]] || exit 1
    local rust_reference rust_slot
    rust_reference="$(readlink "$fixture/unpacked/.tools/rust/rust-set.failed")"
    rust_slot="${rust_reference%/*}"; rust_slot="${rust_slot##*/}"
    [[ "$rust_slot" =~ ^rust-[0-9a-f]{64}$ && "${rust_reference##*/}" == set.failed ]] || exit 1
    [[ ! -e "$fixture/unpacked/$rust_slot/set.unrelated" ]] || exit 1
    if [[ "$selector_status" == 0 ]]; then
        cmp -s "$fixture/unpacked/$stored_slot/set.failed/install.log" <(printf 'failed shared attempt') || exit 1
        cmp -s "$fixture/unpacked/$rust_slot/set.failed/build/evidence" <(printf 'failed shared Cargo build') || exit 1
        cmp -s "$fixture/unpacked/$rust_slot/set.failed/inputs.json" <(printf '{"build":{"compiler":"before"}}\n') || exit 1
        cmp -s "$fixture/unpacked/$rust_slot/set.failed/native-environment.txt" <(printf 'native before\n') || exit 1
        cmp -s "$fixture/unpacked/$rust_slot/set.failed/build-identity.observed.json" <(printf '{"compiler":"after"}\n') || exit 1
        cmp -s "$fixture/unpacked/$rust_slot/set.failed/native-environment.observed.txt" <(printf 'native after\n') || exit 1
    else
        [[ ! -e "$fixture/unpacked/$stored_slot" && ! -e "$fixture/unpacked/$rust_slot" ]] || exit 1
    fi
    [[ ! -e "$fixture/unpacked/outside" && ! -e "$fixture/unpacked/unrelated" ]] || exit 1
    cmp -s "$fixture/unpacked/ic-testkit-tools-install.log" <(printf failed) || exit 1
    cmp -s "$fixture/unpacked/formatting.fixture" <(printf 'formatter diagnostic') || exit 1
    cmp -s "$fixture/unpacked/.tools/rust/build/cargo-attempt.fixture/install.log" <(printf 'failed Cargo attempt') || exit 1
    cmp -s "$fixture/unpacked/.tools/rust/tool-1.0.0-bin-tool-release/installed/selection.json" <(printf 'admitted receipt') || exit 1
    [[ -L "$fixture/unpacked/.tools/rust/common" && "$(readlink "$fixture/unpacked/.tools/rust/common")" == common-set.fixture ]] || exit 1
    [[ -L "$fixture/unpacked/.tools/rust/build/external" && ! -e "$fixture/unpacked/.tools/rust/build/external" ]] || exit 1
    cmp -s "$fixture/unpacked/target/validation-failures/latest.log" <(printf diagnostic) || exit 1
    cmp -s "$fixture/unpacked/step-outcomes.json" <(printf '%s\n' '{"prepare":{"outcome":"failure","status":23}}') || exit 1
    rm -rf "$fixture/unpacked"
}
verify_proof() {
    # These are this workflow's own qualification archives, not arbitrary input.
    verify_archive "$1/evidence.tar.gz" portable 0
    verify_archive "$1/checks-rejected.tar.gz" checks 1
    verify_archive "$1/portable-rejected.tar.gz" portable 1
}
verify_upload() {
    local directory="$1"
    mkdir "$fixture/upload"
    # Our upload is exactly three regular qualification archives. Refuse links,
    # extra members and path redirection before extracting even owned evidence.
    printf '%s\n' evidence.tar.gz checks-rejected.tar.gz portable-rejected.tar.gz > "$fixture/expected-upload"
    tar -tzf "$directory/evidence.tar.gz" > "$fixture/upload-members"
    cmp "$fixture/expected-upload" "$fixture/upload-members" || return
    tar -tvzf "$directory/evidence.tar.gz" > "$fixture/upload-types"
    awk 'substr($0, 1, 1) != "-" { exit 1 }' "$fixture/upload-types" || return
    tar -xzf "$directory/evidence.tar.gz" -C "$fixture/upload"
    verify_proof "$fixture/upload"
}
if [[ $# == 2 && ( "$1" == verify || "$1" == verify-upload ) ]]; then
    if [[ "$1" == verify-upload ]]; then verify_upload "$2"; else verify_proof "$2"; fi
    echo 'Downloaded normal/rejected-selection evidence bytes, modes, links, statuses and outcomes passed'
    fixture_complete=true
    exit 0
fi
[[ $# == 0 ]] || exit 2
# Every setup/evidence operation below uses this fixture's private store.
export SHARED_TOOLING_STORE="$fixture/shared-store"
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
[[ "$(cat "$fixture/early-unpacked/step-outcomes.json")" == "$IC_TESTKIT_FAILURE_STEPS" ]] || exit 1
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
mkdir -p "$GITHUB_WORKSPACE/.tools/rust/build/cargo-attempt.fixture" \
    "$GITHUB_WORKSPACE/.tools/rust/tool-1.0.0-bin-tool-release/installed" \
    "$GITHUB_WORKSPACE/.tools/rust/common-set.fixture/bin"
printf 'failed Cargo attempt' > "$GITHUB_WORKSPACE/.tools/rust/build/cargo-attempt.fixture/install.log"
printf 'admitted receipt' > "$GITHUB_WORKSPACE/.tools/rust/tool-1.0.0-bin-tool-release/installed/selection.json"
ln -s common-set.fixture "$GITHUB_WORKSPACE/.tools/rust/common"
ln -s ../../../unrelated "$GITHUB_WORKSPACE/.tools/rust/build/external"
printf validation > "$RUNNER_TEMP/ic-testkit-validation.log"
printf diagnostic > "$GITHUB_WORKSPACE/target/validation-failures/latest.log"
mkdir "$RUNNER_TEMP/ic-testkit-archive-proof" "$RUNNER_TEMP/ic-testkit-archive-downloaded"
printf 'qualification source' > "$RUNNER_TEMP/ic-testkit-archive-proof/evidence.tar.gz"
printf 'qualification download' > "$RUNNER_TEMP/ic-testkit-archive-downloaded/evidence.tar.gz"
# Exercise the real shared selector through this consumer's collector. Synthetic
# authenticated tools keep the fixture offline and independent of installed sets.
# Installers reject newline-bearing consumer paths; archive roots above still
# exercise those names independently of installation admission.
mv "$GITHUB_WORKSPACE" "$fixture/workspace"
export GITHUB_WORKSPACE="$fixture/workspace"
case "$(uname -s):$(uname -m)" in
    Linux:x86_64|Linux:amd64) host=LINUX_AMD64; target=x86_64-unknown-linux-musl ;;
    Linux:aarch64|Linux:arm64) host=LINUX_ARM64; target=aarch64-unknown-linux-musl ;;
    Darwin:x86_64) host=DARWIN_AMD64; target=x86_64-apple-darwin ;;
    Darwin:arm64|Darwin:aarch64) host=DARWIN_ARM64; target=aarch64-apple-darwin ;;
    *) echo 'unsupported evidence fixture host' >&2; exit 1 ;;
esac
mkdir -p "$fixture/assets" "$fixture/bin" "$GITHUB_WORKSPACE/ci" "$fixture/ripgrep-1.0.0-$target"
for tool in jq yq rg cloc; do
    case "$tool" in
        jq) version='jq-1.0.0' ;;
        yq) version='yq (https://github.com/mikefarah/yq/) version v1.0.0' ;;
        rg) version='ripgrep 1.0.0' ;;
        cloc) version='1.0.0' ;;
    esac
    printf '#!/bin/sh\necho "%s"\n' "$version" > "$fixture/assets/$tool"
    chmod 755 "$fixture/assets/$tool"
done
cp "$fixture/assets/rg" "$fixture/ripgrep-1.0.0-$target/rg"
tar -czf "$fixture/assets/ripgrep.tar.gz" -C "$fixture" "ripgrep-1.0.0-$target/rg"
pins="$GITHUB_WORKSPACE/ci/tool-versions.env"
for tool in JQ YQ RIPGREP CLOC; do
    printf 'export SHARED_TOOLING_%s_VERSION=1.0.0\n' "$tool" >> "$pins"
    case "$tool" in
        JQ) input="$fixture/assets/jq" ;;
        YQ) input="$fixture/assets/yq" ;;
        RIPGREP) input="$fixture/assets/ripgrep.tar.gz" ;;
        CLOC) input="$fixture/assets/cloc" ;;
    esac
    digest="$(bash "$root/scripts/ci/verify-file-checksum.sh" --print sha256 "$input")"
    suffix="_$host"; [[ "$tool" != CLOC ]] || suffix=''
    printf 'export SHARED_TOOLING_%s_SHA256%s=%s\n' "$tool" "$suffix" "$digest" >> "$pins"
done
# Substitute transport only; the canonical installer owns slots and references.
cat > "$fixture/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
while [[ $# -gt 0 ]]; do
    case "$1" in -o) output="$2"; shift 2 ;; https://*) asset="${1##*/}"; shift ;; *) shift ;; esac
done
case "$asset" in jq-*) tool=jq ;; yq_*) tool=yq ;; cloc-*) tool=cloc ;; ripgrep-*) tool=ripgrep.tar.gz ;; *) exit 2 ;; esac
cp "$IC_TESTKIT_EVIDENCE_ASSETS/$tool" "$output"
CURL
chmod +x "$fixture/bin/curl"
IC_TESTKIT_EVIDENCE_ASSETS="$fixture/assets" PATH="$fixture/bin:$PATH" \
    bash "$root/scripts/dev/install-host-tools.sh" --consumer "$GITHUB_WORKSPACE" --versions "$pins"
verified="$GITHUB_WORKSPACE/.tools/$(readlink "$GITHUB_WORKSPACE/.tools/host")"
physical="$(cd "$verified" && pwd -P)"
stored_parent="${physical%/*}"
stored_path="${stored_parent##*/}/${physical##*/}"
# Retain only referenced failed generations, excluding other checkouts' payloads.
(umask 077; mkdir "$stored_parent/set.failed" "$stored_parent/set.unrelated")
printf 'failed shared attempt' > "$stored_parent/set.failed/install.log"
printf 'other checkout' > "$stored_parent/set.unrelated/payload"
ln -s "$stored_parent/set.failed" "$GITHUB_WORKSPACE/.tools/host-set.failed"
# The shared selector now owns both the physical local Rust tree and its
# admitted shared attempts. The consumer collector must not duplicate selection.
rust_slot="$SHARED_TOOLING_STORE/rust-$(printf '0%.0s' {1..64})"
(umask 077; mkdir -p "$rust_slot/set.failed/build" "$rust_slot/set.unrelated")
printf 'failed shared Cargo build' > "$rust_slot/set.failed/build/evidence"
printf '{"build":{"compiler":"before"}}\n' > "$rust_slot/set.failed/inputs.json"
printf 'native before\n' > "$rust_slot/set.failed/native-environment.txt"
printf '{"compiler":"after"}\n' > "$rust_slot/set.failed/build-identity.observed.json"
printf 'native after\n' > "$rust_slot/set.failed/native-environment.observed.txt"
printf 'other checkout' > "$rust_slot/set.unrelated/payload"
ln -s "$rust_slot/set.failed" "$GITHUB_WORKSPACE/.tools/rust/rust-set.failed"
mkdir "$fixture/compact-temp" "$fixture/compact-unpacked"
compact="$(RUNNER_TEMP="$fixture/compact-temp" bash "$collector" checks)"
tar -xzf "$compact" -C "$fixture/compact-unpacked"
[[ ! -e "$fixture/compact-unpacked/.tools/${verified##*/}" && ! -e "$fixture/compact-unpacked/$stored_path" ]] || exit 1
cmp "$stored_parent/set.failed/install.log" "$fixture/compact-unpacked/${stored_parent##*/}/set.failed/install.log"
for evidence in inputs.json native-environment.txt build-identity.observed.json native-environment.observed.txt; do
    cmp "$rust_slot/set.failed/$evidence" "$fixture/compact-unpacked/${rust_slot##*/}/set.failed/$evidence"
done
[[ -L "$fixture/compact-unpacked/.tools/host-set.failed" && ! -e "$fixture/compact-unpacked/${stored_parent##*/}/set.unrelated" ]] || exit 1
[[ -x "$fixture/compact-unpacked/.tools/host-set.fixture/bin/probe" ]] || exit 1
[[ -s "$fixture/compact-unpacked/tool-evidence/host/check.log" ]] || exit 1
cmp "$pins" "$fixture/compact-unpacked/tool-evidence/host/caller-pins"
[[ "$(cat "$fixture/compact-unpacked/tool-evidence/host/selection.txt")" == "$verified" ]] || exit 1
cmp "$GITHUB_WORKSPACE/.tools/rust/tool-1.0.0-bin-tool-release/installed/selection.json" \
    "$fixture/compact-unpacked/.tools/rust/tool-1.0.0-bin-tool-release/installed/selection.json"
cmp "$GITHUB_WORKSPACE/.tools/rust/build/cargo-attempt.fixture/install.log" \
    "$fixture/compact-unpacked/.tools/rust/build/cargo-attempt.fixture/install.log"
cmp "$rust_slot/set.failed/build/evidence" \
    "$fixture/compact-unpacked/${rust_slot##*/}/set.failed/build/evidence"
[[ ! -e "$fixture/compact-unpacked/${rust_slot##*/}/set.unrelated" ]] || exit 1
# Execute the actual workflow collector steps with a rejected store. Both must
# publish a completed local archive while preserving the selector's failure.
ln -s "$SHARED_TOOLING_STORE" "$fixture/redirected-store"
mkdir "$GITHUB_WORKSPACE/scripts"
ln -s "$root/scripts/ci" "$GITHUB_WORKSPACE/scripts/ci"
for mode in checks portable; do
    mkdir "$fixture/rejected-$mode-temp" "$fixture/rejected-$mode-unpacked"
    # Both modes see the same available diagnostics; each selects only its scope.
    cp -p "$RUNNER_TEMP/ic-testkit-tools-install.log" "$RUNNER_TEMP/ic-testkit-validation.log" \
        "$RUNNER_TEMP/formatting.fixture" "$RUNNER_TEMP/outside" "$fixture/rejected-$mode-temp/"
    cp -R -P -p "$RUNNER_TEMP/ic-testkit-portable-fixtures" "$RUNNER_TEMP/ic-testkit-archive-proof" \
        "$RUNNER_TEMP/ic-testkit-archive-downloaded" "$fixture/rejected-$mode-temp/"
    awk -v id="$mode-evidence" '
        index($0, "id: " id) { selected=1 }
        selected && /^        run: \|$/ { body=1; next }
        body && /^          / { print substr($0, 11); next }
        body { exit }
    ' "$root/.github/workflows/ci.yml" > "$fixture/workflow-$mode.sh"
    [[ -s "$fixture/workflow-$mode.sh" ]] || exit 1
    status=0
    (cd "$GITHUB_WORKSPACE" && SHARED_TOOLING_STORE="$fixture/redirected-store" \
        RUNNER_TEMP="$fixture/rejected-$mode-temp" GITHUB_OUTPUT="$fixture/workflow-$mode.output" \
        bash -e -u -o pipefail "$fixture/workflow-$mode.sh") \
        > "$fixture/workflow-$mode.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || exit 1
    rejected="$(cat "$fixture/workflow-$mode.output")"
    [[ "$rejected" == archive=* ]] || exit 1
    tar -xzf "${rejected#archive=}" -C "$fixture/rejected-$mode-unpacked"
    rejected="$fixture/rejected-$mode-unpacked"
    cmp "$GITHUB_WORKSPACE/.tools/rust/build/cargo-attempt.fixture/install.log" \
        "$rejected/.tools/rust/build/cargo-attempt.fixture/install.log"
    cmp "$GITHUB_WORKSPACE/.tools/rust/tool-1.0.0-bin-tool-release/installed/selection.json" \
        "$rejected/.tools/rust/tool-1.0.0-bin-tool-release/installed/selection.json"
    [[ -x "$rejected/.tools/host-set.fixture/bin/probe" ]] || exit 1
    [[ "$(readlink "$rejected/.tools/host-set.failed")" == "$stored_parent/set.failed" ]] || exit 1
    [[ ! -e "$rejected/$stored_path" && ! -e "$rejected/outside" && ! -e "$rejected/unrelated" ]] || exit 1
    [[ ! -e "$rejected/${rust_slot##*/}" && -L "$rejected/.tools/rust/rust-set.failed" ]] || exit 1
    [[ -L "$rejected/.tools/rust/build/external" && ! -e "$rejected/.tools/rust/build/external" ]] || exit 1
    [[ "$(cat "$rejected/ic-testkit-tools-install.log")" == failed ]] || exit 1
    [[ "$(cat "$rejected/step-outcomes.json")" == "$IC_TESTKIT_FAILURE_STEPS" ]] || exit 1
    [[ "$(cat "$rejected/tool-evidence/selection-status")" == 1 ]] || exit 1
    [[ "$(cat "$rejected/tool-evidence/collection-status")" == shared_collection=incomplete ]] || exit 1
    [[ -s "$rejected/tool-evidence/selection.log" ]] || exit 1
done
# A failed selector may emit partial pairs: archive them as diagnostics, never
# use them to read even an otherwise accessible outside payload.
mkdir "$fixture/partial-collector" "$fixture/partial-temp" "$fixture/partial-unpacked"
cp "$collector" "$root/scripts/ci/archive-evidence.sh" "$root/scripts/ci/formatting-evidence.sh" "$fixture/partial-collector/"
cat > "$fixture/partial-collector/select-tool-evidence.sh" <<'SELECTOR'
#!/usr/bin/env bash
printf '%s\0%s\0' "$IC_TESTKIT_EVIDENCE_OUTSIDE" unrelated
echo 'injected selection failure' >&2
exit 23
SELECTOR
status=0
partial="$(IC_TESTKIT_EVIDENCE_OUTSIDE="$GITHUB_WORKSPACE" RUNNER_TEMP="$fixture/partial-temp" \
    bash "$fixture/partial-collector/collect-failure-evidence.sh" checks)" || status=$?
[[ "$status" == 23 ]] || exit 1
tar -xzf "$partial" -C "$fixture/partial-unpacked"
[[ ! -e "$fixture/partial-unpacked/unrelated" ]] || exit 1
[[ -s "$fixture/partial-unpacked/tool-evidence/selections.nul" ]] || exit 1
[[ "$(cat "$fixture/partial-unpacked/tool-evidence/selection-status")" == 23 ]] || exit 1
cmp "$GITHUB_WORKSPACE/.tools/rust/build/cargo-attempt.fixture/install.log" \
    "$fixture/partial-unpacked/.tools/rust/build/cargo-attempt.fixture/install.log"
# A redirected Rust root must not read external receipts or build evidence.
mv "$GITHUB_WORKSPACE/.tools/rust" "$fixture/external-rust"
ln -s "$fixture/external-rust" "$GITHUB_WORKSPACE/.tools/rust"
mkdir "$fixture/redirected-rust-temp" "$fixture/redirected-rust-unpacked"
redirected="$(RUNNER_TEMP="$fixture/redirected-rust-temp" bash "$collector" checks)"
tar -xzf "$redirected" -C "$fixture/redirected-rust-unpacked"
[[ ! -e "$fixture/redirected-rust-unpacked/.tools/rust" ]] || exit 1
cmp "$fixture/compact-unpacked/.tools/rust/build/cargo-attempt.fixture/install.log" \
    "$fixture/external-rust/build/cargo-attempt.fixture/install.log"
rm "$GITHUB_WORKSPACE/.tools/rust"
mv "$fixture/external-rust" "$GITHUB_WORKSPACE/.tools/rust"
# A selected installation that now fails admission must retain its full bytes.
printf '\nchanged\n' >> "$verified/bin/jq"
mkdir "$fixture/changed-temp" "$fixture/changed-unpacked"
changed="$(RUNNER_TEMP="$fixture/changed-temp" bash "$collector" checks)"
tar -xzf "$changed" -C "$fixture/changed-unpacked"
cmp "$verified/bin/jq" "$fixture/changed-unpacked/$stored_path/bin/jq"
[[ -L "$fixture/changed-unpacked/.tools/${verified##*/}" && ! -e "$fixture/changed-unpacked/${stored_parent##*/}/set.unrelated" ]] || exit 1
[[ -s "$fixture/changed-unpacked/tool-evidence/host/check.log" ]] || exit 1
archive="$(TAR_OPTIONS=--dereference bash "$collector" portable)"
mkdir "$fixture/proof"
cp "$archive" "$fixture/proof/evidence.tar.gz"
for mode in checks portable; do
    rejected="$(cat "$fixture/workflow-$mode.output")"
    cp "${rejected#archive=}" "$fixture/proof/$mode-rejected.tar.gz"
done
verify_proof "$fixture/proof"
# Qualify the same separate entry point used after hosted artifact readback.
mkdir "$fixture/verify-valid"
TMPDIR="$fixture/verify-valid" bash "$root/scripts/ci/test-failure-evidence.sh" verify "$fixture/proof" \
    > "$fixture/verify-valid.log" 2>&1
set -- "$fixture/verify-valid"/*
[[ ! -e "$1" ]] || exit 1
# Contradict payload, outcome and selector-status observations without changing verifier code.
# Appended newlines must not disappear through command-substitution normalization.
# Failed verification must preserve extracted evidence, including on Bash 3.2.
for invalid in payload outcomes selection-status payload-newline rejected-log-newline missing-diagnostic missing-portable missing-build-observation changed-native-observation; do
    mkdir "$fixture/corrupt-$invalid" "$fixture/verify-$invalid" "$fixture/corrupt-$invalid-proof"
    cp "$fixture/proof/"*.tar.gz "$fixture/corrupt-$invalid-proof/"
    case "$invalid" in
        selection-status|missing-diagnostic) corrupt_archive='checks-rejected.tar.gz' ;;
        rejected-log-newline|missing-portable) corrupt_archive='portable-rejected.tar.gz' ;;
        *) corrupt_archive=evidence.tar.gz ;;
    esac
    tar -xzf "$fixture/proof/$corrupt_archive" -C "$fixture/corrupt-$invalid"
    if [[ "$invalid" == payload ]]; then
        printf changed > "$fixture/corrupt-$invalid/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
    elif [[ "$invalid" == outcomes ]]; then
        printf '{"prepare":{"outcome":"success","status":0}}' > "$fixture/corrupt-$invalid/step-outcomes.json"
    elif [[ "$invalid" == selection-status ]]; then
        printf 0 > "$fixture/corrupt-$invalid/tool-evidence/selection-status"
    elif [[ "$invalid" == payload-newline ]]; then
        printf '\n' >> "$fixture/corrupt-$invalid/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
    elif [[ "$invalid" == rejected-log-newline ]]; then
        printf '\n' >> "$fixture/corrupt-$invalid/.tools/rust/build/cargo-attempt.fixture/install.log"
    elif [[ "$invalid" == missing-diagnostic ]]; then
        rm "$fixture/corrupt-$invalid/target/validation-failures/latest.log"
    elif [[ "$invalid" == missing-build-observation ]]; then
        rm "$fixture/corrupt-$invalid/${rust_slot##*/}/set.failed/build-identity.observed.json"
    elif [[ "$invalid" == changed-native-observation ]]; then
        printf changed > "$fixture/corrupt-$invalid/${rust_slot##*/}/set.failed/native-environment.observed.txt"
    else
        rm "$fixture/corrupt-$invalid/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
    fi
    TAR_OPTIONS='' tar -czf "$fixture/corrupt-$invalid-proof/$corrupt_archive" -C "$fixture/corrupt-$invalid" .
    status=0
    TMPDIR="$fixture/verify-$invalid" bash "$root/scripts/ci/test-failure-evidence.sh" \
        verify "$fixture/corrupt-$invalid-proof" > "$fixture/verify-$invalid.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || exit 1
    set -- "$fixture/verify-$invalid"/ic-testkit-evidence-test.*
    [[ $# == 1 && -d "$1/unpacked" ]] || exit 1
    cmp "$fixture/corrupt-$invalid/step-outcomes.json" "$1/unpacked/step-outcomes.json"
    if [[ "$corrupt_archive" != evidence.tar.gz ]]; then
        cmp "$fixture/corrupt-$invalid/tool-evidence/selection-status" "$1/unpacked/tool-evidence/selection-status"
        cmp "$fixture/corrupt-$invalid/.tools/rust/build/cargo-attempt.fixture/install.log" \
            "$1/unpacked/.tools/rust/build/cargo-attempt.fixture/install.log"
    else
        cmp "$fixture/corrupt-$invalid/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak' \
            "$1/unpacked/ic-testkit-portable-fixtures/Linux:x86_64/"$'line\nbreak'
    fi
done
checks="$(bash "$collector" checks)"
mkdir "$fixture/checks"
tar -xzf "$checks" -C "$fixture/checks"
[[ "$(cat "$fixture/checks/ic-testkit-validation.log")" == validation ]] || exit 1
[[ "$(cat "$fixture/checks/formatting.fixture")" == 'formatter diagnostic' ]] || exit 1
cmp "$GITHUB_WORKSPACE/.tools/rust/build/cargo-attempt.fixture/install.log" \
    "$fixture/checks/.tools/rust/build/cargo-attempt.fixture/install.log"
[[ ! -e "$fixture/checks/ic-testkit-portable-fixtures" ]] || exit 1
[[ ! -e "$fixture/checks/ic-testkit-archive-proof" && ! -e "$fixture/checks/ic-testkit-archive-downloaded" ]] || exit 1
# Do not overwrite a previous attempt.
cp "$archive" "$fixture/before.tar.gz"
if bash "$collector" portable; then exit 1; fi
cmp "$fixture/before.tar.gz" "$archive"
# A redirected checkout parent must not prevent retention of unrelated logs.
# Preserve the link itself, never inspect/archive the target, and keep failure.
for parent in target .tools; do
    outside="$fixture/linked-$parent-outside"
    mv "$GITHUB_WORKSPACE/$parent" "$outside"
    ln -s "$outside" "$GITHUB_WORKSPACE/$parent"
    for mode in checks portable; do
        for selector in real failed; do
            temporary="$fixture/linked-$parent-$mode-$selector-temp"
            unpacked="$fixture/linked-$parent-$mode-$selector-unpacked"
            mkdir "$temporary" "$unpacked"
            cp "$RUNNER_TEMP/ic-testkit-tools-install.log" "$temporary/"
            selected_collector="$collector"
            expected_status=1
            if [[ "$selector" == failed ]]; then
                selected_collector="$fixture/partial-collector/collect-failure-evidence.sh"
                expected_status=23
            fi
            status=0
            linked="$(IC_TESTKIT_EVIDENCE_OUTSIDE="$GITHUB_WORKSPACE" RUNNER_TEMP="$temporary" \
                bash "$selected_collector" "$mode")" || status=$?
            [[ "$status" == "$expected_status" && -f "$linked" ]] || exit 1
            tar -tzf "$linked" > "$temporary/members"
            if grep -F "./$parent/" "$temporary/members"; then exit 1; fi
            tar -xzf "$linked" -C "$unpacked"
            [[ -L "$unpacked/$parent" && "$(readlink "$unpacked/$parent")" == "$outside" ]] || exit 1
            cmp "$temporary/ic-testkit-tools-install.log" "$unpacked/ic-testkit-tools-install.log"
            cmp "$temporary/ic-testkit-$mode-failure/step-outcomes.json" "$unpacked/step-outcomes.json"
            grep -F "linked evidence parent: $parent" "$unpacked/collection.log" >/dev/null || exit 1
            if [[ "$selector" == failed ]]; then
                [[ "$(cat "$unpacked/tool-evidence/selection-status")" == 23 ]] || exit 1
                [[ ! -e "$unpacked/unrelated" ]] || exit 1
            fi
        done
    done
    rm "$GITHUB_WORKSPACE/$parent"
    mv "$outside" "$GITHUB_WORKSPACE/$parent"
done
# A failed archiver retains its partial output and original evidence.
mkdir "$fixture/failed-temp"
printf '#!%s\nprintf partial\nexit 23\n' "$BASH" > "$fixture/bin/tar"
chmod +x "$fixture/bin/tar"
if PATH="$fixture/bin:$PATH" RUNNER_TEMP="$fixture/failed-temp" bash "$collector" checks; then exit 1; fi
[[ "$(cat "$fixture/failed-temp/ic-testkit-checks-failure/evidence.tar.gz.partial")" == partial ]] || exit 1
[[ -f "$payload" && ! -e "$fixture/failed-temp/ic-testkit-checks-failure/evidence.tar.gz" ]] || exit 1
if [[ -n "${IC_TESTKIT_ARCHIVE_PROOF_DIR:-}" ]]; then
    mkdir "$IC_TESTKIT_ARCHIVE_PROOF_DIR"
    tar -czf "$IC_TESTKIT_ARCHIVE_PROOF_DIR/evidence.tar.gz" -C "$fixture/proof" \
        evidence.tar.gz checks-rejected.tar.gz portable-rejected.tar.gz
    mkdir "$fixture/verify-upload"
    TMPDIR="$fixture/verify-upload" bash "$root/scripts/ci/test-failure-evidence.sh" \
        verify-upload "$IC_TESTKIT_ARCHIVE_PROOF_DIR"
    for invalid in extra link; do
        mkdir "$fixture/upload-$invalid" "$fixture/upload-$invalid-input" "$fixture/verify-upload-$invalid"
        cp "$fixture/proof/"*.tar.gz "$fixture/upload-$invalid-input/"
        members=(evidence.tar.gz checks-rejected.tar.gz portable-rejected.tar.gz)
        if [[ "$invalid" == extra ]]; then
            printf extra > "$fixture/upload-$invalid-input/extra"
            members+=(extra)
        else
            rm "$fixture/upload-$invalid-input/evidence.tar.gz"
            ln -s "$fixture/proof/evidence.tar.gz" "$fixture/upload-$invalid-input/evidence.tar.gz"
        fi
        tar -czf "$fixture/upload-$invalid/evidence.tar.gz" -C "$fixture/upload-$invalid-input" "${members[@]}"
        status=0
        TMPDIR="$fixture/verify-upload-$invalid" bash "$root/scripts/ci/test-failure-evidence.sh" \
            verify-upload "$fixture/upload-$invalid" > "$fixture/upload-$invalid.log" 2>&1 || status=$?
        [[ "$status" != 0 ]] || exit 1
    done
fi
echo 'Failure evidence selections, archive round trips and failure retention passed'
fixture_complete=true
