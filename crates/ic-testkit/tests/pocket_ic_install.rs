use ic_testkit::pic::{CanisterInstallExt, InstallSpec, pic};

#[test]
fn fallible_install_preserves_the_original_failure_after_diagnostics() {
    let pocket_ic = pic();
    let error = pocket_ic
        .try_create_and_install(InstallSpec::new(vec![0xde, 0xad], vec![], 0).label("invalid-wasm"))
        .expect_err("invalid Wasm should fail installation");

    assert_eq!(error.label(), Some("invalid-wasm"));
    assert!(
        error.message().contains("CanisterInvalidWasm"),
        "original PocketIC rejection should be preserved: {}",
        error.message()
    );
    pocket_ic
        .canister_status(error.canister_id(), None)
        .expect("the failed install's canister should remain inspectable");
}
