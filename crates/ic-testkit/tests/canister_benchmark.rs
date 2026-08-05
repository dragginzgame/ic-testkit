use candid::Principal;
use ic_testkit::{
    artifacts::{
        WasmBuildCacheMaintenance, WasmBuildCachePrunePolicy, WasmBuildOutcome, WasmBuildSpec,
        build_wasm_canisters_cached, read_wasm, wasm_path, workspace_root_for,
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
        .with_prune_policy(WasmBuildCachePrunePolicy::new());
    let first = build_wasm_canisters_cached(&spec).expect("first exact Wasm build");
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(first.record().timings().cargo_build().is_some());
    assert_cache_observability(&first);

    let reused = build_wasm_canisters_cached(&spec).expect("reuse exact Wasm build");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    assert_eq!(first.record().fingerprint(), reused.record().fingerprint());
    assert!(reused.record().timings().cargo_build().is_none());

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

fn unique_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale temp dir");
    }
    fs::create_dir_all(&root).expect("create temp dir");
    root
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
