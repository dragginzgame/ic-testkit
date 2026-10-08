#!/usr/bin/env bash
set -euo pipefail

repo_root="${BASH_SOURCE[0]}"
[[ "$repo_root" == /* ]] || repo_root="$PWD/$repo_root"
repo_root="$(cd -P "${repo_root%/*}/../.." && printf '%s/.' "$PWD")"
repo_root="${repo_root%/.}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-testkit-release-metadata.XXXXXX")"
cleanup() {
  local status=$?
  if [[ "$status" -eq 0 ]]; then
    rm -rf "$work_dir"
  else
    echo "Failed metadata qualification retained at $work_dir" >&2
  fi
}
trap cleanup EXIT
fail() { echo "metadata qualification failed: $*" >&2; exit 1; }
# Bash 3.2 does not apply errexit to standalone [[ ... ]] commands. Keep
# rejection explicit in both the command stubs and the fixture assertions.
export REAL_GIT
REAL_GIT="$(command -v git)"
export REAL_CAT
REAL_CAT="$(command -v cat)"
export REAL_CARGO
REAL_CARGO="$(command -v cargo)"
real_make="$(command -v make)"
mkdir -p "$work_dir/bin"
cat > "$work_dir/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  hash-object)
    shift
    "$REAL_GIT" hash-object "$@"
    if [[ -z "${ADAPTER_HASH_PATH:-}" || "${*: -1}" == "$ADAPTER_HASH_PATH" ]]; then
      exit "${ADAPTER_HASH_STATUS:-0}"
    fi
    ;;
  rev-parse)
    case "$*" in
      'rev-parse --git-path release-state') echo .release-state ;;
      'rev-parse --show-prefix') printf '' ;;
      *) exit 97 ;;
    esac
    ;;
  status)
    [[ "$*" == 'status --porcelain=v1 -z --untracked-files=all' ]] || exit 97
    [[ "${ADAPTER_SOURCE_STATUS:-0}" -eq 0 ]] || exit "$ADAPTER_SOURCE_STATUS"
    [[ -z "${ADAPTER_STAGED_DIRTY:-}" ]] || printf 'M  %s\0' "$ADAPTER_STAGED_DIRTY"
    [[ -z "${ADAPTER_DIRTY:-}" ]] || printf ' M %s\0' "$ADAPTER_DIRTY"
    if [[ "${ADAPTER_REAL_UNTRACKED:-}" == true ]]; then
      paths="$(mktemp)"
      trap 'rm -f "$paths"' EXIT
      "$REAL_GIT" ls-files --others --exclude-standard -z > "$paths"
      while IFS= read -r -d '' path; do printf '?? %s\0' "$path"; done < "$paths"
    elif [[ -n "${ADAPTER_UNTRACKED:-}" ]]; then
      printf '?? %s\0' "$ADAPTER_UNTRACKED"
    fi
    exit "${ADAPTER_UNTRACKED_STATUS:-0}"
    ;;
  ls-tree)
    [[ "$2" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb && "$3" == -- ]] || exit 97
    file="$4"
    [[ "${ADAPTER_COMMIT_MISSING:-}" != "$file" ]] || exit 0
    digest="$("$REAL_GIT" hash-object --no-filters -- "${ADAPTER_COMMIT_ROOT:-.release-state/$RELEASE_VERSION.metadata/new}/$file")"
    printf '%s blob %s\t%s\n' "${ADAPTER_COMMIT_MODE:-100644}" "$digest" "$file"
    exit "${ADAPTER_COMMIT_STATUS:-0}"
    ;;
  ls-files)
    [[ "$2" == --error-unmatch ]] || exit 97
    ;;
  diff)
    case "$2" in
      --quiet) exit "${ADAPTER_UNSTAGED:-0}" ;;
      --binary)
        for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
          printf '%s\0' "$file"
          cat "$file"
        done
        if [[ -f input-diff-status ]]; then
          exit "$(cat input-diff-status)"
        fi
        exit "${ADAPTER_INPUT_DIFF_STATUS:-0}"
        ;;
      *) exit 97 ;;
    esac
    ;;
  *) echo "unexpected Git effect: $*" >&2; exit 97 ;;
esac
STUB
cat > "$work_dir/bin/make" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == '--no-print-directory release-check' ]] || exit 97
printf '%s\n' "$*" >> gate-trace
printf '%s\n' "${CARGO_NET_OFFLINE-unset}" >> gate-environment
echo 'substituted complete validation gate'
if [[ -n "${ADAPTER_AFTER_GATE_DIFF_STATUS:-}" ]]; then
  printf '%s\n' "$ADAPTER_AFTER_GATE_DIFF_STATUS" > input-diff-status
fi
if [[ "${ADAPTER_REGISTRY_CHECK:-}" == true &&
      ( "${CARGO_NET_OFFLINE:-}" == true || "${CARGO_NET_OFFLINE:-}" == 1 ) ]]; then
  echo 'substituted registry check requires HTTP' >&2
  exit 101
fi
exit "${ADAPTER_GATE_STATUS:-0}"
STUB
chmod +x "$work_dir/bin/git" "$work_dir/bin/make"
cat > "$work_dir/bin/cat" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
"$REAL_CAT" "$@" || exit $?
if [[ "$#" == 1 && "$1" == "${ADAPTER_READ_PATH:-}" ]]; then
  exit 9
fi
STUB
chmod +x "$work_dir/bin/cat"
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == fetch && "${ADAPTER_FETCH_TEST:-}" == true ]]; then
  printf '%s\n' "$*" >> fetch-trace
  printf '%s\n' "${CARGO_NET_OFFLINE-unset}" >> fetch-policy
  for argument in "$@"; do
    [[ "$argument" != --offline ]] || exit 27
  done
  [[ "${CARGO_NET_OFFLINE:-}" != true && "${CARGO_NET_OFFLINE:-}" != 1 ]] || exit 27
  exit "${ADAPTER_FETCH_STATUS:-0}"
fi
# Real metadata fixtures use the explicitly prepared cache, never live downloads.
CARGO_NET_OFFLINE=true exec "$REAL_CARGO" "$@"
STUB
chmod +x "$work_dir/bin/cargo"
export PATH="$work_dir/bin:$PATH"

previous="$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable "$repo_root/Cargo.toml")"
new_fixture() {
  mkdir -p "$work_dir/$1"
  cd "$work_dir/$1"
  # Exercise the consumer's actual Make callbacks with local copies of their
  # inputs. Only the complete validation gate and Git effects are substituted.
  mkdir -p scripts/release scripts/ci ci
  cp "$repo_root/scripts/release/metadata.sh" scripts/release/
  for helper in check-release-source.sh read-cargo-workspace-version.sh next-release-version.sh check-format-tools.sh check-installation-version.sh finalize-release-changelog.awk; do
    cp "$repo_root/scripts/ci/$helper" scripts/ci/
  done
  cp "$repo_root/ci/tool-versions.env" ci/
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md rust-toolchain.toml; do
    mkdir -p "$(dirname "$file")"
    cp "$repo_root/$file" "$file"
  done
  for member in crates/ic-testkit crates/ic_testkit_perf_probe; do
    mkdir -p "$member/src"
    cp "$repo_root/$member/Cargo.toml" "$member/Cargo.toml"
    : > "$member/src/lib.rs"
  done
  echo 'consumer build artifact' > artifact
  export RELEASE_PREVIOUS="$previous" RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
  export RELEASE_COMMIT=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
  export RELEASE_KIND="${2:-patch}" RELEASE_DATE=2026-10-05
  export RELEASE_VERSION
  RELEASE_VERSION="$(bash "$repo_root/scripts/ci/next-release-version.sh" "$previous" "$RELEASE_KIND")"
  # Own the fixture's candidate independently of the checkout's note state.
  # Keep published history, replacing any real pending batch with fixture notes.
  for file in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
    awk -v version="$RELEASE_VERSION" -v notes="${3:-pending}" '
      /^## \[[0-9]+\.[0-9]+\.[0-9]+\]$/ { pending = 1; next }
      /^## / {
        pending = 0
        if (!history++ && notes == "pending") {
          print "## [" version "]"
          print ""
          print "- Isolated release fixture notes."
          print ""
        }
      }
      !pending { print }
      END { if (!history) exit 1 }
    ' "$file" > candidate-notes
    mv candidate-notes "$file"
  done
}
adapter() {
  local target
  case "$1" in
    preflight) target=release-preflight ;;
    verify) target=release-verify ;;
    prepare) target=release-prepare-version ;;
    prepared) target=release-prepared-check ;;
    commit) target=release-commit-check ;;
    committed) target=release-committed-check ;;
    tagged) target=release-tagged-check ;;
    push) target=release-push-check ;;
    *) fail "unknown fixture phase: $1" ;;
  esac
  # This private release owns its identity. Recursive Make command-line
  # overrides from an outer release otherwise beat the fixture's environment.
  MAKEFLAGS='' MAKEOVERRIDES='' MFLAGS='' \
    "$real_make" --no-print-directory -f "$repo_root/Makefile" "$target"
}
expect_failure() {
  if adapter "$1" > rejected.log 2>&1; then
    fail "adapter unexpectedly accepted $1 in $PWD (${conflict:-no selected-commit conflict})"
  fi
}

# Imported undated history is not another pending release. Exercise both note
# views through the actual consumer callbacks, preserving the historical bytes.
new_fixture undated-history
for file in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
  printf '\n## [0.0.1]\n\nHistorical imported notes.\n' >> "$file"
done
adapter preflight || fail 'undated history preflight rejected'
adapter verify || fail 'undated history validation rejected'
adapter prepare || fail 'undated history preparation rejected'
for file in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
  tail -n 4 "$file" > history-tail
  tail -n 4 ".release-state/$RELEASE_VERSION.metadata/old/$file" > old-history-tail
  cmp history-tail old-history-tail || fail "undated history changed: $file"
done

# One private policy selection drives both cache preparation and gate admission.
# Local exported variables cannot leak into another fixture.
check_network_policy() {
  local policy="$1" ADAPTER_FETCH_TEST=true ADAPTER_REGISTRY_CHECK=true CARGO_NET_OFFLINE
  export ADAPTER_FETCH_TEST ADAPTER_REGISTRY_CHECK CARGO_NET_OFFLINE
  if [[ "$policy" == unset ]]; then unset CARGO_NET_OFFLINE;
  else CARGO_NET_OFFLINE="$policy"; fi
  if [[ "$policy" == true || "$policy" == 1 ]]; then
    expect_failure preflight
  else
    adapter preflight || fail "cache preparation rejected network policy: $policy"
  fi
  [[ "$(cat fetch-trace)" == 'fetch --manifest-path Cargo.toml --locked' ]] || fail 'cache preparation changed locked fetch arguments'
  [[ "$(cat fetch-policy)" == "$policy" ]] || fail 'cache preparation changed caller policy'
  [[ ! -e gate-trace && ! -e .release-state/"$RELEASE_VERSION".metadata ]] || fail 'cache preparation dispatched validation or metadata effects'
  cmp "$repo_root/Cargo.toml" Cargo.toml || fail 'network fixture changed manifest'
  cmp "$repo_root/Cargo.lock" Cargo.lock || fail 'network fixture changed lock selection'

  # Verify the gate independently, even when offline cache admission failed.
  if [[ "$policy" == true || "$policy" == 1 ]]; then
    expect_failure verify
    expect_failure prepare
    [[ ! -e ".release-state/$RELEASE_VERSION.validation" ]] || fail 'offline gate recorded validation'
  else
    adapter verify || fail "validation rejected network policy: $policy"
    [[ -s ".release-state/$RELEASE_VERSION.validation" ]] || fail 'successful gate omitted validation'
  fi
  [[ "$(cat gate-environment)" == "$policy" ]] || fail 'gate changed caller policy'
  [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail 'gate retried implicitly'
  cmp "$repo_root/Cargo.toml" Cargo.toml || fail 'network fixture changed manifest'
  cmp "$repo_root/Cargo.lock" Cargo.lock || fail 'network fixture changed lock selection'
}
for policy in unset false true 1; do
  new_fixture "network-$policy"
  check_network_policy "$policy" || fail "network-policy qualification failed: $policy"
done

for kind in patch minor major; do
  new_fixture "$kind" "$kind"
  if [[ "$kind" == patch ]]; then
    # Exercise the shared finalizer fix through both actual consumer note views.
    for view in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
      awk -v heading="## [$RELEASE_VERSION]" '
        $0 == heading { print $0 " \t"; next }
        { print }
      ' "$view" > "$view.spaced"
      mv "$view.spaced" "$view"
    done
  fi
  cat > readme-history <<'HISTORY'

Historical examples outside the maintained TOML block:
ic-testkit = "0.1"
```text
ic-testkit = "0.2"
```
HISTORY
  printf 'Historical final line without a newline.' >> readme-history
  for readme in README.md crates/ic-testkit/README.md; do
    # Valid formatting and comments survive the targeted requirement update.
    perl -pi -e 's/^ic-testkit = ("[0-9]+\.[0-9]+")$/  ic-testkit  =  $1  # maintained example/' "$readme"
    cat readme-history >> "$readme"
  done
  adapter preflight
  adapter verify
  cp CHANGELOG.md history-root
  cp crates/ic-testkit/CHANGELOG.md history-package
  # Simulate another valid release identity inherited through recursive Make.
  # The private patch/minor/major candidate and its notes must remain authoritative.
  MAKEFLAGS='-- RELEASE_PREVIOUS=98.0.0 RELEASE_KIND=major RELEASE_VERSION=99.0.0 RELEASE_DATE=2099-01-01' \
    adapter prepare
  adapter prepared
  [[ "$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" == "$RELEASE_VERSION" ]] || fail "fixture invariant in $PWD at line $LINENO"
  # The exact generated metadata, dependency selection and retained old bytes
  # are the contract; no test inspects the adapter's implementation layout.
  cmp history-root ".release-state/$RELEASE_VERSION.metadata/old/CHANGELOG.md"
  cmp history-package ".release-state/$RELEASE_VERSION.metadata/old/crates/ic-testkit/CHANGELOG.md"
  grep -Fx "## [$RELEASE_VERSION] - $RELEASE_DATE" CHANGELOG.md
  grep -Fx "## [$RELEASE_VERSION] - $RELEASE_DATE" crates/ic-testkit/CHANGELOG.md
  for readme in README.md crates/ic-testkit/README.md; do
    grep -Fx "  ic-testkit  =  \"${RELEASE_VERSION%.*}\"  # maintained example" "$readme"
    tail -n "$(awk 'END { print NR }' readme-history)" "$readme" > prepared-readme-history
    cmp readme-history prepared-readme-history
  done
  for phase in commit committed tagged push; do adapter "$phase"; done
  [[ "$(cat artifact)" == 'consumer build artifact' ]] || fail "fixture invariant in $PWD at line $LINENO"
  [[ "$(cat gate-trace)" == '--no-print-directory release-check' ]] || fail "fixture invariant in $PWD at line $LINENO"

  # Model an interrupted publication before the manifest commit point. Resume
  # republishes the same saved bytes without another gate or version selection.
  cp ".release-state/$RELEASE_VERSION.metadata/old/Cargo.toml" Cargo.toml
  cp ".release-state/$RELEASE_VERSION.metadata/old/README.md" README.md
  # The shared runner repeats preflight while the manifest is still old. Its
  # cache check must tolerate the saved new lockfile already being published.
  cp Cargo.lock recovery-lock
  cp Cargo.toml recovery-manifest
  cp README.md recovery-readme
  echo 'unvalidated recovery mutation' >> README.md
  expect_failure preflight
  cp recovery-readme README.md
  saved_member=".release-state/$RELEASE_VERSION.metadata/new/crates/ic-testkit/Cargo.toml"
  cp "$saved_member" recovery-member
  echo '# unvalidated saved workspace input' >> "$saved_member"
  expect_failure preflight
  cp recovery-member "$saved_member"
  adapter preflight
  cmp recovery-lock Cargo.lock
  cmp recovery-manifest Cargo.toml
  adapter prepare
  adapter prepared
  [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
  echo 'unvalidated mutation' >> README.md
  expect_failure prepared
  expect_failure prepare
done

new_fixture historical-release-metadata
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
cp -R "$retained/new" committed-release
export ADAPTER_COMMIT_ROOT="$PWD/committed-release"
cp "$retained/ready" selected-ready
cp ".release-state/$RELEASE_VERSION.validation" selected-receipt
# Simulate newer committed documentation while the runner selects the original
# exact release commit. Its own fixture covers ancestry and Git effects.
printf '\nNewer committed documentation.\n' >> README.md
for phase in committed tagged push; do adapter "$phase"; done
expect_failure prepared
expect_failure commit
cp README.md newer-readme
for conflict in payload missing mode inspection selection; do
  case "$conflict" in
    payload) printf '\nChanged selected payload.\n' >> committed-release/README.md ;;
    missing) export ADAPTER_COMMIT_MISSING=README.md ;;
    mode) export ADAPTER_COMMIT_MODE=120000 ;;
    inspection) export ADAPTER_COMMIT_STATUS=9 ;;
    selection) export RELEASE_COMMIT=cccccccccccccccccccccccccccccccccccccccc ;;
  esac
  for phase in committed tagged push; do
    expect_failure "$phase"
    cmp newer-readme README.md
    cmp selected-ready "$retained/ready"
    cmp selected-receipt ".release-state/$RELEASE_VERSION.validation"
  done
  cp "$retained/new/README.md" committed-release/README.md
  unset ADAPTER_COMMIT_MISSING ADAPTER_COMMIT_MODE ADAPTER_COMMIT_STATUS
  export RELEASE_COMMIT=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
done
unset RELEASE_COMMIT
expect_failure committed
export RELEASE_COMMIT=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
for phase in committed tagged push; do adapter "$phase"; done
[[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
unset ADAPTER_COMMIT_ROOT

new_fixture publication-temporaries
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
cp "$retained/old/Cargo.toml" Cargo.toml
cp "$repo_root/.gitignore" .gitignore
# Real Git supplies untracked-path admission for this case. Only the fixture's
# mocked Git-directory state needs an extra private exclusion; no commits occur.
"$REAL_GIT" init --quiet
printf '/.release-state/\n' > .git/info/exclude
: > rejected.log
"$REAL_GIT" add -- .gitignore Cargo.toml Cargo.lock CHANGELOG.md \
  crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md \
  rust-toolchain.toml crates/ic-testkit/Cargo.toml crates/ic-testkit/src/lib.rs \
  crates/ic_testkit_perf_probe/Cargo.toml crates/ic_testkit_perf_probe/src/lib.rs \
  artifact gate-trace gate-environment rejected.log scripts ci
printf 'interrupted root staging bytes\n' > .release-metadata.fixture
printf 'interrupted package staging bytes\n' > crates/ic-testkit/.release-metadata.fixture
ADAPTER_REAL_UNTRACKED=true adapter preflight || fail "tracked fixture preflight rejected"
ADAPTER_REAL_UNTRACKED=true adapter prepare || fail "tracked fixture recovery rejected"
[[ "$(cat .release-metadata.fixture)" == 'interrupted root staging bytes' ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(cat crates/ic-testkit/.release-metadata.fixture)" == 'interrupted package staging bytes' ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
printf 'unrelated same-prefix file\n' > crates/ic_testkit_perf_probe/.release-metadata.unrelated
ADAPTER_REAL_UNTRACKED=true expect_failure prepared

new_fixture finalized-notes patch finalized
cp CHANGELOG.md history-root
cp crates/ic-testkit/CHANGELOG.md history-package
adapter verify
adapter prepare
adapter prepared
for view in root package; do
  file=CHANGELOG.md
  [[ "$view" != package ]] || file=crates/ic-testkit/CHANGELOG.md
  grep -Fx "## [$RELEASE_VERSION] - $RELEASE_DATE" "$file"
  # Preparing with no pending notes inserts an empty release section and
  # preserves all prior history, including the heading and introductory text.
  awk -v heading="## [$RELEASE_VERSION] - $RELEASE_DATE" '
    $0 == heading { inserted = 1; next }
    inserted { inserted = 0; next }
    { print }
  ' "$file" > prepared-history
  cmp "history-$view" prepared-history
done

new_fixture raw-metadata-identities
"$REAL_GIT" init --quiet
printf '*.md text eol=lf\n' > .gitattributes
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
cp "$retained/ready" raw-ready
cp ".release-state/$RELEASE_VERSION.validation" raw-receipt
cp -R "$retained/new" raw-live
for mutated_tree in live old new; do
  mutated_file=README.md
  [[ "$mutated_tree" == live ]] || mutated_file="$retained/$mutated_tree/README.md"
  cp "$mutated_file" raw-before
  # Real Git attributes normalize CRLF to LF. The release identity must still
  # distinguish the actual live, backup and prepared bytes without normalization.
  perl -pi -e 's/\n/\r\n/g' "$mutated_file"
  cp "$mutated_file" raw-after
  cp -R "$retained" "$mutated_tree-retained"
  phases=(prepared prepare commit)
  if [[ "$mutated_tree" != live ]]; then phases+=(committed tagged push); fi
  for phase in "${phases[@]}"; do
    expect_failure "$phase"
    cmp raw-after "$mutated_file"
    cmp raw-ready "$retained/ready"
    cmp raw-receipt ".release-state/$RELEASE_VERSION.validation"
    diff -r "$mutated_tree-retained" "$retained"
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      expected="raw-live/$file"
      if [[ "$mutated_tree" == live && "$file" == README.md ]]; then expected=raw-after; fi
      cmp "$expected" "$file"
    done
  done
  cp raw-before "$mutated_file"
  adapter prepared
done
[[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"

for linked_path in ready old/README.md new; do
  new_fixture "linked-${linked_path//\//-}"
  adapter verify
  adapter prepare
  retained=".release-state/$RELEASE_VERSION.metadata"
  # Recreate the pre-publication state, then replace saved evidence with an
  # alias to identical bytes. Content equality must not authorize linked state.
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
    cp "$retained/old/$file" "$file"
  done
  mv "$retained/$linked_path" foreign-metadata
  cp -R foreign-metadata foreign-before
  ln -s "$PWD/foreign-metadata" "$retained/$linked_path"
  expect_failure prepare
  diff -r foreign-before foreign-metadata
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
    cmp "$retained/old/$file" "$file"
  done
done

new_fixture linked-preparation-directory
adapter verify
retained=".release-state/$RELEASE_VERSION.metadata"
mkdir -p "$retained" foreign-metadata
cp README.md foreign-metadata/README.md
cp -R foreign-metadata foreign-before
ln -s "$PWD/foreign-metadata" "$retained/new"
expect_failure prepare
diff -r foreign-before foreign-metadata
[[ ! -e "$retained/ready" ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]] || fail "fixture invariant in $PWD at line $LINENO"

for view in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
  new_fixture "conflicting-${view//\//-}"
  awk '
    /^## \[[0-9]+\.[0-9]+\.[0-9]+\]$/ && !selected++ {
      print "## [9.9.9]"; next
    }
    { print }
  ' "$view" > conflicting-notes
  mv conflicting-notes "$view"
  cp "$view" notes-before
  adapter verify
  expect_failure prepare
  cmp notes-before "$view"
  [[ "$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]] || fail "fixture invariant in $PWD at line $LINENO"
  [[ ! -e ".release-state/$RELEASE_VERSION.metadata/ready" ]] || fail "fixture invariant in $PWD at line $LINENO"
  logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
  [[ -s "${logs[0]}" ]] || fail "fixture invariant in $PWD at line $LINENO"
done

for view in README.md crates/ic-testkit/README.md; do
  for invalid_example in duplicate malformed; do
    new_fixture "invalid-$invalid_example-${view//\//-}"
    if [[ "$invalid_example" == duplicate ]]; then
      # shellcheck disable=SC2016 # Markdown fences are literal fixture data.
      printf '\n```toml\nic-testkit = "0.1"\n```\n' >> "$view"
    else
      perl -pi -e 's/^ic-testkit = "[0-9]+\.[0-9]+"$/ic-testkit = "invalid"/' "$view"
    fi
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      mkdir -p "before/$(dirname "$file")"
      cp "$file" "before/$file"
    done
    adapter verify
    expect_failure prepare
    [[ ! -e ".release-state/$RELEASE_VERSION.metadata/ready" ]] || fail "fixture invariant in $PWD at line $LINENO"
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      cmp "before/$file" "$file"
    done
  done
done

new_fixture failed-gate
export ADAPTER_GATE_STATUS=23
expect_failure verify
unset ADAPTER_GATE_STATUS
[[ "$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]] || fail "fixture invariant in $PWD at line $LINENO"
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ -s "${logs[0]}" ]] || fail "fixture invariant in $PWD at line $LINENO"
expect_failure prepare

new_fixture validation-retry
adapter preflight
adapter verify
cp ".release-state/$RELEASE_VERSION.validation" passed-receipt
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
first_log="${logs[0]}"
cp "$first_log" passed-log
export ADAPTER_GATE_STATUS=23
expect_failure verify
unset ADAPTER_GATE_STATUS
[[ ! -e ".release-state/$RELEASE_VERSION.validation" ]] || fail "fixture invariant in $PWD at line $LINENO"
expect_failure prepare
cmp passed-log "$first_log"
receipts=(.release-state/"$RELEASE_VERSION".validation.*/prior-validation)
cmp passed-receipt "${receipts[0]}"
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ "${#logs[@]}" -eq 2 ]] || fail "fixture invariant in $PWD at line $LINENO"
for log in "${logs[@]}"; do [[ -s "$log" ]] || fail "empty validation log: $log"; done
adapter preflight
adapter verify
adapter prepare
[[ "$(wc -l < gate-trace)" -eq 3 ]] || fail "fixture invariant in $PWD at line $LINENO"
cmp passed-log "$first_log"

new_fixture failed-input-inspection
adapter verify
cp ".release-state/$RELEASE_VERSION.validation" inspected-receipt
for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
  mkdir -p "inspected/$(dirname "$file")"
  cp "$file" "inspected/$file"
done
export ADAPTER_INPUT_DIFF_STATUS=9
expect_failure prepare
unset ADAPTER_INPUT_DIFF_STATUS
cmp inspected-receipt ".release-state/$RELEASE_VERSION.validation"
[[ ! -e ".release-state/$RELEASE_VERSION.metadata" ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
  cmp "inspected/$file" "$file"
done

new_fixture failed-post-gate-inspection
export ADAPTER_AFTER_GATE_DIFF_STATUS=9
expect_failure verify
unset ADAPTER_AFTER_GATE_DIFF_STATUS
[[ ! -e ".release-state/$RELEASE_VERSION.validation" ]] || fail "fixture invariant in $PWD at line $LINENO"
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ -s "${logs[0]}" ]] || fail "fixture invariant in $PWD at line $LINENO"
expect_failure prepare
[[ ! -e ".release-state/$RELEASE_VERSION.metadata" ]] || fail "fixture invariant in $PWD at line $LINENO"
[[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
first_log="${logs[0]}"
cp "$first_log" failed-inspection-log
rm input-diff-status
adapter preflight
adapter verify
adapter prepare
cmp failed-inspection-log "$first_log"
[[ "$(wc -l < gate-trace)" -eq 2 ]] || fail "fixture invariant in $PWD at line $LINENO"

new_fixture failed-payload-inspection
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
cp "$retained/ready" inspected-ready
cp ".release-state/$RELEASE_VERSION.validation" inspected-receipt
export ADAPTER_HASH_STATUS=9
for failed_path in '' README.md "$retained/old/README.md" "$retained/new/README.md"; do
  export ADAPTER_HASH_PATH="$failed_path"
  for phase in preflight prepare prepared commit committed tagged push; do
    # Late checks read the selected release commit rather than the live file.
    if [[ "$failed_path" == README.md ]]; then
      case "$phase" in committed|tagged|push) continue ;; esac
    fi
    # Model an interrupted manifest publication for the recovery preflight.
    if [[ "$phase" == preflight ]]; then
      cp "$retained/old/Cargo.toml" Cargo.toml
    else
      cp "$retained/new/Cargo.toml" Cargo.toml
    fi
    expect_failure "$phase"
    cmp inspected-ready "$retained/ready"
    cmp inspected-receipt ".release-state/$RELEASE_VERSION.validation"
    [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      tree=new
      if [[ "$phase" == preflight && "$file" == Cargo.toml ]]; then tree=old; fi
      cmp "$retained/$tree/$file" "$file"
    done
  done
done
unset ADAPTER_HASH_STATUS ADAPTER_HASH_PATH
adapter prepared

new_fixture failed-retained-record-read
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
receipt=".release-state/$RELEASE_VERSION.validation"
cp "$retained/ready" inspected-ready
cp "$receipt" inspected-receipt
for failed_path in "$receipt" "$retained/ready"; do
  export ADAPTER_READ_PATH="$failed_path"
  for phase in preflight prepare prepared commit committed tagged push; do
    if [[ "$phase" == preflight ]]; then
      cp "$retained/old/Cargo.toml" Cargo.toml
    else
      cp "$retained/new/Cargo.toml" Cargo.toml
    fi
    expect_failure "$phase"
    cmp inspected-ready "$retained/ready"
    cmp inspected-receipt "$receipt"
    [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail 'record-read rejection dispatched another gate'
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      tree=new
      if [[ "$phase" == preflight && "$file" == Cargo.toml ]]; then tree=old; fi
      cmp "$retained/$tree/$file" "$file"
    done
  done
done
unset ADAPTER_READ_PATH
adapter prepared

new_fixture failed-version-inspection
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
receipt=".release-state/$RELEASE_VERSION.validation"
cp "$retained/ready" inspected-ready
cp "$receipt" inspected-receipt
for helper in scripts/ci/read-cargo-workspace-version.sh scripts/ci/next-release-version.sh; do
  cp "$helper" inspected-helper
  printf '\nexit 9\n' >> "$helper"
  phases=(preflight prepared commit)
  if [[ "$helper" == scripts/ci/next-release-version.sh ]]; then
    phases=(preflight verify prepare prepared commit committed tagged push)
  fi
  for phase in "${phases[@]}"; do
    if [[ "$phase" == preflight ]]; then
      cp "$retained/old/Cargo.toml" Cargo.toml
    else
      cp "$retained/new/Cargo.toml" Cargo.toml
    fi
    expect_failure "$phase"
    cmp inspected-ready "$retained/ready"
    cmp inspected-receipt "$receipt"
    [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail 'version-inspection rejection dispatched another gate'
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      tree=new
      if [[ "$phase" == preflight && "$file" == Cargo.toml ]]; then tree=old; fi
      cmp "$retained/$tree/$file" "$file"
    done
  done
  cp inspected-helper "$helper"
  adapter prepared
done

new_fixture cache-fetch-recovery
adapter verify
adapter prepare
retained=".release-state/$RELEASE_VERSION.metadata"
receipt=".release-state/$RELEASE_VERSION.validation"
cp "$retained/ready" cache-ready
cp "$receipt" cache-receipt
cp "$retained/old/Cargo.toml" Cargo.toml
export ADAPTER_FETCH_TEST=true ADAPTER_FETCH_STATUS=23
CARGO_NET_OFFLINE=false expect_failure preflight
[[ "$(cat fetch-trace)" == "fetch --manifest-path $retained/new/Cargo.toml --locked" ]] || fail 'recovery fetched the mixed live workspace'
cmp cache-ready "$retained/ready"
cmp cache-receipt "$receipt"
[[ "$(wc -l < gate-trace)" -eq 1 ]] || fail 'failed cache fetch dispatched another gate'
for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
  tree=new
  if [[ "$file" == Cargo.toml ]]; then tree=old; fi
  cmp "$retained/$tree/$file" "$file"
done
export ADAPTER_FETCH_STATUS=0
CARGO_NET_OFFLINE=false adapter preflight
[[ "$(wc -l < fetch-trace)" -eq 2 ]] || fail 'cache fetch retried implicitly'
cmp cache-ready "$retained/ready"
cmp cache-receipt "$receipt"
unset ADAPTER_FETCH_TEST ADAPTER_FETCH_STATUS
cp "$retained/new/Cargo.toml" Cargo.toml
adapter prepared

new_fixture changed-input
adapter verify
echo 'source changed after validation' >> CHANGELOG.md
expect_failure prepare
[[ "$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]] || fail "fixture invariant in $PWD at line $LINENO"

new_fixture admission
export ADAPTER_SOURCE_STATUS=9
expect_failure preflight
unset ADAPTER_SOURCE_STATUS
export ADAPTER_STAGED_DIRTY=$'staged unrelated\nfile.rs'
expect_failure preflight
printf -v rendered_path '%q' "$ADAPTER_STAGED_DIRTY"
grep -F -- "$rendered_path" rejected.log >/dev/null || fail 'missing staged-path diagnosis'
unset ADAPTER_STAGED_DIRTY
export ADAPTER_DIRTY='unrelated file.rs'
expect_failure preflight
unset ADAPTER_DIRTY
export ADAPTER_UNTRACKED='untracked release input'
expect_failure preflight
printf -v rendered_path '%q' "$ADAPTER_UNTRACKED"
grep -F -- "$rendered_path" rejected.log >/dev/null || fail 'missing untracked-path diagnosis'
unset ADAPTER_UNTRACKED
adapter verify
cp ".release-state/$RELEASE_VERSION.validation" admitted-receipt
for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
  mkdir -p "admitted/$(dirname "$file")"
  cp "$file" "admitted/$file"
done
# An empty inventory is clean only after Git completed successfully. Failure
# must not run a gate, invalidate its receipt or begin metadata publication.
export ADAPTER_UNTRACKED_STATUS=9
for phase in preflight verify prepare; do
  expect_failure "$phase"
  cmp admitted-receipt ".release-state/$RELEASE_VERSION.validation"
  [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
  [[ ! -e ".release-state/$RELEASE_VERSION.metadata" ]] || fail "fixture invariant in $PWD at line $LINENO"
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
    cmp "admitted/$file" "$file"
  done
done
unset ADAPTER_UNTRACKED_STATUS
adapter prepare
cp ".release-state/$RELEASE_VERSION.metadata/ready" admitted-ready
export ADAPTER_UNTRACKED_STATUS=9
for phase in prepared commit committed tagged push; do
  expect_failure "$phase"
  cmp admitted-receipt ".release-state/$RELEASE_VERSION.validation"
  cmp admitted-ready ".release-state/$RELEASE_VERSION.metadata/ready"
  [[ "$(wc -l < gate-trace)" -eq 1 ]] || fail "fixture invariant in $PWD at line $LINENO"
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
    cmp ".release-state/$RELEASE_VERSION.metadata/new/$file" "$file"
  done
done
unset ADAPTER_UNTRACKED_STATUS
adapter prepared
export ADAPTER_UNSTAGED=1
expect_failure commit
unset ADAPTER_UNSTAGED
export RELEASE_SOURCE=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
expect_failure prepared

echo 'release metadata isolated checks passed'
