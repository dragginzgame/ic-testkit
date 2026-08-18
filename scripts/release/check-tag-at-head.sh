#!/usr/bin/env bash
set -euo pipefail

release_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version="$(/bin/bash "${release_dir}/read-workspace-version.sh" --stable Cargo.toml)"

if ! tag_commit="$(git rev-parse "v${version}^{}" 2>/dev/null)"; then
  echo "error: release tag v${version} does not exist" >&2
  exit 1
fi
head_commit="$(git rev-parse HEAD)"
if [[ "${tag_commit}" != "${head_commit}" ]]; then
  echo "error: release tag v${version} does not point to HEAD" >&2
  exit 1
fi
