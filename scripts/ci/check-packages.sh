#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  package) command=(package) ;;
  publish-dry-run) command=(publish --dry-run) ;;
  *) echo 'expected package or publish-dry-run' >&2; exit 2 ;;
esac
[[ $# == 1 ]] || exit 2
repo_root="${BASH_SOURCE[0]}"
[[ "$repo_root" == /* ]] || repo_root="$PWD/$repo_root"
repo_root="$(CDPATH='' cd -P "${repo_root%/*}/../.." && printf '%s/.' "$PWD")"
repo_root="${repo_root%/.}"
cd "$repo_root"
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
work_dir="$(fixture_create "${TMPDIR:-/tmp}/ic-testkit-packages.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Package verification retained"' EXIT

# Package the coordinated set. Verification below uses those exact archives:
# Cargo otherwise reuses an older published sibling with the current version.
cargo "${command[@]}" -p ic-testkit-candid -p ic-testkit --locked \
  --allow-dirty --no-verify --target-dir "$repo_root/target"
for package in ic-testkit-candid ic-testkit; do
  tar -xzf "$repo_root/target/package/$package-$version.crate" -C "$work_dir"
done
# An explicit verification workspace also isolates TMPDIRs inside the checkout.
# Neither normalized package manifest nor archive is changed.
cat > "$work_dir/Cargo.toml" <<EOF
[workspace]
resolver = "3"
members = ["ic-testkit-candid-$version", "ic-testkit-$version"]
[patch.crates-io]
ic-testkit-candid = { path = "ic-testkit-candid-$version" }
EOF
cp Cargo.lock "$work_dir/Cargo.lock"
cargo build --manifest-path "$work_dir/Cargo.toml" --locked --target-dir "$repo_root/target"
fixture_complete=true
