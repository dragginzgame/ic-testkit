//! Original-byte evidence for bounded synthetic HTTP peers.

use std::{
    io::{self, Read},
    net::TcpStream,
    time::{Duration, Instant},
};

/// Whether an absent Content-Length declares an empty body or is rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HttpContentLength {
    /// An absent field declares an empty body.
    Optional,
    /// The request must declare a length, including zero.
    Required,
}

/// Exact reason why one request could not be captured.
#[derive(Debug)]
pub enum HttpCaptureFailure {
    /// Original socket operation failed; its OS error is retained.
    Io(io::Error),
    /// The single deadline expired before another read.
    Deadline,
    /// The peer closed before the declared request completed.
    Eof,
    /// The header exceeded its byte bound.
    HeaderBound,
    /// The length was absent when required, malformed or ambiguous.
    ContentLength,
    /// The body or total capture exceeded its byte bound.
    BodyBound,
    /// Captured bytes follow the declared request.
    TrailingBytes,
    /// A reader reported more bytes than its supplied buffer.
    InvalidReadCount,
    /// The byte bounds cannot be represented.
    InvalidBounds,
    /// The timeout exceeds the host clock's representable range.
    InvalidDeadline,
}

/// Exact bytes received, including partial bytes when capture fails.
#[derive(Debug)]
pub struct HttpCapture {
    /// Bytes received from the original socket.
    pub bytes: Vec<u8>,
    /// Absent only when exactly one framed request was received.
    pub failure: Option<HttpCaptureFailure>,
}

impl HttpCapture {
    /// Return a complete request as bytes, discarding partial-byte evidence on failure.
    ///
    /// # Errors
    /// Maps a typed framing failure to an I/O kind while retaining original socket errors.
    pub fn into_result(self) -> io::Result<Vec<u8>> {
        match self.failure {
            None => Ok(self.bytes),
            Some(HttpCaptureFailure::Io(error)) => Err(error),
            Some(failure) => Err(io::Error::from(match failure {
                HttpCaptureFailure::Deadline => io::ErrorKind::TimedOut,
                HttpCaptureFailure::Eof => io::ErrorKind::UnexpectedEof,
                HttpCaptureFailure::InvalidDeadline => io::ErrorKind::InvalidInput,
                _ => io::ErrorKind::InvalidData,
            })),
        }
    }
}

/// Capture one raw request on an accepted socket with one total deadline.
///
/// Socket mode and read timeouts are normalized on macOS as well as Linux.
/// The caller retains response behavior, evidence files and application policy.
pub fn capture_http_request(
    stream: &mut TcpStream,
    timeout: Duration,
    max_header_bytes: usize,
    max_body_bytes: usize,
    content_length: HttpContentLength,
) -> HttpCapture {
    if max_header_bytes < 4 || max_header_bytes.checked_add(max_body_bytes).is_none() {
        return failed(Vec::new(), HttpCaptureFailure::InvalidBounds);
    }
    let Some(deadline) = Instant::now().checked_add(timeout) else {
        return failed(Vec::new(), HttpCaptureFailure::InvalidDeadline);
    };
    capture_http_request_until(
        stream,
        Vec::new(),
        deadline,
        max_header_bytes,
        max_body_bytes,
        content_length,
    )
}

pub(super) fn capture_http_request_until(
    stream: &mut TcpStream,
    initial: Vec<u8>,
    deadline: Instant,
    max_header_bytes: usize,
    max_body_bytes: usize,
    content_length: HttpContentLength,
) -> HttpCapture {
    if max_header_bytes < 4
        || max_header_bytes.checked_add(max_body_bytes).is_none()
        || initial.len() > max_header_bytes
    {
        return failed(initial, HttpCaptureFailure::InvalidBounds);
    }
    if let Err(error) = stream.set_nonblocking(false) {
        return failed(initial, HttpCaptureFailure::Io(error));
    }
    capture_with_deadline(
        |buffer, remaining| {
            stream.set_read_timeout(Some(remaining))?;
            stream.read(buffer)
        },
        initial,
        deadline,
        max_header_bytes,
        max_body_bytes,
        content_length,
    )
}

const fn failed(bytes: Vec<u8>, failure: HttpCaptureFailure) -> HttpCapture {
    HttpCapture {
        bytes,
        failure: Some(failure),
    }
}

#[cfg(test)]
fn capture_with(
    read: impl FnMut(&mut [u8], Duration) -> io::Result<usize>,
    budget: Duration,
    max_header_bytes: usize,
    max_body_bytes: usize,
    content_length: HttpContentLength,
) -> HttpCapture {
    let Some(deadline) = Instant::now().checked_add(budget) else {
        return failed(Vec::new(), HttpCaptureFailure::InvalidDeadline);
    };
    capture_with_deadline(
        read,
        Vec::new(),
        deadline,
        max_header_bytes,
        max_body_bytes,
        content_length,
    )
}

fn capture_with_deadline(
    mut read: impl FnMut(&mut [u8], Duration) -> io::Result<usize>,
    mut bytes: Vec<u8>,
    deadline: Instant,
    max_header_bytes: usize,
    max_body_bytes: usize,
    content_length: HttpContentLength,
) -> HttpCapture {
    let Some(capacity) = max_header_bytes.checked_add(max_body_bytes) else {
        return failed(bytes, HttpCaptureFailure::InvalidBounds);
    };
    if max_header_bytes < 4 || bytes.len() > max_header_bytes {
        return failed(bytes, HttpCaptureFailure::InvalidBounds);
    }
    let mut chunk = [0; 4096];
    let failure = loop {
        let Some(remaining) = deadline
            .checked_duration_since(Instant::now())
            .filter(|value| !value.is_zero())
        else {
            break HttpCaptureFailure::Deadline;
        };
        let window = (capacity - bytes.len()).min(chunk.len());
        if window == 0 {
            break HttpCaptureFailure::BodyBound;
        }
        match read(&mut chunk[..window], remaining) {
            Ok(0) => break HttpCaptureFailure::Eof,
            Ok(count) if count > window => break HttpCaptureFailure::InvalidReadCount,
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => break HttpCaptureFailure::Io(error),
        }
        match frame_length(&bytes, max_header_bytes, max_body_bytes, content_length) {
            Ok(Some(length)) if bytes.len() == length => {
                return HttpCapture {
                    bytes,
                    failure: None,
                };
            }
            Ok(Some(length)) if bytes.len() > length => break HttpCaptureFailure::TrailingBytes,
            Ok(_) => {}
            Err(failure) => break failure,
        }
    };
    failed(bytes, failure)
}

fn frame_length(
    bytes: &[u8],
    max_header_bytes: usize,
    max_body_bytes: usize,
    content_length: HttpContentLength,
) -> Result<Option<usize>, HttpCaptureFailure> {
    let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
        return if bytes.len() > max_header_bytes {
            Err(HttpCaptureFailure::HeaderBound)
        } else {
            Ok(None)
        };
    };
    if end + 4 > max_header_bytes {
        return Err(HttpCaptureFailure::HeaderBound);
    }
    let header =
        std::str::from_utf8(&bytes[..end]).map_err(|_| HttpCaptureFailure::ContentLength)?;
    let mut length = None;
    for line in header.lines().skip(1) {
        let (name, value) = line
            .split_once(':')
            .ok_or(HttpCaptureFailure::ContentLength)?;
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(HttpCaptureFailure::ContentLength);
        }
        if name.eq_ignore_ascii_case("content-length") {
            if length.is_some()
                || value.trim().is_empty()
                || !value.trim().bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(HttpCaptureFailure::ContentLength);
            }
            length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| HttpCaptureFailure::ContentLength)?,
            );
        }
    }
    let length = match (length, content_length) {
        (Some(length), _) => length,
        (None, HttpContentLength::Optional) => 0,
        (None, HttpContentLength::Required) => return Err(HttpCaptureFailure::ContentLength),
    };
    if length > max_body_bytes {
        return Err(HttpCaptureFailure::BodyBound);
    }
    end.checked_add(4)
        .and_then(|end| end.checked_add(length))
        .map(Some)
        .ok_or(HttpCaptureFailure::InvalidBounds)
}

#[cfg(test)]
mod tests;
