use candid::Principal;
use ic_testkit::{
    artifacts::{build_wasm_canisters, read_wasm, wasm_path},
    benchmark::{
        BenchmarkEventSource, BenchmarkParserConfig, pair_benchmark_spans,
        parse_benchmark_events_from_source,
    },
    pic::install_prebuilt_canister,
};
use std::{fs, path::PathBuf};

const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";

#[test]
fn perf_probe_canister_emits_parseable_benchmark_markers() {
    let target_dir = unique_temp_dir("ic-testkit-perf-probe-target");
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    build_wasm_canisters(&workspace, &target_dir, &[PERF_PROBE_PACKAGE], &[], &[]);
    assert!(wasm_path(&target_dir, PERF_PROBE_PACKAGE, "debug").is_file());

    let wasm = read_wasm(&target_dir, PERF_PROBE_PACKAGE, "debug");
    let fixture = install_prebuilt_canister(wasm, vec![]);
    let result: u64 = fixture
        .pic()
        .update_call(fixture.canister_id(), "benchmark_once", ())
        .expect("benchmark_once update call");

    assert_eq!(result, 1_498_500);

    let logs = fixture
        .pic()
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

fn unique_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale temp dir");
    }
    fs::create_dir_all(&root).expect("create temp dir");
    root
}
