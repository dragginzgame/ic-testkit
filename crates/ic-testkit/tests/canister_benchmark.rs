#[cfg(unix)]
#[path = "support/executable.rs"]
mod executable_support;
mod support;
#[cfg(unix)]
#[path = "support/wait.rs"]
mod wait_support;

use candid::Principal;
use ic_testkit::{
    artifacts::{
        ArtifactCacheError, ArtifactCacheMaintenance, ArtifactCachePreparation,
        ArtifactCachePrunePolicy, ArtifactCacheSpec, SharedIncrementalTargetMaintenanceOutcome,
        SharedIncrementalTargetPrunePolicy, WasmBuildOutcome, WasmBuildProgressConfig,
        WasmBuildProgressEvent, WasmBuildSpec, build_wasm_canisters_cached,
        build_wasm_canisters_cached_batch, build_wasm_canisters_cached_with_progress,
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
    sync::{Arc, Barrier},
    thread,
    time::Duration,
};

use support::unique_temp_directory as unique_temp_dir;

#[cfg(unix)]
use executable_support::write_executable_script;
#[cfg(unix)]
use ic_testkit::artifacts::WasmBuildError;
#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use wait_support::wait_for_path;

const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";

#[test]
fn independent_wasm_batch_preserves_standalone_feature_resolution() {
    let root = unique_temp_dir("ic-testkit-independent-feature-batch");
    let workspace = root.join("workspace");
    for package in ["shared", "feature_a", "feature_b"] {
        fs::create_dir_all(workspace.join(package).join("src"))
            .expect("create feature fixture package");
    }
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"shared\", \"feature_a\", \"feature_b\"]\nresolver = \"2\"\n",
    )
    .expect("write feature fixture workspace");
    fs::write(
        workspace.join("shared/Cargo.toml"),
        "[package]\nname = \"shared\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [features]\na = []\nb = []\n",
    )
    .expect("write shared feature manifest");
    fs::write(
        workspace.join("shared/src/lib.rs"),
        "#[cfg(all(feature = \"a\", feature = \"b\"))]\n\
         compile_error!(\"standalone package features were unified\");\n\
         pub fn value() -> u8 { 1 }\n",
    )
    .expect("write shared feature source");
    for (package, feature) in [("feature_a", "a"), ("feature_b", "b")] {
        fs::write(
            workspace.join(package).join("Cargo.toml"),
            format!(
                "[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
                 [lib]\ncrate-type = [\"cdylib\"]\n\
                 [dependencies]\nshared = {{ path = \"../shared\", features = [\"{feature}\"] }}\n"
            ),
        )
        .expect("write feature canister manifest");
        fs::write(
            workspace.join(package).join("src/lib.rs"),
            "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u8 { shared::value() }\n",
        )
        .expect("write feature canister source");
    }
    let target = root.join("exact-target");
    let specs = [
        WasmBuildSpec::new(&workspace, &target, &["feature_a"], "debug"),
        WasmBuildSpec::new(&workspace, &target, &["feature_b"], "debug"),
    ];

    let batch = build_wasm_canisters_cached_batch(&specs);

    assert!(batch.is_success());
    assert_eq!(batch.outcomes().count(), 2);
    assert!(
        batch
            .outcomes()
            .all(|(_index, outcome)| matches!(outcome, WasmBuildOutcome::Built(_)))
    );
    assert!(wasm_path(&target, "feature_a", "debug").is_file());
    assert!(wasm_path(&target, "feature_b", "debug").is_file());
    assert!(
        batch
            .outcomes()
            .all(|(_index, outcome)| outcome.record().exact_cache_path().is_dir())
    );
    let first_exact_cache = batch
        .outcomes()
        .next()
        .expect("first batch outcome")
        .1
        .record()
        .exact_cache_path()
        .to_owned();
    fs::remove_dir_all(&first_exact_cache).expect("remove exact entry reconstruction fixture");

    let with_invalid_middle = [
        specs[0].clone(),
        WasmBuildSpec::new(&workspace, &target, &[], "debug"),
        specs[1].clone(),
    ];
    let with_failure = build_wasm_canisters_cached_batch(&with_invalid_middle);
    assert!(!with_failure.is_success());
    assert_eq!(with_failure.outcomes().count(), 2);
    assert!(
        with_failure
            .outcomes()
            .all(|(_index, outcome)| outcome.is_reused())
    );
    assert_eq!(
        with_failure
            .failures()
            .map(|(index, _error)| index)
            .collect::<Vec<_>>(),
        [1]
    );
    assert!(
        first_exact_cache.is_dir(),
        "a validated caller-facing hit must recreate its immutable cache directory"
    );
    fs::remove_dir_all(root).expect("remove independent feature fixture");
}

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
            ArtifactCachePrunePolicy::new(),
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
        .with_extra_env([("IC_TESTKIT_CACHE_PROBE", "changed")]);
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
        shared_target
            .canonicalize()
            .expect("canonicalize shared Cargo target")
    );
    assert!(inspection.logical_size_bytes() > 0);
    let last_used = inspection.last_used();

    let cache_entry = first.record().exact_cache_path();
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
            .expect("reinspect shared Cargo target")
            .expect("shared Cargo target must still exist")
            .last_used(),
        last_used,
        "an exact hit must not touch caller-owned shared Cargo state"
    );

    let changed = spec
        .clone()
        .with_extra_env([("IC_TESTKIT_SHARED_INCREMENTAL_PROBE", "changed")]);
    let rebuilt = build_wasm_canisters_cached(&changed).expect("build changed exact identity");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(first.record().fingerprint(), rebuilt.record().fingerprint());
    assert!(marker.is_file());

    let restored = build_wasm_canisters_cached(&spec).expect("restore original exact output");
    assert!(matches!(restored, WasmBuildOutcome::Reused(_)));
    assert!(restored.record().timings().cargo_build().is_none());

    prune_wasm_build_cache(
        &target_dir,
        ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
    )
    .expect("prune exact output entries");
    assert!(marker.is_file(), "retention must not own the shared target");

    fs::remove_dir_all(target_dir).expect("clean shared-mode exact cache");
    fs::remove_dir_all(shared_target).expect("clean shared Cargo target");
}

#[test]
fn scheduled_shared_target_maintenance_participates_in_wasm_acquisition() {
    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    if !workspace
        .join("canisters/test/perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping integrated shared maintenance test: fixture is not packaged");
        return;
    }
    let root = unique_temp_dir("ic-testkit-integrated-shared-maintenance");
    let target_dir = root.join("exact");
    let shared_target = root.join("shared");
    assert!(!shared_target.exists());
    let spec = WasmBuildSpec::new(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], "debug")
        .with_shared_incremental_target(&shared_target)
        .with_shared_incremental_target_maintenance_at_most_every(
            SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(u64::MAX),
            Duration::from_secs(60 * 60),
        );

    let mut first_events = Vec::new();
    let first = build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().without_heartbeats(),
        |event| first_events.push(event),
    )
    .expect("build with integrated shared-target maintenance");
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(shared_target.is_dir());
    assert!(matches!(
        first.record().shared_incremental_maintenance(),
        Some(SharedIncrementalTargetMaintenanceOutcome::Performed { .. })
    ));
    assert!(first.to_string().contains("shared_maintenance=("));
    let maintenance_started = first_events
        .iter()
        .position(|event| {
            matches!(
                event,
                WasmBuildProgressEvent::SharedTargetMaintenanceStarted { .. }
            )
        })
        .expect("integrated maintenance start event");
    let cargo_started = first_events
        .iter()
        .position(|event| matches!(event, WasmBuildProgressEvent::CargoStarted { .. }))
        .expect("integrated Cargo start event");
    assert!(maintenance_started < cargo_started);
    assert_eq!(
        first_events[..cargo_started]
            .iter()
            .filter(|event| matches!(event, WasmBuildProgressEvent::InputsResolved { .. }))
            .count(),
        1,
        "due maintenance must reuse the acquisition's pre-build resolution",
    );
    assert!(first_events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::SharedTargetMaintenanceFinished {
            outcome: SharedIncrementalTargetMaintenanceOutcome::Performed { .. }
        }
    )));

    let mut reused_events = Vec::new();
    let reused = build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().without_heartbeats(),
        |event| reused_events.push(event),
    )
    .expect("reuse with integrated shared-target maintenance");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    let reused_timings = reused.record().timings();
    assert!(reused_timings.shared_incremental_lock_wait().is_some());
    assert!(matches!(
        reused.record().shared_incremental_maintenance(),
        Some(SharedIncrementalTargetMaintenanceOutcome::Skipped { .. })
    ));
    assert_eq!(
        reused_events
            .iter()
            .filter(|event| matches!(event, WasmBuildProgressEvent::InputsResolved { .. }))
            .count(),
        1,
    );
    assert!(reused_events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::SharedTargetMaintenanceFinished {
            outcome: SharedIncrementalTargetMaintenanceOutcome::Skipped { .. }
        }
    )));

    fs::remove_dir_all(&shared_target).expect("remove shared target before exact hit");
    let recreated = build_wasm_canisters_cached(&spec)
        .expect("exact hit must recreate configured shared maintenance target");
    assert!(matches!(recreated, WasmBuildOutcome::Reused(_)));
    assert!(shared_target.is_dir());
    assert!(matches!(
        recreated.record().shared_incremental_maintenance(),
        Some(SharedIncrementalTargetMaintenanceOutcome::Performed { .. })
    ));

    fs::remove_dir_all(root).expect("clean integrated shared maintenance fixture");
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
        .with_extra_env([("IC_TESTKIT_BRIDGE_MODE", "changed")]);
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
        transaction
            .output_path("transformed.wasm")
            .expect("resolve transformed output path"),
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
        transaction
            .output_path("transformed.wasm")
            .expect("resolve refreshed transformed output path"),
        b"transformed-updated",
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
        .with_cargo_profile_args(["--definitely-not-a-cargo-build-option"]);
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
    .with_extra_env([
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
    wait_for_path(&started);
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

#[cfg(unix)]
fn write_blocking_cargo_wrapper(path: &std::path::Path) {
    write_executable_script(
        path,
        b"#!/bin/sh\n\
if [ \"$1\" = \"build\" ]; then\n\
  : > \"$IC_TESTKIT_BUILD_STARTED\"\n\
  while [ ! -f \"$IC_TESTKIT_RELEASE_BUILD\" ]; do sleep 0.05; done\n\
fi\n\
exec \"$REAL_CARGO\" \"$@\"\n",
    );
}

fn assert_cache_observability(outcome: &WasmBuildOutcome) {
    assert!(matches!(
        outcome.record().maintenance(),
        Some(ArtifactCacheMaintenance::Pruned(_))
    ));
    let timings = outcome.record().timings();
    let input = timings.input_resolution();
    assert!(input.total() >= input.tool_identity());
    assert!(input.total() >= input.cargo_metadata());
    assert!(input.total() >= input.input_discovery());
    assert!(input.total() >= input.content_hashing());
    assert!(timings.cache_maintenance().is_some());
}
