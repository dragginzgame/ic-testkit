#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
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
export REAL_GIT
REAL_GIT="$(command -v git)"
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
  rev-parse) [[ "$*" == 'rev-parse --git-path release-state' ]]; echo .release-state ;;
  ls-files)
    if [[ "$2" == --others ]]; then
      if [[ "${ADAPTER_REAL_UNTRACKED:-}" == true ]]; then exec "$REAL_GIT" "$@"; fi
      printf '%s' "${ADAPTER_UNTRACKED:-}"
      exit "${ADAPTER_UNTRACKED_STATUS:-0}"
    fi
    ;;
  diff)
    case "$2" in
      --name-only)
        [[ "${ADAPTER_DIFF_STATUS:-0}" -eq 0 ]] || exit "$ADAPTER_DIFF_STATUS"
        if [[ "$*" == *--cached* && -n "${ADAPTER_STAGED_DIRTY:-}" ]]; then
          printf '%s\0' "$ADAPTER_STAGED_DIRTY"
        fi
        [[ -z "${ADAPTER_DIRTY:-}" ]] || printf '%s\0' "$ADAPTER_DIRTY"
        ;;
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
[[ "$*" == '--no-print-directory release-check' ]]
printf '%s\n' "$*" >> gate-trace
printf '%s\n' "${CARGO_NET_OFFLINE-unset}" >> gate-environment
echo 'substituted complete validation gate'
if [[ -n "${ADAPTER_AFTER_GATE_DIFF_STATUS:-}" ]]; then
  printf '%s\n' "$ADAPTER_AFTER_GATE_DIFF_STATUS" > input-diff-status
fi
if [[ "${ADAPTER_REGISTRY_CHECK:-}" == true && "${CARGO_NET_OFFLINE:-}" == true ]]; then
  echo 'substituted registry check requires HTTP' >&2
  exit 101
fi
exit "${ADAPTER_GATE_STATUS:-0}"
STUB
chmod +x "$work_dir/bin/git" "$work_dir/bin/make"
export PATH="$work_dir/bin:$PATH"

previous="$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable "$repo_root/Cargo.toml")"
new_fixture() {
  mkdir -p "$work_dir/$1"
  cd "$work_dir/$1"
  for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md rust-toolchain.toml; do
    mkdir -p "$(dirname "$file")"
    cp "$repo_root/$file" "$file"
  done
  for member in crates/ic-testkit canisters/test/perf_probe; do
    mkdir -p "$member/src"
    cp "$repo_root/$member/Cargo.toml" "$member/Cargo.toml"
    : > "$member/src/lib.rs"
  done
  echo 'consumer build artifact' > artifact
  export RELEASE_PREVIOUS="$previous" RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
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
adapter() { bash "$repo_root/scripts/release/metadata.sh" "$1"; }
expect_failure() {
  if adapter "$1" > rejected.log 2>&1; then
    echo "metadata adapter unexpectedly accepted $1" >&2
    exit 1
  fi
}

for kind in patch minor major; do
  new_fixture "$kind" "$kind"
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
  adapter prepare
  adapter prepared
  [[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$RELEASE_VERSION" ]]
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
  [[ "$(cat artifact)" == 'consumer build artifact' ]]
  [[ "$(cat gate-trace)" == '--no-print-directory release-check' ]]

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
  [[ "$(wc -l < gate-trace)" -eq 1 ]]
  echo 'unvalidated mutation' >> README.md
  expect_failure prepared
  expect_failure prepare
done

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
  canisters/test/perf_probe/Cargo.toml canisters/test/perf_probe/src/lib.rs \
  artifact gate-trace gate-environment rejected.log
printf 'interrupted root staging bytes\n' > .release-metadata.fixture
printf 'interrupted package staging bytes\n' > crates/ic-testkit/.release-metadata.fixture
(
  export ADAPTER_REAL_UNTRACKED=true
  adapter preflight
  adapter prepare
  [[ "$(cat .release-metadata.fixture)" == 'interrupted root staging bytes' ]]
  [[ "$(cat crates/ic-testkit/.release-metadata.fixture)" == 'interrupted package staging bytes' ]]
  [[ "$(wc -l < gate-trace)" -eq 1 ]]
  printf 'unrelated same-prefix file\n' > canisters/test/perf_probe/.release-metadata.unrelated
  expect_failure prepared
)

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
  for phase in prepared prepare commit committed tagged push; do
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
[[ "$(wc -l < gate-trace)" -eq 1 ]]

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
[[ ! -e "$retained/ready" ]]
[[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]

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
  [[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]
  [[ ! -e ".release-state/$RELEASE_VERSION.metadata/ready" ]]
  logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
  [[ -s "${logs[0]}" ]]
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
    [[ ! -e ".release-state/$RELEASE_VERSION.metadata/ready" ]]
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      cmp "before/$file" "$file"
    done
  done
done

new_fixture failed-gate
export ADAPTER_GATE_STATUS=23
expect_failure verify
unset ADAPTER_GATE_STATUS
[[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ -s "${logs[0]}" ]]
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
[[ ! -e ".release-state/$RELEASE_VERSION.validation" ]]
expect_failure prepare
cmp passed-log "$first_log"
receipts=(.release-state/"$RELEASE_VERSION".validation.*/prior-validation)
cmp passed-receipt "${receipts[0]}"
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ "${#logs[@]}" -eq 2 ]]
for log in "${logs[@]}"; do [[ -s "$log" ]]; done
adapter preflight
adapter verify
adapter prepare
[[ "$(wc -l < gate-trace)" -eq 3 ]]
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
[[ ! -e ".release-state/$RELEASE_VERSION.metadata" ]]
[[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]
[[ "$(wc -l < gate-trace)" -eq 1 ]]
for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
  cmp "inspected/$file" "$file"
done

new_fixture failed-post-gate-inspection
export ADAPTER_AFTER_GATE_DIFF_STATUS=9
expect_failure verify
unset ADAPTER_AFTER_GATE_DIFF_STATUS
[[ ! -e ".release-state/$RELEASE_VERSION.validation" ]]
logs=(.release-state/"$RELEASE_VERSION".validation.*/validation.log)
[[ -s "${logs[0]}" ]]
expect_failure prepare
[[ ! -e ".release-state/$RELEASE_VERSION.metadata" ]]
[[ "$(wc -l < gate-trace)" -eq 1 ]]
first_log="${logs[0]}"
cp "$first_log" failed-inspection-log
rm input-diff-status
adapter preflight
adapter verify
adapter prepare
cmp failed-inspection-log "$first_log"
[[ "$(wc -l < gate-trace)" -eq 2 ]]

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
    # Model an interrupted manifest publication for the recovery preflight.
    if [[ "$phase" == preflight ]]; then
      cp "$retained/old/Cargo.toml" Cargo.toml
    else
      cp "$retained/new/Cargo.toml" Cargo.toml
    fi
    expect_failure "$phase"
    cmp inspected-ready "$retained/ready"
    cmp inspected-receipt ".release-state/$RELEASE_VERSION.validation"
    [[ "$(wc -l < gate-trace)" -eq 1 ]]
    for file in Cargo.toml Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md; do
      tree=new
      if [[ "$phase" == preflight && "$file" == Cargo.toml ]]; then tree=old; fi
      cmp "$retained/$tree/$file" "$file"
    done
  done
done
unset ADAPTER_HASH_STATUS ADAPTER_HASH_PATH
adapter prepared

new_fixture changed-input
adapter verify
echo 'source changed after validation' >> CHANGELOG.md
expect_failure prepare
[[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]

new_fixture admission
export ADAPTER_DIFF_STATUS=9
expect_failure preflight
unset ADAPTER_DIFF_STATUS
export ADAPTER_STAGED_DIRTY='staged unrelated file.rs'
expect_failure preflight
unset ADAPTER_STAGED_DIRTY
export ADAPTER_DIRTY='unrelated file.rs'
expect_failure preflight
unset ADAPTER_DIRTY
export ADAPTER_UNTRACKED='untracked release input'
expect_failure preflight
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
  [[ "$(wc -l < gate-trace)" -eq 1 ]]
  [[ ! -e ".release-state/$RELEASE_VERSION.metadata" ]]
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
  [[ "$(wc -l < gate-trace)" -eq 1 ]]
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

# The complete gate's registry dry run has a different network boundary from
# offline metadata preparation. Preserve explicit caller policy without fallback.
for policy in unset false true; do
  new_fixture "network-$policy"
  (
    if [[ "$policy" == unset ]]; then
      unset CARGO_NET_OFFLINE
    else
      export CARGO_NET_OFFLINE="$policy"
    fi
    export ADAPTER_REGISTRY_CHECK=true
    if [[ "$policy" == true ]]; then
      expect_failure verify
      expect_failure prepare
      [[ ! -e ".release-state/$RELEASE_VERSION.validation" ]]
    else
      adapter verify
      [[ -s ".release-state/$RELEASE_VERSION.validation" ]]
    fi
    [[ "$(cat gate-environment)" == "$policy" ]]
    [[ "$(wc -l < gate-trace)" -eq 1 ]]
    [[ "$(bash "$repo_root/scripts/release/read-workspace-version.sh" --stable Cargo.toml)" == "$previous" ]]
  )
done
echo 'release metadata isolated checks passed'
