//! Small native helpers for synthetic transport tests.

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
    let invalid = || io::Error::from(io::ErrorKind::InvalidData);
    let capacity = max_header_bytes
        .checked_add(max_body_bytes)
        .filter(|_| max_header_bytes >= 4)
        .ok_or_else(invalid)?;
    let deadline = Instant::now().checked_add(timeout).ok_or_else(invalid)?;
    stream.set_nonblocking(false)?;
    let mut bytes = Vec::new();
    let mut chunk = [0; 4096];
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or(io::ErrorKind::TimedOut)?;
        stream.set_read_timeout(Some(remaining))?;
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
