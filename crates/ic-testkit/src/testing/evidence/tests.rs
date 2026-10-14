use super::*;
use std::io::Cursor;

#[test]
fn framing_preserves_optional_and_required_length_and_rejects_ambiguity() {
    assert!(
        frame_length(b"POST / HTTP/1.1\r\n", 32, 4, HttpContentLength::Optional)
            .unwrap()
            .is_none()
    );
    let request = b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest";
    assert_eq!(
        frame_length(request, 64, 4, HttpContentLength::Required).unwrap(),
        Some(request.len())
    );
    assert_eq!(
        frame_length(
            b"DELETE / HTTP/1.1\r\n\r\n",
            64,
            0,
            HttpContentLength::Optional
        )
        .unwrap(),
        Some(21)
    );
    for bytes in [
        &b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\nContent-Length: 1\r\ncontent-length: 1\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\nContent-Length: -1\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\n\r\n"[..],
    ] {
        assert!(frame_length(bytes, 128, 4, HttpContentLength::Required).is_err());
    }
    assert!(
        frame_length(
            b"POST / HTTP/1.1\r\n\r\n",
            4,
            4,
            HttpContentLength::Optional
        )
        .is_err()
    );
}

#[test]
fn partial_bytes_and_original_errors_survive_interruption_eof_and_timeout() {
    let expected = b"POST / HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc";
    let mut source = Cursor::new(expected);
    let mut interrupted = false;
    let capture = capture_with(
        |bytes, _| {
            if !interrupted {
                interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            source.read(bytes)
        },
        Duration::from_secs(1),
        1024,
        1024,
        HttpContentLength::Required,
    );
    assert!(capture.failure.is_none());
    assert_eq!(capture.bytes, expected);

    let partial = b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\nab";
    let mut source = Cursor::new(partial);
    let capture = capture_with(
        |bytes, _| source.read(bytes),
        Duration::from_secs(1),
        1024,
        1024,
        HttpContentLength::Required,
    );
    assert!(matches!(capture.failure, Some(HttpCaptureFailure::Eof)));
    assert_eq!(capture.bytes, partial);

    let mut source = Cursor::new(partial);
    let capture = capture_with(
        |bytes, _| {
            let count = source.read(bytes)?;
            if count == 0 {
                Err(io::Error::new(io::ErrorKind::TimedOut, "original failure"))
            } else {
                Ok(count)
            }
        },
        Duration::from_secs(1),
        1024,
        1024,
        HttpContentLength::Required,
    );
    assert_eq!(capture.bytes, partial);
    let Some(HttpCaptureFailure::Io(error)) = capture.failure else {
        panic!("original I/O failure must be retained");
    };
    assert_eq!(error.to_string(), "original failure");

    let deadline = capture_with(
        |_, _| Err(io::ErrorKind::Interrupted.into()),
        Duration::from_millis(2),
        1024,
        1024,
        HttpContentLength::Required,
    );
    assert!(matches!(
        deadline.failure,
        Some(HttpCaptureFailure::Deadline)
    ));
    assert_eq!(deadline.bytes, [] as [u8; 0]);

    let invalid = capture_with(
        |buffer, _| Ok(buffer.len() + 1),
        Duration::from_secs(1),
        1024,
        1024,
        HttpContentLength::Required,
    );
    assert!(matches!(
        invalid.failure,
        Some(HttpCaptureFailure::InvalidReadCount)
    ));
    assert_eq!(invalid.bytes, [] as [u8; 0]);
}

#[test]
fn malformed_and_oversized_frames_retain_received_bytes() {
    let oversized = b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\n".to_vec();
    let missing = b"POST / HTTP/1.1\r\n\r\n".to_vec();
    let trailing = b"POST / HTTP/1.1\r\nContent-Length: 3\r\n\r\nabcx".to_vec();
    for (bytes, failure) in [
        (oversized, "BodyBound"),
        (missing, "ContentLength"),
        (trailing, "TrailingBytes"),
    ] {
        let mut source = Cursor::new(bytes.clone());
        let capture = capture_with(
            |buffer, _| source.read(buffer),
            Duration::from_secs(1),
            1024,
            4,
            HttpContentLength::Required,
        );
        assert!(format!("{:?}", capture.failure).contains(failure));
        assert_eq!(capture.bytes, bytes);
    }
}
