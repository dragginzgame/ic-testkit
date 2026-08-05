//! Complete two-canister recipe for `CachedPocketIcBaselinePool`.
//!
//! Run with:
//!
//! ```text
//! cargo run -p ic-testkit --example multi_canister_baseline_pool
//! ```

use std::{error::Error, fmt, num::NonZeroUsize};

use candid::Principal;
use ic_testkit::pic::{
    BaselinePoolContractError, BaselinePoolOutcome, BaselinePreparationStage,
    CachedPocketIcBaseline, CachedPocketIcBaselinePool, CanisterRestoreReceipt,
    ControllerSnapshotError, CycleResetPolicy, FailureDisposition, FixtureRecipeId, PocketIc,
    PocketIcBaselineRecipe, PreparedBaseline, ReadinessReceipt, RebuildReason, ResetReceipt,
    ResetRequirement, ResetRequirements, ValidationReceipt, is_dead_pocket_ic_transport_error,
};

const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

struct TwoCanisterRecipe {
    id: FixtureRecipeId,
    requirements: ResetRequirements,
}

struct TopologyMetadata {
    canister_ids: [Principal; 2],
}

#[derive(Debug)]
enum RecipeError {
    Contract(BaselinePoolContractError),
    Snapshot(ControllerSnapshotError),
    Validation(&'static str),
}

impl TwoCanisterRecipe {
    fn new() -> Result<Self, BaselinePoolContractError> {
        Ok(Self {
            id: FixtureRecipeId::try_new("example/two-empty-canisters/v1")?,
            requirements: ResetRequirements::try_new([
                ResetRequirement::CanisterSnapshots,
                ResetRequirement::CanisterCycles(CycleResetPolicy::PreserveCurrent),
            ])?,
        })
    }
}

impl PocketIcBaselineRecipe for TwoCanisterRecipe {
    type Metadata = TopologyMetadata;
    type Error = RecipeError;

    fn id(&self) -> &FixtureRecipeId {
        &self.id
    }

    fn reset_requirements(&self) -> &ResetRequirements {
        &self.requirements
    }

    fn build(&self) -> Result<CachedPocketIcBaseline<Self::Metadata>, Self::Error> {
        let pocket_ic = PocketIc::new();
        let canister_ids = [pocket_ic.create_canister(), pocket_ic.create_canister()];
        for canister_id in canister_ids {
            pocket_ic.install_canister(canister_id, EMPTY_WASM.to_vec(), vec![], None);
        }

        CachedPocketIcBaseline::capture(
            pocket_ic,
            Principal::anonymous(),
            canister_ids,
            TopologyMetadata { canister_ids },
        )
        .map_err(Into::into)
    }

    fn restore_canisters(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<CanisterRestoreReceipt, Self::Error> {
        baseline.restore(Principal::anonymous())?;
        CanisterRestoreReceipt::try_from_baseline(baseline, CycleResetPolicy::PreserveCurrent)
            .map_err(Into::into)
    }

    fn reset_non_snapshot_state(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ResetReceipt, Self::Error> {
        // This recipe is suitable only for tests whose relevant mutations are
        // fully contained in the two captured canisters.
        Ok(ResetReceipt::empty())
    }

    fn drive_to_readiness(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ReadinessReceipt, Self::Error> {
        ReadinessReceipt::try_new("empty-canisters-ready").map_err(Into::into)
    }

    fn validate(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
        _preparation: &PreparedBaseline,
    ) -> Result<ValidationReceipt, Self::Error> {
        for canister_id in baseline.metadata().canister_ids {
            let status = baseline
                .pocket_ic()
                .canister_status(canister_id, None)
                .map_err(|_| RecipeError::Validation("canister status failed"))?;
            if status.module_hash.is_none() {
                return Err(RecipeError::Validation(
                    "captured canister has no installed module",
                ));
            }
        }

        ValidationReceipt::try_new(self.id.clone(), "two-empty-modules-installed")
            .map_err(Into::into)
    }

    fn classify_failure(
        &self,
        stage: BaselinePreparationStage,
        error: &Self::Error,
    ) -> FailureDisposition {
        if is_dead_pocket_ic_transport_error(error) {
            FailureDisposition::Rebuild(RebuildReason::DeadPocketIcTransport)
        } else {
            FailureDisposition::Rebuild(stage.default_rebuild_reason())
        }
    }
}

impl From<BaselinePoolContractError> for RecipeError {
    fn from(error: BaselinePoolContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<ControllerSnapshotError> for RecipeError {
    fn from(error: ControllerSnapshotError) -> Self {
        Self::Snapshot(error)
    }
}

impl fmt::Display for RecipeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
            Self::Validation(message) => formatter.write_str(message),
        }
    }
}

impl Error for RecipeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Contract(error) => Some(error),
            Self::Snapshot(error) => Some(error),
            Self::Validation(_) => None,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).expect("one is nonzero"),
        TwoCanisterRecipe::new()?,
    );

    let (baseline, first) = pool.acquire()?;
    assert!(matches!(first, BaselinePoolOutcome::Built { .. }));
    println!("first acquisition: {first:?}");
    drop(baseline);

    let (baseline, second) = pool.acquire()?;
    assert!(matches!(second, BaselinePoolOutcome::Restored { .. }));
    println!("second acquisition: {second:?}");
    drop(baseline);

    Ok(())
}
