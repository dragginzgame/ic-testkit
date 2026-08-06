use std::{
    num::NonZeroUsize,
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::OnceLock,
    time::{Duration, Instant},
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
    invalidation_reason: Option<StandaloneFixturePoolRebuildReason>,
}

impl StandaloneFixtureBaseline {
    fn capture(fixture: StandaloneCanisterFixture) -> Result<Self, ControllerSnapshotError> {
        let canister_id = fixture.canister_id();
        let snapshots = fixture
            .pocket_ic()
            .capture_controller_snapshots(canister_id, [canister_id])?;

        Ok(Self {
            fixture,
            snapshots,
            invalidation_reason: None,
        })
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

/// Timings for one standalone fixture-pool acquisition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StandaloneFixturePoolTimings {
    wait: Duration,
    build: Option<Duration>,
    restore: Option<Duration>,
    stale_teardown: Option<Duration>,
    total: Duration,
}

/// Whether a standalone fixture-pool lease was built, restored, or rebuilt.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StandaloneFixturePoolOutcome {
    /// An empty slot was built and its snapshot was captured.
    Built {
        /// Diagnostic slot index.
        slot: usize,
        /// Acquisition phase timings.
        timings: StandaloneFixturePoolTimings,
    },
    /// A populated slot was restored successfully.
    Restored {
        /// Diagnostic slot index.
        slot: usize,
        /// Acquisition phase timings.
        timings: StandaloneFixturePoolTimings,
    },
    /// An invalid or dead slot was rebuilt.
    Rebuilt {
        /// Diagnostic slot index.
        slot: usize,
        /// Reason the previous slot could not be reused.
        reason: StandaloneFixturePoolRebuildReason,
        /// Acquisition phase timings.
        timings: StandaloneFixturePoolTimings,
    },
}

/// Why a standalone fixture-pool slot was rebuilt.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneFixturePoolRebuildReason {
    /// PocketIC transport was no longer reachable during snapshot restoration.
    DeadPocketIcTransport,
    /// A previous restore failed after it may have partially changed the slot.
    PreviousRestoreFailure,
    /// A lease was dropped while its thread was unwinding.
    UnwindWhileLeased,
}

/// Standalone fixture preparation stage associated with a snapshot failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneFixturePoolStage {
    /// Building a fixture and capturing its baseline snapshot.
    Build,
    /// Restoring a populated fixture's baseline snapshot.
    Restore,
}

/// Failure to acquire a standalone fixture-pool lease with structured diagnostics.
#[non_exhaustive]
#[derive(Debug)]
pub enum StandaloneFixturePoolError {
    /// Initial build/capture or snapshot restoration failed.
    Preparation {
        /// Failed lifecycle stage.
        stage: StandaloneFixturePoolStage,
        /// Structured snapshot failure.
        source: Box<ControllerSnapshotError>,
        /// Timings recorded before acquisition failed.
        timings: Box<StandaloneFixturePoolTimings>,
    },
    /// Dead-transport restoration failed and replacement snapshot capture also failed.
    RecoveryFailed {
        /// Original dead-transport restoration failure.
        original: Box<ControllerSnapshotError>,
        /// Snapshot failure while building the replacement slot.
        rebuild: Box<ControllerSnapshotError>,
        /// Combined restore, teardown, and rebuild timings.
        timings: Box<StandaloneFixturePoolTimings>,
    },
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
        self.acquire_with_outcome(build)
            .map(|(guard, outcome)| (guard, outcome.is_reused()))
            .map_err(StandaloneFixturePoolError::into_snapshot_error)
    }

    /// Acquire one fixture with structured lifecycle outcome and phase timings.
    ///
    /// This is the diagnostic counterpart to [`acquire`](Self::acquire). It
    /// distinguishes a new slot from restoration and reconstruction while the
    /// compatibility method continues to report restoration as a boolean.
    ///
    /// # Errors
    ///
    /// Returns the failed preparation stage, the structured snapshot error,
    /// and all phase timings completed before failure. If dead-transport
    /// restoration and replacement capture both fail, both errors are retained.
    pub fn acquire_with_outcome<B>(
        &self,
        build: B,
    ) -> Result<
        (
            CachedStandaloneCanisterFixtureGuard<'_>,
            StandaloneFixturePoolOutcome,
        ),
        StandaloneFixturePoolError,
    >
    where
        B: Fn() -> StandaloneCanisterFixture,
    {
        let total_started = Instant::now();
        self.prepare_slot_with_outcome(self.slots().acquire(), &build, total_started)
    }

    fn prepare_slot_with_outcome<'a, B>(
        &'a self,
        mut slot: BoundedSlotLease<'a, StandaloneFixtureBaseline>,
        build: &B,
        total_started: Instant,
    ) -> Result<
        (
            CachedStandaloneCanisterFixtureGuard<'a>,
            StandaloneFixturePoolOutcome,
        ),
        StandaloneFixturePoolError,
    >
    where
        B: Fn() -> StandaloneCanisterFixture,
    {
        let slot_index = slot.slot_index();
        let mut timings = StandaloneFixturePoolTimings {
            wait: slot.wait(),
            ..StandaloneFixturePoolTimings::default()
        };

        if !slot.is_reusable() {
            let rebuild_reason = Self::rebuild_reason_for_invalid_slot(&slot);
            Self::discard_stale_slot(&mut slot, &mut timings);
            let baseline = match Self::build_slot(build, &mut timings) {
                Ok(baseline) => baseline,
                Err(source) => {
                    timings.total = total_started.elapsed();
                    return Err(StandaloneFixturePoolError::Preparation {
                        stage: StandaloneFixturePoolStage::Build,
                        source: Box::new(source),
                        timings: Box::new(timings),
                    });
                }
            };
            slot.replace(baseline);
            timings.total = total_started.elapsed();
            let outcome = rebuild_reason.map_or_else(
                || StandaloneFixturePoolOutcome::Built {
                    slot: slot_index,
                    timings,
                },
                |reason| StandaloneFixturePoolOutcome::Rebuilt {
                    slot: slot_index,
                    reason,
                    timings,
                },
            );
            return Ok((CachedStandaloneCanisterFixtureGuard { slot }, outcome));
        }

        let restore_started = Instant::now();
        let restore = slot
            .get()
            .expect("populated fixture pool slot must remain present")
            .restore(self.restore_funding);
        timings.restore = Some(restore_started.elapsed());
        match restore {
            Ok(()) => {
                slot.get_mut()
                    .expect("restored fixture pool slot must remain present")
                    .invalidation_reason = None;
                timings.total = total_started.elapsed();
                Ok((
                    CachedStandaloneCanisterFixtureGuard { slot },
                    StandaloneFixturePoolOutcome::Restored {
                        slot: slot_index,
                        timings,
                    },
                ))
            }
            Err(error) if snapshot_error_is_dead_instance_transport(&error) => {
                Self::discard_stale_slot(&mut slot, &mut timings);
                let baseline = match Self::build_slot(build, &mut timings) {
                    Ok(baseline) => baseline,
                    Err(rebuild) => {
                        timings.total = total_started.elapsed();
                        return Err(StandaloneFixturePoolError::RecoveryFailed {
                            original: Box::new(error),
                            rebuild: Box::new(rebuild),
                            timings: Box::new(timings),
                        });
                    }
                };
                slot.replace(baseline);
                timings.total = total_started.elapsed();
                Ok((
                    CachedStandaloneCanisterFixtureGuard { slot },
                    StandaloneFixturePoolOutcome::Rebuilt {
                        slot: slot_index,
                        reason: StandaloneFixturePoolRebuildReason::DeadPocketIcTransport,
                        timings,
                    },
                ))
            }
            Err(source) => {
                if let Some(baseline) = slot.get_mut() {
                    baseline.invalidation_reason =
                        Some(StandaloneFixturePoolRebuildReason::PreviousRestoreFailure);
                }
                // Restoration may have changed an earlier canister before a
                // later snapshot failed. Preserve the current error while
                // preventing a partially restored slot from being reused.
                slot.invalidate();
                timings.total = total_started.elapsed();
                Err(StandaloneFixturePoolError::Preparation {
                    stage: StandaloneFixturePoolStage::Restore,
                    source: Box::new(source),
                    timings: Box::new(timings),
                })
            }
        }
    }

    fn rebuild_reason_for_invalid_slot(
        slot: &BoundedSlotLease<'_, StandaloneFixtureBaseline>,
    ) -> Option<StandaloneFixturePoolRebuildReason> {
        if slot.invalidated_by_unwind() {
            Some(StandaloneFixturePoolRebuildReason::UnwindWhileLeased)
        } else {
            slot.get()
                .and_then(|baseline| baseline.invalidation_reason)
                .or_else(|| {
                    slot.is_populated()
                        .then_some(StandaloneFixturePoolRebuildReason::PreviousRestoreFailure)
                })
        }
    }

    fn build_slot<B>(
        build: &B,
        timings: &mut StandaloneFixturePoolTimings,
    ) -> Result<StandaloneFixtureBaseline, ControllerSnapshotError>
    where
        B: Fn() -> StandaloneCanisterFixture,
    {
        let started = Instant::now();
        let result = StandaloneFixtureBaseline::capture(build());
        timings.build = Some(started.elapsed());
        result
    }

    fn discard_stale_slot(
        slot: &mut BoundedSlotLease<'_, StandaloneFixtureBaseline>,
        timings: &mut StandaloneFixturePoolTimings,
    ) {
        if !slot.is_populated() {
            return;
        }
        let started = Instant::now();
        if let Some(stale) = slot.take() {
            let _ = catch_unwind(AssertUnwindSafe(|| drop(stale)));
        }
        timings.stale_teardown = Some(started.elapsed());
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

impl StandaloneFixturePoolOutcome {
    /// Diagnostic slot index used by this acquisition.
    #[must_use]
    pub const fn slot(&self) -> usize {
        match self {
            Self::Built { slot, .. } | Self::Restored { slot, .. } | Self::Rebuilt { slot, .. } => {
                *slot
            }
        }
    }

    /// Acquisition phase timings.
    #[must_use]
    pub const fn timings(&self) -> StandaloneFixturePoolTimings {
        match self {
            Self::Built { timings, .. }
            | Self::Restored { timings, .. }
            | Self::Rebuilt { timings, .. } => *timings,
        }
    }

    /// Report whether an existing slot was restored without reconstruction.
    #[must_use]
    pub const fn is_reused(&self) -> bool {
        matches!(self, Self::Restored { .. })
    }
}

impl StandaloneFixturePoolTimings {
    /// Time spent waiting for a capacity slot.
    #[must_use]
    pub const fn wait(self) -> Duration {
        self.wait
    }

    /// Time spent building a fixture and capturing its baseline snapshot.
    #[must_use]
    pub const fn build(self) -> Option<Duration> {
        self.build
    }

    /// Time spent restoring a populated slot.
    #[must_use]
    pub const fn restore(self) -> Option<Duration> {
        self.restore
    }

    /// Time spent dropping an invalid or dead slot before rebuilding.
    #[must_use]
    pub const fn stale_teardown(self) -> Option<Duration> {
        self.stale_teardown
    }

    /// Complete acquisition duration.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl std::fmt::Display for StandaloneFixturePoolTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} wait={:?} build={:?} restore={:?} stale_teardown={:?}",
            self.total, self.wait, self.build, self.restore, self.stale_teardown,
        )
    }
}

impl std::fmt::Display for StandaloneFixturePoolOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Built { slot, timings } => write!(formatter, "built slot={slot} {timings}"),
            Self::Restored { slot, timings } => {
                write!(formatter, "restored slot={slot} {timings}")
            }
            Self::Rebuilt {
                slot,
                reason,
                timings,
            } => write!(formatter, "rebuilt slot={slot} reason={reason:?} {timings}"),
        }
    }
}

impl StandaloneFixturePoolError {
    /// Timings recorded before this acquisition failed.
    #[must_use]
    pub const fn timings(&self) -> StandaloneFixturePoolTimings {
        match self {
            Self::Preparation { timings, .. } | Self::RecoveryFailed { timings, .. } => **timings,
        }
    }

    fn into_snapshot_error(self) -> ControllerSnapshotError {
        match self {
            Self::Preparation { source, .. } => *source,
            Self::RecoveryFailed { rebuild, .. } => *rebuild,
        }
    }
}

impl std::fmt::Display for StandaloneFixturePoolStage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Build => "fixture build and snapshot capture",
            Self::Restore => "fixture snapshot restore",
        })
    }
}

impl std::fmt::Display for StandaloneFixturePoolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation { stage, source, .. } => {
                write!(formatter, "standalone {stage} failed: {source}")
            }
            Self::RecoveryFailed {
                original, rebuild, ..
            } => write!(
                formatter,
                "standalone fixture restore failed ({original}); rebuilding the slot also failed: {rebuild}",
            ),
        }
    }
}

impl std::error::Error for StandaloneFixturePoolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Preparation { source, .. } => Some(source.as_ref()),
            Self::RecoveryFailed { original, .. } => Some(original.as_ref()),
        }
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
