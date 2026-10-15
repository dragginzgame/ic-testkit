use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use pocket_ic::PocketIcBuilder;

use super::super::{PocketIcManagedServerOutput, PocketIcStartupConfig, PocketIcStartupFailure};

#[cfg(unix)]
use super::{TestServerScript, process_state};
#[cfg(unix)]
use ic_host_process::testing::write_executable_script;
#[cfg(unix)]
use std::io::Write as _;

fn test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn async_configuration_failures_are_typed_without_server_effects() {
    let runtime = test_runtime();
    for (url, timeout, invalid_configuration) in [
        ("not a URL", Duration::from_secs(1), false),
        ("http://127.0.0.1:1/", Duration::ZERO, true),
    ] {
        let error = runtime
            .block_on(
                PocketIcStartupConfig::connect(url, timeout)
                    .build_instance_async(PocketIcBuilder::new()),
            )
            .err()
            .unwrap();
        assert!(matches!(
            (invalid_configuration, error.failure()),
            (true, PocketIcStartupFailure::InvalidConfiguration { .. })
                | (false, PocketIcStartupFailure::InvalidServerUrl { .. })
        ));
        assert_eq!(error.output(), &PocketIcManagedServerOutput::default());
        assert!(error.server_cleanup().is_none());
    }
}

#[cfg(unix)]
#[test]
fn async_readiness_timeout_and_cancellation_reap_the_exact_server() {
    use std::future::Future as _;
    let runtime = test_runtime();
    for cancel in [false, true] {
        let script = TestServerScript::new(
            "async-readiness",
            "#!/bin/sh\nprintf '%s' \"$$\" > \"$0.pid\"\nprintf 'readiness diagnostic' >&2\nexec sleep 30\n",
        );
        let stdout = script.path().with_extension("stdout");
        let stderr = script.path().with_extension("stderr");
        let config = PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
            .with_server_output_files(&stdout, &stderr);
        runtime.block_on(async {
            let mut construction = Box::pin(config.build_instance_async(PocketIcBuilder::new()));
            if cancel {
                // Poll once to spawn, then cancel while readiness is pending.
                std::future::poll_fn(|cx| {
                    assert!(construction.as_mut().poll(cx).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
                let deadline = Instant::now() + Duration::from_secs(2);
                while fs::read(&stderr).unwrap().is_empty() {
                    assert!(
                        Instant::now() < deadline,
                        "server did not publish diagnostics"
                    );
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                drop(construction);
            } else {
                let error = construction.await.err().unwrap();
                assert!(matches!(
                    error.failure(),
                    PocketIcStartupFailure::ReadinessTimeout { .. }
                ));
                assert_eq!(error.output().stderr(), "readiness diagnostic");
                assert!(error.server_cleanup().is_none());
            }
        });
        let pid_file = script.path().with_extension("pid");
        let pid = fs::read_to_string(&pid_file).unwrap().parse().unwrap();
        assert_eq!(process_state(pid), None);
        assert_eq!(fs::read_to_string(&stderr).unwrap(), "readiness diagnostic");
        for file in [stdout, stderr, pid_file] {
            fs::remove_file(file).unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn async_instance_wait_timeout_exit_and_cancellation_keep_ownership() {
    use std::future::Future as _;
    let runtime = test_runtime();
    for mode in ["timeout", "exit", "panic", "cancel"] {
        let script = TestServerScript::new("async-instance", "#!/bin/sh\n");
        let exit_file = script.path().with_extension("exit");
        let http =
            PendingHttpServer::new((mode == "exit").then(|| exit_file.clone()), mode == "panic");
        let body = if mode == "exit" {
            "while [ ! -f \"$0.exit\" ]; do sleep 0.02; done\nexit 23"
        } else {
            "exec sleep 30"
        };
        write_executable_script(
            &script.path(),
            format!(
                "#!/bin/sh\nprintf '%s' \"$$\" > \"$0.pid\"\nprintf 'instance diagnostic' >&2\nprintf '{}\\n' > \"$2\"\n{body}\n",
                http.port
            ),
        ).expect("write executable test script");
        let stderr = script.path().with_extension("stderr");
        let stdout = script.path().with_extension("stdout");
        runtime.block_on(async {
            let mut construction = Box::pin(
                PocketIcStartupConfig::spawn(script.path(), Duration::from_secs(2))
                    .with_server_output_files(&stdout, &stderr)
                    .build_instance_async(PocketIcBuilder::new().with_application_subnet()));
            if mode == "cancel" {
                let deadline = Instant::now() + Duration::from_secs(2);
                while !http.received.load(std::sync::atomic::Ordering::Acquire) {
                    std::future::poll_fn(|cx| {
                        assert!(construction.as_mut().poll(cx).is_pending());
                        std::task::Poll::Ready(())
                    }).await;
                    assert!(Instant::now() < deadline, "instance request never arrived");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                drop(construction);
            } else {
                let error = construction.await.err().unwrap();
                if mode == "timeout" {
                    assert!(matches!(error.failure(), PocketIcStartupFailure::InstanceCreationTimeout { .. }));
                } else if mode == "exit" {
                    assert!(matches!(error.failure(), PocketIcStartupFailure::ServerExited { status, .. } if status.code() == Some(23)));
                } else {
                    assert!(matches!(error.failure(), PocketIcStartupFailure::BuilderPanicked { .. }));
                }
                assert_eq!(error.output().stderr(), "instance diagnostic");
                assert!(error.server_cleanup().is_none());
            }
        });
        assert!(http.received.load(std::sync::atomic::Ordering::Acquire));
        let pid_file = script.path().with_extension("pid");
        let pid = fs::read_to_string(&pid_file).unwrap().parse().unwrap();
        assert_eq!(process_state(pid), None);
        assert_eq!(fs::read_to_string(&stderr).unwrap(), "instance diagnostic");
        for file in [stdout, stderr, pid_file] {
            fs::remove_file(file).unwrap();
        }
        if exit_file.exists() {
            fs::remove_file(exit_file).unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn async_borrowed_failure_leaves_server_authority_with_the_caller() {
    let http = PendingHttpServer::new(None, true);
    let runtime = test_runtime();
    let error = runtime
        .block_on(
            PocketIcStartupConfig::connect(
                format!("http://127.0.0.1:{}/", http.port),
                Duration::from_secs(2),
            )
            .build_instance_async(PocketIcBuilder::new().with_application_subnet()),
        )
        .err()
        .unwrap();
    assert!(matches!(
        error.failure(),
        PocketIcStartupFailure::BuilderPanicked { .. }
    ));
    assert_eq!(error.output(), &PocketIcManagedServerOutput::default());
    assert!(error.server_cleanup().is_none());
    assert!(std::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, http.port)).is_ok());
}

#[cfg(unix)]
struct PendingHttpServer {
    port: u16,
    received: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

#[cfg(unix)]
impl PendingHttpServer {
    fn new(exit_file: Option<PathBuf>, respond: bool) -> Self {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let received = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let request_received = Arc::clone(&received);
        let stop_worker = Arc::clone(&stop);
        let worker = std::thread::spawn(move || {
            while !stop_worker.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        if crate::testing::capture_http_request(
                            &mut stream,
                            Duration::from_secs(1),
                            65_536,
                            65_536,
                            crate::testing::HttpContentLength::Optional,
                        )
                        .failure
                        .is_some()
                        {
                            continue;
                        }
                        request_received.store(true, Ordering::Release);
                        if let Some(path) = &exit_file {
                            fs::write(path, b"exit").unwrap();
                        }
                        if respond {
                            stream.write_all(b"HTTP/1.1 500 Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                        } else {
                            while !stop_worker.load(Ordering::Acquire) {
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("HTTP fixture accept failed: {error}"),
                }
            }
        });
        Self {
            port,
            received,
            stop,
            worker: Some(worker),
        }
    }
}

#[cfg(unix)]
impl Drop for PendingHttpServer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        self.worker.take().unwrap().join().unwrap();
    }
}

#[cfg(unix)]
#[test]
#[ignore = "requires POCKET_IC_BIN=<caller-provided PocketIC server binary>"]
fn owned_sync_instance_preserves_topology_and_scoped_server_lifetime() {
    let binary = std::env::var_os("POCKET_IC_BIN").unwrap();
    let config = PocketIcStartupConfig::spawn(PathBuf::from(binary), Duration::from_secs(30));
    let owned = config
        .build_instance(
            PocketIcBuilder::new()
                .with_application_subnet()
                .with_nns_subnet(),
        )
        .unwrap();
    let server = owned.server.as_ref().unwrap();
    let pid = server.process_id();
    let directory = server.server.files._directory.path().to_path_buf();
    assert!(owned.instance.topology().get_nns().is_some());
    assert!(process_state(pid).is_some());
    let canister = owned.instance.create_canister();
    assert!(owned.instance.canister_exists(canister));
    let borrowed = PocketIcStartupConfig::connect(server.url(), Duration::from_secs(30))
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    assert!(borrowed.server.is_none());
    drop(borrowed);
    assert!(owned.instance.canister_exists(canister));
    drop(owned);
    assert_eq!(process_state(pid), None);
    assert!(!directory.exists());
}

#[cfg(unix)]
#[test]
#[ignore = "requires POCKET_IC_BIN=<caller-provided PocketIC server binary>"]
fn owned_async_instance_and_borrowed_connections_preserve_lifetimes() {
    let binary = PathBuf::from(std::env::var_os("POCKET_IC_BIN").unwrap());
    let runtime = test_runtime();
    runtime.block_on(async {
        let owned = PocketIcStartupConfig::spawn(&binary, Duration::from_secs(30))
            .build_instance_async(
                PocketIcBuilder::new()
                    .with_application_subnet()
                    .with_nns_subnet(),
            )
            .await
            .unwrap();
        let server = owned.server.as_ref().unwrap();
        let pid = server.process_id();
        let directory = server.server.files._directory.path().to_path_buf();
        assert!(owned.instance.topology().await.get_nns().is_some());
        let canister = owned.instance.create_canister().await;
        assert!(owned.instance.canister_exists(canister).await);
        let borrowed = PocketIcStartupConfig::connect(server.url(), Duration::from_secs(30))
            .build_instance_async(PocketIcBuilder::new().with_application_subnet())
            .await
            .unwrap();
        assert!(borrowed.server.is_none());
        borrowed.instance.drop().await;
        assert!(owned.instance.canister_exists(canister).await);
        owned.instance.drop().await;
        assert!(process_state(pid).is_some());
        drop(owned.server);
        assert_eq!(process_state(pid), None);
        assert!(!directory.exists());
    });
}

#[cfg(unix)]
#[test]
#[ignore = "requires POCKET_IC_BIN=<caller-provided PocketIC server binary>"]
fn install_errors_fixtures_and_baselines_retain_the_owned_server() {
    use crate::pic::{
        CachedPocketIcBaseline, CanisterInstallPhase, InstallSpec, StandaloneCanisterFixture,
    };
    use candid::Principal;

    let binary = PathBuf::from(std::env::var_os("POCKET_IC_BIN").unwrap());
    let owned = PocketIcStartupConfig::spawn(&binary, Duration::from_secs(30))
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pid = owned.server.as_ref().unwrap().process_id();
    let instance_id = owned.instance.instance_id();
    let error = StandaloneCanisterFixture::try_install(
        owned,
        InstallSpec::new(vec![0xde, 0xad], vec![], 5_000_000_000_000).label("invalid-owned-wasm"),
    )
    .err()
    .expect("invalid Wasm must fail installation");
    assert_eq!(
        error.install_error().phase(),
        CanisterInstallPhase::InstallCode
    );
    assert_eq!(error.install_error().label(), Some("invalid-owned-wasm"));
    assert!(
        error
            .install_error()
            .message()
            .contains("CanisterInvalidWasm")
    );
    error
        .pocket_ic()
        .canister_status(error.install_error().canister_id().unwrap(), None)
        .expect("retained server must permit inspecting the failed canister");
    let (owned, _) = error.into_parts();
    assert_eq!(owned.instance.instance_id(), instance_id);
    assert_eq!(owned.server.as_ref().unwrap().process_id(), pid);
    assert!(process_state(pid).is_some());

    let fixture = StandaloneCanisterFixture::install(
        owned,
        InstallSpec::new(b"\0asm\x01\0\0\0".to_vec(), vec![], 5_000_000_000_000),
    );
    let (owned, canister) = fixture.into_parts();
    assert_eq!(owned.server.as_ref().unwrap().process_id(), pid);
    let baseline = CachedPocketIcBaseline::capture(
        owned,
        [crate::pic::CanisterSnapshotTarget::new(canister, None)],
        (),
    )
    .unwrap();
    baseline
        .pocket_ic()
        .uninstall_canister(canister, None)
        .unwrap();
    baseline
        .restore(crate::pic::SnapshotRestoreFunding::Preserve)
        .unwrap();
    assert!(
        baseline
            .pocket_ic()
            .canister_status(canister, None)
            .unwrap()
            .module_hash
            .is_some()
    );
    assert!(process_state(pid).is_some());
    drop(baseline);
    assert_eq!(
        process_state(pid),
        None,
        "baseline drop must reap its server"
    );

    let owned = PocketIcStartupConfig::spawn(binary, Duration::from_secs(30))
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pid = owned.server.as_ref().unwrap().process_id();
    let capture = CachedPocketIcBaseline::capture(
        owned,
        [crate::pic::CanisterSnapshotTarget::new(
            Principal::anonymous(),
            None,
        )],
        (),
    );
    assert!(
        capture.is_err(),
        "missing canister must refuse snapshot capture"
    );
    assert_eq!(
        process_state(pid),
        None,
        "failed capture must release its owner"
    );
}
