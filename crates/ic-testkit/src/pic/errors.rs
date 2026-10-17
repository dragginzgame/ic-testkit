use candid::Principal;
use pocket_ic::{PocketIc, RejectResponse};

use super::{PocketIcInstance, PocketIcOperationError};

/// Structured failure from a [`super::CandidCallExt`] operation.
///
/// Match the variant to inspect its required context and original failure details.
#[non_exhaustive]
#[derive(Debug)]
pub enum CandidCallError {
    /// Arguments could not be Candid-encoded.
    Encode {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Original codec failure text.
        message: String,
    },
    /// Successful response bytes could not be Candid-decoded.
    Decode {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Number of reply bytes presented to the decoder.
        bytes: usize,
        /// Original bounded decoder failure.
        source: Box<ic_testkit_candid::DecodeError>,
    },
    /// PocketIC returned a structured canister rejection.
    CanisterReject {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Complete upstream rejection.
        response: Box<RejectResponse>,
    },
    /// The PocketIC instance became unreachable.
    Transport {
        /// Target, caller, method and raw PocketIC operation.
        context: Box<CandidCallContext>,
        /// Original upstream operation failure and transport classification.
        source: PocketIcOperationError,
    },
}

/// Stable call metadata attached to a contextual call failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidCallContext {
    /// Underlying PocketIC operation, such as `update_call` or `query_call`.
    pub operation: &'static str,
    /// Target canister.
    pub canister_id: Principal,
    /// Effective message caller.
    pub caller: Principal,
    /// Canister method name.
    pub method: String,
}

/// Failed creation, funding, or installation with its original operation error.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum CanisterInstallError {
    /// Canister creation failed before an id was returned.
    CreateCanister {
        /// Optional caller-provided install label.
        label: Option<String>,
        /// Original upstream operation failure.
        source: PocketIcOperationError,
    },
    /// Cycle funding failed for the created canister.
    AddCycles {
        /// Created canister, retained for caller inspection.
        canister_id: Principal,
        /// Optional caller-provided install label.
        label: Option<String>,
        /// Original upstream operation failure.
        source: PocketIcOperationError,
    },
    /// Wasm installation failed for the created canister.
    InstallCode {
        /// Created canister, retained for caller inspection.
        canister_id: Principal,
        /// Optional caller-provided install label.
        label: Option<String>,
        /// Original upstream operation failure.
        source: PocketIcOperationError,
    },
}

/// Failed standalone install retaining the caller-created instance and optional server.
pub struct StandaloneCanisterInstallError {
    instance: Box<PocketIcInstance<PocketIc>>,
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
}

impl std::fmt::Display for CandidCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let context = match self {
            Self::Encode { context, message } => {
                write!(f, "candid encode_args failed: {message}")?;
                context
            }
            Self::Decode {
                context,
                bytes,
                source,
            } => {
                write!(f, "candid decode failed (bytes={bytes}): {source}")?;
                context
            }
            Self::CanisterReject { context, response } => {
                write!(
                    f,
                    "pocket_ic {} was rejected: {response}",
                    context.operation
                )?;
                context
            }
            Self::Transport { context, source } => {
                write!(f, "pocket_ic {} failed: {source}", context.operation)?;
                context
            }
        };
        write!(
            f,
            " (operation={}, canister={}, caller={}, method={})",
            context.operation, context.canister_id, context.caller, context.method
        )
    }
}

impl std::error::Error for CandidCallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transport { source, .. } => Some(source),
            Self::Decode { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl std::fmt::Display for CanisterInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (operation, canister_id, label, source) = match self {
            Self::CreateCanister { label, source } => ("create_canister", None, label, source),
            Self::AddCycles {
                canister_id,
                label,
                source,
            } => ("add_cycles", Some(canister_id), label, source),
            Self::InstallCode {
                canister_id,
                label,
                source,
            } => ("install_canister", Some(canister_id), label, source),
        };
        write!(f, "{operation} failed")?;
        if let Some(canister_id) = canister_id {
            write!(f, " for canister {canister_id}")?;
        }
        if let Some(label) = label {
            write!(f, " ({label})")?;
        }
        write!(f, ": {source}")
    }
}

impl std::error::Error for CanisterInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let (Self::CreateCanister { source, .. }
        | Self::AddCycles { source, .. }
        | Self::InstallCode { source, .. }) = self;
        Some(source)
    }
}

impl StandaloneCanisterInstallError {
    pub(super) fn new(
        instance: PocketIcInstance<PocketIc>,
        install_error: CanisterInstallError,
    ) -> Self {
        Self {
            instance: Box::new(instance),
            install_error,
        }
    }

    /// Borrow the caller-created instance retained after the failed install.
    #[must_use]
    pub fn pocket_ic(&self) -> &PocketIc {
        &self.instance.instance
    }

    /// Inspect the structured install failure.
    #[must_use]
    pub const fn install_error(&self) -> &CanisterInstallError {
        &self.install_error
    }

    /// Recover the instance/server ownership record and install failure.
    pub fn into_parts(self) -> (PocketIcInstance<PocketIc>, CanisterInstallError) {
        (*self.instance, self.install_error)
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

    use super::CanisterInstallError;

    #[test]
    fn labeled_install_error_display_includes_label() {
        let err = CanisterInstallError::InstallCode {
            canister_id: Principal::anonymous(),
            label: Some("authority".into()),
            source: super::PocketIcOperationError::new("trap"),
        };
        assert!(err.to_string().contains("(authority): trap"));
    }
}
