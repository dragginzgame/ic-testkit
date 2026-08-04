use std::{
    collections::{BTreeMap, BTreeSet},
    panic::{AssertUnwindSafe, catch_unwind},
};

use candid::Principal;
use pocket_ic::{PocketIc, RejectResponse};

use super::transport;

const SNAPSHOT_RESTORE_MIN_CYCLES: u128 = 200_000_000_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ControllerSnapshot {
    snapshot_id: Vec<u8>,
    sender: Option<Principal>,
}

/// Deterministically ordered snapshots captured with one controller policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerSnapshots(BTreeMap<Principal, ControllerSnapshot>);

/// One rejected sender attempt for a snapshot operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotAttemptFailure {
    sender: Option<Principal>,
    response: RejectResponse,
}

/// Failure to remove a snapshot while rolling back a partial capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotCleanupFailure {
    canister_id: Principal,
    sender: Option<Principal>,
    response: Option<Box<RejectResponse>>,
    panic_message: Option<String>,
}

/// Structured controller-snapshot failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerSnapshotError {
    DuplicateCanisterId {
        canister_id: Principal,
    },
    CaptureFailed {
        canister_id: Principal,
        attempts: Vec<SnapshotAttemptFailure>,
        cleanup_failures: Vec<SnapshotCleanupFailure>,
    },
    CapturePanicked {
        canister_id: Principal,
        message: String,
        cleanup_failures: Vec<SnapshotCleanupFailure>,
    },
    RestoreFailed {
        canister_id: Principal,
        attempts: Vec<SnapshotAttemptFailure>,
    },
    RestorePanicked {
        canister_id: Principal,
        message: String,
    },
}

enum SnapshotCaptureFailure {
    Rejected(Vec<SnapshotAttemptFailure>),
    Panicked(String),
}

/// Controller-aware capture and restore of related canister snapshots.
pub trait PocketIcSnapshotExt {
    /// Capture one restorable snapshot per unique canister.
    ///
    /// Input is validated before capture begins. If a later capture fails,
    /// snapshots already captured by this operation are deleted before the
    /// structured error is returned.
    fn capture_controller_snapshots<I>(
        &self,
        controller_id: Principal,
        canister_ids: I,
    ) -> Result<ControllerSnapshots, ControllerSnapshotError>
    where
        I: IntoIterator<Item = Principal>;

    /// Restore a previously captured snapshot set using the same controller.
    fn restore_controller_snapshots(
        &self,
        controller_id: Principal,
        snapshots: &ControllerSnapshots,
    ) -> Result<(), ControllerSnapshotError>;
}

impl PocketIcSnapshotExt for PocketIc {
    fn capture_controller_snapshots<I>(
        &self,
        controller_id: Principal,
        canister_ids: I,
    ) -> Result<ControllerSnapshots, ControllerSnapshotError>
    where
        I: IntoIterator<Item = Principal>,
    {
        let canister_ids = ordered_unique_canister_ids(canister_ids)?;
        let mut snapshots = BTreeMap::new();

        for canister_id in canister_ids {
            match try_take_controller_snapshot(self, controller_id, canister_id) {
                Ok(snapshot) => {
                    snapshots.insert(canister_id, snapshot);
                }
                Err(SnapshotCaptureFailure::Rejected(attempts)) => {
                    let cleanup_failures = cleanup_captured_snapshots(self, &snapshots);
                    return Err(ControllerSnapshotError::CaptureFailed {
                        canister_id,
                        attempts,
                        cleanup_failures,
                    });
                }
                Err(SnapshotCaptureFailure::Panicked(message)) => {
                    let cleanup_failures = cleanup_captured_snapshots(self, &snapshots);
                    return Err(ControllerSnapshotError::CapturePanicked {
                        canister_id,
                        message,
                        cleanup_failures,
                    });
                }
            }
        }

        Ok(ControllerSnapshots(snapshots))
    }

    fn restore_controller_snapshots(
        &self,
        controller_id: Principal,
        snapshots: &ControllerSnapshots,
    ) -> Result<(), ControllerSnapshotError> {
        for (canister_id, snapshot_id, sender) in snapshots.iter() {
            restore_controller_snapshot(self, controller_id, canister_id, sender, snapshot_id)?;
        }
        Ok(())
    }
}

impl ControllerSnapshots {
    /// Return the number of captured canisters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Report whether the set contains no snapshots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over captured canister ids in deterministic principal order.
    pub fn canister_ids(&self) -> impl Iterator<Item = Principal> + '_ {
        self.0.keys().copied()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (Principal, &[u8], Option<Principal>)> + '_ {
        self.0.iter().map(|(canister_id, snapshot)| {
            (
                *canister_id,
                snapshot.snapshot_id.as_slice(),
                snapshot.sender,
            )
        })
    }
}

impl SnapshotAttemptFailure {
    /// Read the sender used for this rejected attempt.
    #[must_use]
    pub const fn sender(&self) -> Option<Principal> {
        self.sender
    }

    /// Read PocketIC's structured rejection.
    #[must_use]
    pub const fn response(&self) -> &RejectResponse {
        &self.response
    }
}

impl SnapshotCleanupFailure {
    /// Read the canister whose captured snapshot could not be removed.
    #[must_use]
    pub const fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Read the sender used for the rejected cleanup.
    #[must_use]
    pub const fn sender(&self) -> Option<Principal> {
        self.sender
    }

    /// Read PocketIC's structured rejection.
    #[must_use]
    pub fn response(&self) -> Option<&RejectResponse> {
        self.response.as_deref()
    }

    /// Read a captured PocketIC panic message, when cleanup did not return a rejection.
    #[must_use]
    pub fn panic_message(&self) -> Option<&str> {
        self.panic_message.as_deref()
    }
}

impl std::fmt::Display for ControllerSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCanisterId { canister_id } => {
                write!(f, "duplicate canister id in snapshot set: {canister_id}")
            }
            Self::CaptureFailed {
                canister_id,
                attempts,
                cleanup_failures,
            } => write!(
                f,
                "failed to capture snapshot for {canister_id} after {} sender attempts; {} partial snapshots could not be cleaned up",
                attempts.len(),
                cleanup_failures.len()
            ),
            Self::CapturePanicked {
                canister_id,
                message,
                cleanup_failures,
            } => write!(
                f,
                "snapshot capture panicked for {canister_id}: {message}; {} partial snapshots could not be cleaned up",
                cleanup_failures.len()
            ),
            Self::RestoreFailed {
                canister_id,
                attempts,
            } => write!(
                f,
                "failed to restore snapshot for {canister_id} after {} sender attempts",
                attempts.len()
            ),
            Self::RestorePanicked {
                canister_id,
                message,
            } => write!(f, "snapshot restore panicked for {canister_id}: {message}"),
        }
    }
}

impl std::error::Error for ControllerSnapshotError {}

fn ordered_unique_canister_ids<I>(
    canister_ids: I,
) -> Result<Vec<Principal>, ControllerSnapshotError>
where
    I: IntoIterator<Item = Principal>,
{
    let mut unique = BTreeSet::new();
    for canister_id in canister_ids {
        if !unique.insert(canister_id) {
            return Err(ControllerSnapshotError::DuplicateCanisterId { canister_id });
        }
    }
    Ok(unique.into_iter().collect())
}

fn try_take_controller_snapshot(
    pocket_ic: &PocketIc,
    controller_id: Principal,
    canister_id: Principal,
) -> Result<ControllerSnapshot, SnapshotCaptureFailure> {
    let candidates = controller_sender_candidates(controller_id, canister_id);
    let mut attempts = Vec::new();

    for sender in candidates {
        let capture = catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.take_canister_snapshot(canister_id, sender, None)
        }));
        match capture {
            Err(payload) => {
                return Err(SnapshotCaptureFailure::Panicked(
                    transport::panic_payload_to_string(payload.as_ref()),
                ));
            }
            Ok(snapshot) => match snapshot {
                Ok(snapshot) => {
                    return Ok(ControllerSnapshot {
                        snapshot_id: snapshot.id,
                        sender,
                    });
                }
                Err(response) => attempts.push(SnapshotAttemptFailure { sender, response }),
            },
        }
    }

    Err(SnapshotCaptureFailure::Rejected(attempts))
}

fn cleanup_captured_snapshots(
    pocket_ic: &PocketIc,
    snapshots: &BTreeMap<Principal, ControllerSnapshot>,
) -> Vec<SnapshotCleanupFailure> {
    let mut failures = Vec::new();
    for (canister_id, snapshot) in snapshots {
        let cleanup = catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.delete_canister_snapshot(
                *canister_id,
                snapshot.sender,
                snapshot.snapshot_id.clone(),
            )
        }));
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(response)) => failures.push(SnapshotCleanupFailure {
                canister_id: *canister_id,
                sender: snapshot.sender,
                response: Some(Box::new(response)),
                panic_message: None,
            }),
            Err(payload) => failures.push(SnapshotCleanupFailure {
                canister_id: *canister_id,
                sender: snapshot.sender,
                response: None,
                panic_message: Some(transport::panic_payload_to_string(payload.as_ref())),
            }),
        }
    }
    failures
}

fn restore_controller_snapshot(
    pocket_ic: &PocketIc,
    controller_id: Principal,
    canister_id: Principal,
    snapshot_sender: Option<Principal>,
    snapshot_id: &[u8],
) -> Result<(), ControllerSnapshotError> {
    let fallback_sender = if snapshot_sender.is_some() {
        None
    } else {
        Some(controller_id)
    };
    let candidates = [snapshot_sender, fallback_sender];
    let mut attempts = Vec::new();

    for sender in candidates {
        let restore = catch_unwind(AssertUnwindSafe(|| {
            ensure_snapshot_restore_cycles(pocket_ic, canister_id);
            pocket_ic.load_canister_snapshot(canister_id, sender, snapshot_id.to_vec())
        }));
        match restore {
            Err(payload) => {
                return Err(ControllerSnapshotError::RestorePanicked {
                    canister_id,
                    message: transport::panic_payload_to_string(payload.as_ref()),
                });
            }
            Ok(Ok(())) => return Ok(()),
            Ok(Err(response)) => attempts.push(SnapshotAttemptFailure { sender, response }),
        }
    }

    Err(ControllerSnapshotError::RestoreFailed {
        canister_id,
        attempts,
    })
}

fn ensure_snapshot_restore_cycles(pocket_ic: &PocketIc, canister_id: Principal) {
    let balance = pocket_ic.cycle_balance(canister_id);
    if balance < SNAPSHOT_RESTORE_MIN_CYCLES {
        let top_up = SNAPSHOT_RESTORE_MIN_CYCLES - balance;
        let _ = pocket_ic.add_cycles(canister_id, top_up);
    }
}

fn controller_sender_candidates(
    controller_id: Principal,
    canister_id: Principal,
) -> [Option<Principal>; 2] {
    if canister_id == controller_id {
        [None, Some(controller_id)]
    } else {
        [Some(controller_id), None]
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::{ControllerSnapshotError, ordered_unique_canister_ids};

    #[test]
    fn duplicate_canister_ids_are_rejected_before_capture() {
        let canister_id = Principal::from_slice(&[1]);
        let error = ordered_unique_canister_ids([canister_id, canister_id]).unwrap_err();

        assert_eq!(
            error,
            ControllerSnapshotError::DuplicateCanisterId { canister_id }
        );
    }

    #[test]
    fn canister_ids_are_sorted_deterministically() {
        let first = Principal::from_slice(&[1]);
        let second = Principal::from_slice(&[2]);

        assert_eq!(
            ordered_unique_canister_ids([second, first]).unwrap(),
            vec![first, second]
        );
    }
}
