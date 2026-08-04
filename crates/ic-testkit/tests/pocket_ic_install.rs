use ic_testkit::pic::{CanisterInstallExt, InstallSpec, PocketIc, StandaloneCanisterFixture};

#[test]
fn fallible_install_preserves_the_original_failure_after_diagnostics() {
    let pocket_ic = PocketIc::new();
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

#[test]
fn failed_standalone_install_returns_the_caller_instance() {
    let Err(error) = StandaloneCanisterFixture::try_install(
        PocketIc::new(),
        InstallSpec::new(vec![0xde, 0xad], vec![], 0).label("invalid-standalone-wasm"),
    ) else {
        panic!("invalid Wasm should fail standalone installation");
    };

    let canister_id = error.install_error().canister_id();
    assert_eq!(
        error.install_error().label(),
        Some("invalid-standalone-wasm")
    );
    error
        .pocket_ic()
        .canister_status(canister_id, None)
        .expect("caller-created instance should remain inspectable after install failure");
}
