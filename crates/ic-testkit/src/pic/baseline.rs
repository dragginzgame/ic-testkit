use super::{
    CanisterSnapshotTarget, ControllerSnapshotError, ControllerSnapshots, PocketIcInstance,
    PocketIcSnapshotExt, SnapshotRestoreFunding,
};
use candid::Principal;
use pocket_ic::PocketIc;

/// One owned PocketIC instance/server record with snapshots and caller metadata.
///
/// The value contains no synchronization. A [`super::PocketIcBaselineRecipe`]
/// can own it as its fixture in a [`super::CachedPocketIcBaselinePool`].
pub struct CachedPocketIcBaseline<T> {
    instance: PocketIcInstance<PocketIc>,
    snapshots: ControllerSnapshots,
    metadata: T,
}

impl<T> CachedPocketIcBaseline<T> {
    /// Capture a cached baseline with an explicit sender for every canister.
    ///
    /// Capture is ordered and transactional as documented by
    /// [`PocketIcSnapshotExt::capture_snapshots`]. No sender fallback is performed.
    pub fn capture<I>(
        instance: PocketIcInstance<PocketIc>,
        targets: I,
        metadata: T,
    ) -> Result<Self, ControllerSnapshotError>
    where
        I: IntoIterator<Item = CanisterSnapshotTarget>,
    {
        let snapshots = instance.instance.capture_snapshots(targets)?;
        Ok(Self {
            instance,
            snapshots,
            metadata,
        })
    }

    /// Restore with exactly the captured senders and selected cycle funding.
    pub fn restore(&self, funding: SnapshotRestoreFunding) -> Result<(), ControllerSnapshotError> {
        self.instance
            .instance
            .restore_snapshots(&self.snapshots, funding)
    }

    /// Borrow the owned PocketIC instance behind this cached baseline.
    #[must_use]
    pub const fn pocket_ic(&self) -> &PocketIc {
        &self.instance.instance
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
