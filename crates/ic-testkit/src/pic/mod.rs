//! Direct PocketIC types plus value-adding host-test harness extensions.
//!
//! The complete upstream crate is available through [`crate::pocket_ic`]. This
//! module keeps the common runtime types alongside ic-testkit's focused
//! extension traits and policy types.
//!
//! Construct and own [`PocketIc`] instances normally, then import individual
//! extension traits or [`prelude`] for Candid calls, installation, diagnostics,
//! snapshots, fallible startup, bounded predicate polling and nanosecond time conversion. Native
//! simulator operations remain upstream inherent methods.
//!
//! [`PocketIcManagedServer`] can explicitly own one exact caller-selected
//! server child for a serial suite. No type serializes independent instances or
//! downloads a server implicitly. Explicit provisioning is owned by the
//! published `ic-testkit-server setup` / `check` commands.

pub use pocket_ic::{
    CanisterStatusResult, ErrorCode, LATEST_SERVER_VERSION, PocketIc, PocketIcBuilder, RejectCode,
    RejectResponse,
};

mod server;
pub use server::{POCKET_IC_SERVER_VERSION, supports_pocket_ic_server};

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

pub use baseline::CachedPocketIcBaseline;
pub use baseline_pool::{
    BaselinePoolContractError, BaselinePoolError, BaselinePoolOutcome,
    BaselinePoolPreparationError, BaselinePoolTimings, BaselinePreparationStage,
    CachedPocketIcBaselinePool, CachedPocketIcBaselinePoolGuard, CanisterRestoreReceipt,
    CycleResetPolicy, ExtraCanisterPolicy, FailureDisposition, FixtureRecipeId,
    PocketIcBaselineRecipe, PreparedBaseline, ReadinessReceipt, RebuildReason, ResetDomainKind,
    ResetDomainPolicy, ResetReceipt, ResetRequirements, StateResetPolicy, TimeResetPolicy,
    ValidationReceipt,
};
pub use calls::CandidCallExt;
pub use diagnostics::{
    CanisterDiagnosticFailure, CanisterDiagnosticLogRecord, CanisterDiagnosticLogs,
    CanisterDiagnosticsBatchContractError, CanisterDiagnosticsBatchEntry,
    CanisterDiagnosticsBatchReport, CanisterDiagnosticsReport, CanisterDiagnosticsRequest,
    CanisterLogRenderLimits, DEFAULT_CANISTER_LOG_BYTE_LIMIT, DEFAULT_CANISTER_LOG_RECORD_LIMIT,
    LabeledCanisterDiagnosticsRequest, PocketIcDiagnosticsExt,
};
pub use errors::{
    CandidCallContext, CandidCallError, CandidCallErrorKind, CanisterInstallError,
    CanisterInstallPhase, StandaloneCanisterInstallError,
};
pub use lifecycle::{CanisterInstallExt, InstallSpec, RetryPolicy, RetryPolicyError};
pub use snapshot::{
    CanisterSnapshotTarget, ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt,
    SnapshotAttemptFailure, SnapshotCleanupFailure, SnapshotRestoreFunding,
};
pub use standalone::StandaloneCanisterFixture;
pub use standalone_pool::{
    CachedStandaloneCanisterFixtureGuard, CachedStandaloneCanisterFixturePool,
    StandaloneFixturePoolError, StandaloneFixturePoolOutcome, StandaloneFixturePoolRebuildReason,
    StandaloneFixturePoolStage, StandaloneFixturePoolTimings,
};
pub use startup::{
    PocketIcBuilderExt, PocketIcManagedServer, PocketIcManagedServerOutput, PocketIcStartupConfig,
    PocketIcStartupError,
};
pub use time::{PocketIcTimeExt, TickUntilError, tick_until};
pub use transport::{PocketIcOperationError, is_dead_pocket_ic_transport_error};

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
