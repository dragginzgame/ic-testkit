#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-release-metadata.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT
export REAL_GIT
REAL_GIT="$(command -v git)"
mkdir -p "$work_dir/bin"
cat > "$work_dir/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  hash-object) shift; exec "$REAL_GIT" hash-object "$@" ;;
  rev-parse) [[ "$*" == 'rev-parse --git-path release-state' ]]; echo .release-state ;;
  ls-files)
    if [[ "$2" == --others ]]; then printf '%s' "${ADAPTER_UNTRACKED:-}"; fi
    ;;
  diff)
    case "$2" in
      --name-only)
        [[ "${ADAPTER_DIFF_STATUS:-0}" -eq 0 ]] || exit "$ADAPTER_DIFF_STATUS"
        if [[ "$*" == *--cached* && -n "${ADAPTER_STAGED_DIRTY:-}" ]]; then
          printf '%s\0' "$ADAPTER_STAGED_DIRTY"
        fi
        [[ -z "${ADAPTER_DIRTY:-}" ]] || printf '%s\0' "$ADAPTER_DIRTY"
        ;;
      --quiet) exit "${ADAPTER_UNSTAGED:-0}" ;;
      --binary)
        for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
          printf '%s\0' "$file"
          cat "$file"
        done
        ;;
      *) exit 97 ;;
    esac
    ;;
  *) echo "unexpected Git effect: $*" >&2; exit 97 ;;
esac
STUB
cat > "$work_dir/bin/make" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == '--no-print-directory release-check' && "$CARGO_NET_OFFLINE" == true ]]
printf '%s\n' "$*" >> gate-trace
echo 'substituted complete validation gate'
exit "${ADAPTER_GATE_STATUS:-0}"
STUB
chmod +x "$work_dir/bin/git" "$work_dir/bin/make"
export PATH="$work_dir/bin:$PATH"

previous="$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable "$repo_root/Cargo.toml")"
new_fixture() {
  mkdir -p "$work_dir/$1"
  cd "$work_dir/$1"
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md rust-toolchain.toml; do
    mkdir -p "$(dirname "$file")"
    cp "$repo_root/$file" "$file"
  done
  for member in crates/ic-testkit canisters/test/perf_probe; do
    mkdir -p "$member/src"
    cp "$repo_root/$member/Cargo.toml" "$member/Cargo.toml"
    : > "$member/src/lib.rs"
  done
  echo 'consumer build artifact' > artifact
  export RELEASE_PREVIOUS="$previous" RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
  export RELEASE_KIND="${2:-patch}" RELEASE_DATE=2026-10-05
  export RELEASE_VERSION
  RELEASE_VERSION="$(bash "$repo_root/scripts/ci/next-release-version.sh" "$previous" "$RELEASE_KIND")"
  # Each isolated candidate has matching numbered notes. The real pending
  # batch keeps its minor version; fixtures select their own release identity.
  for file in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
    awk -v version="$RELEASE_VERSION" '
      /^## \[[0-9]+\.[0-9]+\.[0-9]+\]$/ && !selected++ {
        print "## [" version "]"; next
      }
      { print }
    ' "$file" > candidate-notes
    mv candidate-notes "$file"
  done
}
adapter() { bash "$repo_root/scripts/release/metadata.sh" "$1"; }
expect_failure() {
  if adapter "$1" > rejected.log 2>&1; then
    echo "metadata adapter unexpectedly accepted $1" >&2
    exit 1
  fi
}

for kind in patch minor major; do
  new_fixture "$kind" "$kind"
  adapter preflight
  adapter verify
  cp CHANGELOG.md history-root
  cp crates/ic-testkit/CHANGELOG.md history-package
  adapter prepare
  adapter prepared
  [[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$RELEASE_VERSION" ]]
  # The exact generated metadata, dependency selection and retained old bytes
  # are the contract; no test inspects the adapter's implementation layout.
  cmp history-root ".release-state/$RELEASE_VERSION.metadata/old/CHANGELOG.md"
  cmp history-package ".release-state/$RELEASE_VERSION.metadata/old/crates/ic-testkit/CHANGELOG.md"
  grep -Fx "## [$RELEASE_VERSION] - $RELEASE_DATE" CHANGELOG.md
  grep -Fx "## [$RELEASE_VERSION] - $RELEASE_DATE" crates/ic-testkit/CHANGELOG.md
  for phase in commit committed tagged push; do adapter "$phase"; done
  [[ "$(cat artifact)" == 'consumer build artifact' ]]
  [[ "$(cat gate-trace)" == '--no-print-directory release-check' ]]

  # Model an interrupted publication before the manifest commit point. Resume
  # republishes the same saved bytes without another gate or version selection.
  cp ".release-state/$RELEASE_VERSION.metadata/old/Cargo.toml" Cargo.toml
  cp ".release-state/$RELEASE_VERSION.metadata/old/README.md" README.md
  adapter prepare
  adapter prepared
  [[ "$(wc -l < gate-trace)" -eq 1 ]]
  echo 'unvalidated mutation' >> README.md
  expect_failure prepared
  expect_failure prepare
done

for view in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
  new_fixture "conflicting-${view//\//-}"
  awk '
    /^## \[[0-9]+\.[0-9]+\.[0-9]+\]$/ && !selected++ {
      print "## [9.9.9]"; next
    }
    { print }
  ' "$view" > conflicting-notes
  mv conflicting-notes "$view"
  cp "$view" notes-before
  adapter verify
  expect_failure prepare
  cmp notes-before "$view"
  [[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]
  [[ ! -e ".release-state/$RELEASE_VERSION.metadata/ready" ]]
  logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
  [[ -s "${logs[0]}" ]]
done

new_fixture failed-gate
export ADAPTER_GATE_STATUS=23
expect_failure verify
unset ADAPTER_GATE_STATUS
[[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ -s "${logs[0]}" ]]
expect_failure prepare

new_fixture validation-retry
adapter preflight
adapter verify
cp ".release-state/$RELEASE_VERSION.validation" passed-receipt
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
first_log="${logs[0]}"
cp "$first_log" passed-log
export ADAPTER_GATE_STATUS=23
expect_failure verify
unset ADAPTER_GATE_STATUS
[[ ! -e ".release-state/$RELEASE_VERSION.validation" ]]
expect_failure prepare
cmp passed-log "$first_log"
receipts=(.release-state/"$RELEASE_VERSION".validation.*/prior-validation)
cmp passed-receipt "${receipts[0]}"
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ "${#logs[@]}" -eq 2 ]]
for log in "${logs[@]}"; do [[ -s "$log" ]]; done
adapter preflight
adapter verify
adapter prepare
[[ "$(wc -l < gate-trace)" -eq 3 ]]
cmp passed-log "$first_log"

new_fixture changed-input
adapter verify
echo 'source changed after validation' >> CHANGELOG.md
expect_failure prepare
[[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]

new_fixture admission
export ADAPTER_DIFF_STATUS=9
expect_failure preflight
unset ADAPTER_DIFF_STATUS
export ADAPTER_STAGED_DIRTY='staged unrelated file.rs'
expect_failure preflight
unset ADAPTER_STAGED_DIRTY
export ADAPTER_DIRTY='unrelated file.rs'
expect_failure preflight
unset ADAPTER_DIRTY
export ADAPTER_UNTRACKED='untracked release input'
expect_failure preflight
unset ADAPTER_UNTRACKED
adapter verify
adapter prepare
export ADAPTER_UNSTAGED=1
expect_failure commit
unset ADAPTER_UNSTAGED
export RELEASE_SOURCE=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
expect_failure prepared
echo 'release metadata isolated checks passed'
