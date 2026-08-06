use ic_testkit::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCachePrunePolicy, ArtifactCacheSpec,
    prepare_artifact_cache,
};
use std::{io, path::PathBuf, process::Command, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let [tool, input, output, cache_root] = arguments.as_slice() else {
        eprintln!(
            "usage: transactional_artifact_cache <tool> <input> <public-output> <cache-root>"
        );
        return Ok(());
    };
    let tool = PathBuf::from(tool);
    let input = PathBuf::from(input);
    let output = PathBuf::from(output);
    let cache_root = PathBuf::from(cache_root);
    let spec = ArtifactCacheSpec::new(&cache_root, "example-transform", "example/transform/v1")
        .with_input("source", &input)
        .with_tool("transformer", &tool)
        .with_arguments(&["<input>", "<output>"])
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
            let status = Command::new(&tool)
                .arg(&input)
                .arg(staged_output)
                .status()?;
            if !status.success() {
                return Err(
                    io::Error::other(format!("external transformer failed with {status}")).into(),
                );
            }
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
