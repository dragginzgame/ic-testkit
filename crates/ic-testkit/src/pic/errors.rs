use candid::Principal;

use super::{PicSerialGuardError, startup::PicStartError};

///
/// PicCallError
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PicCallError {
    pub message: String,
    pub kind: PicCallErrorKind,
    pub context: Option<Box<PicCallContext>>,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PicCallErrorKind {
    Encode,
    Decode,
    Transport,
    Other,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PicCallContext {
    pub operation: &'static str,
    pub canister_id: Principal,
    pub caller: Principal,
    pub method: String,
}

///
/// PicInstallError
///

#[derive(Debug, Eq, PartialEq)]
pub struct PicInstallError {
    canister_id: Principal,
    label: Option<String>,
    message: String,
}

///
/// StandaloneCanisterFixtureError
///

#[derive(Debug)]
pub enum StandaloneCanisterFixtureError {
    SerialGuard(PicSerialGuardError),
    Start(PicStartError),
    Install(PicInstallError),
}

impl PicCallContext {
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

impl PicCallError {
    /// Capture one PocketIC call/codec failure.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: PicCallErrorKind::Other,
            context: None,
        }
    }

    /// Capture one contextual Candid encode failure.
    #[must_use]
    pub fn encode(context: PicCallContext, source: impl std::fmt::Display) -> Self {
        let message = format!(
            "candid encode_args failed (operation={}, canister={}, caller={}, method={}): {source}",
            context.operation, context.canister_id, context.caller, context.method
        );

        Self {
            message,
            kind: PicCallErrorKind::Encode,
            context: Some(Box::new(context)),
        }
    }

    /// Capture one contextual Candid decode failure.
    #[must_use]
    pub fn decode(context: PicCallContext, bytes: usize, source: impl std::fmt::Display) -> Self {
        let message = format!(
            "candid decode_one failed (operation={}, canister={}, caller={}, method={}, bytes={}): {source}",
            context.operation, context.canister_id, context.caller, context.method, bytes
        );

        Self {
            message,
            kind: PicCallErrorKind::Decode,
            context: Some(Box::new(context)),
        }
    }

    /// Capture one contextual PocketIC transport failure.
    #[must_use]
    pub fn transport(context: PicCallContext, source: impl std::fmt::Display) -> Self {
        let message = format!(
            "pocket_ic {} failed (canister={}, caller={}, method={}): {source}",
            context.operation, context.canister_id, context.caller, context.method
        );

        Self {
            message,
            kind: PicCallErrorKind::Transport,
            context: Some(Box::new(context)),
        }
    }

    /// Read the rendered error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Read the structured failure kind.
    #[must_use]
    pub const fn kind(&self) -> PicCallErrorKind {
        self.kind
    }

    /// Read the structured call context, when available.
    #[must_use]
    pub fn context(&self) -> Option<&PicCallContext> {
        self.context.as_deref()
    }
}

impl std::fmt::Display for PicCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PicCallError {}

impl PicInstallError {
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

impl std::fmt::Display for PicInstallError {
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

impl std::error::Error for PicInstallError {}

impl std::fmt::Display for StandaloneCanisterFixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerialGuard(err) => write!(f, "{err}"),
            Self::Start(err) => write!(f, "{err}"),
            Self::Install(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for StandaloneCanisterFixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SerialGuard(err) => Some(err),
            Self::Start(err) => Some(err),
            Self::Install(err) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::PicInstallError;

    #[test]
    fn labeled_install_error_display_includes_label() {
        let err = PicInstallError::labeled(Principal::anonymous(), "authority", "trap");

        assert_eq!(err.label(), Some("authority"));
        assert!(err.to_string().contains("(authority): trap"));
    }
}
