use candid::Principal;
use ic_testkit::pic::{ControllerSnapshotError, PocketIcSnapshotExt, pic};

#[test]
fn failed_snapshot_set_capture_cleans_up_earlier_snapshots() {
    let pocket_ic = pic();
    let captured_first = pocket_ic.create_canister();
    pocket_ic.install_canister(captured_first, b"\0asm\x01\0\0\0".to_vec(), vec![], None);
    pocket_ic
        .stop_canister(captured_first, None)
        .expect("stop canister before snapshot capture");

    let missing_canister = pocket_ic.create_canister();
    pocket_ic
        .stop_canister(missing_canister, None)
        .expect("stop canister before deletion");
    pocket_ic
        .delete_canister(missing_canister, None)
        .expect("delete canister used for capture failure");
    assert!(captured_first < missing_canister);

    let error = pocket_ic
        .capture_controller_snapshots(Principal::anonymous(), [captured_first, missing_canister])
        .expect_err("missing canister should fail the snapshot set");

    match error {
        ControllerSnapshotError::CaptureFailed {
            canister_id,
            attempts,
            cleanup_failures,
        } => {
            assert_eq!(canister_id, missing_canister, "attempts: {attempts:?}");
            assert!(cleanup_failures.is_empty());
        }
        other => panic!("unexpected snapshot error: {other}"),
    }

    let remaining = pocket_ic
        .list_canister_snapshots(captured_first, None)
        .expect("list snapshots after rollback");
    assert!(remaining.is_empty());
}
