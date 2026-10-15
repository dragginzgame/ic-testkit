use std::io;

use candid::Principal;

use super::{
    CachedPocketIcBaseline, CanisterRestoreReceipt, CanisterSnapshotTarget, CycleResetPolicy,
    FailureDisposition, FixtureRecipeId, PocketIcBaselineRecipe, PreparedBaseline,
    ReadinessReceipt, RebuildReason, ResetReceipt, ResetRequirements, SnapshotRestoreFunding,
    StandaloneCanisterFixture, ValidationReceipt, is_dead_pocket_ic_transport_error,
};

/// Snapshot-only recipe for one caller-built, installed canister.
///
/// Use with [`super::CachedPocketIcBaselinePool`]. The baseline metadata is the
/// installed canister ID; calls go through the baseline's borrowed PocketIC.
/// The builder owns initialization and readiness before capture and must produce
/// the same topology and seeded state for every slot of this recipe.
///
/// Only the captured canister is restored. Time, other canisters, pending work
/// and external resources are not reset. Use a custom recipe or a fresh instance
/// when those mutations matter. Funding is applied immediately before restore.
/// Non-transport restore failures are fatal: the pool discards the affected
/// baseline before returning the error. Dead transport permits one recovery build.
pub struct StandaloneCanisterRecipe<B> {
    id: FixtureRecipeId,
    requirements: ResetRequirements,
    funding: SnapshotRestoreFunding,
    build: B,
}

impl<B> StandaloneCanisterRecipe<B> {
    /// Describe a single-canister recipe without invoking its builder.
    ///
    /// Static suites can initialize the recipe and pool with [`std::sync::LazyLock`].
    #[must_use]
    pub fn new(id: FixtureRecipeId, funding: SnapshotRestoreFunding, build: B) -> Self {
        let cycle_policy = match funding {
            SnapshotRestoreFunding::Preserve => CycleResetPolicy::PreserveCurrent,
            SnapshotRestoreFunding::TopUpTo { minimum_cycles } => {
                CycleResetPolicy::TopUpTo(minimum_cycles)
            }
        };
        Self {
            id,
            requirements: ResetRequirements::try_new(cycle_policy, [])
                .expect("empty non-snapshot requirements are valid"),
            funding,
            build,
        }
    }
}

impl<B> PocketIcBaselineRecipe for StandaloneCanisterRecipe<B>
where
    B: Fn() -> StandaloneCanisterFixture + Send + Sync + 'static,
{
    type Metadata = Principal;
    // Retain the typed snapshot/contract cause without another error vocabulary.
    type Error = io::Error;

    fn id(&self) -> &FixtureRecipeId {
        &self.id
    }

    fn reset_requirements(&self) -> &ResetRequirements {
        &self.requirements
    }

    fn build(&self) -> Result<CachedPocketIcBaseline<Principal>, io::Error> {
        let (pocket_ic, canister_id) = (self.build)().into_parts();
        CachedPocketIcBaseline::capture(
            pocket_ic,
            [CanisterSnapshotTarget::new(canister_id, None)],
            canister_id,
        )
        .map_err(io::Error::other)
    }

    fn restore_canisters(
        &self,
        baseline: &CachedPocketIcBaseline<Principal>,
    ) -> Result<CanisterRestoreReceipt, io::Error> {
        baseline.restore(self.funding).map_err(io::Error::other)?;
        CanisterRestoreReceipt::try_from_baseline(baseline, self.requirements.cycle_policy())
            .map_err(io::Error::other)
    }

    fn reset_non_snapshot_state(
        &self,
        _baseline: &CachedPocketIcBaseline<Principal>,
    ) -> Result<ResetReceipt, io::Error> {
        Ok(ResetReceipt::empty())
    }

    fn drive_to_readiness(
        &self,
        _baseline: &CachedPocketIcBaseline<Principal>,
    ) -> Result<ReadinessReceipt, io::Error> {
        ReadinessReceipt::try_new("snapshot-operation-completed").map_err(io::Error::other)
    }

    fn validate(
        &self,
        _baseline: &CachedPocketIcBaseline<Principal>,
        _preparation: &PreparedBaseline,
    ) -> Result<ValidationReceipt, io::Error> {
        ValidationReceipt::try_new(self.id.clone(), "single-canister-snapshot-completed")
            .map_err(io::Error::other)
    }

    fn classify_failure(
        &self,
        _stage: super::BaselinePreparationStage,
        error: &io::Error,
    ) -> FailureDisposition {
        if is_dead_pocket_ic_transport_error(error) {
            FailureDisposition::Rebuild(RebuildReason::DeadPocketIcTransport)
        } else {
            FailureDisposition::Fatal
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use crate::pic::CachedPocketIcBaselinePool;

    fn build_fixture() -> StandaloneCanisterFixture {
        panic!("constructing an empty pool must not invoke its builder")
    }

    #[test]
    fn runtime_capacity_preserves_lazy_fixture_construction() {
        for capacity in [1, 2, 4, 8].map(|value| NonZeroUsize::new(value).unwrap()) {
            let recipe = StandaloneCanisterRecipe::new(
                FixtureRecipeId::try_new("lazy-single-canister").unwrap(),
                SnapshotRestoreFunding::TopUpTo { minimum_cycles: 1 },
                build_fixture,
            );
            let pool = CachedPocketIcBaselinePool::new(capacity, recipe);
            assert_eq!(pool.capacity(), capacity);
        }
    }
}
