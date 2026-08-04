use candid::Principal;
use ic_testkit::pic::{
    ControllerSnapshotError, PocketIc, PocketIcSnapshotExt, SnapshotRestoreFunding,
};

#[test]
fn failed_snapshot_set_capture_cleans_up_earlier_snapshots() {
    let pocket_ic = PocketIc::new();
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

#[test]
fn snapshot_restore_only_adds_cycles_when_explicitly_requested() {
    let pocket_ic = PocketIc::new();
    let canister_id = pocket_ic.create_canister();
    pocket_ic.install_canister(canister_id, b"\0asm\x01\0\0\0".to_vec(), vec![], None);
    pocket_ic
        .stop_canister(canister_id, None)
        .expect("stop canister before snapshot capture");

    let snapshots = pocket_ic
        .capture_controller_snapshots(Principal::anonymous(), [canister_id])
        .expect("capture snapshot set");
    let balance_before_restore = pocket_ic.cycle_balance(canister_id);

    pocket_ic
        .restore_controller_snapshots(Principal::anonymous(), &snapshots)
        .expect("restore without funding");
    let balance_after_preserved_restore = pocket_ic.cycle_balance(canister_id);
    assert!(
        balance_after_preserved_restore <= balance_before_restore,
        "default restore must not add cycles"
    );

    let minimum_cycles = balance_after_preserved_restore + 10_000_000_000_000;
    pocket_ic
        .restore_controller_snapshots_with_funding(
            Principal::anonymous(),
            &snapshots,
            SnapshotRestoreFunding::TopUpTo { minimum_cycles },
        )
        .expect("restore with explicit funding");
    assert!(
        pocket_ic.cycle_balance(canister_id) > balance_after_preserved_restore,
        "explicit top-up should increase the post-restore balance"
    );
}
