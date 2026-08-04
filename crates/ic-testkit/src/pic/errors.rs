use candid::Principal;
use pocket_ic::{PocketIc, RejectResponse};

///
/// CandidCallError
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidCallError {
    pub message: String,
    pub kind: CandidCallErrorKind,
    pub context: Option<Box<CandidCallContext>>,
    pub reject_response: Option<Box<RejectResponse>>,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidCallErrorKind {
    Encode,
    Decode,
    CanisterReject,
    Transport,
    Other,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidCallContext {
    pub operation: &'static str,
    pub canister_id: Principal,
    pub caller: Principal,
    pub method: String,
}

///
/// CanisterInstallError
///

#[derive(Debug, Eq, PartialEq)]
pub struct CanisterInstallError {
    canister_id: Principal,
    label: Option<String>,
    message: String,
}

/// A failed standalone install that returns ownership of the caller's instance.
pub struct StandaloneCanisterInstallError {
    pocket_ic: Box<PocketIc>,
    install_error: CanisterInstallError,
}

impl CandidCallContext {
    /// Capture the stable call metadata attached to one call failure.
    #[must_use]
    pub fn new(
        operation: &'static str,
        canister_id: Principal,
        caller: Principal,
        method: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            canister_id,
            caller,
            method: method.into(),
        }
    }

    /// Read the PocketIC operation name, such as `update_call` or `query_call`.
    #[must_use]
    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    /// Read the target canister id.
    #[must_use]
    pub const fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Read the caller principal used for the call.
    #[must_use]
    pub const fn caller(&self) -> Principal {
        self.caller
    }

    /// Read the called method name.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
}

impl CandidCallError {
    /// Capture one PocketIC call/codec failure.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: CandidCallErrorKind::Other,
            context: None,
            reject_response: None,
        }
    }

    /// Capture one contextual Candid encode failure.
    #[must_use]
    pub fn encode(context: CandidCallContext, source: impl std::fmt::Display) -> Self {
        let message = format!(
            "candid encode_args failed (operation={}, canister={}, caller={}, method={}): {source}",
            context.operation, context.canister_id, context.caller, context.method
        );

        Self {
            message,
            kind: CandidCallErrorKind::Encode,
            context: Some(Box::new(context)),
            reject_response: None,
        }
    }

    /// Capture one contextual Candid decode failure.
    #[must_use]
    pub fn decode(
        context: CandidCallContext,
        bytes: usize,
        source: impl std::fmt::Display,
    ) -> Self {
        let message = format!(
            "candid decode_one failed (operation={}, canister={}, caller={}, method={}, bytes={}): {source}",
            context.operation, context.canister_id, context.caller, context.method, bytes
        );

        Self {
            message,
            kind: CandidCallErrorKind::Decode,
            context: Some(Box::new(context)),
            reject_response: None,
        }
    }

    /// Capture one structured rejection returned by PocketIC.
    #[must_use]
    pub fn canister_reject(context: CandidCallContext, response: RejectResponse) -> Self {
        let message = format!(
            "pocket_ic {} was rejected (canister={}, caller={}, method={}): {response}",
            context.operation, context.canister_id, context.caller, context.method
        );

        Self {
            message,
            kind: CandidCallErrorKind::CanisterReject,
            context: Some(Box::new(context)),
            reject_response: Some(Box::new(response)),
        }
    }

    /// Capture one contextual PocketIC transport failure.
    #[must_use]
    pub fn transport(context: CandidCallContext, source: impl std::fmt::Display) -> Self {
        let message = format!(
            "pocket_ic {} failed (canister={}, caller={}, method={}): {source}",
            context.operation, context.canister_id, context.caller, context.method
        );

        Self {
            message,
            kind: CandidCallErrorKind::Transport,
            context: Some(Box::new(context)),
            reject_response: None,
        }
    }

    /// Read the rendered error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Read the structured failure kind.
    #[must_use]
    pub const fn kind(&self) -> CandidCallErrorKind {
        self.kind
    }

    /// Read the structured call context, when available.
    #[must_use]
    pub fn context(&self) -> Option<&CandidCallContext> {
        self.context.as_deref()
    }

    /// Read the structured PocketIC rejection, when the call reached the IC.
    #[must_use]
    pub fn reject_response(&self) -> Option<&RejectResponse> {
        self.reject_response.as_deref()
    }
}

impl std::fmt::Display for CandidCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CandidCallError {}

impl CanisterInstallError {
    /// Capture one install failure for a specific canister id.
    #[must_use]
    pub const fn new(canister_id: Principal, message: String) -> Self {
        Self {
            canister_id,
            label: None,
            message,
        }
    }

    /// Capture one labeled install failure for a specific canister id.
    #[must_use]
    pub fn labeled(
        canister_id: Principal,
        label: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            canister_id,
            label: Some(label.into()),
            message: message.into(),
        }
    }

    /// Read the canister id that failed to install.
    #[must_use]
    pub const fn canister_id(&self) -> Principal {
        self.canister_id
    }

    /// Read the captured panic message from the install attempt.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Read the optional caller-provided install label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

impl std::fmt::Display for CanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(label) = &self.label {
            write!(
                f,
                "failed to install canister {} ({label}): {}",
                self.canister_id, self.message
            )
        } else {
            write!(
                f,
                "failed to install canister {}: {}",
                self.canister_id, self.message
            )
        }
    }
}

impl std::error::Error for CanisterInstallError {}

impl StandaloneCanisterInstallError {
    pub(super) fn new(pocket_ic: PocketIc, install_error: CanisterInstallError) -> Self {
        Self {
            pocket_ic: Box::new(pocket_ic),
            install_error,
        }
    }

    /// Borrow the caller-created instance retained after the failed install.
    #[must_use]
    pub fn pocket_ic(&self) -> &PocketIc {
        self.pocket_ic.as_ref()
    }

    /// Inspect the structured install failure.
    #[must_use]
    pub const fn install_error(&self) -> &CanisterInstallError {
        &self.install_error
    }

    /// Recover ownership of the instance and install failure.
    #[must_use]
    pub fn into_parts(self) -> (PocketIc, CanisterInstallError) {
        (*self.pocket_ic, self.install_error)
    }
}

impl std::fmt::Debug for StandaloneCanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandaloneCanisterInstallError")
            .field("install_error", &self.install_error)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for StandaloneCanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.install_error.fmt(f)
    }
}

impl std::error::Error for StandaloneCanisterInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.install_error)
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;
    use pocket_ic::{ErrorCode, RejectCode, RejectResponse};

    use super::{CandidCallContext, CandidCallError, CandidCallErrorKind, CanisterInstallError};

    #[test]
    fn labeled_install_error_display_includes_label() {
        let err = CanisterInstallError::labeled(Principal::anonymous(), "authority", "trap");

        assert_eq!(err.label(), Some("authority"));
        assert!(err.to_string().contains("(authority): trap"));
    }

    #[test]
    fn canister_reject_preserves_the_upstream_response() {
        let response = RejectResponse {
            reject_code: RejectCode::DestinationInvalid,
            reject_message: "missing canister".to_string(),
            error_code: ErrorCode::CanisterNotFound,
            certified: true,
        };
        let error = CandidCallError::canister_reject(
            CandidCallContext::new(
                "query_call",
                Principal::anonymous(),
                Principal::management_canister(),
                "get",
            ),
            response.clone(),
        );

        assert_eq!(error.kind(), CandidCallErrorKind::CanisterReject);
        assert_eq!(error.reject_response(), Some(&response));
    }
}
