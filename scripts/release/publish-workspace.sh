#!/usr/bin/env bash
set -euo pipefail

readonly registry="crates-io"
release_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

version="$(/bin/bash "${release_dir}/read-workspace-version.sh" --stable Cargo.toml)"

if cargo info "ic-testkit@${version}" --registry "${registry}" >/dev/null 2>&1; then
  echo "ic-testkit ${version} is already published; skipping"
else
  cargo publish --locked --registry "${registry}" -p ic-testkit
fi
