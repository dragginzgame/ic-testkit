#!/usr/bin/env bash
set -euo pipefail

release_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version="$(/bin/bash "${release_dir}/read-workspace-version.sh" --stable Cargo.toml)"

head_commit="$(git rev-parse --verify 'HEAD^{commit}')" || exit 1
[[ "$head_commit" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || {
  echo 'error: invalid selected release commit' >&2
  exit 1
}
tag="refs/tags/v$version"
tag_type="$(git cat-file -t "$tag")" || exit 1
[[ "$tag_type" == tag ]] || {
  echo "error: release tag v${version} must be annotated" >&2
  exit 1
}
tag_commit="$(git rev-parse --verify "$tag^{commit}")" || exit 1
if [[ "${tag_commit}" != "${head_commit}" ]]; then
  echo "error: release tag v${version} does not point to HEAD" >&2
  exit 1
fi
