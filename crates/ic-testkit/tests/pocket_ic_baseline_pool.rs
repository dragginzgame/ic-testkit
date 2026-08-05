use std::{
    num::NonZeroUsize,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use candid::Principal;
use ic_testkit::pic::{
    BaselinePoolContractError, BaselinePoolError, BaselinePoolOutcome,
    BaselinePoolPreparationError, CachedPocketIcBaseline, CachedPocketIcBaselinePool,
    CanisterRestoreReceipt, ControllerSnapshotError, CycleResetPolicy, FixtureRecipeId, PocketIc,
    PocketIcBaselineRecipe, PreparedBaseline, ReadinessReceipt, RebuildReason, ResetReceipt,
    ResetRequirement, ResetRequirements, ValidationReceipt,
};

const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Default)]
struct RecipeControls {
    builds: Arc<AtomicUsize>,
    fail_next_build: Arc<AtomicBool>,
    fail_next_restore: Arc<AtomicBool>,
    panic_next_restore: Arc<AtomicBool>,
    report_incomplete_restore: Arc<AtomicBool>,
    report_wrong_validation_recipe: Arc<AtomicBool>,
}

struct TwoCanisterRecipe {
    id: FixtureRecipeId,
    requirements: ResetRequirements,
    controls: RecipeControls,
}

struct TwoCanisterMetadata {
    canister_ids: [Principal; 2],
}

#[derive(Debug)]
enum TestRecipeError {
    Contract(BaselinePoolContractError),
    Snapshot(ControllerSnapshotError),
    Synthetic(&'static str),
}

impl TwoCanisterRecipe {
    fn new(controls: RecipeControls) -> Self {
        Self {
            id: FixtureRecipeId::try_new("ic-testkit/two-empty-canisters/v1").unwrap(),
            requirements: ResetRequirements::try_new([
                ResetRequirement::CanisterSnapshots,
                ResetRequirement::CanisterCycles(CycleResetPolicy::PreserveCurrent),
            ])
            .unwrap(),
            controls,
        }
    }
}

impl PocketIcBaselineRecipe for TwoCanisterRecipe {
    type Error = TestRecipeError;
    type Metadata = TwoCanisterMetadata;

    fn id(&self) -> &FixtureRecipeId {
        &self.id
    }

    fn reset_requirements(&self) -> &ResetRequirements {
        &self.requirements
    }

    fn build(&self) -> Result<CachedPocketIcBaseline<Self::Metadata>, Self::Error> {
        self.controls.builds.fetch_add(1, Ordering::SeqCst);
        if self.controls.fail_next_build.swap(false, Ordering::SeqCst) {
            return Err(TestRecipeError::Synthetic("requested build failure"));
        }

        let pocket_ic = PocketIc::new();
        let canister_ids = [pocket_ic.create_canister(), pocket_ic.create_canister()];
        for canister_id in canister_ids {
            pocket_ic.install_canister(canister_id, EMPTY_WASM.to_vec(), vec![], None);
        }

        CachedPocketIcBaseline::capture(
            pocket_ic,
            Principal::anonymous(),
            canister_ids,
            TwoCanisterMetadata { canister_ids },
        )
        .map_err(Into::into)
    }

    fn restore_canisters(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<CanisterRestoreReceipt, Self::Error> {
        assert!(
            !self
                .controls
                .panic_next_restore
                .swap(false, Ordering::SeqCst),
            "requested recipe-hook panic"
        );
        if self
            .controls
            .fail_next_restore
            .swap(false, Ordering::SeqCst)
        {
            return Err(TestRecipeError::Synthetic("requested restore failure"));
        }

        baseline.restore(Principal::anonymous())?;
        let canister_ids = if self
            .controls
            .report_incomplete_restore
            .swap(false, Ordering::SeqCst)
        {
            baseline.metadata().canister_ids[..1].to_vec()
        } else {
            baseline.metadata().canister_ids.to_vec()
        };
        CanisterRestoreReceipt::try_new(canister_ids, CycleResetPolicy::PreserveCurrent)
            .map_err(Into::into)
    }

    fn reset_non_snapshot_state(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ResetReceipt, Self::Error> {
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
                .map_err(|_| TestRecipeError::Synthetic("canister status failed"))?;
            if status.module_hash.is_none() {
                return Err(TestRecipeError::Synthetic(
                    "baseline canister has no installed module",
                ));
            }
        }

        let recipe_id = if self
            .controls
            .report_wrong_validation_recipe
            .swap(false, Ordering::SeqCst)
        {
            FixtureRecipeId::try_new("ic-testkit/wrong-recipe/v1")?
        } else {
            self.id.clone()
        };
        ValidationReceipt::try_new(recipe_id, "two-empty-modules-installed").map_err(Into::into)
    }
}

impl From<BaselinePoolContractError> for TestRecipeError {
    fn from(error: BaselinePoolContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<ControllerSnapshotError> for TestRecipeError {
    fn from(error: ControllerSnapshotError) -> Self {
        Self::Snapshot(error)
    }
}

impl std::fmt::Display for TestRecipeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
            Self::Synthetic(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for TestRecipeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Contract(error) => Some(error),
            Self::Snapshot(error) => Some(error),
            Self::Synthetic(_) => None,
        }
    }
}

#[test]
fn multi_canister_pool_restores_and_explicitly_rebuilds_one_slot() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    assert_eq!(pool.capacity().get(), 1);

    let (baseline, outcome) = pool.acquire().expect("first baseline should build");
    let timings = outcome.timings();
    assert!(timings.build().is_some());
    assert!(timings.restore().is_none());
    assert!(timings.validation().is_some());
    assert!(matches!(outcome, BaselinePoolOutcome::Built { .. }));
    let first_instance = baseline.pocket_ic().instance_id();
    let canister_ids = baseline.metadata().canister_ids;
    baseline
        .pocket_ic()
        .uninstall_canister(canister_ids[0], None)
        .expect("test should mutate one captured canister");
    drop(baseline);

    let (mut baseline, outcome) = pool.acquire().expect("baseline should restore");
    let timings = outcome.timings();
    assert!(timings.build().is_none());
    assert!(timings.restore().is_some());
    assert!(timings.reset().is_some());
    assert!(timings.readiness().is_some());
    assert!(timings.validation().is_some());
    assert!(matches!(outcome, BaselinePoolOutcome::Restored { .. }));
    assert_eq!(baseline.pocket_ic().instance_id(), first_instance);
    for canister_id in canister_ids {
        assert!(
            baseline
                .pocket_ic()
                .canister_status(canister_id, None)
                .expect("restored canister status should remain readable")
                .module_hash
                .is_some(),
            "snapshot restore should recover every installed module",
        );
    }
    baseline.invalidate(RebuildReason::ExplicitLeaseInvalidation);
    drop(baseline);

    let (baseline, outcome) = pool.acquire().expect("invalid slot should rebuild once");
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::ExplicitLeaseInvalidation,
            ..
        }
    ));
    assert_ne!(baseline.pocket_ic().instance_id(), first_instance);
    assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
}

#[test]
fn reused_slot_failure_rebuilds_once_and_preserves_a_failed_recovery() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("first baseline should build").0);

    controls.fail_next_restore.store(true, Ordering::SeqCst);
    let (baseline, outcome) = pool.acquire().expect("restore failure should rebuild");
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::SnapshotRestoreFailure,
            ..
        }
    ));
    drop(baseline);

    controls.fail_next_restore.store(true, Ordering::SeqCst);
    controls.fail_next_build.store(true, Ordering::SeqCst);
    let Err(error) = pool.acquire() else {
        panic!("a failed restore plus failed rebuild must be returned");
    };
    assert!(matches!(
        error,
        BaselinePoolError::RecoveryFailed {
            original,
            rebuild,
        } if matches!(
            *original,
            BaselinePoolPreparationError::Recipe { .. }
        ) && matches!(
            *rebuild,
            BaselinePoolPreparationError::Recipe { .. }
        )
    ));
}

#[test]
fn incomplete_restore_receipt_forces_a_fresh_baseline() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("first baseline should build").0);

    controls
        .report_incomplete_restore
        .store(true, Ordering::SeqCst);
    let (baseline, outcome) = pool
        .acquire()
        .expect("incomplete restore evidence should rebuild safely");
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::ResetCoverageMismatch,
            ..
        }
    ));
    assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
    drop(baseline);
}

#[test]
fn validation_recipe_mismatch_is_fatal_and_not_retried_as_a_rebuild() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("first baseline should build").0);

    controls
        .report_wrong_validation_recipe
        .store(true, Ordering::SeqCst);
    let Err(error) = pool.acquire() else {
        panic!("a mismatched validation recipe must be rejected");
    };
    assert!(matches!(
        error,
        BaselinePoolError::Preparation(BaselinePoolPreparationError::Contract(
            BaselinePoolContractError::RecipeIdentityMismatch { .. }
        ))
    ));
    assert_eq!(
        controls.builds.load(Ordering::SeqCst),
        1,
        "identity misuse must not trigger an automatic rebuild",
    );

    let (baseline, outcome) = pool
        .acquire()
        .expect("the discarded fatal slot should build on a later acquisition");
    assert!(matches!(outcome, BaselinePoolOutcome::Built { .. }));
    assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
    drop(baseline);
}

#[test]
fn every_recipe_must_declare_snapshot_and_cycle_reset_domains() {
    let error = ResetRequirements::try_new([ResetRequirement::CanisterSnapshots])
        .expect_err("an incomplete reset contract must fail during construction");
    assert!(matches!(
        error,
        BaselinePoolContractError::UndeclaredRequiredResetDomain {
            domain: ic_testkit::pic::ResetDomainKind::CanisterCycles,
        }
    ));
}

#[test]
fn panic_while_leased_invalidates_without_hiding_the_panic() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("first baseline should build").0);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let (_baseline, _outcome) = pool
            .acquire()
            .expect("baseline should restore before panic");
        panic!("synthetic test panic");
    }));
    assert!(panic.is_err(), "the caller panic must keep unwinding");

    let (baseline, outcome) = pool
        .acquire()
        .expect("slot invalidated by unwind should rebuild");
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::UnwindWhileLeased,
            ..
        }
    ));
    assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
    drop(baseline);
}

#[test]
fn recipe_hook_panic_invalidates_without_becoming_a_cache_miss() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("first baseline should build").0);

    controls.panic_next_restore.store(true, Ordering::SeqCst);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = pool.acquire();
    }));
    assert!(panic.is_err(), "the recipe panic must keep unwinding");
    assert_eq!(
        controls.builds.load(Ordering::SeqCst),
        1,
        "the panicking acquisition must not silently rebuild",
    );

    let (baseline, outcome) = pool
        .acquire()
        .expect("slot invalidated by recipe panic should rebuild later");
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::UnwindWhileLeased,
            ..
        }
    ));
    assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
    drop(baseline);
}

#[test]
fn runtime_capacity_allows_two_independent_baseline_leases() {
    let pool = Arc::new(CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(2).unwrap(),
        TwoCanisterRecipe::new(RecipeControls::default()),
    ));
    let (first, first_outcome) = pool.acquire().expect("first slot should build");
    assert!(matches!(first_outcome, BaselinePoolOutcome::Built { .. }));

    let worker_pool = Arc::clone(&pool);
    let (acquired_tx, acquired_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let (second, outcome) = worker_pool.acquire().expect("second slot should build");
        acquired_tx
            .send((second.pocket_ic().instance_id(), outcome))
            .expect("capacity result receiver should remain live");
    });

    let (second_instance, second_outcome) = acquired_rx
        .recv_timeout(OPERATION_TIMEOUT)
        .expect("capacity two should not wait for the first lease");
    assert!(matches!(second_outcome, BaselinePoolOutcome::Built { .. }));
    assert_ne!(first.pocket_ic().instance_id(), second_instance);
    drop(first);
    worker.join().expect("capacity worker should exit cleanly");
}

#[test]
fn one_slot_survives_one_hundred_consecutive_restores() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("stress baseline should build").0);

    for iteration in 0..100 {
        let (baseline, outcome) = pool
            .acquire()
            .unwrap_or_else(|error| panic!("stress restore {iteration} should succeed: {error}"));
        assert!(matches!(outcome, BaselinePoolOutcome::Restored { .. }));

        let canister_id = baseline.metadata().canister_ids[iteration % 2];
        baseline
            .pocket_ic()
            .uninstall_canister(canister_id, None)
            .unwrap_or_else(|error| panic!("stress mutation {iteration} failed: {error}"));
        drop(baseline);
    }

    assert_eq!(
        controls.builds.load(Ordering::SeqCst),
        1,
        "successful restores should not reconstruct the slot",
    );
}
