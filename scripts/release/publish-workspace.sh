#!/usr/bin/env bash
set -euo pipefail

readonly registry="crates-io"
release_dir="${BASH_SOURCE[0]}"
[[ "$release_dir" == /* ]] || release_dir="$PWD/$release_dir"
release_dir="$(cd -P "${release_dir%/*}" && printf '%s/.' "$PWD")"
release_dir="${release_dir%/.}"

version="$(/bin/bash "${release_dir}/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)"

# The shared observer owns registry facts; this adapter owns publication policy.
if [[ "${CARGO_NET_OFFLINE:-false}" == true || "${CARGO_NET_OFFLINE:-false}" == 1 ]]; then
  echo 'publication admission requires registry HTTP access; offline policy is preserved' >&2
  exit 2
fi
status=0
bash "$release_dir/../ci/check-crates-io-version.sh" ic-testkit "$version" || status=$?
case "$status" in
  0) echo "ic-testkit ${version} is already published; skipping" ;;
  1) cargo publish --locked --registry "${registry}" -p ic-testkit ;;
  *) echo "registry observation unavailable for ic-testkit $version" >&2; exit 2 ;;
esac
