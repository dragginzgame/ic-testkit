use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use candid::Principal;
use pocket_ic::{ErrorCode, PocketIc, RejectResponse};

use super::{CanisterInstallError, PocketIcDiagnosticsExt, transport};

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

/// Retry limits and simulated cooldown for install-code operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    max_attempts: usize,
    cooldown: Duration,
}

/// Invalid install-code retry policy configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryPolicyError {
    /// A retry policy must execute its operation at least once.
    ZeroMaxAttempts,
}

impl std::fmt::Display for RetryPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroMaxAttempts => {
                formatter.write_str("retry policy requires at least one attempt")
            }
        }
    }
}

impl std::error::Error for RetryPolicyError {}

impl RetryPolicy {
    /// Create a policy with an exact, non-zero maximum attempt count.
    pub const fn try_new(
        max_attempts: usize,
        cooldown: Duration,
    ) -> Result<Self, RetryPolicyError> {
        if max_attempts == 0 {
            Err(RetryPolicyError::ZeroMaxAttempts)
        } else {
            Ok(Self {
                max_attempts,
                cooldown,
            })
        }
    }

    /// Read the maximum number of operation attempts, including the first.
    #[must_use]
    pub const fn max_attempts(self) -> usize {
        self.max_attempts
    }

    /// Read the simulated cooldown applied between rate-limited attempts.
    #[must_use]
    pub const fn cooldown(self) -> Duration {
        self.cooldown
    }
}

/// Generic canister installation and install-code retry policies.
pub trait CanisterInstallExt {
    /// Create and install one canister from raw wasm and init bytes.
    #[must_use]
    fn create_and_install_with_args(
        &self,
        wasm: Vec<u8>,
        init_bytes: Vec<u8>,
        install_cycles: u128,
    ) -> Principal;

    /// Fallible counterpart to [`create_and_install_with_args`](Self::create_and_install_with_args).
    fn try_create_and_install_with_args(
        &self,
        wasm: Vec<u8>,
        init_bytes: Vec<u8>,
        install_cycles: u128,
    ) -> Result<Principal, CanisterInstallError>;

    /// Create and install one canister from a reusable specification.
    #[must_use]
    fn create_and_install(&self, spec: InstallSpec) -> Principal;

    /// Fallible counterpart to [`create_and_install`](Self::create_and_install).
    fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, CanisterInstallError>;

    /// Sequentially create and install multiple canisters.
    #[must_use]
    fn create_and_install_many<I>(&self, specs: I) -> Vec<Principal>
    where
        I: IntoIterator<Item = InstallSpec>;

    /// Fallible counterpart to [`create_and_install_many`](Self::create_and_install_many).
    fn try_create_and_install_many<I>(
        &self,
        specs: I,
    ) -> Result<Vec<Principal>, CanisterInstallError>
    where
        I: IntoIterator<Item = InstallSpec>;

    /// Advance simulated time and rounds past an install-code cooldown.
    fn wait_out_install_code_rate_limit(&self, cooldown: Duration);

    /// Retry an operation only while PocketIC reports install-code rate limiting.
    fn retry_install_code<T, F>(&self, policy: RetryPolicy, op: F) -> Result<T, RejectResponse>
    where
        F: FnMut() -> Result<T, RejectResponse>;
}

impl CanisterInstallExt for PocketIc {
    /// Install one arbitrary wasm module with caller-provided init bytes.
    ///
    /// This is the generic install path for downstreams that use `ic-testkit`
    /// without depending on application-specific init payload conventions.
    fn create_and_install_with_args(
        &self,
        wasm: Vec<u8>,
        init_bytes: Vec<u8>,
        install_cycles: u128,
    ) -> Principal {
        self.try_create_and_install_with_args(wasm, init_bytes, install_cycles)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Install one arbitrary wasm module with caller-provided init bytes.
    fn try_create_and_install_with_args(
        &self,
        wasm: Vec<u8>,
        init_bytes: Vec<u8>,
        install_cycles: u128,
    ) -> Result<Principal, CanisterInstallError> {
        self.try_create_and_install(InstallSpec::new(wasm, init_bytes, install_cycles))
    }

    /// Install one arbitrary wasm module from a generic install specification.
    fn create_and_install(&self, spec: InstallSpec) -> Principal {
        self.try_create_and_install(spec)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Install one arbitrary wasm module from a generic install specification.
    fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, CanisterInstallError> {
        try_create_funded_and_install(self, spec)
    }

    /// Sequentially install multiple arbitrary wasm modules into this PocketIC instance.
    ///
    /// Installs are attempted in iterator order. If one install fails, earlier
    /// installs remain in the PocketIC instance, the failed canister may exist
    /// with the id exposed by `CanisterInstallError::canister_id()`, and later
    /// installs are not attempted.
    fn create_and_install_many<I>(&self, specs: I) -> Vec<Principal>
    where
        I: IntoIterator<Item = InstallSpec>,
    {
        self.try_create_and_install_many(specs)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Sequentially install multiple arbitrary wasm modules into this PocketIC instance.
    ///
    /// Installs are attempted in iterator order. If one install fails, earlier
    /// installs remain in the PocketIC instance, the failed canister may exist
    /// with the id exposed by `CanisterInstallError::canister_id()`, and later
    /// installs are not attempted.
    fn try_create_and_install_many<I>(
        &self,
        specs: I,
    ) -> Result<Vec<Principal>, CanisterInstallError>
    where
        I: IntoIterator<Item = InstallSpec>,
    {
        specs
            .into_iter()
            .map(|spec| self.try_create_and_install(spec))
            .collect()
    }

    /// Wait out the PocketIC `install_code` cooldown window inside the same instance.
    fn wait_out_install_code_rate_limit(&self, cooldown: Duration) {
        self.advance_time(cooldown);
        self.tick();
        self.tick();
    }

    fn retry_install_code<T, F>(&self, policy: RetryPolicy, op: F) -> Result<T, RejectResponse>
    where
        F: FnMut() -> Result<T, RejectResponse>,
    {
        retry_install_code_with(policy, op, || {
            self.wait_out_install_code_rate_limit(policy.cooldown());
        })
    }
}

// Install a canister after creating it and optionally adding extra cycles.
fn try_create_funded_and_install(
    pocket_ic: &PocketIc,
    spec: InstallSpec,
) -> Result<Principal, CanisterInstallError> {
    let canister_id = pocket_ic.create_canister();
    if spec.cycles > 0 {
        let _ = pocket_ic.add_cycles(canister_id, spec.cycles);
    }

    let install = catch_unwind(AssertUnwindSafe(|| {
        pocket_ic.install_canister(canister_id, spec.wasm, spec.init_bytes, spec.install_sender);
    }));
    if let Err(payload) = install {
        let message = transport::panic_payload_to_string(payload.as_ref());
        let context = if let Some(label) = &spec.label {
            format!("install_canister trapped ({label})")
        } else {
            "install_canister trapped".to_string()
        };
        // Diagnostics are best-effort and must never replace the original
        // structured install failure, including if stderr or PocketIC fails.
        let _ = catch_unwind(AssertUnwindSafe(|| {
            pocket_ic.dump_canister_debug(canister_id, &context);
        }));

        return if let Some(label) = spec.label {
            Err(CanisterInstallError::labeled(canister_id, label, message))
        } else {
            Err(CanisterInstallError::new(canister_id, message))
        };
    }

    Ok(canister_id)
}

fn is_install_code_rate_limited(response: &RejectResponse) -> bool {
    response.error_code == ErrorCode::CanisterInstallCodeRateLimited
}

fn retry_install_code_with<T, F, W>(
    policy: RetryPolicy,
    mut op: F,
    mut wait_out_cooldown: W,
) -> Result<T, RejectResponse>
where
    F: FnMut() -> Result<T, RejectResponse>,
    W: FnMut(),
{
    for attempt in 1..=policy.max_attempts() {
        match op() {
            Ok(value) => return Ok(value),
            Err(err) if is_install_code_rate_limited(&err) && attempt < policy.max_attempts() => {
                wait_out_cooldown();
            }
            Err(err) => return Err(err),
        }
    }

    unreachable!("RetryPolicy guarantees at least one attempt")
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, time::Duration};

    use pocket_ic::{ErrorCode, RejectCode, RejectResponse};

    use super::{RetryPolicy, RetryPolicyError, retry_install_code_with};

    fn rejection(error_code: ErrorCode, message: &str) -> RejectResponse {
        RejectResponse {
            reject_code: RejectCode::SysTransient,
            reject_message: message.to_string(),
            error_code,
            certified: false,
        }
    }

    #[test]
    fn retry_policy_counts_the_first_attempt() {
        let attempts = Cell::new(0);
        let waits = Cell::new(0);
        let rate_limited = rejection(
            ErrorCode::CanisterInstallCodeRateLimited,
            "install-code rate limit",
        );
        let result = retry_install_code_with(
            RetryPolicy::try_new(3, Duration::from_secs(1)).expect("valid retry policy"),
            || {
                attempts.set(attempts.get() + 1);
                Err::<(), _>(rate_limited.clone())
            },
            || waits.set(waits.get() + 1),
        );

        assert_eq!(result, Err(rate_limited));
        assert_eq!(attempts.get(), 3);
        assert_eq!(waits.get(), 2);
    }

    #[test]
    fn retry_policy_stops_on_non_rate_limit_failure() {
        let attempts = Cell::new(0);
        let not_retryable = rejection(ErrorCode::CanisterRejectedMessage, "not retryable");
        let result = retry_install_code_with(
            RetryPolicy::try_new(3, Duration::from_secs(1)).expect("valid retry policy"),
            || {
                attempts.set(attempts.get() + 1);
                Err::<(), _>(not_retryable.clone())
            },
            || panic!("non-rate-limit failure must not wait"),
        );

        assert_eq!(result, Err(not_retryable));
        assert_eq!(attempts.get(), 1);
    }

    #[test]
    fn retry_policy_rejects_zero_attempts() {
        assert_eq!(
            RetryPolicy::try_new(0, Duration::from_secs(1)),
            Err(RetryPolicyError::ZeroMaxAttempts)
        );
    }
}
