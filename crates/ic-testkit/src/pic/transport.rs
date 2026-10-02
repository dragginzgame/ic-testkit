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

// Recognize maintained PocketIC request-error shapes for restore recovery.
pub(super) fn is_dead_instance_transport_error(message: &str) -> bool {
    message_is_dead_instance_transport_error(message)
}

/// Recognize transport failures in PocketIC's unstructured request errors.
///
/// The complete error source chain is inspected so a recipe can classify its
/// own wrapper error as [`super::RebuildReason::DeadPocketIcTransport`]. This
/// requires a reqwest debug error with a PocketIC instance URL and a recognized
/// transport source, optionally prefixed by PocketIC's HTTP panic context.
/// Generic application messages and bare I/O error kinds do not qualify.
/// A testkit [`super::CandidCallError`] tagged as transport is recognized by
/// its structured kind rather than by parsing its contextual display message.
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
            .downcast_ref::<super::CandidCallError>()
            .is_some_and(|error| error.kind() == super::CandidCallErrorKind::Transport)
        {
            return true;
        }
        if message_is_dead_instance_transport_error(&candidate.to_string()) {
            return true;
        }
        current = candidate.source();
    }
    false
}

fn message_is_dead_instance_transport_error(message: &str) -> bool {
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
    (source.starts_with("ConnectError(") && source.contains("kind: ConnectionRefused,"))
        || matches!(
            source,
            "hyper::Error(IncompleteMessage)"
                | "hyper::Error(ChannelClosed)"
                | "hyper::Error(Canceled, \"connection closed before message completed\")"
        )
}

#[cfg(test)]
mod tests {
    use super::{
        PocketIcPanicKind, classify_pocket_ic_panic, is_dead_instance_transport_error,
        is_dead_pocket_ic_transport_error,
    };

    #[derive(Debug)]
    struct WrapperError(std::io::Error);

    const REFUSED: &str = "reqwest::Error { kind: Request, url: \"http://127.0.0.1:1234/instances/0/update/tick\", source: hyper_util::client::legacy::Error(Connect, ConnectError(\"tcp connect error\", 127.0.0.1:1234, Os { code: 111, kind: ConnectionRefused, message: \"Connection refused\" })) }";
    const INCOMPLETE: &str = "reqwest::Error { kind: Request, url: \"http://127.0.0.1:1234/instances/0/read/get_time\", source: hyper::Error(IncompleteMessage) }";

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
    fn classify_pocket_ic_panic_marks_dead_instance_transport() {
        let classified = classify_pocket_ic_panic(Box::new(INCOMPLETE.to_owned()));

        assert!(matches!(
            classified,
            PocketIcPanicKind::DeadInstanceTransport { .. }
        ));
    }

    #[test]
    fn public_classifier_inspects_the_error_source_chain() {
        let dead = WrapperError(std::io::Error::other(REFUSED));
        assert!(is_dead_pocket_ic_transport_error(&dead));

        let unrelated = WrapperError(std::io::Error::other("request rejected"));
        assert!(!is_dead_pocket_ic_transport_error(&unrelated));
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
        ] {
            let error = WrapperError(std::io::Error::other(message.to_owned()));
            assert!(!is_dead_pocket_ic_transport_error(&error), "{message}");
            assert!(matches!(
                classify_pocket_ic_panic(Box::new(message.to_owned())),
                PocketIcPanicKind::Other { .. }
            ));
        }
        let refused = std::io::Error::from(std::io::ErrorKind::ConnectionRefused);
        assert!(!is_dead_pocket_ic_transport_error(&refused));
        assert!(!is_dead_pocket_ic_transport_error(&WrapperError(refused)));
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
}
