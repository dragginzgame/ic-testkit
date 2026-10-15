#!/usr/bin/env bash
set -euo pipefail

# Testkit owns CI roots/retention; shared helpers select tool evidence and archive it.
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
# Shared formatter failures retain exact logs under the workflow temp root.
# shellcheck source=scripts/ci/formatting-evidence.sh
source "${archiver%/*}/formatting-evidence.sh"
select_formatting_evidence "$temporary"
arguments+=(${formatting_evidence[@]+"${formatting_evidence[@]}"})
# Retain a redirected parent itself and unrelated logs, without traversing it.
collection_status=0
for parent in target .tools; do
    if [[ -L "$workspace/$parent" ]]; then
        printf 'linked evidence parent: %s\n' "$parent" >> "$bundle/collection.log"
        printf 'linked evidence parent: %s\n' "$parent" >&2
        arguments+=("$workspace" "$parent")
        collection_status=1
    fi
done
if [[ "$collection_status" != 0 ]]; then arguments+=("$bundle" collection.log); fi
if [[ ! -L "$workspace/target" ]] &&
    [[ -e "$workspace/target/validation-failures" || -L "$workspace/target/validation-failures" ]]; then
    arguments+=("$workspace" target/validation-failures)
fi
diagnostics="$bundle/tool-evidence"
mkdir "$diagnostics"
selector_status=0
bash "${archiver%/*}/select-tool-evidence.sh" compact "$workspace" "$diagnostics" \
    "$workspace/ci/tool-versions.env" "$workspace/ci/ic-tools.tsv" \
    > "$diagnostics/selections.nul" 2> "$diagnostics/selection.log" || selector_status=$?
printf '%s\n' "$selector_status" > "$diagnostics/selection-status"
if [[ "$selector_status" == 0 ]]; then
    while IFS= read -r -d '' selected_root && IFS= read -r -d '' selected_path; do
        arguments+=("$selected_root" "$selected_path")
    done < "$diagnostics/selections.nul"
else
    # A failed selector's partial output is diagnostic data, never authority to
    # traverse a store. Retain the physical checkout's tools and links themselves.
    printf 'shared_collection=incomplete\n' > "$diagnostics/collection-status"
    cat "$diagnostics/selection.log" >&2
    echo 'Shared evidence collection incomplete; retaining local evidence and diagnostics.' >&2
    if [[ ! -L "$workspace/.tools" && -d "$workspace/.tools" ]]; then arguments+=("$workspace" .tools); fi
fi
if [[ "$selector_status" != 0 || -d "$diagnostics/host" || -d "$diagnostics/ic" ]]; then
    arguments+=("$bundle" tool-evidence)
fi
# Never follow links or include Git metadata. Failed archives and all source
# evidence remain available; the uploader selects only the completed filename.
TAR_OPTIONS='' bash "$archiver" \
    "$bundle/evidence.tar.gz.partial" "${arguments[@]}" >/dev/null
mv "$bundle/evidence.tar.gz.partial" "$bundle/evidence.tar.gz"
printf '%s\n' "$bundle/evidence.tar.gz"
# CI publishes the archive path before propagating this failure to the step.
[[ "$selector_status" == 0 ]] || exit "$selector_status"
exit "$collection_status"
