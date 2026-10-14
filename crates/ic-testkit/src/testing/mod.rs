//! Small native helpers for synthetic transport tests.

mod evidence;
pub use evidence::{HttpCapture, HttpCaptureFailure, HttpContentLength, capture_http_request};

#[cfg(test)]
mod tests;
