use super::build_wasm_canisters_cached_batch;
use crate::artifacts::WasmBuildSpec;
use std::path::Path;

#[test]
fn empty_independent_batch_succeeds_without_work() {
    let outcome = build_wasm_canisters_cached_batch(&[]).expect("empty batch");
    assert!(outcome.outcomes().is_empty());
}

#[test]
fn invalid_independent_spec_reports_its_batch_index() {
    let specs = [WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &[],
        "debug",
    )];
    let error = build_wasm_canisters_cached_batch(&specs).expect_err("invalid batch entry");
    assert_eq!(error.failed_index(), 0);
    assert!(error.completed().is_empty());
}
