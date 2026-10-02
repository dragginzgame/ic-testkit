//! Synthetic HTTP/subprocess qualification of the candidate upstream patch.
use pocket_ic::common::rest::{CreateInstanceResponse, RawCanisterId, Topology};
use pocket_ic::{PocketIc, PocketIcBuilder, PocketIcShutdownError};
use std::{
    io::{BufRead as _, BufReader, Read as _, Write as _},
    net::{TcpListener, TcpStream},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};
const DEADLOCK_ESCAPE: Duration = Duration::from_secs(10);
const DEADLINE: Duration = Duration::from_millis(200);

#[test]
fn acknowledged_shutdown_is_idempotent() {
    for mode in ["sync_success", "async_success"] {
        for status in ["200 OK", "204 No Content"] {
            let (listener, mut probe) = spawn(mode);
            create_instance(&listener, 0);
            reply(&listener, "DELETE /instances/0/ HTTP/1.1", status, "");
            assert!(probe.wait().success());
            assert_eq!(
                listener.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }
    }
}

#[test]
fn stalled_shutdown_times_out_retains_ownership_and_preserves_shared_peer() {
    for mode in ["sync_timeout", "async_timeout"] {
        let (listener, mut probe) = spawn(mode);
        create_instance(&listener, 0);
        let (_stalled, request) = accept_request(&listener);
        assert_eq!(request, "DELETE /instances/0/ HTTP/1.1");
        // Keep the first connection open and unanswered throughout. The same
        // peer must serve a new instance and a retry of the timed-out owner.
        create_instance(&listener, 1);
        reply(&listener, "DELETE /instances/1/ HTTP/1.1", "200 OK", "");
        reply(&listener, "DELETE /instances/0/ HTTP/1.1", "200 OK", "");
        assert!(probe.wait().success());
    }
}

#[test]
fn rejected_or_only_accepted_deletion_remains_unconfirmed_and_retryable() {
    for mode in ["sync_status", "async_status"] {
        for status in ["500 Internal Server Error", "202 Accepted", "404 Not Found"] {
            let (listener, mut probe) = spawn(mode);
            create_instance(&listener, 0);
            reply(&listener, "DELETE /instances/0/ HTTP/1.1", status, "");
            reply(&listener, "DELETE /instances/0/ HTTP/1.1", "200 OK", "");
            assert!(probe.wait().success());
        }
    }
}

#[test]
fn refused_deletion_returns_a_transport_error() {
    let (listener, mut probe) = spawn("refused");
    create_instance(&listener, 0);
    drop(listener);
    probe.0.stdin.take().unwrap().write_all(b"x").unwrap();
    assert!(probe.wait().success());
}

#[test]
fn synchronous_drop_finishes_without_a_deletion_response() {
    let (listener, mut probe) = spawn("drop_timeout");
    create_instance(&listener, 0);
    let (_stalled, request) = accept_request(&listener);
    assert_eq!(request, "DELETE /instances/0/ HTTP/1.1");
    // probe.wait has a kill/reap-backed 10s escape; candidate Drop has a 5s
    // deadline. The peer stays open and never supplies a response.
    assert!(probe.wait().success());
}

#[test]
fn borrowed_handle_never_deletes_the_owned_instance() {
    let (listener, mut probe) = spawn("borrowed");
    create_instance(&listener, 0);
    reply(&listener, "DELETE /instances/0/ HTTP/1.1", "200 OK", "");
    assert!(probe.wait().success());
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn borrowed_gateway_body_timeout_preserves_retry_without_instance_deletion() {
    let (listener, mut probe) = spawn("gateway");
    create_instance(&listener, 0);
    reply(
        &listener,
        "GET /instances/0/auto_progress HTTP/1.1",
        "200 OK",
        "true",
    );
    reply(
        &listener,
        "POST /http_gateway HTTP/1.1",
        "200 OK",
        r#"{"Created":{"instance_id":7,"port":12345}}"#,
    );
    let (mut stalled, request) = accept_request(&listener);
    assert_eq!(request, "POST /http_gateway/7/stop HTTP/1.1");
    // Headers arrive, but the JSON body never arrives. The whole shutdown,
    // including acknowledgement decoding, must be inside the deadline.
    write!(
        stalled,
        "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nContent-Type: application/json\r\n\r\n"
    )
    .unwrap();
    reply(
        &listener,
        "POST /http_gateway/7/stop HTTP/1.1",
        "200 OK",
        "null",
    );
    reply(&listener, "DELETE /instances/0/ HTTP/1.1", "200 OK", "");
    assert!(probe.wait().success());
}

#[test]
#[ignore = "isolated helper invoked by the parent tests"]
fn shutdown_probe() {
    let mode = std::env::var("PROBE_MODE").unwrap();
    let url = std::env::var("PROBE_URL").unwrap();
    let builder = || {
        PocketIcBuilder::new()
            .with_application_subnet()
            .with_server_url(url.parse().unwrap())
            .with_max_request_time_ms(Some(1))
    };
    if mode.starts_with("async_") {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let mut pic = builder().build_async().await;
            assert!(matches!(pic.shutdown(Duration::ZERO).await, Err(PocketIcShutdownError::InvalidTimeout)));
            if mode == "async_timeout" {
                assert!(matches!(pic.shutdown(DEADLINE).await, Err(PocketIcShutdownError::DeadlineExceeded(t)) if t == DEADLINE));
                let mut other = builder().build_async().await;
                other.shutdown(DEADLOCK_ESCAPE).await.unwrap();
                other.drop().await;
            } else if mode == "async_status" {
                assert!(matches!(pic.shutdown(DEADLOCK_ESCAPE).await, Err(PocketIcShutdownError::UnexpectedStatus(_))));
            }
            pic.shutdown(DEADLOCK_ESCAPE).await.unwrap();
            pic.shutdown(DEADLINE).await.unwrap();
            pic.drop().await;
        });
    } else {
        let mut pic = builder().build();
        assert!(matches!(
            pic.shutdown(Duration::ZERO),
            Err(PocketIcShutdownError::InvalidTimeout)
        ));
        if mode == "sync_timeout" {
            assert!(
                matches!(pic.shutdown(DEADLINE), Err(PocketIcShutdownError::DeadlineExceeded(t)) if t == DEADLINE)
            );
            let mut other = builder().build();
            other.shutdown(DEADLOCK_ESCAPE).unwrap();
            drop(other);
        } else if mode == "sync_status" {
            assert!(matches!(
                pic.shutdown(DEADLOCK_ESCAPE),
                Err(PocketIcShutdownError::UnexpectedStatus(_))
            ));
        } else if mode == "borrowed" || mode == "gateway" {
            let mut borrowed =
                PocketIc::new_from_existing_instance(url.parse().unwrap(), 0, Some(1));
            if mode == "gateway" {
                borrowed.make_live(None);
                assert!(
                    matches!(borrowed.shutdown(DEADLINE), Err(PocketIcShutdownError::DeadlineExceeded(t)) if t == DEADLINE)
                );
            }
            borrowed.shutdown(DEADLOCK_ESCAPE).unwrap();
            drop(borrowed);
        } else if mode == "refused" {
            std::io::stdin().read_exact(&mut [0]).unwrap();
            assert!(matches!(
                pic.shutdown(DEADLINE),
                Err(PocketIcShutdownError::Request(_))
            ));
            drop(pic);
            return;
        } else if mode == "drop_timeout" {
            drop(pic);
            return;
        }
        pic.shutdown(DEADLOCK_ESCAPE).unwrap();
        pic.shutdown(DEADLINE).unwrap();
        drop(pic);
    }
}

fn spawn(mode: &str) -> (TcpListener, Probe) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let probe = Probe(
        Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "shutdown_probe", "--nocapture"])
            .env("PROBE_MODE", mode)
            .env(
                "PROBE_URL",
                format!("http://{}/", listener.local_addr().unwrap()),
            )
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1")
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    (listener, probe)
}

fn create_instance(listener: &TcpListener, instance_id: usize) {
    let body = serde_json::to_string(&CreateInstanceResponse::Created {
        instance_id,
        topology: Topology {
            subnet_configs: std::collections::BTreeMap::new(),
            default_effective_canister_id: RawCanisterId {
                canister_id: vec![],
            },
        },
        http_gateway_info: None,
    })
    .unwrap();
    reply(listener, "POST /instances HTTP/1.1", "200 OK", &body);
}

fn reply(listener: &TcpListener, expected: &str, status: &str, body: &str) {
    let (mut stream, request) = accept_request(listener);
    assert_eq!(request, expected);
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
}

fn accept_request(listener: &TcpListener) -> (TcpStream, String) {
    let deadline = Instant::now() + DEADLOCK_ESCAPE;
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "peer request did not arrive");
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("accept synthetic request: {error}"),
        }
    };
    stream
        .set_read_timeout(Some(DEADLOCK_ESCAPE))
        .expect("bound request read");
    stream
        .set_write_timeout(Some(DEADLOCK_ESCAPE))
        .expect("bound response write");
    let mut reader = BufReader::new(stream);
    let mut request = String::new();
    reader
        .read_line(&mut request)
        .expect("read HTTP request line");
    let mut body_len = 0;
    let mut header_bytes = request.len();
    loop {
        let mut header = String::new();
        assert!(reader.read_line(&mut header).expect("read HTTP header") > 0);
        header_bytes += header.len();
        assert!(header_bytes <= 65_536, "bound synthetic request headers");
        if header == "\r\n" {
            break;
        }
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            body_len = value.trim().parse().expect("request content length");
        }
    }
    assert!(body_len <= 65_536, "bound synthetic request body");
    reader
        .read_exact(&mut vec![0; body_len])
        .expect("drain request body");
    (reader.into_inner(), request.trim_end().to_owned())
}

struct Probe(Child);

impl Probe {
    fn wait(&mut self) -> ExitStatus {
        let deadline = Instant::now() + DEADLOCK_ESCAPE;
        loop {
            if let Some(status) = self.0.try_wait().expect("inspect teardown probe") {
                return status;
            }
            assert!(Instant::now() < deadline, "teardown probe did not finish");
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
