use std::{
    num::NonZeroUsize,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use candid::Principal;
use ic_testkit::pic::{
    BaselinePoolContractError, BaselinePoolError, BaselinePoolOutcome,
    BaselinePoolPreparationError, BaselinePreparationStage, CachedPocketIcBaseline,
    CachedPocketIcBaselinePool, CanisterRestoreReceipt, ControllerSnapshotError, CycleResetPolicy,
    ExtraCanisterPolicy, FailureDisposition, FixtureRecipeId, PocketIc, PocketIcBaselineRecipe,
    PocketIcBuilder, PocketIcBuilderExt, PocketIcStartupError, PreparedBaseline, ReadinessReceipt,
    RebuildReason, ResetAchievement, ResetReceipt, ResetRequirement, ResetRequirements,
    TimeResetPolicy, ValidationReceipt, is_dead_pocket_ic_transport_error,
};

const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Default)]
struct RecipeControls {
    builds: Arc<AtomicUsize>,
    built_validations: Arc<AtomicUsize>,
    restored_validations: Arc<AtomicUsize>,
    fail_next_build: Arc<AtomicBool>,
    fail_next_restore: Arc<AtomicBool>,
    fail_next_reset: Arc<AtomicBool>,
    fail_next_readiness: Arc<AtomicBool>,
    fail_next_validation: Arc<AtomicBool>,
    panic_next_restore: Arc<AtomicBool>,
    report_incomplete_restore: Arc<AtomicBool>,
    report_wrong_validation_recipe: Arc<AtomicBool>,
    tracked_extra_canister: Arc<Mutex<Option<Principal>>>,
    first_server_url: Arc<Mutex<Option<String>>>,
}

struct TwoCanisterRecipe {
    id: FixtureRecipeId,
    requirements: ResetRequirements,
    controls: RecipeControls,
    guarded_domain: Option<GuardedDomain>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum GuardedDomain {
    Time,
    Cycles,
    ExtraCanisters,
}

struct TwoCanisterMetadata {
    canister_ids: [Principal; 2],
    baseline_time_nanos: u64,
    baseline_cycles: [u128; 2],
}

#[derive(Debug)]
enum TestRecipeError {
    Contract(BaselinePoolContractError),
    Snapshot(ControllerSnapshotError),
    Startup(PocketIcStartupError),
    DomainMutation(&'static str),
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
            guarded_domain: None,
        }
    }

    fn guarding_domain(controls: RecipeControls, domain: GuardedDomain) -> Self {
        let (identity, requirements) = match domain {
            GuardedDomain::Time => (
                "ic-testkit/two-empty-canisters-time-guarded/v1",
                vec![
                    ResetRequirement::CanisterSnapshots,
                    ResetRequirement::CanisterCycles(CycleResetPolicy::PreserveCurrent),
                    ResetRequirement::PocketIcTime(TimeResetPolicy::RebuildOnMutation),
                ],
            ),
            GuardedDomain::Cycles => (
                "ic-testkit/two-empty-canisters-cycle-guarded/v1",
                vec![
                    ResetRequirement::CanisterSnapshots,
                    ResetRequirement::CanisterCycles(CycleResetPolicy::RebuildOnMutation),
                ],
            ),
            GuardedDomain::ExtraCanisters => (
                "ic-testkit/two-empty-canisters-extra-guarded/v1",
                vec![
                    ResetRequirement::CanisterSnapshots,
                    ResetRequirement::CanisterCycles(CycleResetPolicy::PreserveCurrent),
                    ResetRequirement::ExtraCanisters(ExtraCanisterPolicy::RebuildOnChange),
                ],
            ),
        };
        Self {
            id: FixtureRecipeId::try_new(identity).unwrap(),
            requirements: ResetRequirements::try_new(requirements).unwrap(),
            controls,
            guarded_domain: Some(domain),
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
        *self
            .controls
            .tracked_extra_canister
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

        let first_server_url = self
            .controls
            .first_server_url
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let pocket_ic = if let Some(server_url) = first_server_url {
            PocketIcBuilder::new()
                .with_server_url(
                    server_url
                        .parse()
                        .map_err(|_| TestRecipeError::Synthetic("invalid dedicated server URL"))?,
                )
                .with_application_subnet()
                .try_build()
                .map_err(TestRecipeError::Startup)?
        } else {
            PocketIc::new()
        };
        let canister_ids = [pocket_ic.create_canister(), pocket_ic.create_canister()];
        for canister_id in canister_ids {
            pocket_ic.install_canister(canister_id, EMPTY_WASM.to_vec(), vec![], None);
        }

        let mut baseline = CachedPocketIcBaseline::capture(
            pocket_ic,
            Principal::anonymous(),
            canister_ids,
            TwoCanisterMetadata {
                canister_ids,
                baseline_time_nanos: 0,
                baseline_cycles: [0; 2],
            },
        )
        .map_err(TestRecipeError::from)?;
        let baseline_time_nanos = baseline.pocket_ic().get_time().as_nanos_since_unix_epoch();
        let baseline_cycles = canister_ids.map(|id| baseline.pocket_ic().cycle_balance(id));
        let metadata = baseline.metadata_mut();
        metadata.baseline_time_nanos = baseline_time_nanos;
        metadata.baseline_cycles = baseline_cycles;
        Ok(baseline)
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
        if self.guarded_domain == Some(GuardedDomain::Cycles) {
            let current_cycles = baseline
                .metadata()
                .canister_ids
                .map(|id| baseline.pocket_ic().cycle_balance(id));
            if current_cycles != baseline.metadata().baseline_cycles {
                return Err(TestRecipeError::DomainMutation("canister-cycles"));
            }
        }
        if self
            .controls
            .fail_next_restore
            .swap(false, Ordering::SeqCst)
        {
            return Err(TestRecipeError::Synthetic("requested restore failure"));
        }

        baseline.restore(Principal::anonymous())?;
        if self
            .controls
            .report_incomplete_restore
            .swap(false, Ordering::SeqCst)
        {
            return CanisterRestoreReceipt::try_new(
                baseline.metadata().canister_ids[..1].iter().copied(),
                CycleResetPolicy::PreserveCurrent,
            )
            .map_err(Into::into);
        }
        let cycle_policy = if self.guarded_domain == Some(GuardedDomain::Cycles) {
            CycleResetPolicy::RebuildOnMutation
        } else {
            CycleResetPolicy::PreserveCurrent
        };
        CanisterRestoreReceipt::try_from_baseline(baseline, cycle_policy).map_err(Into::into)
    }

    fn reset_non_snapshot_state(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ResetReceipt, Self::Error> {
        if self.controls.fail_next_reset.swap(false, Ordering::SeqCst) {
            return Err(TestRecipeError::Synthetic("requested reset failure"));
        }
        match self.guarded_domain {
            None | Some(GuardedDomain::Cycles) => Ok(ResetReceipt::empty()),
            Some(GuardedDomain::Time) => {
                if baseline.pocket_ic().get_time().as_nanos_since_unix_epoch()
                    != baseline.metadata().baseline_time_nanos
                {
                    return Err(TestRecipeError::DomainMutation("pocket-ic-time"));
                }
                ResetReceipt::try_new([ResetAchievement::PocketIcTime(
                    TimeResetPolicy::RebuildOnMutation,
                )])
                .map_err(Into::into)
            }
            Some(GuardedDomain::ExtraCanisters) => {
                let extra_canister = *self
                    .controls
                    .tracked_extra_canister
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if extra_canister.is_some_and(|canister_id| {
                    baseline
                        .pocket_ic()
                        .canister_status(canister_id, None)
                        .is_ok()
                }) {
                    return Err(TestRecipeError::DomainMutation("extra-canister"));
                }
                ResetReceipt::try_new([ResetAchievement::ExtraCanisters(
                    ExtraCanisterPolicy::RebuildOnChange,
                )])
                .map_err(Into::into)
            }
        }
    }

    fn drive_to_readiness(
        &self,
        _baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ReadinessReceipt, Self::Error> {
        if self
            .controls
            .fail_next_readiness
            .swap(false, Ordering::SeqCst)
        {
            return Err(TestRecipeError::Synthetic("requested readiness failure"));
        }
        ReadinessReceipt::try_new("empty-canisters-ready").map_err(Into::into)
    }

    fn validate(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
        preparation: &PreparedBaseline,
    ) -> Result<ValidationReceipt, Self::Error> {
        if self
            .controls
            .fail_next_validation
            .swap(false, Ordering::SeqCst)
        {
            return Err(TestRecipeError::Synthetic("requested validation failure"));
        }
        match preparation {
            PreparedBaseline::Built => {
                self.controls
                    .built_validations
                    .fetch_add(1, Ordering::SeqCst);
            }
            PreparedBaseline::Restored { .. } => {
                self.controls
                    .restored_validations
                    .fetch_add(1, Ordering::SeqCst);
            }
            _ => {}
        }
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

    fn classify_failure(
        &self,
        stage: BaselinePreparationStage,
        error: &Self::Error,
    ) -> FailureDisposition {
        if is_dead_pocket_ic_transport_error(error) {
            FailureDisposition::Rebuild(RebuildReason::DeadPocketIcTransport)
        } else if let TestRecipeError::DomainMutation(code) = error {
            FailureDisposition::Rebuild(RebuildReason::RecipeClassified {
                code: (*code).to_string(),
            })
        } else {
            FailureDisposition::Rebuild(stage.default_rebuild_reason())
        }
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
            Self::Startup(error) => error.fmt(formatter),
            Self::DomainMutation(domain) => write!(formatter, "{domain} mutated while leased"),
            Self::Synthetic(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for TestRecipeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Contract(error) => Some(error),
            Self::Snapshot(error) => Some(error),
            Self::Startup(error) => Some(error),
            Self::DomainMutation(_) | Self::Synthetic(_) => None,
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
    assert!(matches!(&outcome, BaselinePoolOutcome::Restored { .. }));
    assert_eq!(controls.built_validations.load(Ordering::SeqCst), 1);
    assert_eq!(controls.restored_validations.load(Ordering::SeqCst), 1);
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
    assert_eq!(controls.built_validations.load(Ordering::SeqCst), 2);
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
    let timings = error.timings();
    assert!(timings.restore().is_some());
    assert!(timings.stale_teardown().is_some());
    assert!(timings.build().is_some());
    assert!(timings.total() >= timings.restore().unwrap());
    assert!(matches!(
        &error,
        BaselinePoolError::RecoveryFailed {
            original,
            rebuild,
            ..
        } if matches!(
            **original,
            BaselinePoolPreparationError::Recipe { .. }
        ) && matches!(
            **rebuild,
            BaselinePoolPreparationError::Recipe { .. }
        )
    ));
}

#[test]
fn failed_initial_build_reports_partial_timings() {
    let controls = RecipeControls::default();
    controls.fail_next_build.store(true, Ordering::SeqCst);
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls),
    );

    let Err(error) = pool.acquire() else {
        panic!("the requested initial build failure must be returned");
    };
    let timings = error.timings();
    assert!(timings.build().is_some());
    assert!(timings.restore().is_none());
    assert!(timings.validation().is_none());
    assert!(timings.total() >= timings.build().unwrap());
    assert!(matches!(
        error,
        BaselinePoolError::Preparation {
            error: BaselinePoolPreparationError::Recipe {
                stage: BaselinePreparationStage::Build,
                ..
            },
            ..
        }
    ));
}

#[test]
fn reset_readiness_and_validation_failures_have_distinct_rebuild_reasons() {
    let controls = RecipeControls::default();
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(pool.acquire().expect("first baseline should build").0);

    for (flag, expected) in [
        (&controls.fail_next_reset, RebuildReason::ResetFailure),
        (
            &controls.fail_next_readiness,
            RebuildReason::ReadinessFailure,
        ),
        (
            &controls.fail_next_validation,
            RebuildReason::InvariantValidationFailure,
        ),
    ] {
        flag.store(true, Ordering::SeqCst);
        let (baseline, outcome) = pool
            .acquire()
            .expect("classified preparation failure should rebuild once");
        assert!(matches!(
            outcome,
            BaselinePoolOutcome::Rebuilt { reason, .. } if reason == expected
        ));
        drop(baseline);
    }

    assert_eq!(controls.builds.load(Ordering::SeqCst), 4);
}

#[test]
#[ignore = "launches and kills a dedicated PocketIC server; run explicitly in isolation"]
fn killed_dedicated_server_rebuilds_on_a_fresh_server() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("dedicated server runtime should build");
    let (mut server, server_url) =
        runtime.block_on(pocket_ic::start_server(pocket_ic::StartServerParams {
            reuse: false,
            hard_ttl: Some(Duration::from_secs(60)),
            ..pocket_ic::StartServerParams::default()
        }));

    let controls = RecipeControls::default();
    *controls
        .first_server_url
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(server_url.to_string());
    let pool = CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(controls.clone()),
    );
    drop(
        pool.acquire()
            .expect("dedicated-server baseline should build")
            .0,
    );

    server
        .kill()
        .expect("the test-owned PocketIC server should stop");
    server
        .wait()
        .expect("the test-owned PocketIC server should be reaped");

    let (baseline, outcome) = pool
        .acquire()
        .expect("dead dedicated transport should rebuild on the default server");
    assert!(matches!(
        outcome,
        BaselinePoolOutcome::Rebuilt {
            reason: RebuildReason::DeadPocketIcTransport,
            ..
        }
    ));
    assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
    drop(baseline);
}

#[test]
fn guarded_recipe_rebuilds_after_time_cycle_and_extra_canister_mutations() {
    {
        let controls = RecipeControls::default();
        let pool = guarded_pool(controls.clone(), GuardedDomain::Time);
        let (baseline, _) = pool.acquire().expect("time-guarded baseline should build");
        baseline.pocket_ic().advance_time(Duration::from_secs(1));
        drop(baseline);

        let (baseline, outcome) = pool
            .acquire()
            .expect("time mutation should rebuild the guarded slot");
        assert_rebuilt_for_domain(&outcome, "pocket-ic-time");
        assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
        drop(baseline);
    }

    {
        let controls = RecipeControls::default();
        let pool = guarded_pool(controls.clone(), GuardedDomain::Cycles);
        let (baseline, _) = pool.acquire().expect("cycle-guarded baseline should build");
        let canister_id = baseline.metadata().canister_ids[0];
        let _ = baseline.pocket_ic().add_cycles(canister_id, 1_000_000);
        drop(baseline);

        let (baseline, outcome) = pool
            .acquire()
            .expect("cycle mutation should rebuild the guarded slot");
        assert_rebuilt_for_domain(&outcome, "canister-cycles");
        assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
        drop(baseline);
    }

    {
        let controls = RecipeControls::default();
        let pool = guarded_pool(controls.clone(), GuardedDomain::ExtraCanisters);
        let (baseline, _) = pool
            .acquire()
            .expect("extra-canister-guarded baseline should build");
        let extra_canister = baseline.pocket_ic().create_canister();
        *controls
            .tracked_extra_canister
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(extra_canister);
        drop(baseline);

        let (baseline, outcome) = pool
            .acquire()
            .expect("extra canister should rebuild the guarded slot");
        assert_rebuilt_for_domain(&outcome, "extra-canister");
        assert_eq!(controls.builds.load(Ordering::SeqCst), 2);
        drop(baseline);
    }
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
        &error,
        BaselinePoolError::Preparation {
            error: BaselinePoolPreparationError::Contract(
                BaselinePoolContractError::RecipeIdentityMismatch { .. }
            ),
            ..
        }
    ));
    let timings = error.timings();
    assert!(timings.validation().is_some());
    assert!(timings.stale_teardown().is_some());
    assert!(timings.total() >= timings.validation().unwrap());
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
fn capacity_one_reports_time_waiting_for_the_held_slot() {
    let pool = Arc::new(CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::new(RecipeControls::default()),
    ));
    let (first, _) = pool.acquire().expect("first slot should build");

    let worker_pool = Arc::clone(&pool);
    let (attempting_tx, attempting_rx) = mpsc::channel();
    let (acquired_tx, acquired_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        attempting_tx
            .send(())
            .expect("wait-timing coordinator should remain live");
        let (baseline, outcome) = worker_pool.acquire().expect("held slot should restore");
        acquired_tx
            .send((baseline.pocket_ic().instance_id(), outcome))
            .expect("wait-timing receiver should remain live");
    });

    attempting_rx
        .recv_timeout(OPERATION_TIMEOUT)
        .expect("worker should begin its acquisition");
    assert!(
        acquired_rx
            .recv_timeout(Duration::from_millis(100))
            .is_err(),
        "capacity one must not release a second lease while the first is held",
    );
    let first_instance = first.pocket_ic().instance_id();
    drop(first);

    let (second_instance, outcome) = acquired_rx
        .recv_timeout(OPERATION_TIMEOUT)
        .expect("waiting acquisition should complete after release");
    assert_eq!(second_instance, first_instance);
    assert!(matches!(&outcome, BaselinePoolOutcome::Restored { .. }));
    assert!(
        outcome.timings().wait() >= Duration::from_millis(50),
        "reported queue wait should include the observed capacity block",
    );
    worker
        .join()
        .expect("wait-timing worker should exit cleanly");
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

fn assert_rebuilt_for_domain(outcome: &BaselinePoolOutcome, expected: &str) {
    assert!(
        matches!(
            outcome,
            BaselinePoolOutcome::Rebuilt {
                reason: RebuildReason::RecipeClassified { code },
                ..
            } if code == expected
        ),
        "expected rebuild for {expected}, got {outcome:?}"
    );
}

fn guarded_pool(
    controls: RecipeControls,
    domain: GuardedDomain,
) -> CachedPocketIcBaselinePool<TwoCanisterRecipe> {
    CachedPocketIcBaselinePool::new(
        NonZeroUsize::new(1).unwrap(),
        TwoCanisterRecipe::guarding_domain(controls, domain),
    )
}
