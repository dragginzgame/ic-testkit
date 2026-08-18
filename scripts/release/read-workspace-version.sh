#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 [--stable] [MANIFEST]" >&2
}

stable=false
if [[ "${1:-}" == "--stable" ]]; then
  stable=true
  shift
fi
if [[ "$#" -gt 1 || "${1:-}" == -* ]]; then
  usage
  exit 2
fi

manifest="${1:-Cargo.toml}"
if [[ ! -r "${manifest}" ]]; then
  echo "error: cannot read workspace manifest ${manifest}" >&2
  exit 1
fi

version="$(sed -n '
  /^[[:space:]]*\[workspace\.package\][[:space:]]*$/,/^[[:space:]]*\[/ {
    s/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)"[[:space:]]*$/\1/
    t found
    b
    :found
    p
    q
  }
' "${manifest}")"
if [[ -z "${version}" ]]; then
  echo "error: failed to read package version from ${manifest}" >&2
  exit 1
fi
if [[ "${stable}" == true && ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: unsupported stable release version ${version}" >&2
  exit 2
fi

printf '%s\n' "${version}"
