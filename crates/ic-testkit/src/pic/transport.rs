use std::any::Any;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum PocketIcPanicKind {
    DeadInstanceTransport { message: String },
    Other { message: String },
}

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

    if message_is_dead_instance_transport_error(&message) {
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
    message_is_dead_instance_transport_error(message)
}

/// Detect PocketIC's currently unstructured dead-transport failure class.
///
/// The complete error source chain is inspected so a recipe can classify its
/// own wrapper error as [`super::RebuildReason::DeadPocketIcTransport`]. This
/// remains a conservative message-based boundary until PocketIC exposes a
/// structured transport error.
#[must_use]
pub fn is_dead_pocket_ic_transport_error(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(candidate) = current {
        if message_is_dead_instance_transport_error(&candidate.to_string()) {
            return true;
        }
        current = candidate.source();
    }
    false
}

fn message_is_dead_instance_transport_error(message: &str) -> bool {
    message.contains("ConnectionRefused")
        || message.contains("tcp connect error")
        || message.contains("IncompleteMessage")
        || message.contains("connection closed before message completed")
        || message.contains("channel closed")
}

#[cfg(test)]
mod tests {
    use super::{
        PocketIcPanicKind, classify_pocket_ic_panic, is_dead_instance_transport_error,
        is_dead_pocket_ic_transport_error,
    };

    #[derive(Debug)]
    struct WrapperError(std::io::Error);

    impl std::fmt::Display for WrapperError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("wrapped PocketIC request failed")
        }
    }

    impl std::error::Error for WrapperError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
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

    #[test]
    fn public_classifier_inspects_the_error_source_chain() {
        let dead = WrapperError(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "tcp connect error: ConnectionRefused",
        ));
        assert!(is_dead_pocket_ic_transport_error(&dead));

        let unrelated = WrapperError(std::io::Error::other("request rejected"));
        assert!(!is_dead_pocket_ic_transport_error(&unrelated));
    }
}
