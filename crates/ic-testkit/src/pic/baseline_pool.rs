use candid::Principal;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    time::{Duration, Instant},
};

use super::{
    CachedPocketIcBaseline,
    bounded_pool::{BoundedSlotLease, BoundedSlotPool},
};

/// Caller-owned stable identity for one pooled fixture recipe.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FixtureRecipeId(String);

/// Reset domain whose handling is declared by a pooled baseline recipe.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResetDomainKind {
    /// Captured canister snapshots.
    CanisterSnapshots,
    /// Canister cycle balances.
    CanisterCycles,
    /// PocketIC simulated time.
    PocketIcTime,
    /// Canisters outside the captured baseline set.
    ExtraCanisters,
    /// Pending ingress, timers, or cross-canister messages.
    PendingMessages,
    /// Subnet metrics, routing, allocation, or other subnet-global state.
    SubnetState,
    /// Files, processes, services, or other caller-owned resources.
    ExternalResources,
}

/// Cycle handling required or achieved by one reset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CycleResetPolicy {
    /// Do not proactively add or remove cycles before snapshot restoration.
    ///
    /// PocketIC may still charge cycles while performing the restore.
    PreserveCurrent,
    /// Add cycles as needed to reach this minimum immediately before restore,
    /// without removing excess.
    TopUpTo(u128),
    /// Restore the exact balance recorded by the recipe baseline.
    RestoreExactBaseline,
    /// Treat any relevant cycle mutation as requiring slot reconstruction.
    RebuildOnMutation,
}

/// PocketIC time handling required or achieved by one reset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeResetPolicy {
    /// Preserve the current simulator time rather than claiming it was reset.
    PreserveCurrent,
    /// Restore the exact time recorded by the recipe baseline.
    RestoreBaseline,
    /// Treat any relevant time mutation as requiring slot reconstruction.
    RebuildOnMutation,
}

/// Extra-canister handling required or achieved by one reset.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtraCanisterPolicy {
    /// Validate that the baseline canister set is unchanged.
    RequireBaselineSet,
    /// Remove canisters explicitly tracked by the recipe.
    RemoveTracked,
    /// Treat any extra-canister change as requiring slot reconstruction.
    RebuildOnChange,
}

/// Generic handling for reset domains without a more specific policy.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateResetPolicy {
    /// Reset the domain through caller-owned recipe logic.
    ResetByRecipe,
    /// Validate that the domain remained unchanged.
    ValidateUnchanged,
    /// Explicitly declare the domain irrelevant to this recipe's guarantees.
    IrrelevantByRecipeContract,
    /// Treat any relevant change as requiring slot reconstruction.
    RebuildOnChange,
}

/// One reset guarantee required before a baseline may be reused.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResetRequirement {
    /// Restore every captured canister snapshot.
    CanisterSnapshots,
    /// Apply this cycle policy.
    CanisterCycles(CycleResetPolicy),
    /// Apply this time policy.
    PocketIcTime(TimeResetPolicy),
    /// Apply this extra-canister policy.
    ExtraCanisters(ExtraCanisterPolicy),
    /// Apply this pending-message policy.
    PendingMessages(StateResetPolicy),
    /// Apply this subnet-state policy.
    SubnetState(StateResetPolicy),
    /// Apply this external-resource policy.
    ExternalResources(StateResetPolicy),
}

/// One reset guarantee reported as achieved by a recipe.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResetAchievement {
    /// Every captured canister snapshot was restored.
    CanisterSnapshots,
    /// This cycle policy was achieved.
    CanisterCycles(CycleResetPolicy),
    /// This time policy was achieved.
    PocketIcTime(TimeResetPolicy),
    /// This extra-canister policy was achieved.
    ExtraCanisters(ExtraCanisterPolicy),
    /// This pending-message policy was achieved.
    PendingMessages(StateResetPolicy),
    /// This subnet-state policy was achieved.
    SubnetState(StateResetPolicy),
    /// This external-resource policy was achieved.
    ExternalResources(StateResetPolicy),
}

/// Typed reset guarantees required by one fixture recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetRequirements(BTreeMap<ResetDomainKind, ResetRequirement>);

/// Typed reset guarantees achieved by one preparation pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResetReceipt(BTreeMap<ResetDomainKind, ResetAchievement>);

/// Receipt for restoring the recipe's captured canister set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanisterRestoreReceipt {
    canister_ids: Vec<Principal>,
    cycle_policy: CycleResetPolicy,
}

/// Receipt identifying the readiness boundary reached after reset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessReceipt {
    identity: String,
}

/// Receipt proving the recipe's final invariant validation ran successfully.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReceipt {
    recipe_id: FixtureRecipeId,
    invariant_identity: String,
}

/// Contract failure while constructing or verifying recipe reset evidence.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaselinePoolContractError {
    /// Recipe identity was empty or whitespace-only.
    EmptyRecipeIdentity,
    /// A receipt identity was empty or whitespace-only.
    EmptyReceiptIdentity { receipt: &'static str },
    /// A reset domain was declared more than once.
    DuplicateResetDomain { domain: ResetDomainKind },
    /// A restored canister appeared more than once.
    DuplicateCanisterId { canister_id: Principal },
    /// A restore receipt contained no canisters.
    EmptyCanisterSet,
    /// A recipe omitted a reset domain required by every pooled baseline.
    UndeclaredRequiredResetDomain { domain: ResetDomainKind },
    /// The restored canister receipt did not identify the complete snapshot set.
    RestoreCanisterSetMismatch {
        expected: Vec<Principal>,
        actual: Vec<Principal>,
    },
    /// A required reset domain had no matching achievement.
    MissingResetDomain { domain: ResetDomainKind },
    /// A reset achievement did not satisfy the required policy.
    ResetPolicyMismatch {
        requirement: ResetRequirement,
        achievement: ResetAchievement,
    },
    /// Final validation reported a recipe other than the pool-owned recipe.
    RecipeIdentityMismatch {
        expected: FixtureRecipeId,
        actual: FixtureRecipeId,
    },
}

/// Whether validation is observing a newly built or restored baseline.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreparedBaseline {
    /// The recipe just built this baseline.
    Built,
    /// The recipe restored and reset an existing baseline.
    Restored {
        /// Captured canisters restored by the recipe.
        canisters: CanisterRestoreReceipt,
        /// Combined typed reset receipt.
        reset: ResetReceipt,
        /// Readiness boundary reached after reset.
        readiness: ReadinessReceipt,
    },
}

/// Recipe stage associated with a structured preparation failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselinePreparationStage {
    /// Constructing a new baseline.
    Build,
    /// Restoring captured canisters.
    RestoreCanisters,
    /// Resetting state outside the snapshots.
    ResetNonSnapshotState,
    /// Driving the restored topology to readiness.
    DriveToReadiness,
    /// Validating a newly built baseline.
    ValidateBuilt,
    /// Validating a restored baseline.
    ValidateRestored,
}

/// Why an invalid or failed slot was reconstructed.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebuildReason {
    /// PocketIC transport was no longer reachable.
    DeadPocketIcTransport,
    /// Captured snapshot restoration failed.
    SnapshotRestoreFailure,
    /// Non-snapshot reset failed.
    ResetFailure,
    /// Readiness or quiescence could not be established.
    ReadinessFailure,
    /// Required and achieved reset domains did not match.
    ResetCoverageMismatch,
    /// Final invariant validation failed.
    InvariantValidationFailure,
    /// A caller explicitly invalidated its lease.
    ExplicitLeaseInvalidation,
    /// A lease was dropped while its thread was unwinding.
    UnwindWhileLeased,
    /// Recipe-specific structured reason.
    RecipeClassified { code: String },
}

/// Recipe decision for a failed restored-slot preparation stage.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FailureDisposition {
    /// Return the failure without rebuilding during this acquisition.
    Fatal,
    /// Invalidate and rebuild the slot once.
    Rebuild(RebuildReason),
}

/// Timings for one baseline-pool acquisition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BaselinePoolTimings {
    wait: Duration,
    build: Option<Duration>,
    restore: Option<Duration>,
    reset: Option<Duration>,
    readiness: Option<Duration>,
    validation: Option<Duration>,
    stale_teardown: Option<Duration>,
    total: Duration,
}

/// Whether a baseline-pool lease was built, restored, or rebuilt.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaselinePoolOutcome {
    /// An empty slot was constructed and validated.
    Built {
        /// Diagnostic slot index.
        slot: usize,
        /// Acquisition phase timings.
        timings: BaselinePoolTimings,
    },
    /// An existing slot was restored, reset, and validated.
    Restored {
        /// Diagnostic slot index.
        slot: usize,
        /// Acquisition phase timings.
        timings: BaselinePoolTimings,
    },
    /// An invalid or failed slot was reconstructed and validated.
    Rebuilt {
        /// Diagnostic slot index.
        slot: usize,
        /// Reason the previous slot could not be reused.
        reason: RebuildReason,
        /// Acquisition phase timings.
        timings: BaselinePoolTimings,
    },
}

/// One failed recipe or contract stage while preparing a baseline slot.
#[non_exhaustive]
#[derive(Debug)]
pub enum BaselinePoolPreparationError<E> {
    /// Caller-owned recipe logic returned an error.
    Recipe {
        /// Failed lifecycle stage.
        stage: BaselinePreparationStage,
        /// Caller-owned structured source error.
        source: E,
    },
    /// Typed reset or recipe evidence violated the pool contract.
    Contract(BaselinePoolContractError),
}

/// Failure to acquire a validated baseline-pool lease.
#[non_exhaustive]
#[derive(Debug)]
pub enum BaselinePoolError<E> {
    /// Initial construction or a non-rebuilt preparation failed.
    Preparation(BaselinePoolPreparationError<E>),
    /// Reused-slot preparation failed and its one rebuild attempt also failed.
    RecoveryFailed {
        /// Original restore/reset/readiness/validation failure.
        original: Box<BaselinePoolPreparationError<E>>,
        /// Failure while rebuilding or validating the replacement.
        rebuild: Box<BaselinePoolPreparationError<E>>,
    },
}

/// Complete caller-owned lifecycle recipe for one pooled PocketIC baseline.
pub trait PocketIcBaselineRecipe: Send + Sync + 'static {
    /// Metadata retained beside every baseline owned by this recipe.
    type Metadata: Send + 'static;
    /// Structured caller error shared by recipe lifecycle stages.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Stable caller-owned recipe identity.
    fn id(&self) -> &FixtureRecipeId;

    /// Reset guarantees required before an existing slot may be reused.
    fn reset_requirements(&self) -> &ResetRequirements;

    /// Construct and capture one complete baseline.
    fn build(&self) -> Result<CachedPocketIcBaseline<Self::Metadata>, Self::Error>;

    /// Restore every captured canister and report the cycle policy applied.
    fn restore_canisters(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<CanisterRestoreReceipt, Self::Error>;

    /// Reset state not covered by canister snapshots.
    fn reset_non_snapshot_state(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ResetReceipt, Self::Error>;

    /// Drive the topology to the recipe's readiness boundary.
    fn drive_to_readiness(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
    ) -> Result<ReadinessReceipt, Self::Error>;

    /// Validate the same baseline invariants after build and restore.
    fn validate(
        &self,
        baseline: &CachedPocketIcBaseline<Self::Metadata>,
        preparation: &PreparedBaseline,
    ) -> Result<ValidationReceipt, Self::Error>;

    /// Classify a restored-slot recipe failure as fatal or rebuildable.
    fn classify_failure(
        &self,
        stage: BaselinePreparationStage,
        _error: &Self::Error,
    ) -> FailureDisposition {
        FailureDisposition::Rebuild(stage.default_rebuild_reason())
    }
}

/// Caller-owned runtime-capacity pool of independently restorable PocketIC baselines.
///
/// One pool structurally owns one [`PocketIcBaselineRecipe`]. A warm
/// acquisition restores the complete captured canister set, applies the
/// recipe's non-snapshot reset, reaches its readiness boundary, checks typed
/// reset coverage, and validates final invariants before exposing a lease.
/// Each capacity slot owns an independent PocketIC instance.
///
/// Snapshot reuse is not a complete PocketIC rollback. The recipe must account
/// for time, extra canisters, pending messages, subnet state, cycles, and
/// external resources when those domains matter to its tests.
pub struct CachedPocketIcBaselinePool<R>
where
    R: PocketIcBaselineRecipe,
{
    recipe: R,
    slots: BoundedSlotPool<BaselineSlot<R::Metadata>>,
}

struct BaselineSlot<M> {
    baseline: CachedPocketIcBaseline<M>,
    invalidation_reason: Option<RebuildReason>,
}

/// Exclusive lease of one validated pooled PocketIC baseline.
pub struct CachedPocketIcBaselinePoolGuard<'a, R>
where
    R: PocketIcBaselineRecipe,
{
    slot: BoundedSlotLease<'a, BaselineSlot<R::Metadata>>,
}

impl FixtureRecipeId {
    /// Construct a nonempty caller-owned stable recipe identity.
    pub fn try_new(identity: impl Into<String>) -> Result<Self, BaselinePoolContractError> {
        let identity = identity.into();
        if identity.trim().is_empty() {
            return Err(BaselinePoolContractError::EmptyRecipeIdentity);
        }
        Ok(Self(identity))
    }

    /// Borrow the recipe identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResetRequirement {
    /// Domain governed by this requirement.
    #[must_use]
    pub const fn domain(&self) -> ResetDomainKind {
        match self {
            Self::CanisterSnapshots => ResetDomainKind::CanisterSnapshots,
            Self::CanisterCycles(_) => ResetDomainKind::CanisterCycles,
            Self::PocketIcTime(_) => ResetDomainKind::PocketIcTime,
            Self::ExtraCanisters(_) => ResetDomainKind::ExtraCanisters,
            Self::PendingMessages(_) => ResetDomainKind::PendingMessages,
            Self::SubnetState(_) => ResetDomainKind::SubnetState,
            Self::ExternalResources(_) => ResetDomainKind::ExternalResources,
        }
    }
}

impl ResetAchievement {
    /// Domain governed by this achievement.
    #[must_use]
    pub const fn domain(&self) -> ResetDomainKind {
        match self {
            Self::CanisterSnapshots => ResetDomainKind::CanisterSnapshots,
            Self::CanisterCycles(_) => ResetDomainKind::CanisterCycles,
            Self::PocketIcTime(_) => ResetDomainKind::PocketIcTime,
            Self::ExtraCanisters(_) => ResetDomainKind::ExtraCanisters,
            Self::PendingMessages(_) => ResetDomainKind::PendingMessages,
            Self::SubnetState(_) => ResetDomainKind::SubnetState,
            Self::ExternalResources(_) => ResetDomainKind::ExternalResources,
        }
    }

    fn satisfies(&self, requirement: &ResetRequirement) -> bool {
        match (requirement, self) {
            (ResetRequirement::CanisterSnapshots, Self::CanisterSnapshots) => true,
            (ResetRequirement::CanisterCycles(left), Self::CanisterCycles(right)) => left == right,
            (ResetRequirement::PocketIcTime(left), Self::PocketIcTime(right)) => left == right,
            (ResetRequirement::ExtraCanisters(left), Self::ExtraCanisters(right)) => left == right,
            (ResetRequirement::PendingMessages(left), Self::PendingMessages(right))
            | (ResetRequirement::SubnetState(left), Self::SubnetState(right))
            | (ResetRequirement::ExternalResources(left), Self::ExternalResources(right)) => {
                left == right
            }
            _ => false,
        }
    }
}

impl ResetRequirements {
    /// Construct a duplicate-checked reset requirement set.
    ///
    /// Snapshot restoration and its cycle policy are mandatory for every
    /// pooled baseline recipe.
    pub fn try_new<I>(requirements: I) -> Result<Self, BaselinePoolContractError>
    where
        I: IntoIterator<Item = ResetRequirement>,
    {
        let mut domains = BTreeMap::new();
        for requirement in requirements {
            let domain = requirement.domain();
            if domains.insert(domain, requirement).is_some() {
                return Err(BaselinePoolContractError::DuplicateResetDomain { domain });
            }
        }
        for domain in [
            ResetDomainKind::CanisterSnapshots,
            ResetDomainKind::CanisterCycles,
        ] {
            if !domains.contains_key(&domain) {
                return Err(BaselinePoolContractError::UndeclaredRequiredResetDomain { domain });
            }
        }
        Ok(Self(domains))
    }

    /// Read the requirement for one domain.
    #[must_use]
    pub fn get(&self, domain: ResetDomainKind) -> Option<&ResetRequirement> {
        self.0.get(&domain)
    }

    /// Iterate over requirements in deterministic domain order.
    pub fn iter(&self) -> impl Iterator<Item = &ResetRequirement> {
        self.0.values()
    }

    fn verify(&self, receipt: &ResetReceipt) -> Result<(), BaselinePoolContractError> {
        for (domain, requirement) in &self.0 {
            let Some(achievement) = receipt.0.get(domain) else {
                return Err(BaselinePoolContractError::MissingResetDomain { domain: *domain });
            };
            if !achievement.satisfies(requirement) {
                return Err(BaselinePoolContractError::ResetPolicyMismatch {
                    requirement: requirement.clone(),
                    achievement: achievement.clone(),
                });
            }
        }
        Ok(())
    }
}

impl ResetReceipt {
    /// Construct a duplicate-checked non-snapshot reset achievement set.
    ///
    /// Omit snapshot and cycle achievements from the receipt returned by
    /// [`PocketIcBaselineRecipe::reset_non_snapshot_state`]; the pool derives
    /// those from [`CanisterRestoreReceipt`].
    pub fn try_new<I>(achievements: I) -> Result<Self, BaselinePoolContractError>
    where
        I: IntoIterator<Item = ResetAchievement>,
    {
        let mut domains = BTreeMap::new();
        for achievement in achievements {
            let domain = achievement.domain();
            if domains.insert(domain, achievement).is_some() {
                return Err(BaselinePoolContractError::DuplicateResetDomain { domain });
            }
        }
        Ok(Self(domains))
    }

    /// Create an empty receipt for recipes with no non-snapshot reset achievements.
    #[must_use]
    pub const fn empty() -> Self {
        Self(BTreeMap::new())
    }

    /// Read the achievement for one domain.
    #[must_use]
    pub fn get(&self, domain: ResetDomainKind) -> Option<&ResetAchievement> {
        self.0.get(&domain)
    }

    /// Iterate over achievements in deterministic domain order.
    pub fn iter(&self) -> impl Iterator<Item = &ResetAchievement> {
        self.0.values()
    }

    fn include_restore(
        &mut self,
        restore: &CanisterRestoreReceipt,
    ) -> Result<(), BaselinePoolContractError> {
        self.insert(ResetAchievement::CanisterSnapshots)?;
        self.insert(ResetAchievement::CanisterCycles(restore.cycle_policy))
    }

    fn insert(&mut self, achievement: ResetAchievement) -> Result<(), BaselinePoolContractError> {
        let domain = achievement.domain();
        if self.0.insert(domain, achievement).is_some() {
            return Err(BaselinePoolContractError::DuplicateResetDomain { domain });
        }
        Ok(())
    }
}

impl CanisterRestoreReceipt {
    /// Construct a deterministic, duplicate-checked canister restore receipt.
    pub fn try_new<I>(
        canister_ids: I,
        cycle_policy: CycleResetPolicy,
    ) -> Result<Self, BaselinePoolContractError>
    where
        I: IntoIterator<Item = Principal>,
    {
        let mut unique = BTreeSet::new();
        for canister_id in canister_ids {
            if !unique.insert(canister_id) {
                return Err(BaselinePoolContractError::DuplicateCanisterId { canister_id });
            }
        }
        if unique.is_empty() {
            return Err(BaselinePoolContractError::EmptyCanisterSet);
        }
        Ok(Self {
            canister_ids: unique.into_iter().collect(),
            cycle_policy,
        })
    }

    /// Restored canister ids in deterministic order.
    #[must_use]
    pub fn canister_ids(&self) -> &[Principal] {
        &self.canister_ids
    }

    /// Cycle policy applied while restoring canisters.
    #[must_use]
    pub const fn cycle_policy(&self) -> CycleResetPolicy {
        self.cycle_policy
    }
}

impl ReadinessReceipt {
    /// Construct a nonempty caller-owned readiness identity.
    pub fn try_new(identity: impl Into<String>) -> Result<Self, BaselinePoolContractError> {
        Ok(Self {
            identity: nonempty_receipt_identity("readiness", identity.into())?,
        })
    }

    /// Borrow the readiness identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

impl ValidationReceipt {
    /// Construct final validation evidence for one recipe.
    pub fn try_new(
        recipe_id: FixtureRecipeId,
        invariant_identity: impl Into<String>,
    ) -> Result<Self, BaselinePoolContractError> {
        Ok(Self {
            recipe_id,
            invariant_identity: nonempty_receipt_identity("validation", invariant_identity.into())?,
        })
    }

    /// Recipe identity validated by this receipt.
    #[must_use]
    pub const fn recipe_id(&self) -> &FixtureRecipeId {
        &self.recipe_id
    }

    /// Borrow the caller-owned invariant identity.
    #[must_use]
    pub fn invariant_identity(&self) -> &str {
        &self.invariant_identity
    }
}

impl BaselinePreparationStage {
    fn default_rebuild_reason(self) -> RebuildReason {
        match self {
            Self::RestoreCanisters => RebuildReason::SnapshotRestoreFailure,
            Self::ResetNonSnapshotState => RebuildReason::ResetFailure,
            Self::DriveToReadiness => RebuildReason::ReadinessFailure,
            Self::ValidateRestored | Self::ValidateBuilt => {
                RebuildReason::InvariantValidationFailure
            }
            Self::Build => RebuildReason::RecipeClassified {
                code: "build".to_owned(),
            },
        }
    }
}

impl BaselinePoolTimings {
    /// Time spent waiting for a capacity slot.
    #[must_use]
    pub const fn wait(self) -> Duration {
        self.wait
    }

    /// Time spent constructing a new baseline.
    #[must_use]
    pub const fn build(self) -> Option<Duration> {
        self.build
    }

    /// Time spent restoring captured canisters.
    #[must_use]
    pub const fn restore(self) -> Option<Duration> {
        self.restore
    }

    /// Time spent resetting non-snapshot state.
    #[must_use]
    pub const fn reset(self) -> Option<Duration> {
        self.reset
    }

    /// Time spent driving the topology to readiness.
    #[must_use]
    pub const fn readiness(self) -> Option<Duration> {
        self.readiness
    }

    /// Time spent validating final baseline invariants.
    #[must_use]
    pub const fn validation(self) -> Option<Duration> {
        self.validation
    }

    /// Time spent dropping an invalid baseline before rebuilding.
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

impl BaselinePoolOutcome {
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
    pub const fn timings(&self) -> BaselinePoolTimings {
        match self {
            Self::Built { timings, .. }
            | Self::Restored { timings, .. }
            | Self::Rebuilt { timings, .. } => *timings,
        }
    }
}

impl<R> CachedPocketIcBaselinePool<R>
where
    R: PocketIcBaselineRecipe,
{
    /// Create a runtime-capacity pool that structurally owns one recipe.
    #[must_use]
    pub fn new(capacity: NonZeroUsize, recipe: R) -> Self {
        Self {
            recipe,
            slots: BoundedSlotPool::new(capacity),
        }
    }

    /// Borrow the caller-owned identity of this pool's only recipe.
    #[must_use]
    pub fn recipe_id(&self) -> &FixtureRecipeId {
        self.recipe.id()
    }

    /// Maximum number of simultaneously leased PocketIC baselines.
    #[must_use]
    pub fn capacity(&self) -> NonZeroUsize {
        self.slots.capacity()
    }

    /// Acquire one fully built or restored and validated baseline lease.
    ///
    /// A rebuildable warm-preparation failure discards the stale slot and
    /// performs at most one build attempt. Recipe and caller panics are never
    /// converted into cache misses; the lease is invalidated while the panic
    /// continues unwinding.
    ///
    /// # Errors
    ///
    /// Returns a stage-specific recipe or contract error. If warm preparation
    /// and its one recovery build both fail, the error preserves both causes.
    pub fn acquire(
        &self,
    ) -> Result<
        (CachedPocketIcBaselinePoolGuard<'_, R>, BaselinePoolOutcome),
        BaselinePoolError<R::Error>,
    > {
        let total_started = Instant::now();
        let mut slot = self.slots.acquire();
        let mut timings = BaselinePoolTimings {
            wait: slot.wait(),
            ..BaselinePoolTimings::default()
        };
        let slot_index = slot.slot_index();

        if slot.is_reusable() {
            match self.prepare_reused(&mut slot, &mut timings) {
                Ok(()) => {
                    timings.total = total_started.elapsed();
                    return Ok((
                        CachedPocketIcBaselinePoolGuard { slot },
                        BaselinePoolOutcome::Restored {
                            slot: slot_index,
                            timings,
                        },
                    ));
                }
                Err(original) => {
                    let disposition = self.failure_disposition(&original);
                    match disposition {
                        FailureDisposition::Fatal => {
                            // A failed restore may have partially changed the
                            // instance. Discard it, but do not reinterpret a
                            // caller-declared fatal error as a rebuild reason.
                            Self::discard_stale_slot(&mut slot, &mut timings);
                            return Err(BaselinePoolError::Preparation(original));
                        }
                        FailureDisposition::Rebuild(reason) => {
                            Self::discard_stale_slot(&mut slot, &mut timings);
                            if let Err(rebuild) = self.build_slot(&mut slot, &mut timings) {
                                return Err(BaselinePoolError::RecoveryFailed {
                                    original: Box::new(original),
                                    rebuild: Box::new(rebuild),
                                });
                            }
                            timings.total = total_started.elapsed();
                            return Ok((
                                CachedPocketIcBaselinePoolGuard { slot },
                                BaselinePoolOutcome::Rebuilt {
                                    slot: slot_index,
                                    reason,
                                    timings,
                                },
                            ));
                        }
                    }
                }
            }
        }

        let rebuild_reason = if slot.invalidated_by_unwind() {
            Some(RebuildReason::UnwindWhileLeased)
        } else {
            slot.get()
                .and_then(|slot| slot.invalidation_reason.clone())
                .or_else(|| {
                    slot.is_populated()
                        .then_some(RebuildReason::ExplicitLeaseInvalidation)
                })
        };
        if slot.is_populated() {
            Self::discard_stale_slot(&mut slot, &mut timings);
        }
        self.build_slot(&mut slot, &mut timings)
            .map_err(BaselinePoolError::Preparation)?;
        timings.total = total_started.elapsed();

        let outcome = rebuild_reason.map_or_else(
            || BaselinePoolOutcome::Built {
                slot: slot_index,
                timings,
            },
            |reason| BaselinePoolOutcome::Rebuilt {
                slot: slot_index,
                reason,
                timings,
            },
        );
        Ok((CachedPocketIcBaselinePoolGuard { slot }, outcome))
    }

    fn prepare_reused(
        &self,
        slot: &mut BoundedSlotLease<'_, BaselineSlot<R::Metadata>>,
        timings: &mut BaselinePoolTimings,
    ) -> Result<(), BaselinePoolPreparationError<R::Error>> {
        let baseline = &slot
            .get()
            .expect("reusable baseline slot must be populated")
            .baseline;

        let started = Instant::now();
        let restore = self.recipe.restore_canisters(baseline);
        add_timing(&mut timings.restore, started.elapsed());
        let canisters = restore.map_err(|source| BaselinePoolPreparationError::Recipe {
            stage: BaselinePreparationStage::RestoreCanisters,
            source,
        })?;
        let expected_canisters = baseline.snapshot_canister_ids().collect::<Vec<_>>();
        if canisters.canister_ids() != expected_canisters {
            return Err(BaselinePoolPreparationError::Contract(
                BaselinePoolContractError::RestoreCanisterSetMismatch {
                    expected: expected_canisters,
                    actual: canisters.canister_ids().to_vec(),
                },
            ));
        }

        let started = Instant::now();
        let reset_result = self.recipe.reset_non_snapshot_state(baseline);
        add_timing(&mut timings.reset, started.elapsed());
        let mut reset = reset_result.map_err(|source| BaselinePoolPreparationError::Recipe {
            stage: BaselinePreparationStage::ResetNonSnapshotState,
            source,
        })?;
        reset
            .include_restore(&canisters)
            .map_err(BaselinePoolPreparationError::Contract)?;

        let started = Instant::now();
        let readiness_result = self.recipe.drive_to_readiness(baseline);
        add_timing(&mut timings.readiness, started.elapsed());
        let readiness =
            readiness_result.map_err(|source| BaselinePoolPreparationError::Recipe {
                stage: BaselinePreparationStage::DriveToReadiness,
                source,
            })?;
        self.recipe
            .reset_requirements()
            .verify(&reset)
            .map_err(BaselinePoolPreparationError::Contract)?;

        let preparation = PreparedBaseline::Restored {
            canisters,
            reset,
            readiness,
        };
        self.validate_baseline(
            baseline,
            &preparation,
            BaselinePreparationStage::ValidateRestored,
            timings,
        )?;
        slot.get_mut()
            .expect("validated baseline slot must remain populated")
            .invalidation_reason = None;
        Ok(())
    }

    fn build_slot(
        &self,
        slot: &mut BoundedSlotLease<'_, BaselineSlot<R::Metadata>>,
        timings: &mut BaselinePoolTimings,
    ) -> Result<(), BaselinePoolPreparationError<R::Error>> {
        let started = Instant::now();
        let build = self.recipe.build();
        add_timing(&mut timings.build, started.elapsed());
        let baseline = build.map_err(|source| BaselinePoolPreparationError::Recipe {
            stage: BaselinePreparationStage::Build,
            source,
        })?;

        if let Err(error) = self.validate_baseline(
            &baseline,
            &PreparedBaseline::Built,
            BaselinePreparationStage::ValidateBuilt,
            timings,
        ) {
            drop_baseline_safely(baseline);
            return Err(error);
        }
        let replaced = slot.replace(BaselineSlot {
            baseline,
            invalidation_reason: None,
        });
        debug_assert!(replaced.is_none());
        Ok(())
    }

    fn validate_baseline(
        &self,
        baseline: &CachedPocketIcBaseline<R::Metadata>,
        preparation: &PreparedBaseline,
        stage: BaselinePreparationStage,
        timings: &mut BaselinePoolTimings,
    ) -> Result<(), BaselinePoolPreparationError<R::Error>> {
        let started = Instant::now();
        let validation = self.recipe.validate(baseline, preparation);
        add_timing(&mut timings.validation, started.elapsed());
        let receipt =
            validation.map_err(|source| BaselinePoolPreparationError::Recipe { stage, source })?;
        if receipt.recipe_id() != self.recipe.id() {
            return Err(BaselinePoolPreparationError::Contract(
                BaselinePoolContractError::RecipeIdentityMismatch {
                    expected: self.recipe.id().clone(),
                    actual: receipt.recipe_id().clone(),
                },
            ));
        }
        Ok(())
    }

    fn failure_disposition(
        &self,
        error: &BaselinePoolPreparationError<R::Error>,
    ) -> FailureDisposition {
        match error {
            BaselinePoolPreparationError::Recipe { stage, source } => {
                self.recipe.classify_failure(*stage, source)
            }
            BaselinePoolPreparationError::Contract(
                BaselinePoolContractError::RecipeIdentityMismatch { .. },
            ) => FailureDisposition::Fatal,
            BaselinePoolPreparationError::Contract(_) => {
                FailureDisposition::Rebuild(rebuild_reason_for_error(error))
            }
        }
    }

    fn discard_stale_slot(
        slot: &mut BoundedSlotLease<'_, BaselineSlot<R::Metadata>>,
        timings: &mut BaselinePoolTimings,
    ) {
        let started = Instant::now();
        if let Some(stale) = slot.take() {
            drop_baseline_safely(stale.baseline);
        }
        timings.stale_teardown = Some(started.elapsed());
    }
}

impl<R> CachedPocketIcBaselinePoolGuard<'_, R>
where
    R: PocketIcBaselineRecipe,
{
    /// Diagnostic slot index held by this lease.
    #[must_use]
    pub const fn slot(&self) -> usize {
        self.slot.slot_index()
    }

    /// Mark this slot non-reusable with a structured rebuild reason.
    pub fn invalidate(&mut self, reason: RebuildReason) {
        if let Some(slot) = self.slot.get_mut() {
            slot.invalidation_reason = Some(reason);
        }
        self.slot.invalidate();
    }
}

impl<R> Deref for CachedPocketIcBaselinePoolGuard<'_, R>
where
    R: PocketIcBaselineRecipe,
{
    type Target = CachedPocketIcBaseline<R::Metadata>;

    fn deref(&self) -> &Self::Target {
        &self
            .slot
            .get()
            .expect("leased baseline pool slot must be populated")
            .baseline
    }
}

fn nonempty_receipt_identity(
    receipt: &'static str,
    identity: String,
) -> Result<String, BaselinePoolContractError> {
    if identity.trim().is_empty() {
        return Err(BaselinePoolContractError::EmptyReceiptIdentity { receipt });
    }
    Ok(identity)
}

fn add_timing(total: &mut Option<Duration>, elapsed: Duration) {
    *total = Some(total.unwrap_or_default().saturating_add(elapsed));
}

fn rebuild_reason_for_error<E>(error: &BaselinePoolPreparationError<E>) -> RebuildReason {
    match error {
        BaselinePoolPreparationError::Recipe { stage, .. } => stage.default_rebuild_reason(),
        BaselinePoolPreparationError::Contract(
            BaselinePoolContractError::MissingResetDomain { .. }
            | BaselinePoolContractError::ResetPolicyMismatch { .. }
            | BaselinePoolContractError::DuplicateResetDomain { .. }
            | BaselinePoolContractError::RestoreCanisterSetMismatch { .. },
        ) => RebuildReason::ResetCoverageMismatch,
        BaselinePoolPreparationError::Contract(_) => RebuildReason::InvariantValidationFailure,
    }
}

fn drop_baseline_safely<M>(baseline: CachedPocketIcBaseline<M>) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(baseline)));
}

impl std::fmt::Display for FixtureRecipeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::fmt::Display for BaselinePreparationStage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Build => "build",
            Self::RestoreCanisters => "canister restore",
            Self::ResetNonSnapshotState => "non-snapshot reset",
            Self::DriveToReadiness => "readiness",
            Self::ValidateBuilt => "built-baseline validation",
            Self::ValidateRestored => "restored-baseline validation",
        })
    }
}

impl std::fmt::Display for BaselinePoolContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyRecipeIdentity => formatter.write_str("fixture recipe identity is empty"),
            Self::EmptyReceiptIdentity { receipt } => {
                write!(formatter, "{receipt} receipt identity is empty")
            }
            Self::DuplicateResetDomain { domain } => {
                write!(
                    formatter,
                    "reset domain {domain:?} was reported more than once"
                )
            }
            Self::DuplicateCanisterId { canister_id } => {
                write!(formatter, "restore receipt repeats canister {canister_id}")
            }
            Self::EmptyCanisterSet => formatter.write_str("restore receipt contains no canisters"),
            Self::UndeclaredRequiredResetDomain { domain } => write!(
                formatter,
                "baseline recipe does not declare required reset domain {domain:?}",
            ),
            Self::RestoreCanisterSetMismatch { expected, actual } => write!(
                formatter,
                "restore receipt identified canisters {actual:?}, expected captured set {expected:?}",
            ),
            Self::MissingResetDomain { domain } => {
                write!(
                    formatter,
                    "required reset domain {domain:?} was not achieved"
                )
            }
            Self::ResetPolicyMismatch {
                requirement,
                achievement,
            } => write!(
                formatter,
                "reset achievement {achievement:?} does not satisfy {requirement:?}",
            ),
            Self::RecipeIdentityMismatch { expected, actual } => write!(
                formatter,
                "validation receipt used recipe `{actual}` instead of `{expected}`",
            ),
        }
    }
}

impl std::error::Error for BaselinePoolContractError {}

impl<E> std::fmt::Display for BaselinePoolPreparationError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipe { stage, source } => {
                write!(formatter, "baseline {stage} failed: {source}")
            }
            Self::Contract(error) => write!(formatter, "baseline pool contract failed: {error}"),
        }
    }
}

impl<E> std::error::Error for BaselinePoolPreparationError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Recipe { source, .. } => Some(source),
            Self::Contract(error) => Some(error),
        }
    }
}

impl<E> std::fmt::Display for BaselinePoolError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation(error) => error.fmt(formatter),
            Self::RecoveryFailed { original, rebuild } => write!(
                formatter,
                "baseline preparation failed ({original}); rebuilding the slot also failed: {rebuild}",
            ),
        }
    }
}

impl<E> std::error::Error for BaselinePoolError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Preparation(error) => Some(error),
            Self::RecoveryFailed { original, .. } => Some(original.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BaselinePoolContractError, CycleResetPolicy, FixtureRecipeId, ResetAchievement,
        ResetDomainKind, ResetReceipt, ResetRequirement, ResetRequirements,
    };

    #[test]
    fn recipe_identity_must_be_nonempty() {
        assert!(matches!(
            FixtureRecipeId::try_new("  "),
            Err(BaselinePoolContractError::EmptyRecipeIdentity)
        ));
    }

    #[test]
    fn reset_requirements_reject_duplicate_domains() {
        let result = ResetRequirements::try_new([
            ResetRequirement::CanisterCycles(CycleResetPolicy::PreserveCurrent),
            ResetRequirement::CanisterCycles(CycleResetPolicy::RestoreExactBaseline),
        ]);
        assert!(matches!(
            result,
            Err(BaselinePoolContractError::DuplicateResetDomain {
                domain: ResetDomainKind::CanisterCycles,
            })
        ));
    }

    #[test]
    fn required_policy_must_match_achieved_policy() {
        let requirements = ResetRequirements::try_new([
            ResetRequirement::CanisterSnapshots,
            ResetRequirement::CanisterCycles(CycleResetPolicy::RestoreExactBaseline),
        ])
        .unwrap();
        let receipt = ResetReceipt::try_new([
            ResetAchievement::CanisterSnapshots,
            ResetAchievement::CanisterCycles(CycleResetPolicy::PreserveCurrent),
        ])
        .unwrap();

        assert!(matches!(
            requirements.verify(&receipt),
            Err(BaselinePoolContractError::ResetPolicyMismatch { .. })
        ));
    }
}
