use std::{num::NonZeroUsize, time::Duration};

use candid::Principal;
use pocket_ic::{ErrorCode, PocketIc, RejectResponse};

use super::{
    CanisterDiagnosticsRequest, CanisterInstallError, CanisterInstallPhase, PocketIcDiagnosticsExt,
    transport::capture_operation,
};

/// Inputs and diagnostic context for one generic canister installation.
#[non_exhaustive]
pub struct InstallSpec {
    /// Wasm module installed into the newly created canister.
    pub wasm: Vec<u8>,
    /// Raw Candid or application-specific initialization bytes.
    pub init_bytes: Vec<u8>,
    /// Extra cycles added after PocketIC creates the canister and before install.
    pub cycles: u128,
    /// Optional sender for the management-canister install operation.
    pub install_sender: Option<Principal>,
    /// Optional human-readable context included in install errors.
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
    max_attempts: NonZeroUsize,
    cooldown: Duration,
}

impl RetryPolicy {
    /// Create a policy with an exact, non-zero maximum attempt count.
    #[must_use]
    pub const fn new(max_attempts: NonZeroUsize, cooldown: Duration) -> Self {
        Self {
            max_attempts,
            cooldown,
        }
    }

    /// Read the maximum number of operation attempts, including the first.
    #[must_use]
    pub const fn max_attempts(self) -> NonZeroUsize {
        self.max_attempts
    }

    /// Read the simulated cooldown applied between rate-limited attempts.
    #[must_use]
    pub const fn cooldown(self) -> Duration {
        self.cooldown
    }
}

/// Generic canister installation and structured install-code retry policy.
///
/// The extension creates canisters using PocketIC defaults. `InstallSpec::cycles`
/// is an additional top-up, not the complete initial balance.
pub trait CanisterInstallExt {
    /// Create and install one canister from a reusable specification.
    ///
    /// Captures PocketIC panics during creation, cycle funding, and installation.
    /// The error identifies the stage and retains an id when creation succeeded.
    fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, CanisterInstallError>;

    /// Sequentially create and install multiple canisters, stopping at the first failure.
    fn try_create_and_install_many<I>(
        &self,
        specs: I,
    ) -> Result<Vec<Principal>, CanisterInstallError>
    where
        I: IntoIterator<Item = InstallSpec>;

    /// Advance simulated time and rounds past an install-code cooldown.
    fn wait_out_install_code_rate_limit(&self, cooldown: Duration);

    /// Retry only while PocketIC reports install-code rate limiting.
    ///
    /// `RetryPolicy::max_attempts` includes the initial call. Before each
    /// retry, this advances simulated time by the configured cooldown and
    /// executes two PocketIC ticks. Other rejections are returned unchanged.
    fn retry_install_code<T, F>(&self, policy: RetryPolicy, op: F) -> Result<T, RejectResponse>
    where
        F: FnMut() -> Result<T, RejectResponse>;
}

impl CanisterInstallExt for PocketIc {
    /// Install one arbitrary wasm module from a generic install specification.
    fn try_create_and_install(&self, spec: InstallSpec) -> Result<Principal, CanisterInstallError> {
        let canister_id =
            try_install_step(CanisterInstallPhase::CreateCanister, None, &spec, || {
                self.create_canister()
            })?;
        let diagnostic_sender = spec.install_sender.unwrap_or_else(Principal::anonymous);
        if spec.cycles > 0 {
            try_install_step(
                CanisterInstallPhase::AddCycles,
                Some(canister_id),
                &spec,
                || self.add_cycles(canister_id, spec.cycles),
            )?;
        }

        // Move the Wasm and init payload while retaining the label for failures.
        let install = capture_operation(|| {
            self.install_canister(canister_id, spec.wasm, spec.init_bytes, spec.install_sender);
        });
        if let Err(source) = install {
            let context = if let Some(label) = &spec.label {
                format!("install_canister trapped ({label})")
            } else {
                "install_canister trapped".to_string()
            };
            // Diagnostics are best-effort and must never replace the original
            // structured install failure, including if stderr or PocketIC fails.
            let _ = capture_operation(|| {
                let report = self.collect_canister_diagnostics(CanisterDiagnosticsRequest::new(
                    canister_id,
                    diagnostic_sender,
                    diagnostic_sender,
                ));
                eprintln!("{context}: {report}");
            });

            return Err(CanisterInstallError::new(
                CanisterInstallPhase::InstallCode,
                Some(canister_id),
                spec.label,
                source,
            ));
        }

        Ok(canister_id)
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

fn try_install_step<T>(
    phase: CanisterInstallPhase,
    canister_id: Option<Principal>,
    spec: &InstallSpec,
    operation: impl FnOnce() -> T,
) -> Result<T, CanisterInstallError> {
    capture_operation(operation)
        .map_err(|source| CanisterInstallError::new(phase, canister_id, spec.label.clone(), source))
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
    let mut retries = 1..policy.max_attempts().get();
    loop {
        match op() {
            Err(err)
                if err.error_code == ErrorCode::CanisterInstallCodeRateLimited
                    && retries.next().is_some() =>
            {
                wait_out_cooldown();
            }
            result => return result,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        num::NonZeroUsize,
        time::Duration,
    };

    use pocket_ic::{ErrorCode, RejectCode, RejectResponse};

    use super::{RetryPolicy, retry_install_code_with};

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
        for max_attempts in [1, 3] {
            let attempts = Cell::new(0);
            let waits = Cell::new(0);
            let result = retry_install_code_with(
                RetryPolicy::new(
                    NonZeroUsize::new(max_attempts).unwrap(),
                    Duration::from_secs(1),
                ),
                || {
                    attempts.set(attempts.get() + 1);
                    Err::<(), _>(rejection(
                        ErrorCode::CanisterInstallCodeRateLimited,
                        &format!("rate limit on attempt {}", attempts.get()),
                    ))
                },
                || waits.set(waits.get() + 1),
            );

            assert_eq!(
                result,
                Err(rejection(
                    ErrorCode::CanisterInstallCodeRateLimited,
                    &format!("rate limit on attempt {max_attempts}"),
                ))
            );
            assert_eq!(attempts.get(), max_attempts);
            assert_eq!(waits.get(), max_attempts - 1);
        }
    }

    #[test]
    fn retry_policy_waits_only_between_retryable_attempts() {
        for success_attempt in [1, 3] {
            let events = RefCell::new(Vec::new());
            let attempts = Cell::new(0);
            let result = retry_install_code_with(
                RetryPolicy::new(NonZeroUsize::new(3).unwrap(), Duration::from_secs(1)),
                || {
                    events.borrow_mut().push("attempt");
                    attempts.set(attempts.get() + 1);
                    if attempts.get() == success_attempt {
                        Ok(42)
                    } else {
                        Err(rejection(
                            ErrorCode::CanisterInstallCodeRateLimited,
                            "rate limit",
                        ))
                    }
                },
                || events.borrow_mut().push("wait"),
            );
            assert_eq!(result, Ok(42));
            assert_eq!(attempts.get(), success_attempt);
            assert_eq!(
                events.into_inner(),
                if success_attempt == 1 {
                    vec!["attempt"]
                } else {
                    vec!["attempt", "wait", "attempt", "wait", "attempt"]
                }
            );
        }
    }

    #[test]
    fn retry_policy_stops_on_non_rate_limit_failure() {
        let attempts = Cell::new(0);
        let not_retryable = rejection(ErrorCode::CanisterRejectedMessage, "not retryable");
        let result = retry_install_code_with(
            RetryPolicy::new(NonZeroUsize::new(3).unwrap(), Duration::from_secs(1)),
            || {
                attempts.set(attempts.get() + 1);
                Err::<(), _>(not_retryable.clone())
            },
            || panic!("non-rate-limit failure must not wait"),
        );

        assert_eq!(result, Err(not_retryable));
        assert_eq!(attempts.get(), 1);
    }
}
