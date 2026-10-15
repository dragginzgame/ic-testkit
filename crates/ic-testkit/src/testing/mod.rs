//! Small native helpers for synthetic transport tests.

pub mod http;

use std::{
    io::{self, Read as _},
    net::TcpStream,
    time::{Duration, Instant},
};

/// Read one bounded raw HTTP request from an accepted test socket.
///
/// Selects blocking reads explicitly, including sockets inheriting nonblocking
/// mode on macOS. Every read uses the remaining single deadline. Missing
/// Content-Length means no body; repeated/invalid lengths and transfer encoding
/// are rejected. This helper does not authenticate or interpret application data.
/// The caller owns responses, write timeouts and failure injection.
pub fn read_http_request(
    stream: &mut TcpStream,
    timeout: Duration,
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> io::Result<Vec<u8>> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(io::ErrorKind::InvalidInput)?;
    capture_request(
        stream,
        Vec::new(),
        deadline,
        max_header_bytes,
        max_body_bytes,
    )
}

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
/// in the returned bytes/header budget, and uses [`read_http_request`]'s framing
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
    capture_request(
        stream,
        line.to_vec(),
        deadline,
        max_header_bytes,
        max_body_bytes,
    )
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

fn capture_request(
    stream: &mut TcpStream,
    mut bytes: Vec<u8>,
    deadline: Instant,
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> io::Result<Vec<u8>> {
    let invalid = || io::Error::from(io::ErrorKind::InvalidData);
    let capacity = max_header_bytes
        .checked_add(max_body_bytes)
        .filter(|_| max_header_bytes >= 4)
        .ok_or_else(invalid)?;
    stream.set_nonblocking(false)?;
    let mut chunk = [0; 4096];
    loop {
        set_remaining_timeout(stream, deadline)?;
        let window = (capacity - bytes.len()).min(chunk.len());
        if window == 0 {
            return Err(invalid());
        }
        match stream.read(&mut chunk[..window]) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
        if let Some(length) = request_length(&bytes, max_header_bytes, max_body_bytes)? {
            if bytes.len() == length {
                return Ok(bytes);
            }
            if bytes.len() > length {
                return Err(invalid());
            }
        }
    }
}

fn request_length(
    bytes: &[u8],
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> io::Result<Option<usize>> {
    let invalid = || io::Error::from(io::ErrorKind::InvalidData);
    let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
        return if bytes.len() > max_header_bytes {
            Err(invalid())
        } else {
            Ok(None)
        };
    };
    if end + 4 > max_header_bytes {
        return Err(invalid());
    }
    let header = std::str::from_utf8(&bytes[..end]).map_err(|_| invalid())?;
    let mut length = None;
    for line in header.lines().skip(1) {
        let (name, value) = line.split_once(':').ok_or_else(invalid)?;
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(invalid());
        }
        if name.eq_ignore_ascii_case("content-length") {
            if length.is_some()
                || value.trim().is_empty()
                || !value.trim().bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(invalid());
            }
            length = Some(value.trim().parse::<usize>().map_err(|_| invalid())?);
        }
    }
    let length = length.unwrap_or(0);
    if length > max_body_bytes {
        return Err(invalid());
    }
    end.checked_add(4)
        .and_then(|end| end.checked_add(length))
        .map(Some)
        .ok_or_else(invalid)
}

#[cfg(test)]
mod tests;
