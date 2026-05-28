use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use candid::Principal;

use super::{Pic, PicInstallError, startup};

///
/// InstallSpec
///

#[non_exhaustive]
pub struct InstallSpec {
    pub wasm: Vec<u8>,
    pub init_bytes: Vec<u8>,
    pub cycles: u128,
    pub install_sender: Option<Principal>,
    pub label: Option<String>,
}

impl InstallSpec {
    /// Build one generic canister install specification.
    #[must_use]
    pub const fn new(wasm: Vec<u8>, init_bytes: Vec<u8>, cycles: u128) -> Self {
        Self {
            wasm,
            init_bytes,
            cycles,
            install_sender: None,
            label: None,
        }
    }

    /// Set the management-call sender used for `install_canister`.
    #[must_use]
    pub const fn install_sender(mut self, sender: Principal) -> Self {
        self.install_sender = Some(sender);
        self
    }

    /// Set a diagnostic label for install failures.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Pic {
    /// Install one arbitrary wasm module with caller-provided init bytes.
    ///
    /// This is the generic install path for downstreams that use `ic-testkit`
    /// without depending on application-specific init payload conventions.
    #[must_use]
    pub fn create_and_install_with_args(
        &self,
        wasm: Vec<u8>,
        init_bytes: Vec<u8>,
        install_cycles: u128,
    ) -> Principal {
        self.try_create_and_install_with_args(wasm, init_bytes, install_cycles)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Install one arbitrary wasm module with caller-provided init bytes.
    pub fn try_create_and_install_with_args(
        &self,
        wasm: Vec<u8>,
        init_bytes: Vec<u8>,
        install_cycles: u128,
    ) -> Result<Principal, PicInstallError> {
        self.try_create_and_install(InstallSpec::new(wasm, init_bytes, install_cycles))
    }

    /// Install one arbitrary wasm module from a generic install specification.
    #[must_use]
    pub fn create_and_install(&self, spec: InstallSpec) -> Principal {
        self.try_create_and_install(spec)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Install one arbitrary wasm module from a generic install specification.
    pub fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, PicInstallError> {
        self.try_create_funded_and_install(spec)
    }

    /// Sequentially install multiple arbitrary wasm modules into this `Pic`.
    ///
    /// Installs are attempted in iterator order. If one install fails, earlier
    /// installs remain in the PocketIC instance, the failed canister may exist
    /// with the id exposed by `PicInstallError::canister_id()`, and later
    /// installs are not attempted.
    #[must_use]
    pub fn create_and_install_many<I>(&self, specs: I) -> Vec<Principal>
    where
        I: IntoIterator<Item = InstallSpec>,
    {
        self.try_create_and_install_many(specs)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Sequentially install multiple arbitrary wasm modules into this `Pic`.
    ///
    /// Installs are attempted in iterator order. If one install fails, earlier
    /// installs remain in the PocketIC instance, the failed canister may exist
    /// with the id exposed by `PicInstallError::canister_id()`, and later
    /// installs are not attempted.
    pub fn try_create_and_install_many<I>(
        &self,
        specs: I,
    ) -> Result<Vec<Principal>, PicInstallError>
    where
        I: IntoIterator<Item = InstallSpec>,
    {
        specs
            .into_iter()
            .map(|spec| self.try_create_and_install(spec))
            .collect()
    }

    /// Wait out the PocketIC `install_code` cooldown window inside the same instance.
    pub fn wait_out_install_code_rate_limit(&self, cooldown: Duration) {
        self.advance_time(cooldown);
        self.tick_n(2);
    }

    /// Retry one install_code-like operation while PocketIC still reports rate limiting.
    pub fn retry_install_code_ok<T, F>(
        &self,
        retry_limit: usize,
        cooldown: Duration,
        mut op: F,
    ) -> Result<T, String>
    where
        F: FnMut() -> Result<T, String>,
    {
        let mut last_err = None;

        for _ in 0..retry_limit {
            match op() {
                Ok(value) => return Ok(value),
                Err(err) if is_install_code_rate_limited(&err) => {
                    last_err = Some(err);
                    self.wait_out_install_code_rate_limit(cooldown);
                }
                Err(err) => return Err(err),
            }
        }

        Err(last_err.unwrap_or_else(|| "install_code retry loop exhausted".to_string()))
    }

    /// Retry one install_code-like failure path while PocketIC still reports rate limiting.
    pub fn retry_install_code_err<F>(
        &self,
        retry_limit: usize,
        cooldown: Duration,
        first: Result<(), String>,
        mut op: F,
    ) -> Result<(), String>
    where
        F: FnMut() -> Result<(), String>,
    {
        match first {
            Ok(()) => return Ok(()),
            Err(err) if !is_install_code_rate_limited(&err) => return Err(err),
            Err(_) => {}
        }

        self.wait_out_install_code_rate_limit(cooldown);

        for _ in 1..retry_limit {
            match op() {
                Ok(()) => return Ok(()),
                Err(err) if is_install_code_rate_limited(&err) => {
                    self.wait_out_install_code_rate_limit(cooldown);
                }
                Err(err) => return Err(err),
            }
        }

        op()
    }

    // Install a canister after creating it and optionally adding extra cycles.
    fn try_create_funded_and_install(
        &self,
        spec: InstallSpec,
    ) -> Result<Principal, PicInstallError> {
        let canister_id = self.create_canister();
        if spec.cycles > 0 {
            self.add_cycles(canister_id, spec.cycles);
        }

        let install = catch_unwind(AssertUnwindSafe(|| {
            self.inner.install_canister(
                canister_id,
                spec.wasm,
                spec.init_bytes,
                spec.install_sender,
            );
        }));
        if let Err(payload) = install {
            if let Some(label) = &spec.label {
                eprintln!("install_canister trapped for {canister_id} ({label})");
            } else {
                eprintln!("install_canister trapped for {canister_id}");
            }
            if let Ok(status) = self.inner.canister_status(canister_id, None) {
                eprintln!("canister_status for {canister_id}: {status:?}");
            }
            if let Ok(logs) = self
                .inner
                .fetch_canister_logs(canister_id, Principal::anonymous())
            {
                for record in logs {
                    eprintln!("canister_log {canister_id}: {record:?}");
                }
            }
            let message = startup::panic_payload_to_string(payload.as_ref());
            return if let Some(label) = spec.label {
                Err(PicInstallError::labeled(canister_id, label, message))
            } else {
                Err(PicInstallError::new(canister_id, message))
            };
        }

        Ok(canister_id)
    }
}

fn is_install_code_rate_limited(message: &str) -> bool {
    message.contains("CanisterInstallCodeRateLimited")
}
