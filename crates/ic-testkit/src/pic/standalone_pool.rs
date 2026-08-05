use std::{
    num::NonZeroUsize,
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::OnceLock,
};

use super::{
    ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, SnapshotRestoreFunding,
    StandaloneCanisterFixture,
    bounded_pool::{BoundedSlotLease, BoundedSlotPool},
    transport,
};

struct StandaloneFixtureBaseline {
    fixture: StandaloneCanisterFixture,
    snapshots: ControllerSnapshots,
}

impl StandaloneFixtureBaseline {
    fn capture(fixture: StandaloneCanisterFixture) -> Result<Self, ControllerSnapshotError> {
        let canister_id = fixture.canister_id();
        let snapshots = fixture
            .pocket_ic()
            .capture_controller_snapshots(canister_id, [canister_id])?;

        Ok(Self { fixture, snapshots })
    }

    fn restore(&self, funding: SnapshotRestoreFunding) -> Result<(), ControllerSnapshotError> {
        self.fixture
            .pocket_ic()
            .restore_controller_snapshots_with_funding(
                self.fixture.canister_id(),
                &self.snapshots,
                funding,
            )
    }
}

/// Caller-owned bounded pool of independently restorable standalone fixtures.
///
/// Each slot owns one PocketIC instance, one installed canister, and one
/// captured baseline snapshot. Acquiring a populated slot restores that
/// snapshot before returning it. At most `CAPACITY` leases can overlap; a
/// caller waits only when every slot is in use.
///
/// One pool represents one logical fixture recipe. Every call to
/// [`acquire`](Self::acquire) must supply a builder for the same Wasm, init
/// arguments, topology, and seeded baseline. The builder is not evaluated on
/// a cache hit, so callers should use a separate pool for each recipe.
///
/// Snapshot restoration rewinds the installed canister, not the surrounding
/// PocketIC instance. Instance time, other canisters, and cycle changes not
/// covered by the selected [`SnapshotRestoreFunding`] policy may persist.
///
/// The pool contains no process-global state. Downstream suites select a
/// capacity that fits their host and keep lifecycle-sensitive tests on fresh
/// [`StandaloneCanisterFixture`] values when snapshot restoration is not the
/// intended isolation boundary.
pub struct CachedStandaloneCanisterFixturePool<const CAPACITY: usize> {
    slots: OnceLock<BoundedSlotPool<StandaloneFixtureBaseline>>,
    restore_funding: SnapshotRestoreFunding,
}

/// Exclusive lease of one independently restored standalone fixture.
///
/// The lease dereferences to [`StandaloneCanisterFixture`], so existing call
/// helpers can borrow it without adding a second fixture API.
pub struct CachedStandaloneCanisterFixtureGuard<'a> {
    slot: BoundedSlotLease<'a, StandaloneFixtureBaseline>,
}

impl<const CAPACITY: usize> CachedStandaloneCanisterFixturePool<CAPACITY> {
    /// Create an empty caller-owned fixture pool.
    ///
    /// # Panics
    ///
    /// Panics at compile time for a statically initialized zero-capacity pool,
    /// or at runtime if constructed dynamically with zero capacity.
    #[must_use]
    pub const fn new() -> Self {
        assert!(CAPACITY > 0, "fixture pool capacity must be non-zero");

        Self {
            slots: OnceLock::new(),
            restore_funding: SnapshotRestoreFunding::Preserve,
        }
    }

    /// Select the cycle-funding policy applied immediately before each
    /// snapshot restore.
    #[must_use]
    pub const fn with_restore_funding(mut self, funding: SnapshotRestoreFunding) -> Self {
        self.restore_funding = funding;
        self
    }

    /// Acquire one isolated fixture, building a slot on first use and restoring
    /// its captured snapshot on later uses.
    ///
    /// `build` must create the same logical fixture baseline on every call to
    /// this pool. It runs only when an empty slot is first populated or a dead
    /// PocketIC instance must be replaced.
    ///
    /// A recognized dead-instance transport failure evicts and rebuilds only
    /// the affected slot. Other snapshot failures are returned unchanged and
    /// invalidate the possibly partially restored slot for the next lease.
    ///
    /// # Errors
    ///
    /// Returns the structured snapshot capture or restore failure for the
    /// selected slot.
    pub fn acquire<B>(
        &self,
        build: B,
    ) -> Result<(CachedStandaloneCanisterFixtureGuard<'_>, bool), ControllerSnapshotError>
    where
        B: Fn() -> StandaloneCanisterFixture,
    {
        self.prepare_slot(self.slots().acquire(), &build)
    }

    fn prepare_slot<'a, B>(
        &'a self,
        mut slot: BoundedSlotLease<'a, StandaloneFixtureBaseline>,
        build: &B,
    ) -> Result<(CachedStandaloneCanisterFixtureGuard<'a>, bool), ControllerSnapshotError>
    where
        B: Fn() -> StandaloneCanisterFixture,
    {
        if !slot.is_reusable() {
            if let Some(stale) = slot.take() {
                let _ = catch_unwind(AssertUnwindSafe(|| drop(stale)));
            }
            slot.replace(StandaloneFixtureBaseline::capture(build())?);
            return Ok((CachedStandaloneCanisterFixtureGuard { slot }, false));
        }

        let restore = slot
            .get()
            .expect("populated fixture pool slot must remain present")
            .restore(self.restore_funding);
        match restore {
            Ok(()) => Ok((CachedStandaloneCanisterFixtureGuard { slot }, true)),
            Err(error) if snapshot_error_is_dead_instance_transport(&error) => {
                let stale = slot.take();
                if let Some(stale) = stale {
                    let _ = catch_unwind(AssertUnwindSafe(|| drop(stale)));
                }
                slot.replace(StandaloneFixtureBaseline::capture(build())?);
                Ok((CachedStandaloneCanisterFixtureGuard { slot }, false))
            }
            Err(error) => {
                // Restoration may have changed an earlier canister before a
                // later snapshot failed. Preserve the current error while
                // preventing a partially restored slot from being reused.
                slot.invalidate();
                Err(error)
            }
        }
    }

    fn slots(&self) -> &BoundedSlotPool<StandaloneFixtureBaseline> {
        self.slots.get_or_init(|| {
            BoundedSlotPool::new(
                NonZeroUsize::new(CAPACITY).expect("fixture pool capacity must be non-zero"),
            )
        })
    }
}

impl<const CAPACITY: usize> Default for CachedStandaloneCanisterFixturePool<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CachedStandaloneCanisterFixtureGuard<'_> {
    type Target = StandaloneCanisterFixture;

    fn deref(&self) -> &Self::Target {
        &self
            .slot
            .get()
            .expect("leased fixture pool slot must remain populated")
            .fixture
    }
}

fn snapshot_error_is_dead_instance_transport(error: &ControllerSnapshotError) -> bool {
    matches!(
        error,
        ControllerSnapshotError::RestorePanicked { message, .. }
            if transport::is_dead_instance_transport_error(message)
    )
}

#[cfg(test)]
mod tests {
    use super::CachedStandaloneCanisterFixturePool;

    const _: CachedStandaloneCanisterFixturePool<1> = CachedStandaloneCanisterFixturePool::new();

    #[test]
    fn nonzero_pool_constructs() {
        let _pool = CachedStandaloneCanisterFixturePool::<2>::new();
    }
}
