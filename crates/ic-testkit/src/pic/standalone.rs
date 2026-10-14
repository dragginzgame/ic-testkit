use candid::Principal;
use pocket_ic::PocketIc;

use super::{CanisterInstallExt, InstallSpec, PocketIcInstance, StandaloneCanisterInstallError};

/// One caller-built PocketIC instance/server owner and one installed canister.
///
/// This fixture retains the instance and its optional managed server together.
/// It does not impose process-wide synchronization or hide PocketIC methods.
pub struct StandaloneCanisterFixture {
    instance: PocketIcInstance<PocketIc>,
    canister_id: Principal,
}

impl StandaloneCanisterFixture {
    /// Install one canister into a caller-configured PocketIC instance.
    ///
    /// # Panics
    ///
    /// Panics with the contextual install error when installation fails.
    #[must_use]
    pub fn install(instance: PocketIcInstance<PocketIc>, spec: InstallSpec) -> Self {
        Self::try_install(instance, spec)
            .unwrap_or_else(|err| panic!("failed to install standalone canister fixture: {err}"))
    }

    /// Fallible counterpart to [`install`](Self::install).
    ///
    /// On failure, the error retains both the caller's instance/server ownership record and
    /// the structured install failure.
    pub fn try_install(
        instance: PocketIcInstance<PocketIc>,
        spec: InstallSpec,
    ) -> Result<Self, StandaloneCanisterInstallError> {
        let canister_id = match instance.instance.try_create_and_install(spec) {
            Ok(canister_id) => canister_id,
            Err(error) => return Err(StandaloneCanisterInstallError::new(instance, error)),
        };

        Ok(Self {
            instance,
            canister_id,
        })
    }

    /// Borrow the PocketIC instance that owns this standalone fixture.
    #[must_use]
    pub const fn pocket_ic(&self) -> &PocketIc {
        &self.instance.instance
    }

    /// Read the installed canister id for this standalone fixture.
    #[must_use]
    pub const fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Consume the fixture and return the instance/server ownership record and canister id.
    pub fn into_parts(self) -> (PocketIcInstance<PocketIc>, Principal) {
        (self.instance, self.canister_id)
    }
}
