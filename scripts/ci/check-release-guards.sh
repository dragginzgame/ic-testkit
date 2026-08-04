#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
make_bin="$(command -v make)"
work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

clean_case="${work_dir}/clean"
mkdir -p "${clean_case}/bin"
cat >"${clean_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  diff-index) exit 0 ;;
  ls-files)
    printf 'untracked-release-note.md\n'
    exit 0
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${clean_case}/bin/git"
set +e
(
  cd "${clean_case}"
  PATH="${clean_case}/bin:${PATH}" \
    make --no-print-directory -f "${repo_root}/Makefile" ensure-clean
) >/dev/null 2>&1
clean_status="$?"
set -e
[[ "${clean_status}" -ne 0 ]] || fail "ensure-clean accepted an untracked file"

bump_case="${work_dir}/bump"
mkdir -p "${bump_case}/bin"
printf 'version = "0.8.0"\n' >"${bump_case}/Cargo.toml"
cat >"${bump_case}/bin/git" <<'EOF'
#!/bin/bash
if [[ "${1:-}" == "rev-parse" ]]; then
  exit 1
fi
exit 2
EOF
cat >"${bump_case}/bin/bash" <<'EOF'
#!/bin/bash
printf 'changelog %s\n' "$*" >>"${TRACE_FILE}"
exit "${CHANGELOG_STATUS:-0}"
EOF
cat >"${bump_case}/bin/make" <<'EOF'
#!/bin/bash
printf 'make %s\n' "$*" >>"${TRACE_FILE}"
case "${*: -1}" in
  ensure-clean) exit "${CLEAN_STATUS:-0}" ;;
  ci)
    [[ "${CHANGELOG_VERSION:-}" == "0.8.1" ]] || exit 42
    exit "${CI_STATUS:-0}"
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${bump_case}/bin/git" "${bump_case}/bin/bash" "${bump_case}/bin/make"
before_bump="$(<"${bump_case}/Cargo.toml")"

set +e
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" CHANGELOG_STATUS=29 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" patch
) >/dev/null 2>&1
changelog_status="$?"
set -e
[[ "${changelog_status}" -eq 29 ]] \
  || fail "the bump script did not preserve a changelog failure"
[[ "$(<"${bump_case}/Cargo.toml")" == "${before_bump}" ]] \
  || fail "the bump script edited version metadata after a failed changelog gate"
mapfile -t changelog_trace <"${bump_case}/trace"
[[ "${changelog_trace[0]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.8.1" ]] \
  || fail "the bump script did not check the target-version changelog first"
[[ "${#changelog_trace[@]}" -eq 1 ]] \
  || fail "the bump script continued after a failed changelog gate"

: >"${bump_case}/trace"
set +e
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" CI_STATUS=23 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" patch
) >/dev/null 2>&1
ci_status="$?"
set -e
[[ "${ci_status}" -eq 23 ]] || fail "the bump script did not preserve a CI failure"
[[ "$(<"${bump_case}/Cargo.toml")" == "${before_bump}" ]] \
  || fail "the bump script edited version metadata before CI passed"
mapfile -t ci_trace <"${bump_case}/trace"
[[ "${ci_trace[1]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the bump script did not check cleanliness before CI"
[[ "${ci_trace[2]:-}" == "make --no-print-directory ci" ]] \
  || fail "the bump script did not run CI before editing version metadata"

commit_case="${work_dir}/commit"
mkdir -p "${commit_case}/bin"
printf 'version = "0.8.1"\n' >"${commit_case}/Cargo.toml"
cat >"${commit_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  rev-parse) exit 1 ;;
  commit) exit 37 ;;
  tag)
    : >"${TAG_MARKER}"
    exit 0
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${commit_case}/bin/git"
set +e
(
  cd "${commit_case}"
  PATH="${commit_case}/bin:${PATH}" TAG_MARKER="${commit_case}/tagged" \
    make --no-print-directory -f "${repo_root}/Makefile" release-commit
) >/dev/null 2>&1
commit_status="$?"
set -e
[[ "${commit_status}" -ne 0 ]] || fail "release-commit hid a failed commit"
[[ ! -e "${commit_case}/tagged" ]] || fail "release-commit tagged after a failed commit"

push_case="${work_dir}/push"
mkdir -p "${push_case}/bin"
printf 'version = "0.8.1"\n' >"${push_case}/Cargo.toml"
cat >"${push_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
printf 'make %s\n' "$*" >"${TRACE_FILE}"
exit 41
EOF
cat >"${push_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  diff-index) exit 0 ;;
  ls-files) exit 0 ;;
  rev-parse)
    case "${2:-}" in
      'v0.8.1^{}') printf '%s\n' "${TAG_COMMIT}" ;;
      HEAD) printf '%s\n' "${HEAD_COMMIT}" ;;
      *) exit 2 ;;
    esac
    ;;
  push)
    : >"${PUSH_MARKER}"
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${push_case}/bin/make" "${push_case}/bin/git"
set +e
(
  cd "${push_case}"
  PATH="${push_case}/bin:${PATH}" TRACE_FILE="${push_case}/trace" \
    PUSH_MARKER="${push_case}/pushed" TAG_COMMIT=release HEAD_COMMIT=release \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      MAKE="${push_case}/bin/make" release-push
) >/dev/null 2>&1
push_status="$?"
set -e
[[ "${push_status}" -ne 0 ]] || fail "release-push hid a failing CI gate"
[[ ! -e "${push_case}/pushed" ]] || fail "release-push pushed after a failed CI gate"
[[ "$(<"${push_case}/trace")" == "make --no-print-directory ci" ]] \
  || fail "release-push did not run CI before pushing"

rm -f "${push_case}/trace" "${push_case}/pushed"
set +e
(
  cd "${push_case}"
  PATH="${push_case}/bin:${PATH}" TRACE_FILE="${push_case}/trace" \
    PUSH_MARKER="${push_case}/pushed" TAG_COMMIT=stale HEAD_COMMIT=current \
    "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      MAKE="${push_case}/bin/make" release-push
) >/dev/null 2>&1
stale_tag_status="$?"
set -e
[[ "${stale_tag_status}" -ne 0 ]] || fail "release-push accepted a stale release tag"
[[ ! -e "${push_case}/trace" ]] || fail "release-push ran CI with a stale tag"
[[ ! -e "${push_case}/pushed" ]] || fail "release-push pushed a stale tag"

release_block="$(awk '
  $0 == "release-patch:" { found = 1; next }
  found && /^[^[:space:]].*:/ { exit }
  found { print }
' "${repo_root}/Makefile")"
expected_block="$(printf '\t+$(MAKE) --no-print-directory patch\n\t+$(MAKE) --no-print-directory release-stage\n\t+$(MAKE) --no-print-directory release-commit\n\t+$(MAKE) --no-print-directory release-push')"
[[ "${release_block}" == "${expected_block}" ]] \
  || fail "release-patch is not a sequential fail-closed recipe"
