//! Direct PocketIC types plus value-adding host-test harness extensions.
//!
//! Construct and own [`PocketIc`] instances normally, then import individual
//! extension traits or [`prelude`] for Candid calls, installation, diagnostics,
//! snapshots, fallible startup, and nanosecond time conversion. Native
//! simulator operations remain upstream inherent methods.
//!
//! No type in this module serializes independent instances or owns PocketIC's
//! server download/cache policy.

pub use pocket_ic::{
    ErrorCode, LATEST_SERVER_VERSION, PocketIc, PocketIcBuilder, RejectCode, RejectResponse,
};

mod baseline;
mod baseline_pool;
mod bounded_pool;
mod calls;
mod diagnostics;
mod errors;
mod lifecycle;
mod snapshot;
mod standalone;
mod standalone_pool;
mod startup;
mod time;
mod transport;

pub use baseline::{
    CachedPocketIcBaseline, CachedPocketIcBaselineGuard,
    restore_or_rebuild_cached_pocket_ic_baseline,
};
pub use baseline_pool::{
    BaselinePoolContractError, BaselinePoolError, BaselinePoolOutcome,
    BaselinePoolPreparationError, BaselinePoolTimings, BaselinePreparationStage,
    CachedPocketIcBaselinePool, CachedPocketIcBaselinePoolGuard, CanisterRestoreReceipt,
    CycleResetPolicy, ExtraCanisterPolicy, FailureDisposition, FixtureRecipeId,
    PocketIcBaselineRecipe, PreparedBaseline, ReadinessReceipt, RebuildReason, ResetAchievement,
    ResetDomainKind, ResetReceipt, ResetRequirement, ResetRequirements, StateResetPolicy,
    TimeResetPolicy, ValidationReceipt,
};
pub use calls::CandidCallExt;
pub use diagnostics::PocketIcDiagnosticsExt;
pub use errors::{
    CandidCallContext, CandidCallError, CandidCallErrorKind, CanisterInstallError,
    StandaloneCanisterInstallError,
};
pub use lifecycle::{CanisterInstallExt, InstallSpec, RetryPolicy, RetryPolicyError};
pub use snapshot::{
    ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, SnapshotAttemptFailure,
    SnapshotCleanupFailure, SnapshotRestoreFunding,
};
pub use standalone::StandaloneCanisterFixture;
pub use standalone_pool::{
    CachedStandaloneCanisterFixtureGuard, CachedStandaloneCanisterFixturePool,
};
pub use startup::{PocketIcBuilderExt, PocketIcStartupError};
pub use time::PocketIcTimeExt;
pub use transport::is_dead_pocket_ic_transport_error;

/// All PocketIC extension traits, and no data types.
///
/// Importing this module keeps policy/data types explicit while avoiding
/// repeated trait lists across a large integration-test crate.
pub mod prelude {
    pub use super::{
        CandidCallExt, CanisterInstallExt, PocketIcBuilderExt, PocketIcDiagnosticsExt,
        PocketIcSnapshotExt, PocketIcTimeExt,
    };
}
