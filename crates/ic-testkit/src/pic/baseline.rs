use candid::Principal;
use pocket_ic::PocketIc;
use std::{
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::{Mutex, MutexGuard},
};

use super::{
    ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, SnapshotRestoreFunding,
    transport,
};

/// One owned PocketIC instance with captured snapshots and caller metadata.
///
/// The value contains no global synchronization. Callers choose the specific
/// [`Mutex`] slot passed to [`restore_or_rebuild_cached_pocket_ic_baseline`].
pub struct CachedPocketIcBaseline<T> {
    pocket_ic: PocketIc,
    snapshots: ControllerSnapshots,
    metadata: T,
}

/// Exclusive access to one caller-provided cached-baseline slot.
///
/// The slot remains locked for this guard's lifetime. Other slots and fresh
/// PocketIC instances remain independent.
pub struct CachedPocketIcBaselineGuard<'a, T> {
    guard: MutexGuard<'a, Option<CachedPocketIcBaseline<T>>>,
}

enum CachedBaselineRestoreFailure {
    DeadInstanceTransport,
    Panic(Box<dyn std::any::Any + Send>),
}

/// Acquire one process-local cached PocketIC baseline, building it on first use.
fn acquire_cached_pocket_ic_baseline<T, F>(
    slot: &'static Mutex<Option<CachedPocketIcBaseline<T>>>,
    build: F,
) -> (CachedPocketIcBaselineGuard<'static, T>, bool)
where
    F: FnOnce() -> CachedPocketIcBaseline<T>,
{
    let mut guard = slot
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let cache_hit = guard.is_some();

    if !cache_hit {
        *guard = Some(build());
    }

    (CachedPocketIcBaselineGuard { guard }, cache_hit)
}

/// Restore one cached PocketIC baseline, rebuilding it if the owned PocketIC
/// instance has died between tests.
///
/// On the first call, `build` creates the baseline and `restore` is not run.
/// On a cache hit, `restore` runs while the slot is locked. Only a recognized
/// dead-instance transport panic causes eviction and rebuilding; unrelated
/// panics resume unwinding.
///
/// The returned boolean is `true` only when the existing baseline was restored
/// successfully.
pub fn restore_or_rebuild_cached_pocket_ic_baseline<T, B, R>(
    slot: &'static Mutex<Option<CachedPocketIcBaseline<T>>>,
    build: B,
    restore: R,
) -> (CachedPocketIcBaselineGuard<'static, T>, bool)
where
    B: Fn() -> CachedPocketIcBaseline<T>,
    R: Fn(&CachedPocketIcBaseline<T>),
{
    let (baseline, cache_hit) = acquire_cached_pocket_ic_baseline(slot, &build);
    if !cache_hit {
        return (baseline, false);
    }

    match try_restore_cached_pocket_ic_baseline(
        baseline
            .guard
            .as_ref()
            .expect("cached PocketIC baseline must exist"),
        restore,
    ) {
        Ok(()) => return (baseline, true),
        Err(CachedBaselineRestoreFailure::DeadInstanceTransport) => {}
        Err(CachedBaselineRestoreFailure::Panic(payload)) => {
            resume_unwind(payload);
        }
    }

    drop(baseline);
    drop_stale_cached_pocket_ic_baseline(slot);

    let (rebuilt, _cache_hit) = acquire_cached_pocket_ic_baseline(slot, build);
    (rebuilt, false)
}

// Attempt one cached baseline restore and classify only the one recovery path
// we intentionally swallow: a dead PocketIC transport instance.
fn try_restore_cached_pocket_ic_baseline<T, R>(
    baseline: &CachedPocketIcBaseline<T>,
    restore: R,
) -> Result<(), CachedBaselineRestoreFailure>
where
    R: Fn(&CachedPocketIcBaseline<T>),
{
    match catch_unwind(AssertUnwindSafe(|| restore(baseline))) {
        Ok(()) => Ok(()),
        Err(payload) => {
            if transport::panic_is_dead_instance_transport(payload.as_ref()) {
                Err(CachedBaselineRestoreFailure::DeadInstanceTransport)
            } else {
                Err(CachedBaselineRestoreFailure::Panic(payload))
            }
        }
    }
}

/// Remove one dead cached baseline and swallow teardown panics from a broken
/// PocketIC instance so callers can rebuild cleanly.
fn drop_stale_cached_pocket_ic_baseline<T>(
    slot: &'static Mutex<Option<CachedPocketIcBaseline<T>>>,
) {
    let stale = {
        let mut slot = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slot.take()
    };

    if let Some(stale) = stale {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(stale);
        }));
    }
}

impl<T> CachedPocketIcBaselineGuard<'_, T> {
    /// Borrow the owned PocketIC instance behind this cached baseline guard.
    #[must_use]
    pub fn pocket_ic(&self) -> &PocketIc {
        self.guard
            .as_ref()
            .expect("cached PocketIC baseline must exist")
            .pocket_ic()
    }

    /// Borrow the captured metadata behind this cached baseline guard.
    #[must_use]
    pub fn metadata(&self) -> &T {
        self.guard
            .as_ref()
            .expect("cached PocketIC baseline must exist")
            .metadata()
    }

    /// Mutably borrow the captured metadata behind this cached baseline guard.
    #[must_use]
    pub fn metadata_mut(&mut self) -> &mut T {
        self.guard
            .as_mut()
            .expect("cached PocketIC baseline must exist")
            .metadata_mut()
    }

    /// Restore the captured snapshot set without adding cycles.
    pub fn restore(&self, controller_id: Principal) -> Result<(), ControllerSnapshotError> {
        self.guard
            .as_ref()
            .expect("cached PocketIC baseline must exist")
            .restore(controller_id)
    }

    /// Restore the captured snapshot set with an explicit cycle-funding policy.
    pub fn restore_with_funding(
        &self,
        controller_id: Principal,
        funding: SnapshotRestoreFunding,
    ) -> Result<(), ControllerSnapshotError> {
        self.guard
            .as_ref()
            .expect("cached PocketIC baseline must exist")
            .restore_with_funding(controller_id, funding)
    }
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
