#!/usr/bin/env bash
set -euo pipefail

# Testkit-owned CI selection; the shared archiver owns archive mechanics.
# Requires Bash 3.2, tar/gzip and the CI workspace/temp/outcome environment.
[[ $# -eq 1 ]] || { echo 'expected checks or portable evidence selection' >&2; exit 2; }
mode="$1"
case "$mode" in checks|portable) ;; *) echo 'unknown evidence selection' >&2; exit 2 ;; esac
workspace="${GITHUB_WORKSPACE:?}"
[[ "$workspace" == /* ]] || workspace="$PWD/$workspace"
workspace="$(cd -P "$workspace" && printf '%s/.' "$PWD")"
workspace="${workspace%/.}"
temporary="${RUNNER_TEMP:?}"
[[ "$temporary" == /* ]] || temporary="$PWD/$temporary"
temporary="$(cd -P "$temporary" && printf '%s/.' "$PWD")"
temporary="${temporary%/.}"
archiver="$0"
[[ "$archiver" == /* ]] || archiver="$PWD/$archiver"
archiver="${archiver%/*}/archive-evidence.sh"
outcomes="${IC_TESTKIT_FAILURE_STEPS:?}"
bundle="$temporary/ic-testkit-$mode-failure"
# A previous attempt is evidence, not an output to overwrite.
mkdir "$bundle"
printf '%s\n' "$outcomes" > "$bundle/step-outcomes.json"
arguments=("$bundle" step-outcomes.json)
names=(ic-testkit-tools-install.log)
if [[ "$mode" == checks ]]; then
    names+=(ic-testkit-validation.log)
else
    names+=(ic-testkit-portable-fixtures ic-testkit-archive-proof ic-testkit-archive-downloaded)
fi
for name in "${names[@]}"; do
    if [[ -e "$temporary/$name" || -L "$temporary/$name" ]]; then
        arguments+=("$temporary" "$name")
    fi
done
# Do not traverse an intermediate link into unrelated filesystem contents.
for parent in target .tools; do
    [[ ! -L "$workspace/$parent" ]] || { echo "linked evidence parent: $parent" >&2; exit 1; }
done
if [[ -e "$workspace/target/validation-failures" || -L "$workspace/target/validation-failures" ]]; then
    arguments+=("$workspace" target/validation-failures)
fi
shopt -s nullglob
for path in "$workspace"/.tools/host-set.* "$workspace"/.tools/ic-set.*; do
    arguments+=("$workspace" "${path#"$workspace"/}")
done
# Never follow links or include Git metadata. Failed archives and all source
# evidence remain available; the uploader selects only the completed filename.
TAR_OPTIONS='' bash "$archiver" \
    "$bundle/evidence.tar.gz.partial" "${arguments[@]}" >/dev/null
mv "$bundle/evidence.tar.gz.partial" "$bundle/evidence.tar.gz"
printf '%s\n' "$bundle/evidence.tar.gz"
