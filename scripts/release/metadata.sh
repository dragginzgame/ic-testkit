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
fail() { echo "release metadata refused: $*" >&2; exit 1; }
hash_file() { git hash-object -- "$1"; }
input_digest() { git diff --binary HEAD -- | git hash-object --stdin; }

check_paths() {
  local path allowed file path_list
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
  [[ -z "$(git ls-files --others --exclude-standard)" ]] || fail "untracked work"
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
  [[ -f "$validation" && ! -L "$validation" ]] || fail "validation evidence missing"
  validated_input="$(tail -n 1 "$validation")"
  [[ "$validated_input" =~ ^[0-9a-f]{40,64}$ ]] || fail "invalid validation digest"
  [[ "$(cat "$validation")" == "$(validation_identity "$validated_input")" ]] \
    || fail "validation belongs to another release"
}

prepared_identity() {
  local file old_digest new_digest
  printf '%s\n' release-metadata-v1 "$RELEASE_SOURCE" "$RELEASE_VERSION" "$RELEASE_DATE"
  for file in "${files[@]}"; do
    old_digest="$(hash_file "$prepared/old/$file")"
    new_digest="$(hash_file "$prepared/new/$file")"
    printf '%s\n' "$old_digest" "$new_digest"
  done
}
check_prepared() {
  check_validation
  [[ -f "$prepared/ready" && ! -L "$prepared/ready" ]] || fail "prepared metadata missing"
  [[ "$(cat "$prepared/ready")" == "$(prepared_identity)" ]] || fail "prepared metadata changed"
  local file
  for file in "${files[@]}"; do
    [[ "$(hash_file "$file")" == "$(hash_file "$prepared/new/$file")" ]] \
      || fail "release payload changed: $file"
  done
  [[ "$(bash "$script_dir/read-workspace-version.sh" --stable Cargo.toml)" == "$RELEASE_VERSION" ]] \
    || fail "prepared version mismatch"
  bash "$script_dir/../ci/check-format-tools.sh"
  cargo sort --workspace --check
  cargo metadata --locked --offline --format-version 1 --no-deps >/dev/null
  bash "$script_dir/../ci/check-installation-version.sh"
}

[[ "${RELEASE_SOURCE:-}" =~ ^[0-9a-f]{40,64}$ && "${RELEASE_DATE:-}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] \
  || fail "missing saved source/date"
[[ "$(bash "$script_dir/../ci/next-release-version.sh" "$RELEASE_PREVIOUS" "$RELEASE_KIND")" == "$RELEASE_VERSION" ]] \
  || fail "inconsistent saved release selection"
state="$(git rev-parse --git-path release-state)"
[[ ! -L "$state" ]] || fail "symlinked evidence directory"
mkdir -p "$state"
validation="$state/$RELEASE_VERSION.validation"
prepared="$state/$RELEASE_VERSION.metadata"
[[ ! -L "$prepared" ]] || fail "symlinked prepared metadata"
check_paths

case "$mode" in
  preflight)
    [[ "$(bash "$script_dir/read-workspace-version.sh" --stable Cargo.toml)" == "$RELEASE_PREVIOUS" ]] \
      || fail "source version differs from saved selection"
    bash "$script_dir/../ci/check-format-tools.sh"
    cargo fetch --locked --offline
    ;;
  verify)
    attempt="$(mktemp -d "$state/$RELEASE_VERSION.validation.XXXXXX")"
    # A new attempt cannot reuse an earlier pass if its own gate fails. Retain
    # that receipt and each attempt's exact identity/log rather than overwrite it.
    [[ ! -L "$validation" ]] || fail "symlinked validation evidence"
    if [[ -e "$validation" ]]; then mv "$validation" "$attempt/prior-validation"; fi
    before="$(input_digest)"
    validation_identity "$before" > "$attempt/identity"
    # Complete gate, identical for patch/minor/major. Retain its full log even
    # on failure, without cleaning Cargo artifacts or resolving dependencies.
    CARGO_NET_OFFLINE=true make --no-print-directory release-check 2>&1 \
      | tee "$attempt/validation.log"
    [[ "$before" == "$(input_digest)" ]] || fail "inputs changed during validation"
    temporary="$(mktemp "$validation.tmp.XXXXXX")"
    cp "$attempt/identity" "$temporary"
    mv "$temporary" "$validation"
    ;;
  prepare)
    check_validation
    if [[ ! -f "$prepared/ready" ]]; then
      [[ "$(input_digest)" == "$validated_input" ]] || fail "inputs changed after validation"
      mkdir -p "$prepared/old" "$prepared/new"
      for file in "${files[@]}"; do
        mkdir -p "$prepared/old/$(dirname "$file")" "$prepared/new/$(dirname "$file")"
        cp -p "$file" "$prepared/old/$file"
        cp -p "$file" "$prepared/new/$file"
      done
      # Cargo owns lockfile generation. Resolve metadata in a private copy of
      # this workspace's manifests; no builds or real version changes occur.
      for member in crates/ic-testkit canisters/test/perf_probe; do
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
      bash "$script_dir/../ci/check-format-tools.sh"
      cargo sort --workspace "$prepared/new"
      # Member manifests are already sorted inputs, outside the release file
      # set. Formatting must not introduce an unstaged change to those inputs.
      for member in crates/ic-testkit canisters/test/perf_probe; do
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
        awk -v version="$RELEASE_VERSION" -v date="$RELEASE_DATE" \
          -f "$script_dir/../ci/finalize-release-changelog.awk" "$prepared/old/$file" \
          > "$prepared/new/$file"
      done
      for file in README.md crates/ic-testkit/README.md; do
        REQUIREMENT="${RELEASE_VERSION%.*}" perl -pi -e \
          's/^(\s*ic-testkit\s*=\s*")[^"]+(".*)$/$1$ENV{REQUIREMENT}$2/' "$prepared/new/$file"
      done
      (cd "$prepared/new"; bash "$script_dir/../ci/check-installation-version.sh" "$RELEASE_VERSION")
      prepared_identity > "$prepared/ready.tmp"
      mv "$prepared/ready.tmp" "$prepared/ready"
    fi
    [[ "$(cat "$prepared/ready")" == "$(prepared_identity)" ]] || fail "corrupt prepared metadata"
    # Intent and exact old/new bytes precede publication. On interruption,
    # accept only those two identities; never incorporate new dirty input.
    for file in "${files[@]}"; do
      digest="$(hash_file "$file")"
      [[ "$digest" == "$(hash_file "$prepared/old/$file")" || "$digest" == "$(hash_file "$prepared/new/$file")" ]] \
        || fail "metadata changed during recovery: $file"
    done
    for file in "${files[@]}"; do
      temporary="$(mktemp "$(dirname "$file")/.release-metadata.XXXXXX")"
      cp -p "$prepared/new/$file" "$temporary"
      mv "$temporary" "$file"
    done
    check_prepared
    ;;
  prepared | commit | committed | tagged | push)
    check_prepared
    if [[ "$mode" == commit ]]; then
      git diff --quiet -- || fail "unstaged changes after release staging"
    fi
    ;;
  *) fail "unknown adapter phase: $mode" ;;
esac
