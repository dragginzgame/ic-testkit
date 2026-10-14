use super::*;
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

#[test]
fn a_nonblocking_accepted_stream_can_capture_a_fragmented_body() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let bytes = b"POST / HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc".to_vec();
    let expected = bytes.clone();
    let address = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(&bytes[..bytes.len() - 2]).unwrap();
        thread::sleep(Duration::from_millis(20));
        stream.write_all(&bytes[bytes.len() - 2..]).unwrap();
    });
    let (mut stream, _) = listener.accept().unwrap();
    stream.set_nonblocking(true).unwrap();
    let capture = capture_http_request(
        &mut stream,
        Duration::from_secs(1),
        1024,
        1024,
        HttpContentLength::Required,
    );
    client.join().unwrap();
    assert!(capture.failure.is_none(), "{:?}", capture.failure);
    assert_eq!(capture.bytes, expected);
}

#[test]
fn line_phase_returns_before_headers_and_completion_preserves_original_bytes() {
    use std::{io::Write as _, net::TcpListener, sync::mpsc};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let (proceed, waiting) = mpsc::channel();
    let mut writer_socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut socket = listener.accept().unwrap().0;
    let writer = std::thread::spawn(move || {
        let socket = &mut writer_socket;
        socket.write_all(b"POST /selected HTTP/1.1\r\n").unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        socket.write_all(b"Content-Length: 4\r\n\r\ntest").unwrap();
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    let line = read_http_request_line(&mut socket, deadline, 128).unwrap();
    assert_eq!(line, b"POST /selected HTTP/1.1\r\n");
    proceed.send(()).unwrap();
    let request = read_http_request_with_line(&mut socket, &line, deadline, 128, 4).unwrap();
    assert_eq!(
        request,
        b"POST /selected HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest"
    );
    writer.join().unwrap();
}

#[test]
fn prefetched_line_and_elapsed_deadline_refuse_before_waiting_for_headers() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let _writer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let mut socket = listener.accept().unwrap().0;
    let deadline = Instant::now() + Duration::from_secs(1);
    for line in [
        b"\r\n".as_slice(),
        b"POST / HTTP/1.1\n",
        b"POST / HTTP/1.1\r\nInjected: 1\r\n",
        &[255, 13, 10],
    ] {
        assert_eq!(
            read_http_request_with_line(&mut socket, line, deadline, 128, 4)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
    let elapsed = Instant::now();
    assert_eq!(
        read_http_request_line(&mut socket, elapsed, 128)
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(
        read_http_request_with_line(&mut socket, b"POST / HTTP/1.1\r\n", elapsed, 128, 4)
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
}

#[test]
fn actual_capture_refuses_header_body_overflow_and_truncated_payloads() {
    use std::{io::Write as _, net::TcpListener};
    for (request, headers, body, expected) in [
        (
            b"POST / HTTP/1.1\r\n\r\n".as_slice(),
            4,
            4,
            io::ErrorKind::InvalidData,
        ),
        (
            b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\n12345",
            128,
            4,
            io::ErrorKind::InvalidData,
        ),
        (
            b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\nab",
            128,
            4,
            io::ErrorKind::UnexpectedEof,
        ),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut writer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        writer.write_all(request).unwrap();
        drop(writer);
        let mut socket = listener.accept().unwrap().0;
        assert_eq!(
            capture_http_request(
                &mut socket,
                Duration::from_secs(1),
                headers,
                body,
                HttpContentLength::Optional
            )
            .into_result()
            .unwrap_err()
            .kind(),
            expected
        );
    }
}
