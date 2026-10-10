use std::any::Any;

/// Captured PocketIC operation failure, retaining transport classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PocketIcOperationError {
    message: String,
    transport: bool,
}

impl PocketIcOperationError {
    /// Capture an upstream PocketIC panic message and classify its transport.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        let transport = is_dead_instance_transport_error(&message);
        Self { message, transport }
    }

    pub(super) fn from_panic(payload: &(dyn Any + Send)) -> Self {
        Self::new(panic_payload_to_string(payload))
    }

    /// Read the original upstream message without contextual wrappers.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether the message matches a recognized PocketIC transport failure.
    #[must_use]
    pub const fn is_transport(&self) -> bool {
        self.transport
    }
}

impl std::fmt::Display for PocketIcOperationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PocketIcOperationError {}

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

// Check whether one panic payload belongs to the dead-instance transport class
// without consuming it, so callers can still resume the original panic.
pub(super) fn panic_is_dead_instance_transport(payload: &(dyn Any + Send)) -> bool {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&'static str>().copied())
        .is_some_and(is_dead_instance_transport_error)
}

/// Recognize PocketIC transport failures in typed causes and request-error text.
///
/// The complete error source chain is inspected so a recipe can classify its
/// own wrapper error as [`super::RebuildReason::DeadPocketIcTransport`]. This
/// preserves the classification already captured by testkit's structured errors.
/// Testkit [`super::CandidCallError`], [`super::CanisterDiagnosticFailure`], and
/// [`PocketIcOperationError`] causes retain their transport classification through
/// contextual display wrappers, including nested [`std::io::Error`] values.
/// Snapshot and installation failures expose their operation cause through
/// the error source chain.
///
/// For other errors, message recognition requires a reqwest debug error with a
/// PocketIC instance URL and a recognized transport source, optionally prefixed
/// by PocketIC's HTTP panic context. Generic application messages and bare I/O
/// error kinds do not qualify.
/// Recognized sources include OS connection resets during an HTTP request.
///
/// Use this only for errors originating in a PocketIC operation. This is a
/// message-based heuristic, not proof that the server or instance has died;
/// another service or application can reproduce the same text. PocketIC does
/// not yet expose a structured transport error.
#[must_use]
pub fn is_dead_pocket_ic_transport_error(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(candidate) = current {
        if candidate
            .downcast_ref::<PocketIcOperationError>()
            .is_some_and(PocketIcOperationError::is_transport)
        {
            return true;
        }
        if candidate
            .downcast_ref::<super::CandidCallError>()
            .is_some_and(|error| error.kind() == super::CandidCallErrorKind::Transport)
        {
            return true;
        }
        if matches!(
            candidate.downcast_ref::<super::CanisterDiagnosticFailure>(),
            Some(super::CanisterDiagnosticFailure::InstanceUnavailable { .. })
        ) {
            return true;
        }
        if is_dead_instance_transport_error(&candidate.to_string()) {
            return true;
        }
        // io::Error::source delegates to the contained error's source, skipping
        // the contained error itself. Inspect it before following its causes so
        // contextual errors keep their own structured classification.
        current = if let Some(inner) = candidate
            .downcast_ref::<std::io::Error>()
            .and_then(std::io::Error::get_ref)
        {
            Some(inner)
        } else {
            candidate.source()
        };
    }
    false
}

// Recognize maintained PocketIC request-error shapes for restore recovery.
pub(super) fn is_dead_instance_transport_error(message: &str) -> bool {
    let message = message
        .strip_prefix("HTTP failure: ")
        .or_else(|| message.strip_prefix("called `Result::unwrap()` on an `Err` value: "))
        .unwrap_or(message);
    let Some(error) = message.strip_prefix("reqwest::Error { kind: ") else {
        return false;
    };
    let Some((kind, request)) = error.split_once(", url: \"") else {
        return false;
    };
    if !matches!(kind, "Request" | "Body" | "Decode") {
        return false;
    }
    let Some((url, source)) = request.split_once("\", source: ") else {
        return false;
    };
    let Some(source) = source.strip_suffix(" }") else {
        return false;
    };
    let Some(authority_and_path) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    else {
        return false;
    };
    let Some((authority, path)) = authority_and_path.split_once('/') else {
        return false;
    };
    let Some(instance_path) = path.strip_prefix("instances/") else {
        return false;
    };
    let instance_id = instance_path.split('/').next().unwrap_or_default();
    if authority.is_empty()
        || instance_id.is_empty()
        || !instance_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        return false;
    }
    let source = source
        .strip_prefix("hyper_util::client::legacy::Error(Connect, ")
        .or_else(|| source.strip_prefix("hyper_util::client::legacy::Error(SendRequest, "))
        .and_then(|source| source.strip_suffix(')'))
        .unwrap_or(source);
    let reset = source
        .strip_prefix("hyper::Error(Io, Os { code: ")
        .and_then(|source| source.split_once(", kind: ConnectionReset, message: \""))
        .is_some_and(|(code, message)| code.parse::<i32>().is_ok() && message.ends_with("\" })"));
    (source.starts_with("ConnectError(") && source.contains("kind: ConnectionRefused,"))
        || reset
        || matches!(
            source,
            "hyper::Error(IncompleteMessage)"
                | "hyper::Error(ChannelClosed)"
                | "hyper::Error(Canceled, \"connection closed before message completed\")"
        )
}

#[cfg(test)]
mod tests {
    use super::super::{CanisterInstallError, CanisterInstallPhase, ControllerSnapshotError};
    use super::{
        PocketIcOperationError, is_dead_instance_transport_error,
        is_dead_pocket_ic_transport_error, panic_is_dead_instance_transport,
    };

    #[derive(Debug)]
    struct WrapperError(std::io::Error);

    const REFUSED: &str = "reqwest::Error { kind: Request, url: \"http://127.0.0.1:1234/instances/0/update/tick\", source: hyper_util::client::legacy::Error(Connect, ConnectError(\"tcp connect error\", 127.0.0.1:1234, Os { code: 111, kind: ConnectionRefused, message: \"Connection refused\" })) }";
    const INCOMPLETE: &str = "reqwest::Error { kind: Request, url: \"http://127.0.0.1:1234/instances/0/read/get_time\", source: hyper::Error(IncompleteMessage) }";
    const RESET: &str = "reqwest::Error { kind: Request, url: \"http://127.0.0.1:49238/instances/0/update/submit_ingress_message\", source: hyper_util::client::legacy::Error(SendRequest, hyper::Error(Io, Os { code: 54, kind: ConnectionReset, message: \"Connection reset by peer\" })) }";

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
        assert!(is_dead_instance_transport_error(REFUSED));
        assert!(is_dead_instance_transport_error(&format!(
            "HTTP failure: {REFUSED}"
        )));
    }

    #[test]
    fn dead_instance_transport_error_detects_incomplete_message() {
        assert!(is_dead_instance_transport_error(INCOMPLETE));
        assert!(is_dead_instance_transport_error(&format!(
            "called `Result::unwrap()` on an `Err` value: {INCOMPLETE}"
        )));
    }

    #[test]
    fn dead_instance_transport_error_detects_native_connection_reset() {
        for message in [RESET.to_owned(), RESET.replace("code: 54", "code: 104")] {
            assert!(is_dead_instance_transport_error(&message));
            assert!(panic_is_dead_instance_transport(&format!(
                "HTTP failure: {message}"
            )));
            assert!(is_dead_instance_transport_error(&format!(
                "called `Result::unwrap()` on an `Err` value: {message}"
            )));
        }
    }

    #[test]
    fn classify_pocket_ic_panic_marks_dead_instance_transport() {
        let classified = PocketIcOperationError::new(INCOMPLETE);

        assert!(classified.is_transport());
        assert!(panic_is_dead_instance_transport(&INCOMPLETE));
    }

    #[test]
    fn public_classifier_inspects_the_error_source_chain() {
        let dead = WrapperError(std::io::Error::other(REFUSED));
        assert!(is_dead_pocket_ic_transport_error(&dead));

        let unrelated = WrapperError(std::io::Error::other("request rejected"));
        assert!(!is_dead_pocket_ic_transport_error(&unrelated));
    }

    #[test]
    fn snapshot_and_install_wrappers_preserve_transport_causes() {
        let canister_id = candid::Principal::anonymous();
        for message in [REFUSED, RESET, "unrelated application panic"] {
            let expected = message != "unrelated application panic";
            let capture = ControllerSnapshotError::CapturePanicked {
                canister_id,
                source: PocketIcOperationError::new(message),
                cleanup_failures: vec![],
            };
            let restore = ControllerSnapshotError::RestorePanicked {
                canister_id,
                source: PocketIcOperationError::new(message),
            };
            for snapshot in [capture, restore] {
                assert_eq!(is_dead_pocket_ic_transport_error(&snapshot), expected);
                let wrapper = WrapperError(std::io::Error::other(snapshot));
                assert_eq!(is_dead_pocket_ic_transport_error(&wrapper), expected);
            }
            for phase in [
                CanisterInstallPhase::CreateCanister,
                CanisterInstallPhase::AddCycles,
                CanisterInstallPhase::InstallCode,
            ] {
                let install = CanisterInstallError::new(
                    phase,
                    None,
                    Some("fixture".into()),
                    PocketIcOperationError::new(message),
                );
                assert_eq!(is_dead_pocket_ic_transport_error(&install), expected);
                let wrapper = WrapperError(std::io::Error::other(install));
                assert_eq!(is_dead_pocket_ic_transport_error(&wrapper), expected);
            }
        }
    }

    #[test]
    fn unrelated_and_quoted_transport_text_does_not_qualify() {
        for message in [
            "application worker channel closed",
            "fixture expected ConnectionRefused but observed another result",
            "tcp connect error",
            "connection closed before message completed",
            "IncompleteMessage",
            &format!("fixture quoted {REFUSED}"),
            &REFUSED.replace("/instances/0/update/tick", "/application/worker"),
            &REFUSED.replace("/instances/0/", "/instances/quoted/"),
            &INCOMPLETE.replace(
                "hyper::Error(IncompleteMessage)",
                "Custom(\"channel closed\")",
            ),
            &INCOMPLETE.replace(
                "hyper::Error(IncompleteMessage)",
                "Custom(\"hyper::Error(IncompleteMessage)\")",
            ),
            &format!("fixture quoted {RESET}"),
            &RESET.replace("/instances/0/", "/application/worker/"),
            &RESET.replace("/instances/0/", "/instances/quoted/"),
            &RESET.replace("kind: ConnectionReset,", "kind: ConnectionAborted,"),
            &RESET.replace("code: 54", "code: quoted"),
            &RESET.replace("hyper::Error(Io, Os {", "Custom(Os {"),
        ] {
            let error = WrapperError(std::io::Error::other(message.to_owned()));
            assert!(!is_dead_pocket_ic_transport_error(&error), "{message}");
            assert!(!PocketIcOperationError::new(message).is_transport());
            assert!(!panic_is_dead_instance_transport(&message.to_owned()));
        }
        let refused = std::io::Error::from(std::io::ErrorKind::ConnectionRefused);
        assert!(!is_dead_pocket_ic_transport_error(&refused));
        assert!(!is_dead_pocket_ic_transport_error(&WrapperError(refused)));
        let reset = std::io::Error::from(std::io::ErrorKind::ConnectionReset);
        assert!(!is_dead_pocket_ic_transport_error(&reset));
        assert!(!is_dead_pocket_ic_transport_error(&WrapperError(reset)));
    }

    #[test]
    fn structured_channel_closure_on_an_instance_request_qualifies() {
        let message = INCOMPLETE.replace("IncompleteMessage", "ChannelClosed");
        assert!(is_dead_instance_transport_error(&message));
    }

    #[test]
    fn contextual_call_errors_use_their_structured_kind() {
        let context = super::super::CandidCallContext::new(
            "query_call",
            candid::Principal::anonymous(),
            candid::Principal::anonymous(),
            "get",
        );
        let error = super::super::CandidCallError::transport(context.clone(), REFUSED);
        assert!(is_dead_pocket_ic_transport_error(&error));
        let error = super::super::CandidCallError::decode(context, 0, "channel closed");
        assert!(!is_dead_pocket_ic_transport_error(&error));
    }

    #[test]
    fn io_wrappers_preserve_contextual_call_error_kinds() {
        let context = super::super::CandidCallContext::new(
            "query_call",
            candid::Principal::anonymous(),
            candid::Principal::anonymous(),
            "get",
        );
        for (error, expected) in [
            (
                super::super::CandidCallError::transport(context.clone(), REFUSED),
                true,
            ),
            (
                super::super::CandidCallError::decode(context, 0, REFUSED),
                false,
            ),
        ] {
            let inner = std::io::Error::other(error);
            assert_eq!(is_dead_pocket_ic_transport_error(&inner), expected);
            let wrapped = WrapperError(std::io::Error::other(inner));
            assert_eq!(is_dead_pocket_ic_transport_error(&wrapped), expected);
        }
    }
}
