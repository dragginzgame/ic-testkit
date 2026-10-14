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
