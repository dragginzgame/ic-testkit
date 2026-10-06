use ic_testkit::{
    artifacts::{
        ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCachePrunePolicy,
        ArtifactCacheSpec, prepare_artifact_cache,
    },
    ic_host_tools::{
        artifact::Sha256Digest,
        tool::{AdmittedTool, ExecutionContext, OutputLimits, ToolSpec, resolve_executable},
    },
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let [tool, sha256, version, input, output, cache_root] = arguments.as_slice() else {
        eprintln!(
            "usage: transactional_artifact_cache <tool-path> <sha256> <exact-version-stdout> <input> <public-output> <cache-root>"
        );
        return Ok(());
    };
    let current_dir = std::env::current_dir()?;
    let tool = resolve_executable(&current_dir.join(tool), &current_dir, &[])?;
    let context = ExecutionContext {
        current_dir: &current_dir,
        environment: &[],
    };
    let limits = OutputLimits {
        stdout_bytes: 64 * 1024,
        stderr_bytes: 64 * 1024,
        timeout: Duration::from_secs(60),
    };
    // Admission uses caller authority, never a digest computed from untrusted
    // selected bytes. Transform tools must work with this empty environment.
    let admitted = AdmittedTool::admit(
        &ToolSpec {
            executable: &tool,
            sha256: sha256
                .to_str()
                .ok_or("SHA-256 must be UTF-8")?
                .parse::<Sha256Digest>()?,
            executable_bytes: 256 * 1024 * 1024,
            version_arguments: &["--version".into()],
            version_identity: version.to_str().ok_or("version must be UTF-8")?,
        },
        &context,
        limits,
    )?;
    let input = current_dir.join(input);
    let output = current_dir.join(output);
    let cache_root = current_dir.join(cache_root);
    let spec = ArtifactCacheSpec::new(
        &cache_root,
        "example-transform",
        "example/admitted-transform/v1",
    )
    .with_input("source", &input)
    .with_tool("transformer", &tool)
    .with_arguments(["<input>", "<output>"])
    .with_output("result", &output)
    .with_prune_policy(
        ArtifactCachePrunePolicy::new()
            .with_max_age(Duration::from_secs(7 * 24 * 60 * 60))
            .with_max_size_bytes(2 * 1024 * 1024 * 1024),
    );

    let outcome = match prepare_artifact_cache(&spec)? {
        ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
        ArtifactCachePreparation::Build(transaction) => {
            let staged_output = transaction.output_path("result")?;
            admitted.run(
                &[input.into_os_string(), staged_output.into_os_string()],
                &context,
                limits,
            )?;
            transaction.commit()?
        }
    };

    let disposition = if outcome.is_reused() {
        "reused"
    } else {
        "built"
    };
    println!("{disposition} artifact set {}", outcome.record().key());
    Ok(())
}
