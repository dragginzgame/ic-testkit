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

if grep -R -n -E -- 'cargo[[:space:]]+clean' \
  "${repo_root}/.github/workflows" \
  "${repo_root}/scripts/ci" \
  "${repo_root}/scripts/release" >/dev/null; then
  fail "a CI, release, or publish script invokes the Cargo clean subcommand"
fi

mapfile -t makefile_cargo_clean < <(
  grep -n -E -- 'cargo[[:space:]]+clean' "${repo_root}/Makefile"
)
[[ "${#makefile_cargo_clean[@]}" -eq 1 ]] \
  || fail "Cargo clean must exist only as the standalone Make target"
expected_manual_clean_recipe=$'\tcargo '"clean"
[[ "${makefile_cargo_clean[0]#*:}" == "${expected_manual_clean_recipe}" ]] \
  || fail "the standalone Make clean target has an unexpected recipe"

ci_targets_block="$(awk '
  /^CI_TARGETS :=/ { found = 1 }
  found { print }
  found && $0 !~ /\\$/ { exit }
' "${repo_root}/Makefile")"
for ci_target in ${ci_targets_block//\\/}; do
  [[ "${ci_target}" != "clean" ]] \
    || fail "the standalone clean target is reachable from make ci"
done

version_case="${work_dir}/version"
mkdir -p "${version_case}"
printf '[workspace.package]\nversion = "0.8.3"\n' >"${version_case}/Cargo.toml"
version="$(
  /bin/bash "${repo_root}/scripts/release/read-workspace-version.sh" \
    --stable "${version_case}/Cargo.toml"
)"
[[ "${version}" == "0.8.3" ]] \
  || fail "the version reader did not return a stable workspace version"

printf '[workspace.package]\nversion = "0.8.3-rc.1"\n' >"${version_case}/Cargo.toml"
version="$(
  /bin/bash "${repo_root}/scripts/release/read-workspace-version.sh" \
    "${version_case}/Cargo.toml"
)"
[[ "${version}" == "0.8.3-rc.1" ]] \
  || fail "the version reader rejected a bump-compatible prerelease version"
set +e
/bin/bash "${repo_root}/scripts/release/read-workspace-version.sh" \
  --stable "${version_case}/Cargo.toml" >/dev/null 2>&1
prerelease_status="$?"
set -e
[[ "${prerelease_status}" -eq 2 ]] \
  || fail "the stable version reader accepted a prerelease version"

printf '[workspace]\n\n[unrelated]\nversion = "9.9.9"\n' \
  >"${version_case}/Cargo.toml"
set +e
/bin/bash "${repo_root}/scripts/release/read-workspace-version.sh" \
  "${version_case}/Cargo.toml" >/dev/null 2>&1
missing_version_status="$?"
set -e
[[ "${missing_version_status}" -eq 1 ]] \
  || fail "the version reader accepted a manifest without a package version"

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
printf '[workspace.package]\nversion = "0.8.0"\n' >"${bump_case}/Cargo.toml"
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
  release-ci)
    [[ "${CHANGELOG_VERSION:-}" == "${EXPECTED_CHANGELOG_VERSION:-0.8.1}" ]] || exit 42
    exit "${CI_STATUS:-0}"
    ;;
  *) exit 2 ;;
esac
EOF
cat >"${bump_case}/bin/cargo" <<'EOF'
#!/bin/bash
[[ "$*" == "generate-lockfile" ]] || exit 2
EOF
chmod +x "${bump_case}/bin/git" "${bump_case}/bin/bash" \
  "${bump_case}/bin/make" "${bump_case}/bin/cargo"
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
[[ "${ci_trace[2]:-}" == "make --no-print-directory release-ci" ]] \
  || fail "the bump script did not run release CI before editing version metadata"

: >"${bump_case}/trace"
(
  cd "${bump_case}"
  PATH="${bump_case}/bin:${PATH}" TRACE_FILE="${bump_case}/trace" \
    EXPECTED_CHANGELOG_VERSION=0.9.0 \
    /bin/bash "${repo_root}/scripts/release/bump-version.sh" minor
) >/dev/null 2>&1
[[ "$(<"${bump_case}/Cargo.toml")" == $'[workspace.package]\nversion = "0.9.0"' ]] \
  || fail "the minor bump script did not reset the patch component"
mapfile -t minor_trace <"${bump_case}/trace"
[[ "${minor_trace[0]:-}" == "changelog scripts/ci/check-changelog-version.sh 0.9.0" ]] \
  || fail "the minor bump script did not check the target-version changelog first"
[[ "${minor_trace[1]:-}" == "make --no-print-directory ensure-clean" ]] \
  || fail "the minor bump script did not check cleanliness before CI"
[[ "${minor_trace[2]:-}" == "make --no-print-directory release-ci" ]] \
  || fail "the minor bump script did not run release CI before editing version metadata"

cleanup_case="${work_dir}/cleanup"
mkdir -p "${cleanup_case}/bin" "${cleanup_case}/tmp"
cat >"${cleanup_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
printf 'make %s\n' "$*" >>"${TRACE_FILE}"
[[ "$*" == "--no-print-directory ci" ]] || exit 2
printf '%s\n' "${TMPDIR:-}" >"${TMPDIR_TRACE}"
mkdir -p "${TMPDIR}/nested-artifact"
exit "${CI_STATUS:-0}"
EOF
cat >"${cleanup_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*" >>"${TRACE_FILE}"
exit 97
EOF
chmod +x "${cleanup_case}/bin/make" "${cleanup_case}/bin/cargo"
set +e
(
  cd "${repo_root}"
  PATH="${cleanup_case}/bin:${PATH}" TRACE_FILE="${cleanup_case}/trace" \
    TMPDIR_TRACE="${cleanup_case}/tmpdir" TMPDIR="${cleanup_case}/tmp" CI_STATUS=23 \
    /bin/bash scripts/release/run-ci.sh
) >/dev/null 2>&1
cleanup_ci_status="$?"
set -e
[[ "${cleanup_ci_status}" -eq 23 ]] \
  || fail "release CI cleanup did not preserve the CI failure"
mapfile -t cleanup_trace <"${cleanup_case}/trace"
[[ "${cleanup_trace[0]:-}" == "make --no-print-directory ci" ]] \
  || fail "the release CI wrapper did not run the CI gate"
[[ "${#cleanup_trace[@]}" -eq 1 ]] \
  || fail "the failed release CI wrapper invoked Cargo during cleanup"
ci_tmp_dir="$(<"${cleanup_case}/tmpdir")"
[[ "${ci_tmp_dir}" == "${cleanup_case}/tmp/ic-testkit-release-ci."* ]] \
  || fail "the release CI wrapper did not isolate temporary artifacts"
[[ ! -e "${ci_tmp_dir}" ]] \
  || fail "the release CI wrapper left its temporary directory behind"

: >"${cleanup_case}/trace"
(
  cd "${repo_root}"
  PATH="${cleanup_case}/bin:${PATH}" TRACE_FILE="${cleanup_case}/trace" \
    TMPDIR_TRACE="${cleanup_case}/tmpdir" TMPDIR="${cleanup_case}/tmp" CI_STATUS=0 \
    /bin/bash scripts/release/run-ci.sh
) >/dev/null
mapfile -t successful_cleanup_trace <"${cleanup_case}/trace"
[[ "${successful_cleanup_trace[0]:-}" == "make --no-print-directory ci" ]] \
  || fail "the successful release CI wrapper did not run the CI gate"
[[ "${#successful_cleanup_trace[@]}" -eq 1 ]] \
  || fail "the successful release CI wrapper invoked Cargo during cleanup"
successful_ci_tmp_dir="$(<"${cleanup_case}/tmpdir")"
[[ ! -e "${successful_ci_tmp_dir}" ]] \
  || fail "the successful release CI wrapper left its temporary directory behind"

commit_case="${work_dir}/commit"
mkdir -p "${commit_case}/bin"
printf '[workspace.package]\nversion = "0.8.1"\n' >"${commit_case}/Cargo.toml"
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
printf '[workspace.package]\nversion = "0.8.1"\n' >"${push_case}/Cargo.toml"
cat >"${push_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
printf 'git %s\n' "$*" >>"${TRACE_FILE}"
case "${1:-}" in
  diff-index) exit "${CLEAN_STATUS:-0}" ;;
  ls-files)
    [[ -z "${UNTRACKED_FILE:-}" ]] || printf '%s\n' "${UNTRACKED_FILE}"
    ;;
  rev-parse)
    case "${2:-}" in
      'v0.8.1^{}') printf '%s\n' "${TAG_COMMIT}" ;;
      HEAD) printf '%s\n' "${HEAD_COMMIT}" ;;
      *) exit 2 ;;
    esac
    ;;
  push)
    exit "${PUSH_STATUS:-0}"
    ;;
  *) exit 2 ;;
esac
EOF
# A push may not start another build or validation stage. These command
# doubles make such a regression harmless and visible in the trace.
for program in make cargo; do
  cat >"${push_case}/bin/${program}" <<'EOF'
#!/usr/bin/env bash
printf 'unexpected %s %s\n' "${0##*/}" "$*" >>"${TRACE_FILE}"
exit 97
EOF
  chmod +x "${push_case}/bin/${program}"
done
chmod +x "${push_case}/bin/git"

for push_scenario in success stale-tag dirty untracked push-failure; do
  : >"${push_case}/trace"
  clean_status=0
  push_status=0
  tag_commit=release
  untracked_file=""
  expected_trace=$'git diff-index --quiet HEAD --\ngit ls-files --others --exclude-standard'
  case "${push_scenario}" in
    stale-tag) tag_commit=stale ;;
    dirty) clean_status=23; expected_trace='git diff-index --quiet HEAD --' ;;
    untracked) untracked_file=untracked-release-note.md ;;
    push-failure) push_status=37 ;;
  esac
  if [[ "${clean_status}" -eq 0 && -z "${untracked_file}" ]]; then
    expected_trace+=$'\ngit rev-parse v0.8.1^{}\ngit rev-parse HEAD'
    [[ "${tag_commit}" != release ]] || expected_trace+=$'\ngit push --follow-tags'
  fi
  status=0
  (
    cd "${push_case}"
    PATH="${push_case}/bin:${PATH}" TRACE_FILE="${push_case}/trace" \
      TAG_COMMIT="${tag_commit}" HEAD_COMMIT=release CLEAN_STATUS="${clean_status}" \
      UNTRACKED_FILE="${untracked_file}" PUSH_STATUS="${push_status}" \
      "${make_bin}" --no-print-directory -f "${repo_root}/Makefile" \
      MAKE="${push_case}/bin/make" release-push
  ) >/dev/null 2>&1 || status="$?"
  if [[ "${push_scenario}" == success ]]; then
    [[ "${status}" -eq 0 ]] || fail "release-push rejected a clean release tag"
  else
    [[ "${status}" -ne 0 ]] || fail "release-push hid ${push_scenario}"
  fi
  [[ "$(<"${push_case}/trace")" == "${expected_trace}" ]] \
    || fail "release-push ran unexpected commands for ${push_scenario}"
done

sequence_case="${work_dir}/sequence"
mkdir -p "${sequence_case}/bin"
cat >"${sequence_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
stage="${*: -1}"
printf '%s\n' "${stage}" >>"${TRACE_FILE}"
[[ "${stage}" != "${FAIL_STAGE:-}" ]] || exit 23
EOF
cat >"${sequence_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
exit 97
EOF
chmod +x "${sequence_case}/bin/make" "${sequence_case}/bin/git"

# Execute the real outer recipes; recursive stages are harmless recorded commands.
# Inject failure at every stage to prove ordering and fail-closed execution.
check_make_sequence() {
  local target="$1" stages="$2" failed_stage stage status expected
  local -a ordered_stages
  read -r -a ordered_stages <<<"${stages}"
  for failed_stage in "" "${ordered_stages[@]}"; do
    : >"${sequence_case}/trace"
    status=0
    (
      cd "${sequence_case}"
      PATH="${sequence_case}/bin:${PATH}" TRACE_FILE="${sequence_case}/trace" \
        FAIL_STAGE="${failed_stage}" \
        "${make_bin}" --no-print-directory --jobs=4 -f "${repo_root}/Makefile" \
        MAKE="${sequence_case}/bin/make" CI_TARGETS="${stages}" "${target}"
    ) >/dev/null 2>&1 || status="$?"
    if [[ -z "${failed_stage}" ]]; then
      [[ "${status}" -eq 0 ]] || fail "${target} failed with successful stages"
    else
      [[ "${status}" -ne 0 ]] || fail "${target} hid failure in ${failed_stage}"
    fi
    expected=""
    for stage in "${ordered_stages[@]}"; do
      expected+="${stage}"$'\n'
      [[ "${stage}" != "${failed_stage}" ]] || break
    done
    [[ "$(<"${sequence_case}/trace")" == "${expected%$'\n'}" ]] \
      || fail "${target} reordered stages or continued after ${failed_stage:-success}"
  done
}

check_make_sequence release-patch "patch release-stage release-commit release-push"
check_make_sequence release-minor "minor release-stage release-commit release-push"
check_make_sequence ci "check-first check-second check-third"

PYTHONDONTWRITEBYTECODE=1 python3 "${repo_root}/scripts/ci/test-release-pocketic-cleanup.py"
