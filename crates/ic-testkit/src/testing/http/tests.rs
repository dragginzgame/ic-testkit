use super::*;
use std::io::Read as _;

fn limits() -> HttpFixtureLimits {
    HttpFixtureLimits {
        timeout: Duration::from_secs(10),
        max_header_bytes: 1024,
        max_body_bytes: 4,
    }
}

fn client(fixture: &HttpFixture) -> TcpStream {
    let stream = TcpStream::connect(fixture.endpoint().strip_prefix("http://").unwrap()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

#[test]
fn responds_with_exact_bytes_and_returns_the_unmodified_request() {
    let fixture = HttpFixture::spawn(vec![0, 255, 13, 10], "application/cbor", limits()).unwrap();
    let mut stream = client(&fixture);
    let request = b"POST /query HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest";
    stream.write_all(request).unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    assert_eq!(response, b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: 4\r\nConnection: close\r\n\r\n\x00\xff\r\n");
    assert_eq!(fixture.finish().unwrap(), request);
}

#[test]
fn refuses_invalid_setup_and_request_budgets() {
    for content_type in ["", "text/plain\r\nInjected: yes", "text/\0plain"] {
        assert_eq!(
            HttpFixture::spawn(vec![], content_type, limits())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
    for selected in [
        HttpFixtureLimits {
            timeout: Duration::ZERO,
            ..limits()
        },
        HttpFixtureLimits {
            max_header_bytes: 0,
            ..limits()
        },
    ] {
        assert_eq!(
            HttpFixture::spawn(vec![], "text/plain", selected)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
    for selected in [
        HttpFixtureLimits {
            max_header_bytes: 8,
            ..limits()
        },
        HttpFixtureLimits {
            max_body_bytes: 3,
            ..limits()
        },
    ] {
        let fixture = HttpFixture::spawn(vec![], "text/plain", selected).unwrap();
        let mut stream = client(&fixture);
        stream
            .write_all(b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest")
            .unwrap();
        assert_eq!(
            fixture.finish().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn deadline_covers_accept_and_an_incomplete_request() {
    for connect in [false, true] {
        let fixture = HttpFixture::spawn(
            vec![],
            "text/plain",
            HttpFixtureLimits {
                timeout: Duration::from_millis(100),
                ..limits()
            },
        )
        .unwrap();
        let mut stream = connect.then(|| client(&fixture));
        if let Some(stream) = stream.as_mut() {
            stream.write_all(b"GET / HTTP/1.1\r\n").unwrap();
        }
        let error = fixture.finish().unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ));
    }
}

#[test]
fn dropping_cancels_an_unstarted_or_incomplete_exchange() {
    for connect in [false, true] {
        let fixture = HttpFixture::spawn(
            vec![],
            "text/plain",
            HttpFixtureLimits {
                timeout: Duration::from_secs(30),
                ..limits()
            },
        )
        .unwrap();
        let mut stream = connect.then(|| client(&fixture));
        if let Some(stream) = stream.as_mut() {
            stream.write_all(b"GET / HTTP/1.1\r\n").unwrap();
        }
        let started = Instant::now();
        drop(fixture);
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}

#[test]
fn dropping_interrupts_a_response_to_a_stalled_reader() {
    let fixture = HttpFixture::spawn(
        vec![1; 8 * 1024 * 1024],
        "application/octet-stream",
        HttpFixtureLimits {
            timeout: Duration::from_secs(30),
            ..limits()
        },
    )
    .unwrap();
    let mut stream = client(&fixture);
    stream.write_all(b"GET / HTTP/1.1\r\n\r\n").unwrap();
    stream.read_exact(&mut [0]).unwrap();
    let started = Instant::now();
    drop(fixture);
    assert!(started.elapsed() < Duration::from_secs(5));
}
