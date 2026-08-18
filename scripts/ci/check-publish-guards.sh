#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

publish_case="${work_dir}/publish"
mkdir -p "${publish_case}/bin" "${publish_case}/state"
cat >"${publish_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >>"${TRACE_FILE}"
case "${1:-}" in
  info)
    package="${2%@*}"
    [[ -e "${STATE_DIR}/${package}" ]]
    ;;
  publish)
    package=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == "-p" ]]; then
        package="${2:-}"
        break
      fi
      shift
    done
    [[ -n "${package}" ]] || exit 2
    if [[ -n "${PUBLISH_STATUS:-}" ]]; then
      exit "${PUBLISH_STATUS}"
    fi
    : >"${STATE_DIR}/${package}"
    ;;
  *)
    exit 2
    ;;
esac
EOF
chmod +x "${publish_case}/bin/cargo"

current_version="$(
  /bin/bash "${repo_root}/scripts/release/read-workspace-version.sh" \
    --stable "${repo_root}/Cargo.toml"
)"
(
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" \
    bash scripts/release/publish-workspace.sh
) >/dev/null

mapfile -t publish_trace <"${publish_case}/trace"
expected_publish_trace=(
  "cargo info ic-testkit@${current_version} --registry crates-io"
  "cargo publish --locked --registry crates-io -p ic-testkit"
)
[[ "${#publish_trace[@]}" -eq "${#expected_publish_trace[@]}" ]] \
  || fail "the publisher ran an unexpected number of Cargo commands"
for index in "${!expected_publish_trace[@]}"; do
  [[ "${publish_trace[index]}" == "${expected_publish_trace[index]}" ]] \
    || fail "the publisher ran an unexpected Cargo command"
done

: >"${publish_case}/trace"
(
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" \
    bash scripts/release/publish-workspace.sh
) >/dev/null
mapfile -t republish_trace <"${publish_case}/trace"
[[ "${republish_trace[0]:-}" == "cargo info ic-testkit@${current_version} --registry crates-io" ]] \
  || fail "the publisher did not check the existing release"
[[ "${#republish_trace[@]}" -eq 1 ]] \
  || fail "the publisher was not retry-safe for an existing release"

mkdir -p "${publish_case}/failure-state"
if (
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/failure-trace" \
    STATE_DIR="${publish_case}/failure-state" PUBLISH_STATUS=47 \
    bash scripts/release/publish-workspace.sh
) >/dev/null 2>&1; then
  failed_publish_status=0
else
  failed_publish_status="$?"
fi
[[ "${failed_publish_status}" -eq 47 ]] \
  || fail "the publisher did not preserve a cargo publish failure"
[[ ! -e "${publish_case}/failure-state/ic-testkit" ]] \
  || fail "the publisher recorded a failed release as published"
