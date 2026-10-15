use std::collections::BTreeMap;

use candid::Principal;
use pocket_ic::{PocketIc, RejectResponse};

use super::{PocketIcOperationError, transport};

#[derive(Clone, Debug, Eq, PartialEq)]
struct ControllerSnapshot {
    snapshot_id: Vec<u8>,
    sender: Option<Principal>,
}

/// Deterministically ordered snapshots retaining each successful capture sender.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerSnapshots(BTreeMap<Principal, ControllerSnapshot>);

/// One canister and exact management-call sender used for snapshot capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanisterSnapshotTarget {
    canister_id: Principal,
    sender: Option<Principal>,
}

/// Failure to remove a snapshot while rolling back a partial capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotCleanupFailure {
    canister_id: Principal,
    sender: Option<Principal>,
    response: Option<Box<RejectResponse>>,
    panic_message: Option<String>,
}

/// Caller-selected cycle funding applied immediately before snapshot restore.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotRestoreFunding {
    /// Do not add cycles before the restore operation.
    Preserve,
    /// Add cycles only when needed to reach the given minimum balance.
    TopUpTo {
        /// Minimum balance established immediately before each restore.
        minimum_cycles: u128,
    },
}

/// Structured controller-snapshot failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerSnapshotError {
    /// Input contained the same canister more than once.
    DuplicateCanisterId {
        /// Repeated canister id.
        canister_id: Principal,
    },
    /// PocketIC rejected the exact sender selected for capture.
    CaptureFailed {
        /// Canister whose capture failed.
        canister_id: Principal,
        /// Exact sender used for the rejected call.
        sender: Option<Principal>,
        /// Complete upstream rejection.
        response: Box<RejectResponse>,
        /// Failures while deleting snapshots captured earlier in the set.
        cleanup_failures: Vec<SnapshotCleanupFailure>,
    },
    /// PocketIC panicked while capturing a snapshot.
    CapturePanicked {
        /// Canister whose capture panicked.
        canister_id: Principal,
        /// Original operation failure and transport classification.
        source: PocketIcOperationError,
        /// Failures while deleting snapshots captured earlier in the set.
        cleanup_failures: Vec<SnapshotCleanupFailure>,
    },
    /// PocketIC rejected the retained capture sender during restore.
    RestoreFailed {
        /// Canister whose restore failed.
        canister_id: Principal,
        /// Exact sender used for the rejected call.
        sender: Option<Principal>,
        /// Complete upstream rejection.
        response: Box<RejectResponse>,
    },
    /// PocketIC panicked while restoring a snapshot.
    RestorePanicked {
        /// Canister whose restore panicked.
        canister_id: Principal,
        /// Original operation failure and transport classification.
        source: PocketIcOperationError,
    },
}

enum SnapshotCaptureFailure {
    Rejected(RejectResponse),
    Panicked(PocketIcOperationError),
}

/// Exact-sender capture and restore of related canister snapshots.
pub trait PocketIcSnapshotExt {
    /// Capture snapshots with one explicit sender per canister.
    ///
    /// Input is duplicate-checked before any effects and captured in deterministic
    /// canister-id order. A later failure rolls back snapshots captured earlier
    /// in the set. No other sender is attempted after a rejection.
    fn capture_snapshots<I>(
        &self,
        targets: I,
    ) -> Result<ControllerSnapshots, ControllerSnapshotError>
    where
        I: IntoIterator<Item = CanisterSnapshotTarget>;

    /// Restore each snapshot with its exact captured sender and selected funding.
    ///
    /// `Preserve` never adds cycles; `TopUpTo` is evaluated before each restore.
    /// PocketIC may charge cycles for the restore itself. Rejection or panic stops
    /// the set without retrying another sender or rewinding earlier restores.
    fn restore_snapshots(
        &self,
        snapshots: &ControllerSnapshots,
        funding: SnapshotRestoreFunding,
    ) -> Result<(), ControllerSnapshotError>;
}

impl PocketIcSnapshotExt for PocketIc {
    fn capture_snapshots<I>(
        &self,
        targets: I,
    ) -> Result<ControllerSnapshots, ControllerSnapshotError>
    where
        I: IntoIterator<Item = CanisterSnapshotTarget>,
    {
        capture_snapshot_set(self, targets)
    }

    fn restore_snapshots(
        &self,
        snapshots: &ControllerSnapshots,
        funding: SnapshotRestoreFunding,
    ) -> Result<(), ControllerSnapshotError> {
        for (canister_id, snapshot_id, sender) in snapshots.iter() {
            restore_controller_snapshot(self, canister_id, snapshot_id, funding, sender)?;
        }
        Ok(())
    }
}

impl CanisterSnapshotTarget {
    /// Select one canister and exact sender for snapshot capture.
    #[must_use]
    pub const fn new(canister_id: Principal, sender: Option<Principal>) -> Self {
        Self {
            canister_id,
            sender,
        }
    }

    /// Canister whose snapshot will be captured.
    #[must_use]
    pub const fn canister_id(self) -> Principal {
        self.canister_id
    }

    /// Exact management-call sender, including `None` for the default sender.
    #[must_use]
    pub const fn sender(self) -> Option<Principal> {
        self.sender
    }
}

fn capture_snapshot_set<I>(
    pocket_ic: &PocketIc,
    targets: I,
) -> Result<ControllerSnapshots, ControllerSnapshotError>
where
    I: IntoIterator<Item = CanisterSnapshotTarget>,
{
    // Validate the entire input before issuing any management calls. The map
    // also owns the deterministic canister order.
    let mut ordered_targets = BTreeMap::new();
    for target in targets {
        if ordered_targets
            .insert(target.canister_id, target.sender)
            .is_some()
        {
            return Err(ControllerSnapshotError::DuplicateCanisterId {
                canister_id: target.canister_id,
            });
        }
    }
    let mut snapshots = BTreeMap::new();
    for (canister_id, sender) in ordered_targets {
        let failure = match try_take_snapshot(pocket_ic, canister_id, sender) {
            Ok(snapshot) => {
                snapshots.insert(canister_id, snapshot);
                continue;
            }
            Err(failure) => failure,
        };
        let cleanup_failures = cleanup_captured_snapshots(pocket_ic, snapshots);
        return Err(match failure {
            SnapshotCaptureFailure::Rejected(response) => ControllerSnapshotError::CaptureFailed {
                canister_id,
                sender,
                response: Box::new(response),
                cleanup_failures,
            },
            SnapshotCaptureFailure::Panicked(source) => ControllerSnapshotError::CapturePanicked {
                canister_id,
                source,
                cleanup_failures,
            },
        });
    }
    Ok(ControllerSnapshots(snapshots))
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
                sender,
                response,
                cleanup_failures,
            } => write!(
                f,
                "failed to capture snapshot for {canister_id} with sender {sender:?}: {response:?}; {} partial snapshots could not be cleaned up",
                cleanup_failures.len()
            ),
            Self::CapturePanicked {
                canister_id,
                source,
                cleanup_failures,
            } => write!(
                f,
                "snapshot capture panicked for {canister_id}: {source}; {} partial snapshots could not be cleaned up",
                cleanup_failures.len()
            ),
            Self::RestoreFailed {
                canister_id,
                sender,
                response,
            } => write!(
                f,
                "failed to restore snapshot for {canister_id} with sender {sender:?}: {response:?}",
            ),
            Self::RestorePanicked {
                canister_id,
                source,
            } => write!(f, "snapshot restore panicked for {canister_id}: {source}"),
        }
    }
}

impl std::error::Error for ControllerSnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CapturePanicked { source, .. } | Self::RestorePanicked { source, .. } => {
                Some(source)
            }
            _ => None,
        }
    }
}

fn try_take_snapshot(
    pocket_ic: &PocketIc,
    canister_id: Principal,
    sender: Option<Principal>,
) -> Result<ControllerSnapshot, SnapshotCaptureFailure> {
    let snapshot = transport::capture_operation(|| {
        pocket_ic.take_canister_snapshot(canister_id, sender, None)
    })
    .map_err(SnapshotCaptureFailure::Panicked)?
    .map_err(SnapshotCaptureFailure::Rejected)?;
    Ok(ControllerSnapshot {
        snapshot_id: snapshot.id,
        sender,
    })
}

fn cleanup_captured_snapshots(
    pocket_ic: &PocketIc,
    snapshots: BTreeMap<Principal, ControllerSnapshot>,
) -> Vec<SnapshotCleanupFailure> {
    let mut failures = Vec::new();
    for (canister_id, snapshot) in snapshots {
        let cleanup = transport::capture_operation(|| {
            pocket_ic.delete_canister_snapshot(canister_id, snapshot.sender, snapshot.snapshot_id)
        });
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(response)) => failures.push(SnapshotCleanupFailure {
                canister_id,
                sender: snapshot.sender,
                response: Some(Box::new(response)),
                panic_message: None,
            }),
            Err(source) => failures.push(SnapshotCleanupFailure {
                canister_id,
                sender: snapshot.sender,
                response: None,
                panic_message: Some(source.to_string()),
            }),
        }
    }
    failures
}

fn restore_controller_snapshot(
    pocket_ic: &PocketIc,
    canister_id: Principal,
    snapshot_id: &[u8],
    funding: SnapshotRestoreFunding,
    sender: Option<Principal>,
) -> Result<(), ControllerSnapshotError> {
    transport::capture_operation(|| {
        apply_snapshot_restore_funding(pocket_ic, canister_id, funding);
        pocket_ic.load_canister_snapshot(canister_id, sender, snapshot_id.to_vec())
    })
    .map_err(|source| ControllerSnapshotError::RestorePanicked {
        canister_id,
        source,
    })?
    .map_err(|response| ControllerSnapshotError::RestoreFailed {
        canister_id,
        sender,
        response: Box::new(response),
    })
}

fn apply_snapshot_restore_funding(
    pocket_ic: &PocketIc,
    canister_id: Principal,
    funding: SnapshotRestoreFunding,
) {
    if funding == SnapshotRestoreFunding::Preserve {
        return;
    }

    let balance = pocket_ic.cycle_balance(canister_id);
    let top_up = snapshot_restore_top_up(balance, funding);
    if top_up > 0 {
        let _ = pocket_ic.add_cycles(canister_id, top_up);
    }
}

const fn snapshot_restore_top_up(balance: u128, funding: SnapshotRestoreFunding) -> u128 {
    match funding {
        SnapshotRestoreFunding::Preserve => 0,
        SnapshotRestoreFunding::TopUpTo { minimum_cycles } => {
            minimum_cycles.saturating_sub(balance)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SnapshotRestoreFunding, snapshot_restore_top_up};

    #[test]
    fn snapshot_restore_funding_is_explicit() {
        assert_eq!(
            snapshot_restore_top_up(10, SnapshotRestoreFunding::Preserve),
            0
        );
        assert_eq!(
            snapshot_restore_top_up(10, SnapshotRestoreFunding::TopUpTo { minimum_cycles: 25 }),
            15
        );
        assert_eq!(
            snapshot_restore_top_up(30, SnapshotRestoreFunding::TopUpTo { minimum_cycles: 25 }),
            0
        );
    }
}
