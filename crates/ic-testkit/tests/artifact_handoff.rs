#![cfg(unix)]

#[path = "support/executable.rs"]
mod executable_support;
mod support;

use ic_testkit::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCachePrunePolicy, ArtifactCacheSpec,
    LabeledArtifactCacheSpec, LabeledWasmBuildSpec, SharedIncrementalTargetPrunePolicy,
    WasmBuildSpec, build_artifact_caches_batch, build_wasm_canisters_cached,
    build_wasm_canisters_cached_batch, maintain_shared_incremental_target, prepare_artifact_cache,
    prune_artifact_cache, prune_wasm_build_cache,
};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

fn fixture() -> PathBuf {
    let root = support::unique_temp_directory("handoff");
    fs::create_dir_all(root.join("workspace/src")).unwrap();
    fs::write(
        root.join("workspace/Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n",
    )
    .unwrap();
    fs::write(root.join("workspace/src/lib.rs"), "pub fn fixture() {}\n").unwrap();
    // Use real Cargo metadata but synthetic variant-specific build bytes. No
    // Wasm toolchain or canister runtime is needed to exercise cache ownership.
    executable_support::write_executable_script(
        &root.join("cargo.sh"),
        br#"#!/bin/sh
if [ "$1" != build ]; then exec "$REAL_CARGO" "$@"; fi
if [ "$VARIANT" = failure ]; then exit 23; fi
output="$CARGO_TARGET_DIR"
mkdir -p "$output/wasm32-unknown-unknown/debug"
printf '%s' "$VARIANT" > "$output/wasm32-unknown-unknown/debug/fixture.wasm"
"#,
    );
    root
}

fn isolated_wasm_spec(root: &Path, variant: &str) -> WasmBuildSpec {
    WasmBuildSpec::new(
        &root.join("workspace"),
        &root.join("wasm"),
        &["fixture"],
        "debug",
    )
    .with_cargo_program(root.join("cargo.sh"))
    .with_extra_env([
        (
            "REAL_CARGO",
            std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()),
        ),
        ("VARIANT", variant.to_owned()),
    ])
    .with_prune_policy(ArtifactCachePrunePolicy::new().with_max_size_bytes(0))
}

fn wasm_spec(root: &Path, variant: &str) -> WasmBuildSpec {
    isolated_wasm_spec(root, variant).with_shared_incremental_target(root.join("shared"))
}

#[test]
fn isolated_build_records_retain_cold_and_warm_artifacts() {
    let root = fixture();
    let first = build_wasm_canisters_cached(&isolated_wasm_spec(&root, "A")).unwrap();
    assert!(!first.is_reused());
    let other = build_wasm_canisters_cached(&isolated_wasm_spec(&root, "B")).unwrap();
    let reused = build_wasm_canisters_cached(&isolated_wasm_spec(&root, "A")).unwrap();
    assert!(reused.is_reused());
    let path = reused.record().artifacts()[0].clone();
    drop(first);
    prune_wasm_build_cache(
        &root.join("wasm"),
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    )
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"A");
    assert_eq!(fs::read(&other.record().artifacts()[0]).unwrap(), b"B");
    drop(reused);
    prune_wasm_build_cache(
        &root.join("wasm"),
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    )
    .unwrap();
    assert!(!path.exists());
    drop(other);
    fs::remove_dir_all(root).unwrap();
}

fn post_spec(root: &Path, input: &Path) -> ArtifactCacheSpec {
    ArtifactCacheSpec::new(&root.join("post"), "handoff", "post-link/v1")
        .with_input("wasm", input)
        .with_output("deploy", &root.join("deploy.wasm"))
        .with_prune_policy(ArtifactCachePrunePolicy::new().with_max_size_bytes(0))
}

fn post_link(spec: &ArtifactCacheSpec, input: &Path) -> ArtifactCacheOutcome {
    match prepare_artifact_cache(spec).unwrap() {
        ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
        ArtifactCachePreparation::Build(transaction) => {
            transaction.import_output("deploy", input).unwrap();
            transaction.commit().unwrap()
        }
    }
}

fn prune(root: &Path, policy: ArtifactCachePrunePolicy) {
    prune_wasm_build_cache(&root.join("wasm"), policy).unwrap();
    prune_artifact_cache(&root.join("post"), "handoff", policy).unwrap();
}

struct Worker(Child);
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Coordination {
    reader: Box<dyn BufRead>,
    writer: Box<dyn Write>,
}

fn receive(stream: &mut Coordination) -> PathBuf {
    loop {
        let mut line = String::new();
        assert!(
            stream.reader.read_line(&mut line).unwrap() > 0,
            "worker closed coordination pipe"
        );
        if let Some((_, path)) = line.split_once("retained:") {
            return serde_json::from_str(path).unwrap();
        }
    }
}

fn release(stream: &mut Coordination) {
    stream.writer.write_all(b"continue\n").unwrap();
}

fn announce_and_wait(stream: &mut Coordination, path: &Path) {
    writeln!(
        stream.writer,
        "retained:{}",
        serde_json::to_string(path).unwrap()
    )
    .unwrap();
    let mut line = String::new();
    assert!(stream.reader.read_line(&mut line).unwrap() > 0);
    assert_eq!(line, "continue\n");
}

#[test]
fn retained_handoff_survives_other_process_builds_and_pruning() {
    for warm in [false, true] {
        exercise_process_handoff(warm, false);
    }
}

#[test]
fn terminated_consumer_releases_retention_without_stale_pins() {
    exercise_process_handoff(false, true);
}

fn exercise_process_handoff(warm: bool, terminate: bool) {
    let root = fixture();
    if warm {
        let seed = build_wasm_canisters_cached(&wasm_spec(&root, "A")).unwrap();
        let input = &seed.record().artifacts()[0];
        drop(post_link(&post_spec(&root, input), input));
        drop(seed);
        // Force the worker's warm hit to rematerialize A, not just reuse an
        // already matching public output. Do not prune A during this setup.
        let alternate = wasm_spec(&root, "B").with_prune_policy(ArtifactCachePrunePolicy::new());
        drop(build_wasm_canisters_cached(&alternate).unwrap());
        fs::write(root.join("deploy.wasm"), b"B").unwrap();
    }
    let mut worker = Worker(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "retained_handoff_worker",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .env("IC_TESTKIT_HANDOFF_ROOT", &root)
            .env("IC_TESTKIT_HANDOFF_WARM", if warm { "yes" } else { "no" })
            .spawn()
            .unwrap(),
    );
    let mut stream = Coordination {
        reader: Box::new(BufReader::new(worker.0.stdout.take().unwrap())),
        writer: Box::new(worker.0.stdin.take().unwrap()),
    };
    let wasm_path = receive(&mut stream);
    let other = build_wasm_canisters_cached(&wasm_spec(&root, "B")).unwrap();
    let other_path = &other.record().artifacts()[0];
    assert_eq!(fs::read(other_path).unwrap(), b"B");
    assert_eq!(
        fs::read(root.join("wasm/wasm32-unknown-unknown/debug/fixture.wasm")).unwrap(),
        b"B"
    );
    maintain_shared_incremental_target(
        &wasm_spec(&root, "B"),
        SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(0),
    )
    .unwrap();
    for policy in [
        ArtifactCachePrunePolicy::new().with_max_age(Duration::ZERO),
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    ] {
        prune_wasm_build_cache(&root.join("wasm"), policy).unwrap();
        assert_eq!(fs::read(&wasm_path).unwrap(), b"A");
    }
    release(&mut stream);
    let post_path = receive(&mut stream);
    let other_post = post_link(&post_spec(&root, other_path), other_path);
    assert_eq!(fs::read(root.join("deploy.wasm")).unwrap(), b"B");
    prune(
        &root,
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    );
    assert_eq!(fs::read(&post_path).unwrap(), b"A");
    if terminate {
        worker.0.kill().unwrap();
        worker.0.wait().unwrap();
    } else {
        release(&mut stream);
        assert!(worker.0.wait().unwrap().success());
    }
    prune(
        &root,
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    );
    assert!(!wasm_path.exists());
    assert!(!post_path.exists());
    assert_eq!(
        fs::read(other_post.record().artifacts()[0].path()).unwrap(),
        b"B"
    );
    drop(other_post);
    drop(other);
    prune(
        &root,
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "subprocess consumer coordinated over pipes by the handoff tests"]
fn retained_handoff_worker() {
    let root = PathBuf::from(std::env::var_os("IC_TESTKIT_HANDOFF_ROOT").unwrap());
    let mut stream = Coordination {
        reader: Box::new(BufReader::new(std::io::stdin())),
        writer: Box::new(std::io::stdout()),
    };
    let outcome = build_wasm_canisters_cached(&wasm_spec(&root, "A")).unwrap();
    assert_eq!(
        outcome.is_reused(),
        std::env::var("IC_TESTKIT_HANDOFF_WARM").unwrap() == "yes"
    );
    let record = outcome.record().clone();
    drop(outcome);
    let input = &record.artifacts()[0];
    announce_and_wait(&mut stream, input);
    assert_eq!(fs::read(input).unwrap(), b"A");
    let post = post_link(&post_spec(&root, input), input);
    assert_eq!(
        post.is_reused(),
        std::env::var("IC_TESTKIT_HANDOFF_WARM").unwrap() == "yes"
    );
    let post_record = post.record().clone();
    drop(post);
    let post = post_record;
    announce_and_wait(&mut stream, post.artifacts()[0].path());
    assert_eq!(fs::read(post.artifacts()[0].path()).unwrap(), b"A");
}

#[test]
fn batches_retain_distinct_successes_through_later_failure() {
    let root = fixture();
    let specs = ["A", "B", "failure"]
        .map(|variant| LabeledWasmBuildSpec::new(variant, wasm_spec(&root, variant)));
    let report = build_wasm_canisters_cached_batch(&specs).unwrap();
    assert_eq!(report.failures().count(), 1, "{report:?}");
    let records: Vec<_> = report
        .outcomes()
        .map(|entry| entry.outcome().record().clone())
        .collect();
    drop(report);
    for (record, expected) in records.iter().zip([b"A", b"B"]) {
        assert_eq!(fs::read(&record.artifacts()[0]).unwrap(), expected);
    }
    let specs = records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            LabeledArtifactCacheSpec::new(
                index.to_string(),
                post_spec(&root, &record.artifacts()[0]),
            )
        })
        .chain(std::iter::once(LabeledArtifactCacheSpec::new(
            "failure",
            post_spec(&root, &records[0].artifacts()[0]).with_arguments(["fail"]),
        )))
        .collect::<Vec<_>>();
    let mut previous = Vec::new();
    for warm in [false, true] {
        let report = build_artifact_caches_batch(&specs, |label, transaction| {
            if label == "failure" {
                return Err("deliberate recipe failure");
            }
            let index: usize = label.parse().unwrap();
            transaction
                .import_output("deploy", &records[index].artifacts()[0])
                .unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(report.failures().count(), 1);
        let retained: Vec<_> = report
            .outcomes()
            .map(|entry| {
                assert_eq!(entry.outcome().is_reused(), warm);
                entry.outcome().record().clone()
            })
            .collect();
        drop(report);
        prune(
            &root,
            ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
        );
        for (record, expected) in retained.iter().zip([b"A", b"B"]) {
            assert_eq!(fs::read(record.artifacts()[0].path()).unwrap(), expected);
        }
        previous = retained;
    }
    let post_paths: Vec<_> = previous
        .iter()
        .map(|record| record.artifacts()[0].path().to_owned())
        .collect();
    drop(previous);
    let paths: Vec<_> = records
        .iter()
        .map(|record| record.artifacts()[0].clone())
        .collect();
    drop(records);
    prune(
        &root,
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    );
    assert!(paths.iter().chain(&post_paths).all(|path| !path.exists()));
    fs::remove_dir_all(root).unwrap();
}
