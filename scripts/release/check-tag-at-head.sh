#!/usr/bin/env bash
set -euo pipefail

release_dir="${BASH_SOURCE[0]}"
[[ "$release_dir" == /* ]] || release_dir="$PWD/$release_dir"
release_dir="$(cd -P "${release_dir%/*}" && printf '%s/.' "$PWD")"
release_dir="${release_dir%/.}"
version="$(/bin/bash "${release_dir}/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)"

head_commit="$(git rev-parse --verify 'HEAD^{commit}')" || exit 1
exec /bin/bash "${release_dir}/../ci/check-release-tag.sh" "$head_commit" "$version"
