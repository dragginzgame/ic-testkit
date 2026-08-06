#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp_root="${TMPDIR:-/tmp}"
make_bin="${MAKE:-make}"

if [[ "${tmp_root}" != /* || "${tmp_root}" == "/" || ! -d "${tmp_root}" ]]; then
  echo "error: TMPDIR must be an existing absolute directory other than /" >&2
  exit 1
fi

ci_tmp_dir="$(mktemp -d "${tmp_root%/}/ic-testkit-release-ci.XXXXXX")"

cleanup() {
  local ci_status="$?"
  local cleanup_status=0
  trap - EXIT

  if [[ "${ci_status}" -eq 0 ]]; then
    echo "Release CI succeeded; cleaning Cargo artifacts..."
    if ! cargo clean; then
      echo "error: failed to clean Cargo artifacts" >&2
      cleanup_status=1
    fi
  else
    echo "Release CI failed; preserving Cargo artifacts for diagnosis." >&2
  fi
  if ! rm -rf -- "${ci_tmp_dir}"; then
    echo "error: failed to remove release CI temporary directory ${ci_tmp_dir}" >&2
    cleanup_status=1
  fi

  if [[ "${ci_status}" -ne 0 ]]; then
    exit "${ci_status}"
  fi
  exit "${cleanup_status}"
}

trap cleanup EXIT

cd "${repo_root}"
TMPDIR="${ci_tmp_dir}" "${make_bin}" --no-print-directory ci
