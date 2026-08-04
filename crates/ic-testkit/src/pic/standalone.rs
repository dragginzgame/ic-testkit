use candid::{CandidType, Principal, utils::ArgumentEncoder};
use pocket_ic::PocketIc;
use serde::de::DeserializeOwned;

use super::{
    CandidCallError, CandidCallExt, CanisterInstallExt, InstallSpec, StandaloneCanisterInstallError,
};

///
/// StandaloneCanisterFixture
///

pub struct StandaloneCanisterFixture {
    pocket_ic: PocketIc,
    canister_id: Principal,
}

impl StandaloneCanisterFixture {
    /// Install one canister into a caller-configured PocketIC instance.
    #[must_use]
    pub fn install(pocket_ic: PocketIc, spec: InstallSpec) -> Self {
        Self::try_install(pocket_ic, spec)
            .unwrap_or_else(|err| panic!("failed to install standalone canister fixture: {err}"))
    }

    /// Fallible counterpart to [`install`](Self::install).
    ///
    /// On failure, the error retains both the caller's PocketIC instance and
    /// the structured install failure.
    pub fn try_install(
        pocket_ic: PocketIc,
        spec: InstallSpec,
    ) -> Result<Self, StandaloneCanisterInstallError> {
        let canister_id = match pocket_ic.try_create_and_install(spec) {
            Ok(canister_id) => canister_id,
            Err(error) => return Err(StandaloneCanisterInstallError::new(pocket_ic, error)),
        };

        Ok(Self {
            pocket_ic,
            canister_id,
        })
    }

    /// Borrow the PocketIC instance that owns this standalone fixture.
    #[must_use]
    pub const fn pocket_ic(&self) -> &PocketIc {
        &self.pocket_ic
    }

    /// Read the installed canister id for this standalone fixture.
    #[must_use]
    pub const fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Consume the fixture and return the owned PocketIC instance and canister id.
    #[must_use]
    pub fn into_parts(self) -> (PocketIc, Principal) {
        (self.pocket_ic, self.canister_id)
    }

    /// Forward one typed update call to this fixture's canister id.
    pub fn update_call<T, A>(&self, method: &str, args: A) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic.update_candid(self.canister_id, method, args)
    }

    /// Forward one typed update call to this fixture's canister id, panicking
    /// on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `update_call_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn update_call_or_panic<T, A>(&self, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic
            .update_candid_or_panic(self.canister_id, method, args)
    }

    /// Forward one typed update call with an explicit caller to this fixture's canister id.
    pub fn update_call_as<T, A>(
        &self,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic
            .update_candid_as(self.canister_id, caller, method, args)
    }

    /// Forward one typed update call with an explicit caller to this fixture's
    /// canister id, panicking on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `update_call_as_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn update_call_as_or_panic<T, A>(&self, caller: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic
            .update_candid_as_or_panic(self.canister_id, caller, method, args)
    }

    /// Forward one typed query call to this fixture's canister id.
    pub fn query_call<T, A>(&self, method: &str, args: A) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic.query_candid(self.canister_id, method, args)
    }

    /// Forward one typed query call to this fixture's canister id, panicking on
    /// rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `query_call_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn query_call_or_panic<T, A>(&self, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic
            .query_candid_or_panic(self.canister_id, method, args)
    }

    /// Forward one typed query call with an explicit caller to this fixture's canister id.
    pub fn query_call_as<T, A>(
        &self,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic
            .query_candid_as(self.canister_id, caller, method, args)
    }

    /// Forward one typed query call with an explicit caller to this fixture's
    /// canister id, panicking on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `query_call_as_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn query_call_as_or_panic<T, A>(&self, caller: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.pocket_ic
            .query_candid_as_or_panic(self.canister_id, caller, method, args)
    }
}
