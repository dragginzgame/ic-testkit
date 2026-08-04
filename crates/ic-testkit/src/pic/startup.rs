use std::{any::Any, panic::catch_unwind};

use pocket_ic::{PocketIc, PocketIcBuilder};

///
/// PocketIcStartError
///

#[derive(Debug, Eq, PartialEq)]
pub enum PocketIcStartError {
    BinaryUnavailable { message: String },
    BinaryInvalid { message: String },
    DownloadFailed { message: String },
    ServerStartFailed { message: String },
    StartupTimedOut { message: String },
    Panic { message: String },
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum PocketIcPanicKind {
    DeadInstanceTransport { message: String },
    Other { message: String },
}

pub(super) fn try_build_pocket_ic(
    builder: PocketIcBuilder,
) -> Result<PocketIc, PocketIcStartError> {
    let build = catch_unwind(|| builder.build());

    match build {
        Ok(pocket_ic) => Ok(pocket_ic),
        Err(payload) => Err(classify_pocket_ic_start_panic(payload)),
    }
}

impl std::fmt::Display for PocketIcStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryUnavailable { message }
            | Self::BinaryInvalid { message }
            | Self::DownloadFailed { message }
            | Self::ServerStartFailed { message }
            | Self::StartupTimedOut { message }
            | Self::Panic { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for PocketIcStartError {}

// Extract a stable string message from one panic payload.
pub(super) fn panic_payload_to_string(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }

    "non-string panic payload".to_string()
}

// Classify one panic payload so callers can recover dead-instance restores
// without repeating transport-string matching at each call site.
pub(super) fn classify_pocket_ic_panic(payload: Box<dyn Any + Send>) -> PocketIcPanicKind {
    let message = panic_payload_to_string(payload.as_ref());

    if is_dead_instance_transport_error(&message) {
        return PocketIcPanicKind::DeadInstanceTransport { message };
    }

    PocketIcPanicKind::Other { message }
}

// Check whether one panic payload belongs to the dead-instance transport class
// without consuming it, so callers can still resume the original panic.
pub(super) fn panic_is_dead_instance_transport(payload: &(dyn Any + Send)) -> bool {
    matches!(
        classify_pocket_ic_panic(Box::new(panic_payload_to_string(payload))),
        PocketIcPanicKind::DeadInstanceTransport { .. }
    )
}

// Detect the PocketIC transport failure class that means the owned instance
// has already died and cached snapshot restore should rebuild from scratch.
pub(super) fn is_dead_instance_transport_error(message: &str) -> bool {
    message.contains("ConnectionRefused")
        || message.contains("tcp connect error")
        || message.contains("IncompleteMessage")
        || message.contains("connection closed before message completed")
        || message.contains("channel closed")
}

// Classify one PocketIC startup panic into a typed public error.
fn classify_pocket_ic_start_panic(payload: Box<dyn Any + Send>) -> PocketIcStartError {
    let message = match classify_pocket_ic_panic(payload) {
        PocketIcPanicKind::DeadInstanceTransport { message }
        | PocketIcPanicKind::Other { message } => message,
    };

    if message.starts_with("Failed to validate PocketIC server binary") {
        if message.contains("No such file or directory") || message.contains("os error 2") {
            return PocketIcStartError::BinaryUnavailable { message };
        }

        return PocketIcStartError::BinaryInvalid { message };
    }

    if message.starts_with("Failed to download PocketIC server")
        || message.starts_with("Failed to write PocketIC server binary")
    {
        return PocketIcStartError::DownloadFailed { message };
    }

    if message.starts_with("Failed to start PocketIC binary")
        || message.starts_with("Failed to create PocketIC server directory")
    {
        return PocketIcStartError::ServerStartFailed { message };
    }

    if message.starts_with("Timed out waiting for PocketIC server being available") {
        return PocketIcStartError::StartupTimedOut { message };
    }

    PocketIcStartError::Panic { message }
}

#[cfg(test)]
mod tests {
    use super::{
        PocketIcPanicKind, PocketIcStartError, classify_pocket_ic_panic,
        classify_pocket_ic_start_panic, is_dead_instance_transport_error,
    };

    #[test]
    fn pocket_ic_start_error_classifies_missing_binary() {
        let error = classify_pocket_ic_start_panic(Box::new(
            "Failed to validate PocketIC server binary `/tmp/pocket-ic`: `No such file or directory (os error 2)`.".to_string(),
        ));

        assert!(matches!(
            error,
            PocketIcStartError::BinaryUnavailable { .. }
        ));
    }

    #[test]
    fn pocket_ic_start_error_classifies_failed_spawn() {
        let error = classify_pocket_ic_start_panic(Box::new(
            "Failed to start PocketIC binary (/tmp/pocket-ic)".to_string(),
        ));

        assert!(matches!(
            error,
            PocketIcStartError::ServerStartFailed { .. }
        ));
    }

    #[test]
    fn dead_instance_transport_error_detects_connection_refused() {
        assert!(is_dead_instance_transport_error(
            "reqwest::Error { source: ConnectError(\"tcp connect error\", 127.0.0.1:1234, Os { code: 111, kind: ConnectionRefused, message: \"Connection refused\" }) }"
        ));
    }

    #[test]
    fn dead_instance_transport_error_detects_incomplete_message() {
        assert!(is_dead_instance_transport_error(
            "reqwest::Error { source: hyper::Error(IncompleteMessage) }"
        ));
    }

    #[test]
    fn classify_pocket_ic_panic_marks_dead_instance_transport() {
        let classified = classify_pocket_ic_panic(Box::new(
            "reqwest::Error { source: hyper::Error(IncompleteMessage) }".to_string(),
        ));

        assert!(matches!(
            classified,
            PocketIcPanicKind::DeadInstanceTransport { .. }
        ));
    }
}
