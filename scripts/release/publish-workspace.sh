#!/usr/bin/env bash
set -euo pipefail

readonly registry="crates-io"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: failed to read a release version from Cargo.toml" >&2
  exit 1
fi

if cargo info "ic-testkit@${version}" --registry "${registry}" >/dev/null 2>&1; then
  echo "ic-testkit ${version} is already published; skipping"
else
  cargo publish --locked --registry "${registry}" -p ic-testkit
fi
