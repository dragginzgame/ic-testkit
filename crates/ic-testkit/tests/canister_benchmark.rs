mod support;
use candid::Principal;
use ic_build::{
    artifacts::{WasmBuildSpec, build_wasm_canisters_cached},
    cargo::observation::workspace_root_for,
};
use ic_host_fs::read::read_file;
use ic_host_process::tool::{OutputLimit, OutputLimits};
use ic_testkit::{
    benchmark::{
        BenchmarkEventSource, BenchmarkParserConfig, pair_benchmark_spans,
        parse_benchmark_events_from_source,
    },
    pic::{CandidCallExt, InstallSpec, StandaloneCanisterFixture},
};
use std::process::Command;
use support::unique_temp_directory as unique_temp_dir;
const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";

#[test]
fn perf_probe_canister_emits_parseable_benchmark_markers() {
    let workspace = workspace_root_for(
        env!("CARGO_MANIFEST_DIR"),
        OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        },
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into())),
    )
    .expect("resolve Cargo workspace");
    if !workspace
        .join("crates/ic_testkit_perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping perf probe canister test: fixture canister is not packaged");
        return;
    }
    let target_dir_directory = unique_temp_dir("ic-testkit-perf-probe-target");
    let target_dir = target_dir_directory.path();

    let spec = WasmBuildSpec::new(&workspace, target_dir, &[PERF_PROBE_PACKAGE])
        .with_cargo_profile_args(["--locked"]);
    let outcome = build_wasm_canisters_cached(&spec).expect("build perf probe Wasm");
    let wasm_path = outcome
        .record()
        .artifacts()
        .first()
        .expect("perf probe build returns its retained Wasm");
    let wasm = read_file(wasm_path, 128 * 1024 * 1024).expect("read retained bounded Wasm");
    let fixture = StandaloneCanisterFixture::try_install(
        support::pocket_ic(),
        InstallSpec::new(wasm, vec![], 0),
    )
    .expect("install standalone fixture");
    let result: u64 = fixture
        .pocket_ic()
        .update_candid(
            fixture.canister_id(),
            "benchmark_once",
            (),
            support::REPLY_LIMITS,
        )
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

    drop(fixture);
    drop(outcome);
}
