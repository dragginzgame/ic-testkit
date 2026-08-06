use candid::Principal;
use ic_testkit::{
    artifacts::{
        ArtifactCacheError, ArtifactCachePreparation, ArtifactCacheSpec, WasmBuildCacheMaintenance,
        WasmBuildCachePrunePolicy, WasmBuildOutcome, WasmBuildSpec, build_wasm_canisters_cached,
        inspect_shared_incremental_target, prepare_artifact_cache, prune_wasm_build_cache,
        read_wasm, resolve_cargo_build_inputs, wasm_path, workspace_root_for,
    },
    benchmark::{
        BenchmarkEventSource, BenchmarkParserConfig, pair_benchmark_spans,
        parse_benchmark_events_from_source,
    },
    pic::{InstallSpec, PocketIc, StandaloneCanisterFixture},
};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
};

#[cfg(unix)]
use ic_testkit::artifacts::WasmBuildError;
#[cfg(unix)]
use std::{ffi::OsString, os::unix::fs::PermissionsExt as _};

const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";

#[test]
fn perf_probe_canister_emits_parseable_benchmark_markers() {
    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    if !workspace
        .join("canisters/test/perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping perf probe canister test: fixture canister is not packaged");
        return;
    }
    let target_dir = unique_temp_dir("ic-testkit-perf-probe-target");

    let spec = WasmBuildSpec::new(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], "debug")
        .with_prune_policy_at_most_every(
            WasmBuildCachePrunePolicy::new(),
            std::time::Duration::from_secs(60 * 60),
        );
    let first = build_wasm_canisters_cached(&spec).expect("first exact Wasm build");
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(first.record().timings().cargo_build().is_some());
    assert_cache_observability(&first);

    let reused = build_wasm_canisters_cached(&spec).expect("reuse exact Wasm build");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    assert_eq!(first.record().fingerprint(), reused.record().fingerprint());
    assert!(reused.record().timings().cargo_build().is_none());
    assert!(
        reused.record().maintenance().is_none(),
        "scheduled maintenance should skip a second immediate cache hit"
    );
    assert!(reused.record().timings().cache_maintenance().is_some());

    let wasm_path = wasm_path(&target_dir, PERF_PROBE_PACKAGE, "debug");
    fs::write(&wasm_path, b"tampered").expect("overwrite cached Wasm artifact");
    let repaired = build_wasm_canisters_cached(&spec)
        .expect("artifact content mismatch should invalidate Wasm build");
    assert!(matches!(repaired, WasmBuildOutcome::Reused(_)));
    assert_ne!(
        fs::read(&wasm_path).expect("read repaired Wasm artifact"),
        b"tampered",
    );

    let changed_environment = spec
        .clone()
        .with_extra_env(&[("IC_TESTKIT_CACHE_PROBE", "changed")]);
    let rebuilt = build_wasm_canisters_cached(&changed_environment)
        .expect("declared environment should invalidate Wasm build");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(first.record().fingerprint(), rebuilt.record().fingerprint());
    assert_eq!(
        first.record().input_digest(),
        rebuilt.record().input_digest(),
        "declared environment changes the build fingerprint, not source content",
    );

    let restored = build_wasm_canisters_cached(&spec)
        .expect("original content-addressed artifact should remain reusable");
    assert!(matches!(restored, WasmBuildOutcome::Reused(_)));
    assert!(wasm_path.is_file());

    let wasm = read_wasm(&target_dir, PERF_PROBE_PACKAGE, "debug");
    let fixture =
        StandaloneCanisterFixture::install(PocketIc::new(), InstallSpec::new(wasm, vec![], 0));
    let result: u64 = fixture
        .update_candid("benchmark_once", ())
        .expect("benchmark_once update call");

    assert_eq!(result, 1_498_500);

    let logs = fixture
        .pocket_ic()
        .fetch_canister_logs(fixture.canister_id(), Principal::anonymous())
        .expect("fetch perf probe logs");
    let log_text = logs
        .iter()
        .map(|record| String::from_utf8_lossy(&record.content))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = parse_benchmark_events_from_source(
        &log_text,
        &BenchmarkParserConfig::default(),
        BenchmarkEventSource::FetchedLog,
    );
    let spans = pair_benchmark_spans(&parsed.events);

    assert!(
        parsed.malformed_markers.is_empty(),
        "unexpected malformed markers: {:?}",
        parsed.malformed_markers
    );
    assert!(
        spans.invalid_spans.is_empty(),
        "unexpected invalid spans: {:?}",
        spans.invalid_spans
    );
    assert!(
        spans.unpaired_markers.is_empty(),
        "unexpected unpaired markers: {:?}",
        spans.unpaired_markers
    );
    assert!(
        spans
            .spans
            .iter()
            .any(|span| span.span_label == "probe/benchmark_once" && span.delta.instructions > 0),
        "expected a positive instruction delta in spans: {:?}",
        spans.spans
    );

    fs::remove_dir_all(target_dir).expect("clean temp target dir");
}

#[test]
fn exact_wasm_cache_coordinates_overlapping_builds() {
    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    if !workspace
        .join("canisters/test/perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping exact Wasm cache test: fixture canister is not packaged");
        return;
    }
    let target_dir = unique_temp_dir("ic-testkit-overlapping-wasm-cache-target");
    let spec = WasmBuildSpec::new(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], "debug");
    let start = Arc::new(Barrier::new(3));

    let workers = std::array::from_fn::<_, 2, _>(|_| {
        let worker_spec = spec.clone();
        let worker_start = Arc::clone(&start);
        thread::spawn(move || {
            worker_start.wait();
            build_wasm_canisters_cached(&worker_spec)
                .expect("coordinated exact Wasm build should succeed")
        })
    });
    start.wait();

    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("Wasm build worker should not panic"))
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, WasmBuildOutcome::Built(_)))
            .count(),
        1,
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, WasmBuildOutcome::Reused(_)))
            .count(),
        1,
    );
    assert_eq!(
        outcomes[0].record().fingerprint(),
        outcomes[1].record().fingerprint(),
    );

    fs::remove_dir_all(target_dir).expect("clean overlapping cache target dir");
}

#[test]
fn shared_incremental_wasm_cache_keeps_mutable_cargo_state_outside_exact_entries() {
    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    if !workspace
        .join("canisters/test/perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping shared-incremental Wasm cache test: fixture is not packaged");
        return;
    }
    let target_dir = unique_temp_dir("ic-testkit-shared-incremental-cache");
    let shared_target = unique_temp_dir("ic-testkit-shared-incremental-cargo-target");
    let marker = shared_target.join("caller-owned-marker");
    fs::write(&marker, b"preserve").expect("write caller-owned shared-target marker");
    let spec = WasmBuildSpec::new(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], "debug")
        .with_shared_incremental_target(&shared_target);

    let resolved = resolve_cargo_build_inputs(&spec).expect("resolve exact Cargo inputs");
    assert!(!resolved.inputs().is_empty());
    assert!(resolved.is_current(&spec).expect("revalidate Cargo inputs"));

    let first = build_wasm_canisters_cached(&spec).expect("build through shared Cargo target");
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(
        first
            .record()
            .timings()
            .shared_incremental_lock_wait()
            .is_some()
    );
    assert!(shared_target.join("wasm32-unknown-unknown").is_dir());
    assert!(marker.is_file());
    let inspection = inspect_shared_incremental_target(&spec)
        .expect("inspect shared Cargo target")
        .expect("built shared Cargo target should exist");
    assert_eq!(
        inspection.target_dir(),
        shared_target.canonicalize().unwrap()
    );
    assert!(inspection.logical_size_bytes() > 0);
    let last_used = inspection.last_used();

    let cache_entry = target_dir
        .join(".ic-testkit/wasm-targets")
        .join(first.record().fingerprint().to_hex());
    assert!(cache_entry.is_dir());
    assert!(
        !cache_entry
            .join("wasm32-unknown-unknown/debug/incremental")
            .exists()
    );

    let reused = build_wasm_canisters_cached(&spec).expect("reuse exact shared-mode output");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    assert!(
        reused
            .record()
            .timings()
            .shared_incremental_lock_wait()
            .is_none(),
        "an immediate exact hit should not acquire the shared-target lock"
    );
    assert_eq!(
        inspect_shared_incremental_target(&spec)
            .unwrap()
            .unwrap()
            .last_used(),
        last_used,
        "an exact hit must not touch caller-owned shared Cargo state"
    );

    let changed = spec
        .clone()
        .with_extra_env(&[("IC_TESTKIT_SHARED_INCREMENTAL_PROBE", "changed")]);
    let rebuilt = build_wasm_canisters_cached(&changed).expect("build changed exact identity");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(first.record().fingerprint(), rebuilt.record().fingerprint());
    assert!(marker.is_file());

    let restored = build_wasm_canisters_cached(&spec).expect("restore original exact output");
    assert!(matches!(restored, WasmBuildOutcome::Reused(_)));
    assert!(restored.record().timings().cargo_build().is_none());

    prune_wasm_build_cache(
        &target_dir,
        WasmBuildCachePrunePolicy::new().with_max_size_bytes(0),
    )
    .expect("prune exact output entries");
    assert!(marker.is_file(), "retention must not own the shared target");

    fs::remove_dir_all(target_dir).expect("clean shared-mode exact cache");
    fs::remove_dir_all(shared_target).expect("clean shared Cargo target");
}

#[test]
fn resolved_cargo_inputs_guard_transactional_artifacts_through_commit() {
    let root = unique_temp_dir("ic-testkit-cargo-input-artifact-bridge");
    let workspace = root.join("workspace");
    let package = workspace.join("package");
    fs::create_dir_all(package.join("src")).expect("create Cargo bridge fixture");
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"package\"]\nresolver = \"2\"\n",
    )
    .expect("write bridge workspace manifest");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"artifact_bridge_probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write bridge package manifest");
    let source = package.join("src/lib.rs");
    fs::write(&source, "pub fn value() -> u8 { 1 }\n").expect("write bridge source");
    let wasm_spec = WasmBuildSpec::new(
        &workspace,
        &workspace.join("target/exact-wasm"),
        &["artifact_bridge_probe"],
        "debug",
    );
    let resolved = resolve_cargo_build_inputs(&wasm_spec).expect("resolve bridge Cargo inputs");
    let destination = workspace.join("target/public/transformed.wasm");
    let mismatched_build_spec = wasm_spec
        .clone()
        .with_extra_env(&[("IC_TESTKIT_BRIDGE_MODE", "changed")]);
    let mismatched = ArtifactCacheSpec::new(
        &workspace.join("target/artifact-cache"),
        "cargo-bridge",
        "pipeline/v1",
    )
    .with_cargo_build_inputs("wasm-source", &mismatched_build_spec, &resolved)
    .with_output("transformed.wasm", &destination);
    assert!(matches!(
        prepare_artifact_cache(&mismatched),
        Err(ArtifactCacheError::CargoBuildInputsChanged { .. })
    ));

    let artifact_spec = ArtifactCacheSpec::new(
        &workspace.join("target/artifact-cache"),
        "cargo-bridge",
        "pipeline/v1",
    )
    .with_cargo_build_inputs("wasm-source", &wasm_spec, &resolved)
    .with_output("transformed.wasm", &destination);
    let transaction = match prepare_artifact_cache(&artifact_spec).expect("prepare bridge miss") {
        ArtifactCachePreparation::Build(transaction) => transaction,
        ArtifactCachePreparation::Reused(_) => panic!("new bridge fixture must miss"),
    };
    fs::write(
        transaction.output_path("transformed.wasm").unwrap(),
        b"transformed",
    )
    .expect("write transformed output");
    fs::write(&source, "pub fn value() -> u8 { 2 }\n").expect("change exact Cargo input");

    assert!(matches!(
        transaction.commit(),
        Err(ArtifactCacheError::CargoBuildInputsChanged { .. })
    ));
    assert!(!destination.exists());

    let refreshed = resolve_cargo_build_inputs(&wasm_spec).expect("refresh bridge Cargo inputs");
    let refreshed_spec = ArtifactCacheSpec::new(
        &workspace.join("target/artifact-cache"),
        "cargo-bridge",
        "pipeline/v1",
    )
    .with_cargo_build_inputs("wasm-source", &wasm_spec, &refreshed)
    .with_output("transformed.wasm", &destination);
    let transaction = match prepare_artifact_cache(&refreshed_spec).expect("prepare refreshed miss")
    {
        ArtifactCachePreparation::Build(transaction) => transaction,
        ArtifactCachePreparation::Reused(_) => panic!("changed Cargo inputs must select a miss"),
    };
    fs::write(
        transaction.output_path("transformed.wasm").unwrap(),
        b"transformed-v2",
    )
    .expect("write refreshed transformed output");
    transaction.commit().expect("commit refreshed artifact");
    assert!(matches!(
        prepare_artifact_cache(&refreshed_spec).expect("reuse refreshed artifact"),
        ArtifactCachePreparation::Reused(_)
    ));
    fs::remove_dir_all(root).expect("remove Cargo bridge fixture");
}

#[test]
fn failed_shared_incremental_build_preserves_cargo_state_without_publishing_an_entry() {
    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    if !workspace
        .join("canisters/test/perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping failed shared-incremental test: fixture is not packaged");
        return;
    }
    let target_dir = unique_temp_dir("ic-testkit-failed-shared-cache");
    let shared_target = unique_temp_dir("ic-testkit-failed-shared-cargo-target");
    let marker = shared_target.join("caller-owned-marker");
    fs::write(&marker, b"preserve").expect("write failed-build marker");
    let spec = WasmBuildSpec::new(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], "debug")
        .with_shared_incremental_target(&shared_target)
        .with_cargo_profile_args(&["--definitely-not-a-cargo-build-option"]);
    let fingerprint = resolve_cargo_build_inputs(&spec)
        .expect("resolve failing build identity")
        .fingerprint();

    let error = build_wasm_canisters_cached(&spec).expect_err("invalid Cargo option must fail");

    assert!(error.to_string().contains("cargo build failed"));
    assert!(marker.is_file());
    assert!(shared_target.is_dir());
    assert!(
        !target_dir
            .join(".ic-testkit/wasm-targets")
            .join(fingerprint.to_hex())
            .exists(),
        "failed build must not publish an exact entry"
    );

    fs::remove_dir_all(target_dir).expect("clean failed shared-mode exact cache");
    fs::remove_dir_all(shared_target).expect("clean failed shared Cargo target");
}

#[test]
#[cfg(unix)]
fn source_changes_during_shared_incremental_build_reject_exact_publication() {
    let root = unique_temp_dir("ic-testkit-shared-input-race");
    let workspace = root.join("workspace");
    let package = workspace.join("race_probe");
    fs::create_dir_all(package.join("src")).expect("create race fixture source directory");
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"race_probe\"]\nresolver = \"2\"\n",
    )
    .expect("write race workspace manifest");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"race_probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [lib]\ncrate-type = [\"cdylib\"]\n",
    )
    .expect("write race package manifest");
    let source = package.join("src/lib.rs");
    fs::write(
        &source,
        "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u32 { 1 }\n",
    )
    .expect("write original race source");
    let wrapper = root.join("cargo-wrapper.sh");
    let started = root.join("build-started");
    let release = root.join("release-build");
    write_blocking_cargo_wrapper(&wrapper);
    let real_cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join("exact-cache"),
        &["race_probe"],
        "debug",
    )
    .with_shared_incremental_target(root.join("shared-target"))
    .with_cargo_program(&wrapper)
    .with_extra_env_os([
        (OsString::from("REAL_CARGO"), real_cargo),
        (
            OsString::from("IC_TESTKIT_BUILD_STARTED"),
            started.clone().into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_RELEASE_BUILD"),
            release.clone().into_os_string(),
        ),
    ]);
    let fingerprint = resolve_cargo_build_inputs(&spec)
        .expect("resolve original race fingerprint")
        .fingerprint();
    let worker = thread::spawn(move || build_wasm_canisters_cached(&spec));
    wait_for_test_path(&started);
    fs::write(
        &source,
        "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u32 { 2 }\n",
    )
    .expect("change source during blocked Cargo build");
    fs::write(&release, b"go").expect("release blocked Cargo build");

    let error = worker
        .join()
        .expect("shared input-race worker must not panic")
        .expect_err("changed source must reject exact publication");

    assert!(matches!(
        error,
        WasmBuildError::InputsChangedDuringBuild { .. }
    ));
    assert!(root.join("shared-target").is_dir());
    assert!(
        !root
            .join("exact-cache/.ic-testkit/wasm-targets")
            .join(fingerprint.to_hex())
            .exists()
    );
    fs::remove_dir_all(root).expect("clean shared input-race fixture");
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale temp dir");
    }
    fs::create_dir_all(&root).expect("create temp dir");
    root
}

#[cfg(unix)]
fn write_blocking_cargo_wrapper(path: &std::path::Path) {
    fs::write(
        path,
        b"#!/bin/sh\n\
if [ \"$1\" = \"build\" ]; then\n\
  : > \"$IC_TESTKIT_BUILD_STARTED\"\n\
  while [ ! -f \"$IC_TESTKIT_RELEASE_BUILD\" ]; do sleep 0.05; done\n\
fi\n\
exec \"$REAL_CARGO\" \"$@\"\n",
    )
    .expect("write blocking Cargo wrapper");
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make blocking Cargo wrapper executable");
}

#[cfg(unix)]
fn wait_for_test_path(path: &std::path::Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !path.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn assert_cache_observability(outcome: &WasmBuildOutcome) {
    assert!(matches!(
        outcome.record().maintenance(),
        Some(WasmBuildCacheMaintenance::Pruned(_))
    ));
    let timings = outcome.record().timings();
    let input = timings.input_resolution_detail();
    assert_eq!(timings.input_resolution(), input.total());
    assert!(input.total() >= input.tool_identity());
    assert!(input.total() >= input.cargo_metadata());
    assert!(input.total() >= input.input_discovery());
    assert!(input.total() >= input.content_hashing());
    assert!(timings.cache_maintenance().is_some());
}
