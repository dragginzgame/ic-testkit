#!/usr/bin/env bash
# Exercise the upstream proposal against an isolated published PocketIC source copy.
set -euo pipefail

if [[ $# != 1 ]]; then
    echo "Usage: $0 /path/to/pocket-ic-16.0.0" >&2
    exit 2
fi

source_root=$1
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
probe_root=$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-pocketic-teardown.XXXXXX")
trap 'rm -rf -- "$probe_root"' EXIT

cp -R -- "$source_root" "$probe_root/pocket-ic"
git -C "$probe_root/pocket-ic" apply \
    "$repo_root/docs/upstream/pocket-ic-bounded-teardown.patch"
cp -- "$repo_root/scripts/dev/pocketic-teardown-probe.rs" "$probe_root/probe.rs"
cp -- "$repo_root/Cargo.lock" "$probe_root/Cargo.lock"

cat > "$probe_root/Cargo.toml" <<'MANIFEST'
[package]
name = "pocketic-teardown-probe"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]
resolver = "2"
exclude = ["pocket-ic"]

[dependencies]
pocket-ic = { path = "pocket-ic" }
serde_json = "1"
tokio = { version = "1", features = ["full"] }

[[test]]
name = "bounded_teardown"
path = "probe.rs"
MANIFEST

cargo test --offline --manifest-path "$probe_root/Cargo.toml" \
    --target-dir "${CARGO_TARGET_DIR:-$repo_root/target}" \
    --test bounded_teardown -- --nocapture
