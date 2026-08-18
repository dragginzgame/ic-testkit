#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$#" -gt 1 ]]; then
  echo "Usage: $0 [VERSION]" >&2
  exit 2
fi

version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(/bin/bash "${script_dir}/../release/read-workspace-version.sh" --stable Cargo.toml)"
fi
if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: unsupported version format ${version}" >&2
  exit 2
fi

version_pattern="${version//./\\.}"
heading_pattern="^## \\[${version_pattern}\\]( - .+)?$"

if ! grep -Eq -- "${heading_pattern}" CHANGELOG.md; then
  echo "error: CHANGELOG.md has no release heading for package version ${version}" >&2
  exit 1
fi
if ! head_changelog="$(git show HEAD:CHANGELOG.md 2>/dev/null)"; then
  echo "error: CHANGELOG.md is not committed in HEAD" >&2
  exit 1
fi
if ! grep -Eq -- "${heading_pattern}" <<<"${head_changelog}"; then
  echo "error: CHANGELOG.md in HEAD has no release heading for package version ${version}" >&2
  exit 1
fi
