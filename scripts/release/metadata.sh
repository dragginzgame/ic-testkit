#!/usr/bin/env bash
set -euo pipefail

# Consumer-owned metadata and evidence. The reviewed shared runner alone owns
# release selection, staging, commits, tags, remote refs and phase recovery.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
files=(Cargo.lock CHANGELOG.md crates/ic-testkit/CHANGELOG.md README.md crates/ic-testkit/README.md Cargo.toml)
mode="${1:-}"
[[ $# -eq 1 ]] || { echo "expected one adapter phase" >&2; exit 2; }
if [[ "$mode" == files ]]; then
  printf '%s\0' "${files[@]}"
  exit 0
fi
# shellcheck source=/dev/null
source "$script_dir/../../ci/tool-versions.env"
fail() { echo "release metadata refused: $*" >&2; exit 1; }
# Bind evidence to the bytes published, independent of Git's checkout filters.
hash_file() { git hash-object --no-filters -- "$1"; }
input_digest() { git diff --binary HEAD -- | git hash-object --stdin; }

check_paths() {
  local path allowed file path_list untracked
  # Git paths are NUL-delimited; whitespace in an unrelated name cannot evade
  # admission. Inspect index and worktree separately so opposing staged and
  # unstaged changes cannot cancel out and evade preflight.
  path_list="$(mktemp "$state/paths.XXXXXX")"
  git diff --name-only -z -- > "$path_list"
  git diff --name-only -z --cached -- >> "$path_list"
  while IFS= read -r -d '' path; do
    allowed=false
    for file in "${files[@]}"; do
      [[ "$path" != "$file" ]] || allowed=true
    done
    [[ "$allowed" == true ]] || fail "unrelated work: $path"
  done < "$path_list"
  rm -f "$path_list"
  untracked="$(git ls-files --others --exclude-standard)" || fail "cannot inspect untracked work"
  [[ -z "$untracked" ]] || fail "untracked work"
  for file in "${files[@]}"; do
    [[ -f "$file" && ! -L "$file" ]] || fail "missing or symlinked metadata: $file"
    git ls-files --error-unmatch -- "$file" >/dev/null
  done
}

validation_identity() {
  printf '%s\n' release-validation-v1 "$RELEASE_SOURCE" "$RELEASE_PREVIOUS" \
    "$RELEASE_VERSION" "$RELEASE_KIND" "$RELEASE_DATE" "$1"
}
check_validation() {
  local observed_validation expected_validation
  [[ -f "$validation" && ! -L "$validation" ]] || fail "validation evidence missing"
  observed_validation="$(cat "$validation")" || fail "cannot read validation evidence"
  validated_input="${observed_validation##*$'\n'}"
  [[ "$validated_input" =~ ^[0-9a-f]{40,64}$ ]] || fail "invalid validation digest"
  expected_validation="$(validation_identity "$validated_input")" || fail "cannot derive validation identity"
  [[ "$observed_validation" == "$expected_validation" ]] \
    || fail "validation belongs to another release"
}

prepared_identity() {
  local file old_digest new_digest
  printf '%s\n' release-metadata-v1 "$RELEASE_SOURCE" "$RELEASE_VERSION" "$RELEASE_DATE"
  for file in "${files[@]}"; do
    old_digest="$(hash_file "$prepared/old/$file")" || return
    new_digest="$(hash_file "$prepared/new/$file")" || return
    printf '%s\n' "$old_digest" "$new_digest"
  done
}
check_saved_metadata() {
  local expected_identity observed_identity
  check_validation
  [[ -f "$prepared/ready" && ! -L "$prepared/ready" ]] || fail "prepared metadata missing"
  expected_identity="$(prepared_identity)" || fail "cannot inspect prepared metadata"
  observed_identity="$(cat "$prepared/ready")" || fail "cannot read prepared metadata identity"
  [[ "$observed_identity" == "$expected_identity" ]] || fail "prepared metadata changed"
}
check_recovery() {
  check_saved_metadata
  local file digest old_digest new_digest
  # Intent and exact old/new bytes precede publication. On interruption,
  # accept only those two identities; never incorporate new dirty input.
  for file in "${files[@]}"; do
    digest="$(hash_file "$file")"
    old_digest="$(hash_file "$prepared/old/$file")"
    new_digest="$(hash_file "$prepared/new/$file")"
    [[ "$digest" == "$old_digest" || "$digest" == "$new_digest" ]] \
      || fail "metadata changed during recovery: $file"
  done
}
check_prepared() {
  check_saved_metadata
  local file digest expected_digest observed_version
  for file in "${files[@]}"; do
    digest="$(hash_file "$file")"
    expected_digest="$(hash_file "$prepared/new/$file")"
    [[ "$digest" == "$expected_digest" ]] \
      || fail "release payload changed: $file"
  done
  observed_version="$(bash "$script_dir/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" \
    || fail "cannot read prepared version"
  [[ "$observed_version" == "$RELEASE_VERSION" ]] \
    || fail "prepared version mismatch"
  bash "$script_dir/../ci/check-format-tools.sh" "$SHARED_TOOLING_CARGO_SORT_VERSION"
  cargo sort --workspace --check
  cargo metadata --locked --offline --format-version 1 --no-deps >/dev/null
  bash "$script_dir/../ci/check-installation-version.sh"
}
check_committed() {
  check_saved_metadata
  [[ "${RELEASE_COMMIT:-}" =~ ^[0-9a-f]{40,64}$ ]] || fail "missing selected release commit"
  local file entry entry_mode entry_type digest entry_path expected_digest
  # The runner verifies the exact release tree and its ancestry. Read that
  # selected tree here; newer committed fixes are inputs to a separate release.
  for file in "${files[@]}"; do
    entry="$(git ls-tree "$RELEASE_COMMIT" -- "$file")" || fail "cannot inspect committed metadata: $file"
    read -r entry_mode entry_type digest entry_path <<< "$entry" || fail "missing committed metadata: $file"
    [[ "$entry_mode" =~ ^100(644|755)$ && "$entry_type" == blob && "$entry_path" == "$file" ]] \
      || fail "missing or non-regular committed metadata: $file"
    expected_digest="$(hash_file "$prepared/new/$file")"
    [[ "$digest" == "$expected_digest" ]] || fail "committed release payload changed: $file"
  done
}

[[ "${RELEASE_SOURCE:-}" =~ ^[0-9a-f]{40,64}$ && "${RELEASE_DATE:-}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] \
  || fail "missing saved source/date"
calculated_version="$(bash "$script_dir/../ci/next-release-version.sh" "$RELEASE_PREVIOUS" "$RELEASE_KIND")" \
  || fail "cannot derive saved release selection"
[[ "$calculated_version" == "$RELEASE_VERSION" ]] \
  || fail "inconsistent saved release selection"
state="$(git rev-parse --git-path release-state)"
[[ ! -L "$state" ]] || fail "symlinked evidence directory"
mkdir -p "$state"
validation="$state/$RELEASE_VERSION.validation"
prepared="$state/$RELEASE_VERSION.metadata"
[[ ! -L "$prepared" ]] || fail "symlinked prepared metadata"
if [[ -e "$prepared" ]]; then
  [[ -d "$prepared" ]] || fail "prepared metadata is not a directory"
  # Retained intent and backups must stay inside their owned tree. Check before
  # copying or publishing; rejecting a linked ready file afterward is too late.
  retained_links="$(find "$prepared" -type l -print)"
  [[ -z "$retained_links" ]] || fail "symlink in prepared metadata"
fi
check_paths

case "$mode" in
  preflight)
    observed_version="$(bash "$script_dir/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" \
      || fail "cannot read source version"
    [[ "$observed_version" == "$RELEASE_PREVIOUS" ]] \
      || fail "source version differs from saved selection"
    bash "$script_dir/../ci/check-format-tools.sh" "$SHARED_TOOLING_CARGO_SORT_VERSION"
    cache_manifest=Cargo.toml
    if [[ -e "$prepared/ready" ]]; then
      check_recovery
      for member in crates/ic-testkit crates/ic_testkit_perf_probe; do
        cmp "$member/Cargo.toml" "$prepared/new/$member/Cargo.toml" \
          || fail "saved member manifest changed: $member"
      done
      # The new lockfile can be published before the root manifest. Check the
      # verified, consistent saved workspace without rewriting live metadata.
      cache_manifest="$prepared/new/Cargo.toml"
    fi
    # Prepare the selected cache using Cargo's caller-owned network policy.
    # An explicit offline policy still fails on a miss; never retry it online.
    cargo fetch --manifest-path "$cache_manifest" --locked
    ;;
  verify)
    attempt="$(mktemp -d "$state/$RELEASE_VERSION.validation.XXXXXX")"
    # A new attempt cannot reuse an earlier pass if its own gate fails. Retain
    # that receipt and each attempt's exact identity/log rather than overwrite it.
    [[ ! -L "$validation" ]] || fail "symlinked validation evidence"
    if [[ -e "$validation" ]]; then mv "$validation" "$attempt/prior-validation"; fi
    before="$(input_digest)"
    validation_identity "$before" > "$attempt/identity"
    # Complete gate, identical for patch/minor/major. Preserve the caller's
    # network policy: its registry dry run needs HTTP even with cached crates.
    # Retain the full log on failure; never retry offline failures online.
    make --no-print-directory release-check 2>&1 \
      | tee "$attempt/validation.log"
    after="$(input_digest)"
    [[ "$before" == "$after" ]] || fail "inputs changed during validation"
    temporary="$(mktemp "$validation.tmp.XXXXXX")"
    cp "$attempt/identity" "$temporary"
    mv "$temporary" "$validation"
    ;;
  prepare)
    check_validation
    if [[ ! -f "$prepared/ready" ]]; then
      current_input="$(input_digest)"
      [[ "$current_input" == "$validated_input" ]] || fail "inputs changed after validation"
      mkdir -p "$prepared/old" "$prepared/new"
      for file in "${files[@]}"; do
        mkdir -p "$prepared/old/$(dirname "$file")" "$prepared/new/$(dirname "$file")"
        cp -p "$file" "$prepared/old/$file"
        cp -p "$file" "$prepared/new/$file"
      done
      # Cargo owns lockfile generation. Resolve metadata in a private copy of
      # this workspace's manifests; no builds or real version changes occur.
      for member in crates/ic-testkit crates/ic_testkit_perf_probe; do
        mkdir -p "$prepared/new/$member/src"
        cp "$member/Cargo.toml" "$prepared/new/$member/Cargo.toml"
        : > "$prepared/new/$member/src/lib.rs"
      done
      cp rust-toolchain.toml "$prepared/new/rust-toolchain.toml"
      RELEASE_PREVIOUS="$RELEASE_PREVIOUS" RELEASE_VERSION="$RELEASE_VERSION" \
        perl -0pi -e '
          $n = s/(\[workspace\.package\]\n(?:(?!\n\[).)*?\nversion = ")\Q$ENV{RELEASE_PREVIOUS}\E("\n)/$1$ENV{RELEASE_VERSION}$2/s;
          die "workspace version not found\n" unless $n == 1;
        ' "$prepared/new/Cargo.toml"
      bash "$script_dir/../ci/check-format-tools.sh" "$SHARED_TOOLING_CARGO_SORT_VERSION"
      cargo sort --workspace "$prepared/new"
      # Member manifests are already sorted inputs, outside the release file
      # set. Formatting must not introduce an unstaged change to those inputs.
      for member in crates/ic-testkit crates/ic_testkit_perf_probe; do
        cmp "$member/Cargo.toml" "$prepared/new/$member/Cargo.toml" \
          || fail "member manifest requires formatting before release: $member"
      done
      cargo metadata --manifest-path "$prepared/new/Cargo.toml" --offline --format-version 1 >/dev/null
      # Ignore only generated versions of this workspace's two local packages;
      # all registry records and dependency edges must remain byte-identical.
      for tree in old new; do
        perl -0pe 's/(\[\[package\]\]\nname = "(?:ic-testkit|ic_testkit_perf_probe)"\nversion = ")[^"]+("\n)/${1}WORKSPACE${2}/g' \
          "$prepared/$tree/Cargo.lock" > "$prepared/$tree/selected-dependencies"
      done
      cmp "$prepared/old/selected-dependencies" "$prepared/new/selected-dependencies" \
        || fail "dependency selections changed"
      for file in CHANGELOG.md crates/ic-testkit/CHANGELOG.md; do
        awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" \
          -v date="$RELEASE_DATE" \
          -f "$script_dir/../ci/finalize-release-changelog.awk" "$prepared/old/$file" \
          > "$prepared/new/$file"
      done
      (
        # Bash 3.2 does not stop the parent on a failed subshell. Check every
        # command explicitly before publishing the readiness record or files.
        cd "$prepared/new" || fail "cannot enter prepared workspace"
        bash "$script_dir/../ci/check-installation-version.sh" --rewrite "$RELEASE_VERSION" \
          || fail "cannot prepare installation requirements"
        bash "$script_dir/../ci/check-installation-version.sh" "$RELEASE_VERSION" \
          || fail "prepared installation requirements are invalid"
      ) || fail "installation preparation failed"
      prepared_identity > "$prepared/ready.tmp"
      mv "$prepared/ready.tmp" "$prepared/ready"
    fi
    check_recovery
    for file in "${files[@]}"; do
      temporary="$(mktemp "$(dirname "$file")/.release-metadata.XXXXXX")"
      cp -p "$prepared/new/$file" "$temporary"
      mv "$temporary" "$file"
    done
    check_prepared
    ;;
  prepared | commit)
    check_prepared
    if [[ "$mode" == commit ]]; then
      git diff --quiet -- || fail "unstaged changes after release staging"
    fi
    ;;
  committed | tagged | push)
    check_committed
    ;;
  *) fail "unknown adapter phase: $mode" ;;
esac
