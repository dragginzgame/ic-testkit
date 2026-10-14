//! Small native helpers for synthetic transport tests.

mod evidence;
pub use evidence::{HttpCapture, HttpCaptureFailure, HttpContentLength, capture_http_request};

use std::{
    io::{self, Read as _},
    net::TcpStream,
    time::Instant,
};

/// Read only a bounded CRLF request line, without waiting for headers or a body.
///
/// Leaves later bytes in the original socket, so a fixture can classify a
/// concurrent request before deciding whether to complete it. Uses blocking
/// reads and the caller's absolute deadline; no buffering consumes later bytes.
/// Callers retain stream provenance, application classification and response policy.
///
/// # Errors
/// Rejects invalid bounds, truncated/oversized/non-UTF-8 lines and deadline/I/O failures.
pub fn read_http_request_line(
    stream: &mut TcpStream,
    deadline: Instant,
    max_line_bytes: usize,
) -> io::Result<Vec<u8>> {
    if max_line_bytes < 3 {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    stream.set_nonblocking(false)?;
    let mut line = Vec::new();
    while !line.ends_with(b"\r\n") {
        if line.len() == max_line_bytes {
            return Err(io::ErrorKind::InvalidData.into());
        }
        set_remaining_timeout(stream, deadline)?;
        let mut byte = [0];
        match stream.read(&mut byte) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(_) => line.push(byte[0]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
        if byte[0] == b'\n' && !line.ends_with(b"\r\n") {
            return Err(io::ErrorKind::InvalidData.into());
        }
    }
    validate_line(&line)?;
    Ok(line)
}

/// Complete a bounded request after [`read_http_request_line`] on the same socket.
///
/// Preserves the caller's original deadline across both phases, includes the line
/// in the returned bytes/header budget, and uses [`capture_http_request`]'s framing
/// admission. The caller may instead retain an incomplete socket without calling
/// this function, for example while another request determines a cancellation.
/// This never reopens the socket or reads the request line a second time.
///
/// # Errors
/// Returns framing, resource, deadline and I/O failures; malformed prefetched lines refuse.
pub fn read_http_request_with_line(
    stream: &mut TcpStream,
    line: &[u8],
    deadline: Instant,
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> io::Result<Vec<u8>> {
    validate_line(line)?;
    if line.len() > max_header_bytes {
        return Err(io::ErrorKind::InvalidData.into());
    }
    evidence::capture_http_request_until(
        stream,
        line.to_vec(),
        deadline,
        max_header_bytes,
        max_body_bytes,
        HttpContentLength::Optional,
    )
    .into_result()
}

fn validate_line(line: &[u8]) -> io::Result<()> {
    let text = std::str::from_utf8(line).map_err(|_| io::ErrorKind::InvalidData)?;
    let body = text
        .strip_suffix("\r\n")
        .ok_or(io::ErrorKind::InvalidData)?;
    if body.is_empty() || body.contains(['\r', '\n']) {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(())
}

fn set_remaining_timeout(stream: &TcpStream, deadline: Instant) -> io::Result<()> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or(io::ErrorKind::TimedOut)?;
    stream.set_read_timeout(Some(remaining))
}

#[cfg(test)]
mod tests;
