use candid::{CandidType, Principal, utils::ArgumentEncoder};
use pocket_ic::PocketIc;
use serde::de::DeserializeOwned;

use super::{
    CandidCallError, CandidCallExt, CanisterInstallExt, InstallSpec,
    StandaloneCanisterFixtureError, try_pic,
};

const DEFAULT_EXTRA_INSTALL_CYCLES: u128 = 0;

///
/// StandaloneCanisterFixture
///

pub struct StandaloneCanisterFixture {
    pocket_ic: PocketIc,
    canister_id: Principal,
}

impl StandaloneCanisterFixture {
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

// Install one already-built wasm module into a fresh PocketIC instance with
// caller-provided init args and no application-specific bootstrap assumptions.
#[must_use]
pub fn install_prebuilt_canister(wasm: Vec<u8>, init_bytes: Vec<u8>) -> StandaloneCanisterFixture {
    try_install_prebuilt_canister(wasm, init_bytes)
        .unwrap_or_else(|err| panic!("failed to install prebuilt canister fixture: {err}"))
}

// Install one already-built wasm module into a fresh PocketIC instance with
// caller-provided init args and no application-specific bootstrap assumptions.
pub fn try_install_prebuilt_canister(
    wasm: Vec<u8>,
    init_bytes: Vec<u8>,
) -> Result<StandaloneCanisterFixture, StandaloneCanisterFixtureError> {
    try_install_prebuilt_canister_from_spec(InstallSpec::new(
        wasm,
        init_bytes,
        DEFAULT_EXTRA_INSTALL_CYCLES,
    ))
}

// Install one already-built wasm module into a fresh PocketIC instance with
// caller-provided init args and explicit install cycles.
#[must_use]
pub fn install_prebuilt_canister_with_cycles(
    wasm: Vec<u8>,
    init_bytes: Vec<u8>,
    install_cycles: u128,
) -> StandaloneCanisterFixture {
    try_install_prebuilt_canister_with_cycles(wasm, init_bytes, install_cycles)
        .unwrap_or_else(|err| panic!("failed to install prebuilt canister fixture: {err}"))
}

// Install one already-built wasm module into a fresh PocketIC instance with
// caller-provided init args and explicit install cycles.
pub fn try_install_prebuilt_canister_with_cycles(
    wasm: Vec<u8>,
    init_bytes: Vec<u8>,
    install_cycles: u128,
) -> Result<StandaloneCanisterFixture, StandaloneCanisterFixtureError> {
    try_install_prebuilt_canister_from_spec(InstallSpec::new(wasm, init_bytes, install_cycles))
}

// Install one already-built wasm module from a generic install specification
// into a fresh PocketIC instance.
#[must_use]
pub fn install_prebuilt_canister_from_spec(spec: InstallSpec) -> StandaloneCanisterFixture {
    try_install_prebuilt_canister_from_spec(spec)
        .unwrap_or_else(|err| panic!("failed to install prebuilt canister fixture: {err}"))
}

// Install one already-built wasm module from a generic install specification
// into a fresh PocketIC instance.
pub fn try_install_prebuilt_canister_from_spec(
    spec: InstallSpec,
) -> Result<StandaloneCanisterFixture, StandaloneCanisterFixtureError> {
    let pocket_ic = try_pic().map_err(StandaloneCanisterFixtureError::Start)?;
    let canister_id = pocket_ic
        .try_create_and_install(spec)
        .map_err(StandaloneCanisterFixtureError::Install)?;

    Ok(StandaloneCanisterFixture {
        pocket_ic,
        canister_id,
    })
}
