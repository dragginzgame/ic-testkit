#!/usr/bin/env bash
set -euo pipefail

readonly registry="crates-io"
release_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

version="$(/bin/bash "${release_dir}/read-workspace-version.sh" --stable Cargo.toml)"

# Registry admission is read-only and never infers absence from a failed lookup.
# Keep this consumer policy outside the shared snapshot until its canonical
# exact-version observer is committed and reviewed for adoption.
if [[ "${CARGO_NET_OFFLINE:-false}" == true || "${CARGO_NET_OFFLINE:-false}" == 1 ]]; then
  echo 'publication admission requires registry HTTP access; offline policy is preserved' >&2
  exit 2
fi
status="$(curl --disable --silent --show-error --location \
  --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --connect-timeout 10 --max-time 30 --output /dev/null --write-out '%{http_code}' \
  --user-agent 'dragginzgame-ic-testkit (https://github.com/dragginzgame/ic-testkit)' \
  "https://crates.io/api/v1/crates/ic-testkit/$version")" || {
  echo "registry observation unavailable for ic-testkit $version" >&2
  exit 2
}
case "$status" in
  200) echo "ic-testkit ${version} is already published; skipping" ;;
  404) cargo publish --locked --registry "${registry}" -p ic-testkit ;;
  *) echo "registry observation unavailable for ic-testkit $version (HTTP $status)" >&2; exit 2 ;;
esac
