#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-publish-guards.XXXXXX")"
cleanup() {
  local status=$?
  if [[ "$status" == 0 ]]; then rm -rf "$work_dir";
  else echo "Failed publication qualification retained at $work_dir" >&2; fi
}
trap cleanup EXIT
fail() { echo "publication qualification failed: $*" >&2; exit 1; }
mkdir -p "$work_dir/bin" "$work_dir/state"
export QUALIFICATION_CARGO
QUALIFICATION_CARGO="$(command -v cargo)"
version="$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable "$repo_root/Cargo.toml")"
registry_event="registry https://crates.io/api/v1/crates/ic-testkit/$version"
cat > "$work_dir/bin/curl" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == --disable ]] || exit 98
url="${!#}"
[[ "$url" == "https://crates.io/api/v1/crates/ic-testkit/$EXPECTED_VERSION" ]] || exit 98
printf 'registry %s\n' "$url" >> "$TRACE_FILE"
if [[ -e "$STATE_DIR/ic-testkit" ]]; then selected=200; else selected=404; fi
printf '%s' "${HTTP_STATUS-$selected}"
exit "${LOOKUP_STATUS:-0}"
STUB
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == locate-project ]]; then exec "$QUALIFICATION_CARGO" "$@"; fi
[[ "$*" == 'publish --locked --registry crates-io -p ic-testkit' ]] || exit 98
printf 'cargo %s\n' "$*" >> "$TRACE_FILE"
[[ "${PUBLISH_STATUS:-0}" == 0 ]] || exit "$PUBLISH_STATUS"
: > "$STATE_DIR/ic-testkit"
STUB
chmod +x "$work_dir/bin/curl" "$work_dir/bin/cargo"
run_publish() (
  cd "$repo_root" || exit 1
  PATH="$work_dir/bin:$PATH" TRACE_FILE="$work_dir/trace" \
    STATE_DIR="$work_dir/state" EXPECTED_VERSION="$version" \
    CARGO_NET_OFFLINE="${TEST_OFFLINE:-false}" \
    bash scripts/release/publish-workspace.sh
)
expect_trace() {
  printf '%s' "$1" > "$work_dir/expected"
  cmp "$work_dir/expected" "$work_dir/trace" || fail 'unexpected registry/publication effects'
}
: > "$work_dir/trace"
run_publish > "$work_dir/first.log"
expect_trace "$registry_event"$'\ncargo publish --locked --registry crates-io -p ic-testkit\n'
[[ -f "$work_dir/state/ic-testkit" ]] || fail 'publication did not reach Cargo'
: > "$work_dir/trace"
run_publish > "$work_dir/retry.log"
expect_trace "$registry_event"$'\n'
rm "$work_dir/state/ic-testkit"
for http in 429 500 503 000 malformed ''; do
  : > "$work_dir/trace"
  status=0
  HTTP_STATUS="$http" run_publish > "$work_dir/http-${http:-empty}.log" 2>&1 || status=$?
  [[ "$status" == 2 ]] || fail "HTTP '$http' was not unavailable"
  expect_trace "$registry_event"$'\n'
  [[ ! -e "$work_dir/state/ic-testkit" ]] || fail 'unavailable lookup reached publication'
done
# Complete status output cannot hide a failed transport, including apparent absence.
for http in 200 404; do
  : > "$work_dir/trace"
  status=0
  HTTP_STATUS="$http" LOOKUP_STATUS=28 run_publish > "$work_dir/transport-$http.log" 2>&1 || status=$?
  [[ "$status" == 2 ]] || fail 'transport failure was not unavailable'
  expect_trace "$registry_event"$'\n'
done
: > "$work_dir/trace"
status=0
TEST_OFFLINE=true run_publish > "$work_dir/offline.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || fail 'offline admission was not rejected'
expect_trace ''
: > "$work_dir/trace"
status=0
PUBLISH_STATUS=47 run_publish > "$work_dir/publication-failure.log" 2>&1 || status=$?
[[ "$status" == 47 ]] || fail 'Cargo publication failure was not preserved'
expect_trace "$registry_event"$'\ncargo publish --locked --registry crates-io -p ic-testkit\n'
[[ ! -e "$work_dir/state/ic-testkit" ]] || fail 'failed publication was recorded as complete'
echo 'Publication admission, retry and failure checks passed'

real_make="$(command -v make)"
cat > "$work_dir/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'git %s\n' "$*" >> "$GIT_TRACE_FILE"
case "$*" in
  'ls-files --others --exclude-standard'|'diff-index --quiet HEAD --') exit 0 ;;
  'rev-parse --verify HEAD^{commit}')
    printf '%s\n' "${FAKE_HEAD:-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa}"
    exit "${HEAD_STATUS:-0}" ;;
  "rev-parse --verify ${FAKE_HEAD:-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa}^{commit}")
    printf '%s\n' "${FAKE_HEAD:-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa}"
    exit "${HEAD_STATUS:-0}" ;;
  "cat-file -t refs/tags/v$EXPECTED_VERSION")
    printf '%s\n' "${TAG_TYPE-tag}"
    exit "${TYPE_STATUS:-0}" ;;
  "rev-parse --verify refs/tags/v$EXPECTED_VERSION^{commit}")
    printf '%s\n' "${FAKE_TAG_COMMIT:-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa}"
    exit "${TAG_STATUS:-0}" ;;
  *) exit 98 ;;
esac
STUB
chmod +x "$work_dir/bin/git"
run_admitted_publish() (
  cd "$repo_root" || exit 1
  PATH="$work_dir/bin:$PATH" TRACE_FILE="$work_dir/trace" \
    GIT_TRACE_FILE="$work_dir/git-trace" STATE_DIR="$work_dir/state" \
    EXPECTED_VERSION="$version" CARGO_NET_OFFLINE=false \
    MAKEFLAGS='' MAKEOVERRIDES='' MFLAGS='' \
    "$real_make" --no-print-directory -f "$repo_root/Makefile" publish
)
: > "$work_dir/trace"
run_admitted_publish > "$work_dir/annotated-tag.log" 2>&1
expect_trace "$registry_event"$'\ncargo publish --locked --registry crates-io -p ic-testkit\n'
rm "$work_dir/state/ic-testkit"
for invalid in lightweight missing wrong-commit invalid-head failed-head failed-type failed-tag; do
  : > "$work_dir/trace"
  status=0
  case "$invalid" in
    lightweight) TAG_TYPE=commit run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
    missing) TAG_TYPE='' TYPE_STATUS=128 run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
    wrong-commit) FAKE_TAG_COMMIT=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
    invalid-head) FAKE_HEAD=invalid run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
    failed-head) HEAD_STATUS=9 run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
    failed-type) TYPE_STATUS=9 run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
    failed-tag) TAG_STATUS=9 run_admitted_publish > "$work_dir/tag-$invalid.log" 2>&1 || status=$? ;;
  esac
  [[ "$status" != 0 ]] || fail "invalid tag/inspection '$invalid' reached publication"
  expect_trace ''
  [[ ! -e "$work_dir/state/ic-testkit" ]] || fail 'invalid tag reached Cargo publication'
done
echo 'Actual Make publication tag-admission checks passed'
