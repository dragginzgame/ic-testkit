use candid::Principal;
use pocket_ic::PocketIc;
use std::{
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::{Mutex, MutexGuard},
};

use super::{ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, startup};

///
/// CachedPocketIcBaseline
///

pub struct CachedPocketIcBaseline<T> {
    pocket_ic: PocketIc,
    snapshots: ControllerSnapshots,
    metadata: T,
}

///
/// CachedPocketIcBaselineGuard
///

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
            if startup::panic_is_dead_instance_transport(payload.as_ref()) {
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

    /// Restore the captured snapshot set back into the owned PocketIC instance.
    pub fn restore(&self, controller_id: Principal) -> Result<(), ControllerSnapshotError> {
        self.guard
            .as_ref()
            .expect("cached PocketIC baseline must exist")
            .restore(controller_id)
    }
}

impl<T> CachedPocketIcBaseline<T> {
    /// Capture one immutable cached baseline from the current PocketIC instance.
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

    /// Restore the captured snapshot set back into the owned PocketIC instance.
    pub fn restore(&self, controller_id: Principal) -> Result<(), ControllerSnapshotError> {
        self.pocket_ic
            .restore_controller_snapshots(controller_id, &self.snapshots)
    }

    /// Borrow the owned PocketIC instance behind this cached baseline.
    #[must_use]
    pub const fn pocket_ic(&self) -> &PocketIc {
        &self.pocket_ic
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
