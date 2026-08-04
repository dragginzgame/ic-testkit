//! PocketIC re-exports and value-adding host-test harness helpers.

pub use pocket_ic::{
    ErrorCode, LATEST_SERVER_VERSION, PocketIc, PocketIcBuilder, RejectCode, RejectResponse,
};

mod baseline;
mod calls;
mod diagnostics;
mod errors;
mod lifecycle;
mod snapshot;
mod standalone;
mod transport;

pub use baseline::{
    CachedPocketIcBaseline, CachedPocketIcBaselineGuard,
    restore_or_rebuild_cached_pocket_ic_baseline,
};
pub use calls::CandidCallExt;
pub use diagnostics::PocketIcDiagnosticsExt;
pub use errors::{
    CandidCallContext, CandidCallError, CandidCallErrorKind, CanisterInstallError,
    StandaloneCanisterInstallError,
};
pub use lifecycle::{CanisterInstallExt, InstallSpec, RetryPolicy};
pub use snapshot::{
    ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, SnapshotAttemptFailure,
    SnapshotCleanupFailure,
};
pub use standalone::StandaloneCanisterFixture;
