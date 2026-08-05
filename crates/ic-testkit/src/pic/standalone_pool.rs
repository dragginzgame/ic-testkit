use std::{
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Condvar, Mutex, MutexGuard, TryLockError,
        atomic::{AtomicUsize, Ordering},
    },
};

use super::{
    ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, SnapshotRestoreFunding,
    StandaloneCanisterFixture, transport,
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
    slots: [Mutex<Option<StandaloneFixtureBaseline>>; CAPACITY],
    next_slot: AtomicUsize,
    wait_lock: Mutex<()>,
    slot_released: Condvar,
    restore_funding: SnapshotRestoreFunding,
}

/// Exclusive lease of one independently restored standalone fixture.
///
/// The lease dereferences to [`StandaloneCanisterFixture`], so existing call
/// helpers can borrow it without adding a second fixture API.
pub struct CachedStandaloneCanisterFixtureGuard<'a> {
    slot: Option<MutexGuard<'a, Option<StandaloneFixtureBaseline>>>,
    wait_lock: &'a Mutex<()>,
    slot_released: &'a Condvar,
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
            slots: [const { Mutex::new(None) }; CAPACITY],
            next_slot: AtomicUsize::new(0),
            wait_lock: Mutex::new(()),
            slot_released: Condvar::new(),
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
    /// the affected slot. Other snapshot failures are returned unchanged.
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
        let mut start = self.next_slot.fetch_add(1, Ordering::Relaxed) % CAPACITY;

        loop {
            if let Some(slot) = self.try_acquire_slot(start) {
                return self.prepare_slot(slot, &build);
            }

            let wait_guard = self
                .wait_lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(slot) = self.try_acquire_slot(start) {
                drop(wait_guard);
                return self.prepare_slot(slot, &build);
            }

            drop(
                self.slot_released
                    .wait(wait_guard)
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
            start = self.next_slot.fetch_add(1, Ordering::Relaxed) % CAPACITY;
        }
    }

    fn try_acquire_slot(
        &self,
        start: usize,
    ) -> Option<MutexGuard<'_, Option<StandaloneFixtureBaseline>>> {
        for offset in 0..CAPACITY {
            let slot_index = (start + offset) % CAPACITY;
            match self.slots[slot_index].try_lock() {
                Ok(slot) => return Some(slot),
                Err(TryLockError::Poisoned(error)) => return Some(error.into_inner()),
                Err(TryLockError::WouldBlock) => {}
            }
        }

        None
    }

    fn prepare_slot<'a, B>(
        &'a self,
        slot: MutexGuard<'a, Option<StandaloneFixtureBaseline>>,
        build: &B,
    ) -> Result<(CachedStandaloneCanisterFixtureGuard<'a>, bool), ControllerSnapshotError>
    where
        B: Fn() -> StandaloneCanisterFixture,
    {
        // Wrap the reservation before calling caller code. Its Drop path
        // releases the slot and wakes another waiter on success, error, or
        // unwind.
        let mut guard = self.guard(slot);
        let slot = guard
            .slot
            .as_mut()
            .expect("fixture pool reservation must retain its slot");
        let cache_hit = slot.is_some();
        if !cache_hit {
            **slot = Some(StandaloneFixtureBaseline::capture(build())?);
            return Ok((guard, false));
        }

        let restore = slot
            .as_ref()
            .expect("populated fixture pool slot must remain present")
            .restore(self.restore_funding);
        match restore {
            Ok(()) => Ok((guard, true)),
            Err(error) if snapshot_error_is_dead_instance_transport(&error) => {
                let stale = slot.take();
                if let Some(stale) = stale {
                    let _ = catch_unwind(AssertUnwindSafe(|| drop(stale)));
                }
                **slot = Some(StandaloneFixtureBaseline::capture(build())?);
                Ok((guard, false))
            }
            Err(error) => Err(error),
        }
    }

    const fn guard<'a>(
        &'a self,
        slot: MutexGuard<'a, Option<StandaloneFixtureBaseline>>,
    ) -> CachedStandaloneCanisterFixtureGuard<'a> {
        CachedStandaloneCanisterFixtureGuard {
            slot: Some(slot),
            wait_lock: &self.wait_lock,
            slot_released: &self.slot_released,
        }
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
            .as_ref()
            .expect("fixture pool guard must retain its slot")
            .as_ref()
            .expect("leased fixture pool slot must remain populated")
            .fixture
    }
}

impl Drop for CachedStandaloneCanisterFixtureGuard<'_> {
    fn drop(&mut self) {
        drop(self.slot.take());

        // Pair the notification with the same lock used by acquire's second
        // availability check so a release cannot be lost between that check
        // and the condvar wait.
        let wait_guard = self
            .wait_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.slot_released.notify_one();
        drop(wait_guard);
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
