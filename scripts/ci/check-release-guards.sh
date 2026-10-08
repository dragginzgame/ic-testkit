#!/usr/bin/env bash
set -euo pipefail

repo_root="${BASH_SOURCE[0]}"
[[ "$repo_root" == /* ]] || repo_root="$PWD/$repo_root"
repo_root="$(cd -P "${repo_root%/*}/../.." && printf '%s/.' "$PWD")"
repo_root="${repo_root%/.}"
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

makefile_cargo_clean=()
trace_count=0
while IFS= read -r trace_line; do
  makefile_cargo_clean[trace_count]="$trace_line"
  trace_count=$((trace_count + 1))
done < <(
  grep -n -E -- 'cargo[[:space:]]+clean' "${repo_root}/Makefile"
)
[[ "${trace_count}" -eq 1 ]] \
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

installation_case="${work_dir}/installation"
mkdir -p "${installation_case}/crates/ic-testkit" \
  "${installation_case}/scripts/ci" "${installation_case}/scripts/release"
cp "${repo_root}/scripts/ci/check-installation-version.sh" \
  "${installation_case}/scripts/ci/"
cp "${repo_root}/scripts/ci/read-cargo-workspace-version.sh" \
  "${installation_case}/scripts/ci/"
printf '[workspace.package]\nversion = "0.8.3"\n' >"${installation_case}/Cargo.toml"
# shellcheck disable=SC2016 # Markdown fences are literal fixture data.
for scenario in current patch target-minor whitespace stale-root stale-package \
  missing duplicate malformed outside-toml invalid-version; do
  target_version=""
  requirement="0.8"
  expected_status=0
  case "${scenario}" in
    patch) target_version="0.8.4" ;;
    target-minor) target_version="0.9.0"; requirement="0.9" ;;
    current | whitespace) ;;
    *) expected_status=1 ;;
  esac
  for readme in README.md crates/ic-testkit/README.md; do
    printf '```toml\n[dev-dependencies]\nic-testkit = "%s"\n```\n' \
      "${requirement}" >"${installation_case}/${readme}"
  done
  # Historical mentions outside the maintained TOML example are unrestricted.
  printf 'Historical migration: ic-testkit = "0.1"\n' >>"${installation_case}/README.md"
  case "${scenario}" in
    whitespace)
      printf '```toml\n  ic-testkit  =  "0.8"  # current line\n```\n' \
        >"${installation_case}/README.md"
      ;;
    stale-root | stale-package)
      readme="README.md"
      [[ "${scenario}" != stale-package ]] || readme="crates/ic-testkit/README.md"
      printf '```toml\nic-testkit = "0.7"\n```\n' \
        >"${installation_case}/${readme}"
      ;;
    missing) rm "${installation_case}/crates/ic-testkit/README.md" ;;
    duplicate) cat "${installation_case}/crates/ic-testkit/README.md" >>"${installation_case}/README.md" ;;
    malformed)
      printf '```toml\nic-testkit = "0.8\n```\n' >"${installation_case}/README.md"
      ;;
    outside-toml) printf 'ic-testkit = "0.8"\n' >"${installation_case}/README.md" ;;
    invalid-version) target_version="0.8.4-rc.1" ;;
  esac
  status=0
  (
    cd "${installation_case}"
    /bin/bash scripts/ci/check-installation-version.sh "${target_version}"
  ) >/dev/null 2>&1 || status="$?"
  if [[ "${expected_status}" -eq 0 ]]; then
    [[ "${status}" -eq 0 ]] || fail "installation check rejected ${scenario}"
  else
    [[ "${status}" -ne 0 ]] || fail "installation check accepted ${scenario}"
  fi
done

clean_case="${work_dir}/clean"
mkdir -p "${clean_case}/bin"
cat >"${clean_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  diff-index) exit 0 ;;
  ls-files)
    printf '%s' "${CLEAN_UNTRACKED:-}"
    exit "${CLEAN_INVENTORY_STATUS:-0}"
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${clean_case}/bin/git"
for scenario in clean untracked failed-empty failed-partial; do
  clean_untracked=""
  inventory_status=0
  case "$scenario" in
    untracked) clean_untracked=untracked-release-note.md ;;
    failed-empty) inventory_status=9 ;;
    failed-partial) inventory_status=9; clean_untracked=partial-inventory ;;
  esac
  clean_status=0
  (
    cd "${clean_case}"
    PATH="${clean_case}/bin:${PATH}" CLEAN_UNTRACKED="$clean_untracked" \
      CLEAN_INVENTORY_STATUS="$inventory_status" \
      make --no-print-directory -f "${repo_root}/Makefile" ensure-clean
  ) >/dev/null 2>&1 || clean_status="$?"
  if [[ "$scenario" == clean ]]; then
    [[ "$clean_status" -eq 0 ]] || fail "ensure-clean rejected a clean inventory"
  else
    [[ "$clean_status" -ne 0 ]] || fail "ensure-clean accepted $scenario"
  fi
done

sequence_case="${work_dir}/sequence"
mkdir -p "${sequence_case}/bin" "${sequence_case}/scripts/ci"
cp "${repo_root}/scripts/ci/run-validation-targets.sh" \
  "${repo_root}/scripts/ci/check-make-execution.sh" "${sequence_case}/scripts/ci/"
cat >"${sequence_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"-f - "* ]]; then
  exec "$REAL_MAKE" "$@"
fi
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
        REAL_MAKE="${make_bin}" VALIDATION_FAILURE_LOG_DIR="${sequence_case}/failures" \
        FAIL_STAGE="${failed_stage}" \
        "${make_bin}" --no-print-directory --jobs=4 -f "${repo_root}/Makefile" \
        MAKE="${sequence_case}/bin/make" CI_TARGETS="${stages}" RELEASE_CHECK_TARGETS="${stages}" "${target}"
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

check_make_sequence ci "check-first check-second check-third"
check_make_sequence release-check "check-first check-second check-third"

bash "${repo_root}/scripts/ci/check-release-commands.sh" "${repo_root}" make/tools.mk

/bin/bash "${repo_root}/scripts/ci/check-release-metadata.sh"
