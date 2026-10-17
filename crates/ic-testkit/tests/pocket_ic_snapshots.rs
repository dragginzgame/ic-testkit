mod support;

use candid::Principal;
use ic_testkit::pic::{
    CanisterSnapshotTarget, ControllerSnapshotError, PocketIcCallFailure, PocketIcSnapshotExt,
    SnapshotRestoreFunding,
};
use pocket_ic::CanisterSettings;

#[test]
fn explicit_snapshot_senders_support_mixed_controller_sets_without_fallback() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let explicit_controller = Principal::from_slice(&[42]);
    let anonymous_canister = pocket_ic.create_canister();
    let explicit_canister = pocket_ic.create_canister_with_settings(
        None,
        Some(CanisterSettings {
            controllers: Some(vec![explicit_controller]),
            ..CanisterSettings::default()
        }),
    );
    for (canister_id, sender) in [
        (anonymous_canister, None),
        (explicit_canister, Some(explicit_controller)),
    ] {
        pocket_ic.install_canister(canister_id, b"\0asm\x01\0\0\0".to_vec(), vec![], sender);
        pocket_ic
            .stop_canister(canister_id, sender)
            .expect("stop mixed-controller canister before snapshot capture");
    }

    let error = pocket_ic
        .capture_snapshots([CanisterSnapshotTarget::new(explicit_canister, None)])
        .expect_err("incorrect explicit sender must fail without fallback");
    match error {
        ControllerSnapshotError::CaptureFailed {
            canister_id,
            sender,
            source,
            cleanup_failures,
        } => {
            assert_eq!(canister_id, explicit_canister);
            assert_eq!(sender, None);
            let PocketIcCallFailure::Rejected(response) = *source else {
                panic!("expected a complete rejection");
            };
            assert_ne!(response.reject_message, "");
            assert_eq!(cleanup_failures.len(), 0);
        }
        other => panic!("unexpected snapshot error: {other}"),
    }

    let snapshots = pocket_ic
        .capture_snapshots([
            CanisterSnapshotTarget::new(explicit_canister, Some(explicit_controller)),
            CanisterSnapshotTarget::new(anonymous_canister, None),
        ])
        .expect("capture mixed-controller snapshots with exact senders");

    assert_eq!(snapshots.len(), 2);
    let mut expected = vec![anonymous_canister, explicit_canister];
    expected.sort();
    assert_eq!(snapshots.canister_ids().collect::<Vec<_>>(), expected);
    pocket_ic
        .restore_snapshots(&snapshots, SnapshotRestoreFunding::Preserve)
        .expect("restore mixed-controller snapshots with exact captured senders");

    pocket_ic
        .set_controllers(
            explicit_canister,
            Some(explicit_controller),
            vec![Principal::anonymous()],
        )
        .expect("replace the captured controller with anonymous authority");
    let error = pocket_ic
        .restore_snapshots(&snapshots, SnapshotRestoreFunding::Preserve)
        .expect_err("restore must retain the captured sender after controller changes");
    match error {
        ControllerSnapshotError::RestoreFailed {
            canister_id,
            sender,
            source,
        } => {
            assert_eq!(canister_id, explicit_canister);
            assert_eq!(sender, Some(explicit_controller));
            let PocketIcCallFailure::Rejected(response) = *source else {
                panic!("expected a complete rejection");
            };
            assert_ne!(response.reject_message, "");
        }
        other => panic!("unexpected restore failure: {other}"),
    }
}

#[test]
fn duplicate_snapshot_inputs_are_rejected_before_any_capture() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let canister_id = pocket_ic.create_canister();
    pocket_ic.install_canister(canister_id, b"\0asm\x01\0\0\0".to_vec(), vec![], None);
    pocket_ic.stop_canister(canister_id, None).unwrap();
    let duplicate = Principal::from_slice(&[42]);

    let error = pocket_ic
        .capture_snapshots([
            CanisterSnapshotTarget::new(canister_id, None),
            CanisterSnapshotTarget::new(duplicate, None),
            CanisterSnapshotTarget::new(duplicate, Some(Principal::anonymous())),
        ])
        .unwrap_err();
    assert!(
        matches!(error, ControllerSnapshotError::DuplicateCanisterId { canister_id } if canister_id == duplicate)
    );
    assert!(
        pocket_ic
            .list_canister_snapshots(canister_id, None)
            .unwrap()
            .is_empty(),
        "duplicate validation must precede all snapshot creation",
    );
}

#[test]
fn snapshot_capture_reports_first_failure_in_canister_order() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let mut missing = [pocket_ic.create_canister(), pocket_ic.create_canister()];
    for canister_id in missing {
        pocket_ic.stop_canister(canister_id, None).unwrap();
        pocket_ic.delete_canister(canister_id, None).unwrap();
    }
    missing.sort();
    let reversed = [missing[1], missing[0]];
    let error = pocket_ic
        .capture_snapshots(reversed.map(|id| CanisterSnapshotTarget::new(id, None)))
        .unwrap_err();
    match error {
        ControllerSnapshotError::CaptureFailed { canister_id, .. } => {
            assert_eq!(canister_id, missing[0]);
        }
        other => panic!("unexpected snapshot error: {other}"),
    }
}

#[test]
fn failed_snapshot_set_capture_cleans_up_earlier_snapshots() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
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
        .capture_snapshots(
            [captured_first, missing_canister].map(|id| CanisterSnapshotTarget::new(id, None)),
        )
        .expect_err("missing canister should fail the snapshot set");

    match error {
        ControllerSnapshotError::CaptureFailed {
            canister_id,
            sender,
            source,
            cleanup_failures,
        } => {
            assert_eq!(canister_id, missing_canister);
            assert_eq!(sender, None);
            let PocketIcCallFailure::Rejected(response) = *source else {
                panic!("expected a complete rejection");
            };
            assert_ne!(response.reject_message, "");
            assert_eq!(cleanup_failures.len(), 0);
        }
        other => panic!("unexpected snapshot error: {other}"),
    }

    let remaining = pocket_ic
        .list_canister_snapshots(captured_first, None)
        .expect("list snapshots after rollback");
    assert_eq!(remaining, [] as [pocket_ic::Snapshot; 0]);
}

#[test]
fn snapshot_restore_only_adds_cycles_when_explicitly_requested() {
    let owned = support::pocket_ic();
    let pocket_ic = &owned.instance;
    let canister_id = pocket_ic.create_canister();
    pocket_ic.install_canister(canister_id, b"\0asm\x01\0\0\0".to_vec(), vec![], None);
    pocket_ic
        .stop_canister(canister_id, None)
        .expect("stop canister before snapshot capture");

    let snapshots = pocket_ic
        .capture_snapshots([CanisterSnapshotTarget::new(canister_id, None)])
        .expect("capture snapshot set");
    let balance_before_restore = pocket_ic.cycle_balance(canister_id);

    pocket_ic
        .restore_snapshots(&snapshots, SnapshotRestoreFunding::Preserve)
        .expect("restore without funding");
    let balance_after_preserved_restore = pocket_ic.cycle_balance(canister_id);
    assert!(
        balance_after_preserved_restore <= balance_before_restore,
        "preserve funding must not add cycles"
    );

    let minimum_cycles = balance_after_preserved_restore + 10_000_000_000_000;
    pocket_ic
        .restore_snapshots(
            &snapshots,
            SnapshotRestoreFunding::TopUpTo { minimum_cycles },
        )
        .expect("restore with explicit funding");
    assert!(
        pocket_ic.cycle_balance(canister_id) > balance_after_preserved_restore,
        "explicit top-up should increase the post-restore balance"
    );
}
