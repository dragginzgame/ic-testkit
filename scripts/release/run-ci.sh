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

# Keep this default path aligned with PocketIC's LATEST_SERVER_VERSION. The
# upstream client creates it lazily inside this invocation's private TMPDIR.
server_binaries=()
if [[ ! -v POCKET_IC_BIN ]]; then
  server_binaries+=("${ci_tmp_dir}/pocket-ic-server-16.0.0/pocket-ic")
fi
for server_binary in "${POCKET_IC_BIN-}" "${IC_TESTKIT_POCKET_IC_SERVER-}"; do
  if [[ -n "${server_binary}" ]]; then
    if [[ "${server_binary}" != /* ]]; then
      if [[ "${server_binary}" == */* ]]; then
        server_binary="${repo_root}/${server_binary}"
      else
        # Command::new resolves bare executable names through PATH.
        server_binary="$(cd "${repo_root}" && command -v -- "${server_binary}")" || server_binary="${repo_root}/${server_binary}"
        [[ "${server_binary}" == /* ]] || server_binary="${repo_root}/${server_binary}"
      fi
    fi
    server_binaries+=("${server_binary}")
  fi
done

cleanup() {
  local ci_status="$?"
  local cleanup_status=0
  trap - EXIT

  if [[ "${ci_status}" -ne 0 ]]; then
    echo "Release CI failed; preserving Cargo artifacts for diagnosis." >&2
  fi
  if ! python3 "${repo_root}/scripts/release/stop-owned-pocketic-servers.py" "${ci_tmp_dir}" "${server_binaries[@]}"; then
    echo "error: preserving release CI temporary directory ${ci_tmp_dir} after server cleanup failure" >&2
    cleanup_status=1
  elif ! rm -rf -- "${ci_tmp_dir}"; then
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
