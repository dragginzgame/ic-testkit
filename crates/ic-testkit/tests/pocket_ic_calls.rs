mod support;

use std::panic::{AssertUnwindSafe, catch_unwind};

use candid::Principal;
use ic_testkit::pic::{CandidCallErrorKind, CandidCallExt, ErrorCode, RejectCode};

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
            pocket_ic.query_candid::<(), _>(removed_canister, "missing", ()),
            "query_call",
            Principal::anonymous(),
        ),
        (
            pocket_ic.update_candid::<(), _>(removed_canister, "missing", ()),
            "update_call",
            Principal::anonymous(),
        ),
        (
            pocket_ic.query_candid_as::<(), _>(removed_canister, sender, "missing", ()),
            "query_call",
            sender,
        ),
        (
            pocket_ic.update_candid_as::<(), _>(removed_canister, sender, "missing", ()),
            "update_call",
            sender,
        ),
    ] {
        let error = result.expect_err("a deleted canister must reject the call");
        let rejection = error.reject_response.as_deref().unwrap();
        assert_eq!(error.kind, CandidCallErrorKind::CanisterReject);
        assert_eq!(rejection.reject_code, RejectCode::DestinationInvalid);
        assert_eq!(rejection.error_code, ErrorCode::CanisterNotFound);
        let context = error.context.as_deref().unwrap();
        assert_eq!(context.operation, operation);
        assert_eq!(context.canister_id, removed_canister);
        assert_eq!(context.caller, caller);
        assert_eq!(context.method, "missing");
    }

    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.query_candid_or_panic::<(), _>(removed_canister, "missing", ());
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.update_candid_or_panic::<(), _>(removed_canister, "missing", ());
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.query_candid_as_or_panic::<(), _>(removed_canister, sender, "missing", ());
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.update_candid_as_or_panic::<(), _>(removed_canister, sender, "missing", ());
        }))
        .is_err()
    );
}
