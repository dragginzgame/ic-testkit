#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
rewrite=false
if [[ "${1:-}" == --rewrite ]]; then
  rewrite=true
  shift
fi
if [[ "$#" -gt 1 || ( "$rewrite" == true && ( "$#" -ne 1 || -z "${1:-}" ) ) ]]; then
  echo "Usage: $0 [VERSION] | --rewrite VERSION" >&2
  exit 2
fi

# Ordinary CI checks the current workspace package authority; the release
# adapter uses the same locator to update its private copy before checking it.
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
  # historical changelogs. Locate one valid example; ordinary checks require
  # its current line, while preparation replaces that requirement alone.
  if [[ "$rewrite" == true && ( ! -f "$readme" || -L "$readme" ) ]]; then
    echo "error: cannot rewrite missing or symlinked ${readme}" >&2
    exit 1
  fi
  documented="$(awk -v locate="$rewrite" '
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
      if (locate == "true") {
        printf "%s %d\n", line, NR
      } else {
        print line
      }
    }
  ' "${readme}")"
  valid=false
  if [[ "$rewrite" == true ]]; then
    [[ ! "$documented" =~ ^[0-9]+\.[0-9]+[[:blank:]][0-9]+$ ]] || valid=true
  else
    [[ "$documented" != "$requirement" ]] || valid=true
  fi
  if [[ "$valid" != true ]]; then
    echo "error: ${readme} must contain one TOML ic-testkit requirement selecting ${requirement}" >&2
    exit 1
  fi
  if [[ "$rewrite" == true ]]; then
    # The parser owns membership and validates one matching line. Perl only
    # replaces that line's quoted requirement, preserving all other bytes.
    REQUIREMENT="$requirement" REQUIREMENT_LINE="${documented##* }" perl -pi -e '
      if ($. == $ENV{REQUIREMENT_LINE}) {
        s/"[0-9]+\.[0-9]+"/"$ENV{REQUIREMENT}"/
          or die "installation requirement changed before replacement\n";
      }
    ' "$readme"
  fi
done
