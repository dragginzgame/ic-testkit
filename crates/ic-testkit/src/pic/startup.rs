use std::panic::{AssertUnwindSafe, catch_unwind};

use pocket_ic::{PocketIc, PocketIcBuilder};

use super::transport;

/// A panic raised while PocketIC constructs an instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PocketIcStartupError {
    message: String,
}

/// Fallible construction at PocketIC's currently panicking builder boundary.
pub trait PocketIcBuilderExt {
    /// Build one PocketIC instance while capturing an upstream startup panic.
    ///
    /// This method deliberately does not classify panic text. It exists so a
    /// test harness can apply its own bounded retry policy until PocketIC
    /// provides a native fallible builder API.
    fn try_build(self) -> Result<PocketIc, PocketIcStartupError>;
}

impl PocketIcBuilderExt for PocketIcBuilder {
    fn try_build(self) -> Result<PocketIc, PocketIcStartupError> {
        catch_unwind(AssertUnwindSafe(|| self.build()))
            .map_err(|payload| PocketIcStartupError::from_panic(payload.as_ref()))
    }
}

impl PocketIcStartupError {
    fn from_panic(payload: &(dyn std::any::Any + Send)) -> Self {
        Self {
            message: transport::panic_payload_to_string(payload),
        }
    }

    /// Read the unclassified upstream panic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for PocketIcStartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "PocketIC startup panicked: {}", self.message)
    }
}

impl std::error::Error for PocketIcStartupError {}

#[cfg(test)]
mod tests {
    use super::PocketIcStartupError;

    #[test]
    fn startup_error_preserves_string_panic_message() {
        let error = PocketIcStartupError::from_panic(&"startup failed");

        assert_eq!(error.message(), "startup failed");
        assert_eq!(
            error.to_string(),
            "PocketIC startup panicked: startup failed"
        );
    }
}
