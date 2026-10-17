mod support;

use candid::Principal;
use ic_testkit::pic::{
    CandidCallError, CandidCallExt, ErrorCode, RejectCode, query_candid_args, update_candid_args,
};

#[test]
fn candid_call_errors_preserve_live_pocket_ic_rejections() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let removed_canister = pocket_ic.create_canister();
    pocket_ic
        .stop_canister(removed_canister, None)
        .expect("stop canister before deletion");
    pocket_ic
        .delete_canister(removed_canister, None)
        .expect("delete canister used for rejection test");

    let sender = Principal::management_canister();
    for (result, operation, caller) in [
        (
            pocket_ic.query_candid::<(), _>(removed_canister, "missing", (), support::REPLY_LIMITS),
            "query_call",
            Principal::anonymous(),
        ),
        (
            pocket_ic.update_candid::<(), _>(
                removed_canister,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "update_call",
            Principal::anonymous(),
        ),
        (
            pocket_ic.query_candid_as::<(), _>(
                removed_canister,
                sender,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "query_call",
            sender,
        ),
        (
            pocket_ic.update_candid_as::<(), _>(
                removed_canister,
                sender,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "update_call",
            sender,
        ),
        (
            query_candid_args::<(), _>(
                pocket_ic,
                removed_canister,
                sender,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "query_call",
            sender,
        ),
        (
            update_candid_args::<(), _>(
                pocket_ic,
                removed_canister,
                sender,
                "missing",
                (),
                support::REPLY_LIMITS,
            ),
            "update_call",
            sender,
        ),
    ] {
        let error = result.expect_err("a deleted canister must reject the call");
        let CandidCallError::CanisterReject {
            context,
            response: rejection,
        } = error
        else {
            panic!("expected canister rejection: {error}");
        };
        assert_eq!(rejection.reject_code, RejectCode::DestinationInvalid);
        assert_eq!(rejection.error_code, ErrorCode::CanisterNotFound);
        assert_eq!(context.operation, operation);
        assert_eq!(context.canister_id, removed_canister);
        assert_eq!(context.caller, caller);
        assert_eq!(context.method, "missing");
    }
}
