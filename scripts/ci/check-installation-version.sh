#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "$#" -gt 1 ]]; then
  echo "Usage: $0 [VERSION]" >&2
  exit 2
fi

# Release preparation supplies the selected target through CHANGELOG_VERSION;
# ordinary CI checks the current workspace package authority.
version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(/bin/bash "${script_dir}/../release/read-workspace-version.sh" --stable Cargo.toml)"
fi
if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: unsupported installation version ${version}" >&2
  exit 2
fi

requirement="${version%.*}"
for readme in README.md crates/ic-testkit/README.md; do
  # Inspect the maintained TOML dependency example, not migration prose or
  # historical changelogs. Exactly one example must select the current line.
  documented="$(awk '
    /^```toml[[:space:]]*$/ { toml = 1; next }
    /^```/ { toml = 0 }
    toml && /^[[:space:]]*ic-testkit[[:space:]]*=/ {
      if ($0 !~ /^[[:space:]]*ic-testkit[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+"[[:space:]]*(#.*)?$/) {
        print "invalid dependency requirement"
        next
      }
      line = $0
      sub(/^[^"]*"/, "", line)
      sub(/".*$/, "", line)
      print line
    }
  ' "${readme}")"
  if [[ "${documented}" != "${requirement}" ]]; then
    echo "error: ${readme} must contain one TOML ic-testkit requirement selecting ${requirement}" >&2
    exit 1
  fi
done
