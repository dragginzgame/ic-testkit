use ic_testkit::pic::{CandidCallErrorKind, CandidCallExt, ErrorCode, PocketIc, RejectCode};

#[test]
fn candid_call_errors_preserve_live_pocket_ic_rejections() {
    let pocket_ic = PocketIc::new();
    let removed_canister = pocket_ic.create_canister();
    pocket_ic
        .stop_canister(removed_canister, None)
        .expect("stop canister before deletion");
    pocket_ic
        .delete_canister(removed_canister, None)
        .expect("delete canister used for rejection test");

    let error = pocket_ic
        .query_candid::<(), _>(removed_canister, "missing", ())
        .expect_err("querying a deleted canister should be rejected");
    let rejection = error
        .reject_response()
        .expect("PocketIC rejection should remain structured");

    assert_eq!(error.kind(), CandidCallErrorKind::CanisterReject);
    assert_eq!(rejection.reject_code, RejectCode::DestinationInvalid);
    assert_eq!(rejection.error_code, ErrorCode::CanisterNotFound);
}
