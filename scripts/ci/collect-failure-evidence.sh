#!/usr/bin/env bash
set -euo pipefail

# Testkit-owned CI selection; tar preserves legal Unix names, modes and links.
# Requires Bash 3.2, tar/gzip and the CI workspace/temp/outcome environment.
[[ $# -eq 1 ]] || { echo 'expected checks or portable evidence selection' >&2; exit 2; }
mode="$1"
case "$mode" in checks|portable) ;; *) echo 'unknown evidence selection' >&2; exit 2 ;; esac
workspace="$(cd "${GITHUB_WORKSPACE:?}" && pwd -P)"
temporary="$(cd "${RUNNER_TEMP:?}" && pwd -P)"
outcomes="${IC_TESTKIT_FAILURE_STEPS:?}"
bundle="$temporary/ic-testkit-$mode-failure"
# A previous attempt is evidence, not an output to overwrite.
mkdir "$bundle"
printf '%s\n' "$outcomes" > "$bundle/step-outcomes.json"
arguments=(-C "$bundle" ./step-outcomes.json)
names=(ic-testkit-tools-install.log)
if [[ "$mode" == checks ]]; then
    names+=(ic-testkit-validation.log)
else
    names+=(ic-testkit-portable-fixtures ic-testkit-archive-proof ic-testkit-archive-downloaded)
fi
for name in "${names[@]}"; do
    if [[ -e "$temporary/$name" || -L "$temporary/$name" ]]; then
        arguments+=(-C "$temporary" "./$name")
    fi
done
# Do not traverse an intermediate link into unrelated filesystem contents.
for parent in target .tools; do
    [[ ! -L "$workspace/$parent" ]] || { echo "linked evidence parent: $parent" >&2; exit 1; }
done
if [[ -e "$workspace/target/validation-failures" || -L "$workspace/target/validation-failures" ]]; then
    arguments+=(-C "$workspace" ./target/validation-failures)
fi
shopt -s nullglob
for path in "$workspace"/.tools/host-set.* "$workspace"/.tools/ic-set.*; do
    arguments+=(-C "$workspace" "./${path#"$workspace"/}")
done
# Never follow links or include Git metadata. Failed archives and all source
# evidence remain available; the uploader selects only the completed filename.
TAR_OPTIONS='' tar -czf "$bundle/evidence.tar.gz.partial" \
    --exclude=.git --exclude='*/.git' --exclude='*/.git/*' "${arguments[@]}"
mv "$bundle/evidence.tar.gz.partial" "$bundle/evidence.tar.gz"
printf '%s\n' "$bundle/evidence.tar.gz"
