#!/usr/bin/env bash
set -euo pipefail

release_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version="$(/bin/bash "${release_dir}/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)"

head_commit="$(git rev-parse --verify 'HEAD^{commit}')" || exit 1
[[ "$head_commit" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || {
  echo 'error: invalid selected release commit' >&2
  exit 1
}
exec /bin/bash "${release_dir}/../ci/check-release-tag.sh" "$head_commit" "$version"
