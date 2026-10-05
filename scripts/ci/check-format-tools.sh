#!/usr/bin/env bash
set -euo pipefail
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=/dev/null
source "$root/ci/tool-versions.env"
[[ "$(cargo sort --version)" == "cargo-sort $IC_TESTKIT_CARGO_SORT_VERSION" ]] || {
    echo "Formatting requires prepared cargo-sort $IC_TESTKIT_CARGO_SORT_VERSION; run make install-format-tools during setup." >&2
    exit 1
}
cargo fmt --version >/dev/null || {
    echo "Formatting requires prepared rustfmt for the selected toolchain; run rustup component add rustfmt during setup." >&2
    exit 1
}
