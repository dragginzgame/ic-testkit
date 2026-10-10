#!/usr/bin/env bash
set -euo pipefail

repo_root="${BASH_SOURCE[0]}"
[[ "$repo_root" == /* ]] || repo_root="$PWD/$repo_root"
repo_root="$(cd -P "${repo_root%/*}/../.." && printf '%s/.' "$PWD")"
repo_root="${repo_root%/.}"
make_bin="$(command -v make)"
work_dir="$(mktemp -d)"
fixture_complete=false
cleanup() {
  local status=$?
  [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
  if [[ "$status" == 0 ]]; then rm -rf "$work_dir"
  else echo "Failed release guard qualification retained at $work_dir" >&2; fi
  exit "$status"
}
trap cleanup EXIT

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

# Exercise the actual consumer Makefile with harmless formatter/runner effects.
# Even replacing flag variables must not conceal the real outer invocation.
(
  unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
  modes_case="${work_dir}/modes"
  mkdir -p "$modes_case/make" "$modes_case/scripts/ci" "$modes_case/ci"
  cp "$repo_root/Makefile" "$modes_case/"
  cp "$repo_root/make/"*.mk "$modes_case/make/"
  cp "$repo_root/ci/tool-versions.env" "$modes_case/ci/"
  for helper in check-make-execution.sh check-format-tools.sh run-formatting.sh; do
    cp "$repo_root/scripts/ci/$helper" "$modes_case/scripts/ci/"
  done
  cat > "$modes_case/scripts/ci/run-release.sh" <<'EOF'
#!/usr/bin/env bash
printf 'runner\n' >> "$MODE_EVENTS"
exit 23
EOF
  cat > "$modes_case/cargo" <<'EOF'
#!/usr/bin/env bash
case "$*" in
  'sort --version') printf 'cargo-sort 2.1.4\n'; exit 0 ;;
  'fmt --version') exit 0 ;;
esac
printf 'formatter\n' >> "$MODE_EVENTS"
exit 23
EOF
  chmod +x "$modes_case/cargo"
  export MODE_EVENTS="$modes_case/events"
  for target in fmt fmt-check release-patch release-minor release-major release-resume; do
    for mode in -i --ignore-errors -n --dry-run --just-print --recon -t --touch -q --question -kin; do
      for selection in direct inherited cleared replaced both-hidden; do
        : > "$MODE_EVENTS"
        arguments=("$mode" "$target" VERSION=0.28.2 "FORMAT_CARGO=$modes_case/cargo")
        case "$selection" in
          cleared) arguments+=(MAKEFLAGS=) ;;
          replaced) arguments+=(MAKEFLAGS=--no-print-directory) ;;
          both-hidden) arguments+=(MAKEFLAGS= MFLAGS=) ;;
        esac
        status=0
        if [[ "$selection" == inherited ]]; then
          MAKEFLAGS="$mode" "$make_bin" -C "$modes_case" "${arguments[@]:1}" \
            > "$modes_case/mode.log" 2>&1 || status=$?
        else
          "$make_bin" -C "$modes_case" "${arguments[@]}" \
            > "$modes_case/mode.log" 2>&1 || status=$?
        fi
        [[ "$status" -eq 2 && ! -s "$MODE_EVENTS" ]] \
          || fail "unsafe Make admission: $target $mode $selection (status $status)"
      done
    done
  done
  "$make_bin" -C "$modes_case" -j2 help MAKEFLAGS= > "$modes_case/help.log"
  [[ ! -s "$MODE_EVENTS" ]] || fail "harmless help dispatched a tool"
  status=0
  RUNNER_TEMP="$modes_case" "$make_bin" -C "$modes_case" fmt-check \
    "FORMAT_CARGO=$modes_case/cargo" > "$modes_case/format-failure.log" 2>&1 || status=$?
  [[ "$status" -eq 2 && "$(<"$MODE_EVENTS")" == formatter ]] \
    || fail "safe formatting did not propagate the formatter's failure"
)

bash "${repo_root}/scripts/ci/check-release-commands.sh" "${repo_root}" \
  make/tools.mk make/release.mk make/rust-format.mk make/execution.mk \
  scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh

# Qualify the actual consumer's complete aggregates with substituted installers,
# Cargo and owner CLI. Parallel Make must retain ordering and checks stay offline.
(
  unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
  export TOOL_ROOT="${work_dir}/tools" TOOL_TRACE="${work_dir}/tools/trace"
  mkdir -p "$TOOL_ROOT/make" "$TOOL_ROOT/scripts/dev" "$TOOL_ROOT/scripts/ci" "$TOOL_ROOT/bin" "$TOOL_ROOT/ci"
  cp "$repo_root/Makefile" "$TOOL_ROOT/"
  cp "$repo_root"/make/*.mk "$TOOL_ROOT/make/"
  cp "$repo_root/ci/tool-versions.env" "$TOOL_ROOT/ci/"
  cp "$repo_root/scripts/ci/check-make-execution.sh" "$TOOL_ROOT/scripts/ci/"
  for name in host ic rust; do
    cat > "$TOOL_ROOT/scripts/dev/install-$name-tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
name="${0##*/}"; name="${name#install-}"; name="${name%-tools.sh}"
mode=setup
[[ "${*: -1}" != --check ]] || mode=check
[[ "${*: -1}" != --preflight ]] || mode=preflight
printf '%s:%s\n' "$name" "$mode" >> "$TOOL_TRACE"
[[ "${TOOL_FAIL:-}" != "$name:$mode" ]] || exit 23
[[ "$mode" != preflight ]] || exit 0
if [[ "$mode" == setup ]]; then : > "$TOOL_ROOT/$name-ready"
else [[ -f "$TOOL_ROOT/$name-ready" ]]; fi
STUB
  done
  cat > "$TOOL_ROOT/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == build && "$*" == *--locked* ]] || exit 97
printf 'cargo:build\n' >> "$TOOL_TRACE"
mkdir -p "$TOOL_ROOT/target/debug"
cat > "$TOOL_ROOT/target/debug/ic-testkit-server" <<'SERVER'
#!/usr/bin/env bash
set -euo pipefail
printf 'server:%s\n' "$1" >> "$TOOL_TRACE"
case "$1" in
  setup) printf 'retained server bytes\n' > "$TOOL_ROOT/server-bytes" ;;
  check) [[ -f "$TOOL_ROOT/server-bytes" ]]; printf '/prepared/server\n' ;;
  *) exit 97 ;;
esac
SERVER
chmod +x "$TOOL_ROOT/target/debug/ic-testkit-server"
STUB
  chmod +x "$TOOL_ROOT/bin/cargo"
  export PATH="$TOOL_ROOT/bin:$PATH"
  "$make_bin" --no-print-directory -C "$TOOL_ROOT" > "$TOOL_ROOT/help.log"
  [[ ! -e "$TOOL_TRACE" ]] || fail 'default goal dispatched tools'
  status=0
  "$make_bin" --no-print-directory -j4 -C "$TOOL_ROOT" tools-check > "$TOOL_ROOT/missing.log" 2>&1 || status=$?
  [[ "$status" == 2 && "$(cat "$TOOL_TRACE")" == host:check ]] || fail 'missing common tool check installed or continued'
  : > "$TOOL_TRACE"
  "$make_bin" --no-print-directory -j4 -C "$TOOL_ROOT" install-tools > "$TOOL_ROOT/install.log" 2>&1
  printf '%s\n' ic:preflight rust:preflight host:setup ic:setup rust:setup cargo:build server:setup > "$TOOL_ROOT/expected"
  cmp "$TOOL_TRACE" "$TOOL_ROOT/expected" || fail 'complete setup ordering changed'
  cp "$TOOL_ROOT/server-bytes" "$TOOL_ROOT/retained-server"
  : > "$TOOL_TRACE"
  "$make_bin" --no-print-directory -j4 -C "$TOOL_ROOT" tools-check > "$TOOL_ROOT/check.log" 2>&1
  printf '%s\n' host:check ic:check rust:check server:check > "$TOOL_ROOT/expected"
  cmp "$TOOL_TRACE" "$TOOL_ROOT/expected" || fail 'complete offline check rebuilt or installed'
  cmp "$TOOL_ROOT/server-bytes" "$TOOL_ROOT/retained-server"
  : > "$TOOL_TRACE"
  status=0
  TOOL_FAIL=rust:setup "$make_bin" --no-print-directory -j4 -C "$TOOL_ROOT" install-tools > "$TOOL_ROOT/failed.log" 2>&1 || status=$?
  printf '%s\n' ic:preflight rust:preflight host:setup ic:setup rust:setup > "$TOOL_ROOT/expected"
  [[ "$status" == 2 ]] || fail 'setup failure status lost'
  cmp "$TOOL_TRACE" "$TOOL_ROOT/expected" || fail 'failed common setup dispatched product'
  cmp "$TOOL_ROOT/server-bytes" "$TOOL_ROOT/retained-server"
  for prerequisite in ic rust; do
    : > "$TOOL_TRACE"
    status=0
    TOOL_FAIL="$prerequisite:preflight" "$make_bin" --no-print-directory -j4 -C "$TOOL_ROOT" install-tools > "$TOOL_ROOT/preflight-$prerequisite.log" 2>&1 || status=$?
    printf '%s\n' ic:preflight > "$TOOL_ROOT/expected"
    [[ "$prerequisite" != rust ]] || printf '%s\n' rust:preflight >> "$TOOL_ROOT/expected"
    [[ "$status" == 2 ]] || fail 'preflight failure status lost'
    cmp "$TOOL_TRACE" "$TOOL_ROOT/expected" || fail 'preflight failure dispatched setup'
    cmp "$TOOL_ROOT/server-bytes" "$TOOL_ROOT/retained-server"
  done
  rm "$TOOL_ROOT/target/debug/ic-testkit-server"
  : > "$TOOL_TRACE"
  status=0
  "$make_bin" --no-print-directory -j4 -C "$TOOL_ROOT" tools-check > "$TOOL_ROOT/missing-cli.log" 2>&1 || status=$?
  printf '%s\n' host:check ic:check rust:check > "$TOOL_ROOT/expected"
  [[ "$status" == 2 ]] || fail 'missing owner CLI was accepted'
  cmp "$TOOL_TRACE" "$TOOL_ROOT/expected" || fail 'missing CLI check compiled or installed'
)

/bin/bash "${repo_root}/scripts/ci/check-release-metadata.sh"
fixture_complete=true
