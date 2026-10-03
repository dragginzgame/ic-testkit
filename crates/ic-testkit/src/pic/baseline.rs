use super::{
    CanisterSnapshotTarget, ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt,
    SnapshotRestoreFunding,
};
use candid::Principal;
use pocket_ic::PocketIc;

/// One owned PocketIC instance with captured snapshots and caller metadata.
///
/// The value contains no synchronization. A [`super::PocketIcBaselineRecipe`]
/// can own it as its fixture in a [`super::CachedPocketIcBaselinePool`].
pub struct CachedPocketIcBaseline<T> {
    pocket_ic: PocketIc,
    snapshots: ControllerSnapshots,
    metadata: T,
}

impl<T> CachedPocketIcBaseline<T> {
    /// Capture one cached baseline from the current PocketIC instance.
    ///
    /// Snapshot capture is ordered and transactional as documented by
    /// [`PocketIcSnapshotExt::capture_controller_snapshots`].
    pub fn capture<I>(
        pocket_ic: PocketIc,
        controller_id: Principal,
        canister_ids: I,
        metadata: T,
    ) -> Result<Self, ControllerSnapshotError>
    where
        I: IntoIterator<Item = Principal>,
    {
        let snapshots = pocket_ic.capture_controller_snapshots(controller_id, canister_ids)?;

        Ok(Self {
            pocket_ic,
            snapshots,
            metadata,
        })
    }

    /// Capture one cached baseline with an explicit sender for every canister.
    ///
    /// This avoids fallback rejections in mixed-controller topologies.
    pub fn capture_with_senders<I>(
        pocket_ic: PocketIc,
        targets: I,
        metadata: T,
    ) -> Result<Self, ControllerSnapshotError>
    where
        I: IntoIterator<Item = CanisterSnapshotTarget>,
    {
        let snapshots = pocket_ic.capture_snapshots_with_senders(targets)?;

        Ok(Self {
            pocket_ic,
            snapshots,
            metadata,
        })
    }

    /// Restore the captured snapshot set without adding cycles.
    pub fn restore(&self, controller_id: Principal) -> Result<(), ControllerSnapshotError> {
        self.pocket_ic
            .restore_controller_snapshots(controller_id, &self.snapshots)
    }

    /// Restore the captured snapshot set with an explicit cycle-funding policy.
    pub fn restore_with_funding(
        &self,
        controller_id: Principal,
        funding: SnapshotRestoreFunding,
    ) -> Result<(), ControllerSnapshotError> {
        self.pocket_ic.restore_controller_snapshots_with_funding(
            controller_id,
            &self.snapshots,
            funding,
        )
    }

    /// Restore with exactly the senders retained during snapshot capture.
    pub fn restore_with_captured_senders(&self) -> Result<(), ControllerSnapshotError> {
        self.pocket_ic
            .restore_snapshots_with_captured_senders(&self.snapshots)
    }

    /// Restore with captured senders and an explicit cycle-funding policy.
    pub fn restore_with_captured_senders_and_funding(
        &self,
        funding: SnapshotRestoreFunding,
    ) -> Result<(), ControllerSnapshotError> {
        self.pocket_ic
            .restore_snapshots_with_captured_senders_and_funding(&self.snapshots, funding)
    }

    /// Borrow the owned PocketIC instance behind this cached baseline.
    #[must_use]
    pub const fn pocket_ic(&self) -> &PocketIc {
        &self.pocket_ic
    }

    /// Return the number of canisters captured by this baseline.
    #[must_use]
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Iterate over captured canister ids in deterministic principal order.
    pub fn snapshot_canister_ids(&self) -> impl Iterator<Item = Principal> + '_ {
        self.snapshots.canister_ids()
    }

    /// Borrow the captured metadata associated with this cached baseline.
    #[must_use]
    pub const fn metadata(&self) -> &T {
        &self.metadata
    }

    /// Mutably borrow the captured metadata associated with this cached baseline.
    #[must_use]
    pub const fn metadata_mut(&mut self) -> &mut T {
        &mut self.metadata
    }
}
