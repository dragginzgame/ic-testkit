use super::request_length;

#[test]
fn request_framing_preserves_body_bounds_and_rejects_ambiguous_lengths() {
    assert_eq!(request_length(b"POST / HTTP/1.1\r\n", 32, 4).unwrap(), None);
    let request = b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest";
    assert_eq!(request_length(request, 64, 4).unwrap(), Some(request.len()));
    assert_eq!(
        request_length(b"DELETE / HTTP/1.1\r\n\r\n", 64, 0).unwrap(),
        Some(21)
    );
    for request in [
        &b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\nContent-Length: 1\r\ncontent-length: 1\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\nContent-Length: -1\r\n\r\n"[..],
        &b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"[..],
    ] {
        assert!(request_length(request, 128, 4).is_err());
    }
    assert!(request_length(b"POST / HTTP/1.1\r\n\r\n", 4, 4).is_err());
}
