use super::test_support::{unique_temp_directory, write_executable_script};
use super::{ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache};
use crate::{
    ic_host_artifacts::artifact::{ArtifactError, Sha256Digest},
    ic_host_process::tool::{
        AdmittedTool, ExecutionContext, OutputLimits, ToolError, ToolSpec, resolve_executable,
    },
};
use std::{ffi::OsString, fs, path::Path, time::Duration};

#[test]
fn admitted_shared_tool_output_is_published_and_invalidated_by_tool_bytes() {
    let root = unique_temp_directory("shared-admitted-tool")
        .canonicalize()
        .unwrap();
    let script = b"#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'fixture-tool 1.0\\n'; else printf 'compiled output'; fi\n";
    let executable = root.join("producer");
    write_executable_script(&executable, script);
    let executable = resolve_executable(Path::new("./producer"), &root, &[]).unwrap();
    let context = ExecutionContext {
        current_dir: &root,
        environment: &[],
    };
    let limits = OutputLimits {
        stdout_bytes: 128,
        stderr_bytes: 128,
        timeout: Duration::from_secs(5),
    };
    let tool = AdmittedTool::admit(
        &ToolSpec {
            executable: &executable,
            sha256: Sha256Digest::compute(script),
            executable_bytes: script.len() as u64,
            version_arguments: &[OsString::from("--version")],
            version_identity: "fixture-tool 1.0",
        },
        &context,
        limits,
    )
    .unwrap();
    let destination = root.join("output");
    let spec = ArtifactCacheSpec::new(&root.join("cache"), "shared-tool", "compile/v1")
        .with_tool("producer", &executable)
        .with_output("compiled", &destination);
    let ArtifactCachePreparation::Build(transaction) = prepare_artifact_cache(&spec).unwrap()
    else {
        panic!("first acquisition must build");
    };
    let evidence = tool
        .run(&[OsString::from("compile")], &context, limits)
        .unwrap();
    fs::write(
        transaction.output_path("compiled").unwrap(),
        &evidence.stdout,
    )
    .unwrap();
    let record = transaction.commit().unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"compiled output");
    let ArtifactCachePreparation::Reused(reused) = prepare_artifact_cache(&spec).unwrap() else {
        panic!("same admitted producer must reuse its output");
    };
    write_executable_script(&executable, b"#!/bin/sh\nexit 1\n");
    assert!(matches!(
        tool.run(&[OsString::from("compile")], &context, limits),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    assert!(matches!(
        prepare_artifact_cache(&spec).unwrap(),
        ArtifactCachePreparation::Build(_)
    ));
    drop(reused);
    drop(record);
    fs::remove_dir_all(root).unwrap();
}
