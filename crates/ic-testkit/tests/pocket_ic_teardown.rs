//! Controlled reproduction of PocketIC's synchronous instance-deletion wait.
//! The parent owns a synthetic HTTP peer and can always kill/reap the probe.

use candid::Principal;
use ic_testkit::pic::{
    CandidCallErrorKind, CandidCallExt, CanisterDiagnosticFailure, CanisterDiagnosticsRequest,
    CanisterInstallPhase, InstallSpec, PocketIc, PocketIcBuilder, PocketIcBuilderExt,
    PocketIcDiagnosticsExt, PocketIcStartupConfig, StandaloneCanisterFixture,
    is_dead_pocket_ic_transport_error,
};
use ic_testkit::pocket_ic::common::rest::{CreateInstanceResponse, RawCanisterId, Topology};
use std::{
    io::{BufRead as _, BufReader, Read as _, Write as _},
    net::{TcpListener, TcpStream},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

const PROBE_URL_ENV: &str = "IC_TESTKIT_TEARDOWN_PROBE_URL";
const DEADLOCK_ESCAPE: Duration = Duration::from_secs(10);

#[test]
fn instance_drop_waits_for_the_deletion_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind synthetic PocketIC peer");
    listener.set_nonblocking(true).expect("bound accept wait");
    let mut probe = spawn_probe(&listener, "instance_drop_probe");
    create_instance(&listener);

    let (mut deletion, request) = accept_request(&listener);
    assert_eq!(request, "DELETE /instances/0/ HTTP/1.1");
    // Receipt of DELETE is the barrier: the child is inside upstream teardown.
    // This checks ordering, not an elapsed-time performance threshold.
    assert!(probe.0.try_wait().expect("inspect probe").is_none());
    respond(&mut deletion, "{}");
    drop(deletion);
    assert!(
        probe.wait().success(),
        "probe must finish after DELETE is acknowledged"
    );
}

#[test]
#[ignore = "subprocess probe selected explicitly by the parent teardown test"]
fn instance_drop_probe() {
    let pic = synthetic_instance();
    drop(pic);
}

#[test]
fn refused_instance_request_is_classified_at_the_call_boundary() {
    run_refused_operation_probe("refused_instance_request_probe");
}

#[test]
fn refused_creation_returns_the_standalone_instance() {
    run_refused_operation_probe("refused_creation_probe");
}

fn run_refused_operation_probe(name: &str) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind synthetic PocketIC peer");
    listener.set_nonblocking(true).expect("bound accept wait");
    let mut probe = spawn_probe(&listener, name);
    create_instance(&listener);
    // The probe waits on stdin so its first call starts after the peer is gone.
    drop(listener);
    probe
        .0
        .stdin
        .take()
        .expect("probe release pipe")
        .write_all(b"x")
        .expect("release refused call");
    assert!(probe.wait().success(), "real reqwest refusal must qualify");
}

#[test]
#[ignore = "subprocess probe selected explicitly by the parent transport test"]
fn refused_instance_request_probe() {
    let pic = synthetic_instance();
    std::io::stdin()
        .read_exact(&mut [0])
        .expect("parent releases refused call");
    let error = pic
        .query_candid::<u64, _>(Principal::anonymous(), "get", ())
        .expect_err("peer is gone");
    assert_eq!(error.kind(), CandidCallErrorKind::Transport);
    assert!(is_dead_pocket_ic_transport_error(&error));
    let error = std::io::Error::other(error);
    assert!(is_dead_pocket_ic_transport_error(&error));

    let request = CanisterDiagnosticsRequest::new(
        Principal::anonymous(),
        Principal::anonymous(),
        Principal::anonymous(),
    );
    let (_, status, logs) = pic.collect_canister_diagnostics(request).into_parts();
    for error in [
        status.expect_err("status peer is gone"),
        logs.expect_err("log peer is gone"),
    ] {
        assert!(matches!(
            error,
            CanisterDiagnosticFailure::InstanceUnavailable { .. }
        ));
        assert!(is_dead_pocket_ic_transport_error(&error));
        let error = std::io::Error::other(error);
        assert!(is_dead_pocket_ic_transport_error(&error));
    }
    drop(pic);
}

#[test]
#[ignore = "subprocess probe selected explicitly by the parent install test"]
fn refused_creation_probe() {
    let pic = synthetic_instance();
    let instance_id = pic.instance_id();
    std::io::stdin()
        .read_exact(&mut [0])
        .expect("parent releases refused creation");
    let error = StandaloneCanisterFixture::try_install(
        pic,
        InstallSpec::new(vec![], vec![], 1).label("unreachable-fixture"),
    )
    .err()
    .expect("creation should return an error");
    assert_eq!(
        error.install_error().phase(),
        CanisterInstallPhase::CreateCanister
    );
    assert_eq!(error.install_error().canister_id(), None);
    assert_eq!(error.install_error().label(), Some("unreachable-fixture"));
    assert!(is_dead_pocket_ic_transport_error(&error));
    let (pic, install_error) = error.into_parts();
    assert_eq!(pic.instance_id(), instance_id);
    assert!(install_error.operation_error().is_transport());
    drop(pic);
}

fn synthetic_instance() -> PocketIc {
    let url = std::env::var(PROBE_URL_ENV).expect("parent supplies synthetic peer URL");
    PocketIcBuilder::new()
        .with_application_subnet()
        .with_max_request_time_ms(Some(1))
        .try_build(PocketIcStartupConfig::connect(url, DEADLOCK_ESCAPE))
        .expect("construct synthetic instance through bounded startup")
}

fn spawn_probe(listener: &TcpListener, name: &str) -> Probe {
    let url = format!("http://{}/", listener.local_addr().expect("peer address"));
    Probe(
        Command::new(std::env::current_exe().expect("test executable"))
            .args(["--ignored", "--exact", name, "--nocapture"])
            .env(PROBE_URL_ENV, url)
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1")
            .stdin(Stdio::piped())
            .spawn()
            .expect("spawn isolated PocketIC probe"),
    )
}

fn create_instance(listener: &TcpListener) {
    let (mut creation, request) = accept_request(listener);
    assert_eq!(request, "POST /instances HTTP/1.1");
    let body = serde_json::to_string(&CreateInstanceResponse::Created {
        instance_id: 0,
        topology: Topology {
            subnet_configs: std::collections::BTreeMap::new(),
            default_effective_canister_id: RawCanisterId {
                canister_id: vec![],
            },
        },
        http_gateway_info: None,
    })
    .expect("serialize upstream instance creation response");
    respond(&mut creation, &body);
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

fn respond(stream: &mut TcpStream, body: &str) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("acknowledge synthetic request");
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
