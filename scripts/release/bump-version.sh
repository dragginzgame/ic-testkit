#!/usr/bin/env bash
set -euo pipefail

release_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  echo "Usage: $0 patch|minor" >&2
}

bump="${1:-}"
case "${bump}" in
  patch | minor) ;;
  *)
    usage
    exit 2
    ;;
esac

previous_version="$(/bin/bash "${release_dir}/read-workspace-version.sh" Cargo.toml)"

IFS=. read -r major minor patch_extra <<<"${previous_version}"
patch="${patch_extra%%[-+]*}"
if [[ ! "${major}" =~ ^[0-9]+$ || ! "${minor}" =~ ^[0-9]+$ || ! "${patch}" =~ ^[0-9]+$ ]]; then
  echo "error: unsupported version format ${previous_version}" >&2
  exit 1
fi

case "${bump}" in
  patch)
    patch=$((patch + 1))
    ;;
  minor)
    minor=$((minor + 1))
    patch=0
    ;;
esac

new_version="${major}.${minor}.${patch}"

if git rev-parse "v${new_version}" >/dev/null 2>&1; then
  echo "error: tag v${new_version} already exists; aborting" >&2
  exit 1
fi

echo "Checking committed changelog for target version ${new_version}..."
bash scripts/ci/check-changelog-version.sh "${new_version}"

echo "Running full CI gate before version bump..."
make --no-print-directory ensure-clean
CHANGELOG_VERSION="${new_version}" make --no-print-directory release-ci

perl -0pi -e "s/version = \"\\Q${previous_version}\\E\"/version = \"${new_version}\"/g" Cargo.toml
cargo generate-lockfile >/dev/null

echo "Bumped: ${previous_version} -> ${new_version}"
echo "Next:"
echo "  git diff"
echo "  make release-stage"
echo "  make release-commit"
echo "  make release-push"
