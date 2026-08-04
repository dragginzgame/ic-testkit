//! PocketIC re-exports and value-adding host-test harness helpers.

pub use pocket_ic::{ErrorCode, PocketIc, PocketIcBuilder, RejectCode, RejectResponse};

mod baseline;
mod calls;
mod diagnostics;
mod errors;
mod lifecycle;
mod snapshot;
mod standalone;
mod startup;
mod time;

pub use baseline::{
    CachedPocketIcBaseline, CachedPocketIcBaselineGuard,
    restore_or_rebuild_cached_pocket_ic_baseline,
};
pub use calls::CandidCallExt;
pub use diagnostics::PocketIcDiagnosticsExt;
pub use errors::{
    CandidCallContext, CandidCallError, CandidCallErrorKind, CanisterInstallError,
    StandaloneCanisterFixtureError,
};
pub use lifecycle::{CanisterInstallExt, InstallSpec, RetryPolicy};
pub use snapshot::{
    ControllerSnapshotError, ControllerSnapshots, PocketIcSnapshotExt, SnapshotAttemptFailure,
    SnapshotCleanupFailure,
};
pub use standalone::{
    StandaloneCanisterFixture, install_prebuilt_canister, install_prebuilt_canister_from_spec,
    install_prebuilt_canister_with_cycles, try_install_prebuilt_canister,
    try_install_prebuilt_canister_from_spec, try_install_prebuilt_canister_with_cycles,
};
pub use startup::PocketIcStartError;
pub use time::PocketIcTimeExt;

/// Create a fresh PocketIC instance with the default application subnet layout.
#[must_use]
pub fn pic() -> PocketIc {
    try_pic().unwrap_or_else(|err| panic!("failed to start PocketIC: {err}"))
}

/// Create a fresh PocketIC instance without panicking on startup failures.
pub fn try_pic() -> Result<PocketIc, PocketIcStartError> {
    try_build_pocket_ic(PocketIcBuilder::new().with_application_subnet())
}

/// Build a custom PocketIC topology with typed startup diagnostics.
#[must_use]
pub fn build_pocket_ic(builder: PocketIcBuilder) -> PocketIc {
    try_build_pocket_ic(builder).unwrap_or_else(|err| panic!("failed to start PocketIC: {err}"))
}

/// Build a custom PocketIC topology without panicking on startup failures.
pub fn try_build_pocket_ic(builder: PocketIcBuilder) -> Result<PocketIc, PocketIcStartError> {
    startup::try_build_pocket_ic(builder)
}
