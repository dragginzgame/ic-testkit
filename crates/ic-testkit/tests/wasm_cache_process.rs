#![cfg(unix)]

use ic_testkit::artifacts::{
    WasmBuildOutcome, WasmBuildSpec, build_wasm_canisters_cached, workspace_root_for,
};
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

const PERF_PROBE_PACKAGE: &str = "ic_testkit_perf_probe";
const WORKER_ROOT_ENV: &str = "IC_TESTKIT_WASM_PROCESS_ROOT";
const WORKER_ID_ENV: &str = "IC_TESTKIT_WASM_PROCESS_WORKER";
static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn different_cache_roots_coordinate_one_shared_incremental_target_across_processes() {
    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    if !workspace
        .join("canisters/test/perf_probe/Cargo.toml")
        .is_file()
    {
        eprintln!("skipping shared-target process test: fixture canister is not packaged");
        return;
    }
    let root = unique_temp_directory("wasm-shared-process-lock");
    write_cargo_wrapper(&root.join("cargo-wrapper.sh"));
    let executable = std::env::current_exe().expect("resolve current process-test executable");
    let mut first = spawn_worker(&executable, &root, "first");
    let mut second = spawn_worker(&executable, &root, "second");

    wait_for_path(&root.join("ready-first"));
    wait_for_path(&root.join("ready-second"));
    fs::write(root.join("go"), b"go").expect("release shared-target workers");

    assert!(first.wait().expect("wait for first worker").success());
    assert!(second.wait().expect("wait for second worker").success());
    assert!(
        !root.join("overlap").exists(),
        "Cargo builds using one shared target must not overlap"
    );
    assert_eq!(
        fs::read_to_string(root.join("builds"))
            .expect("read shared-target build log")
            .lines()
            .count(),
        2,
        "different exact caches should each build once"
    );
    fs::remove_dir_all(root).expect("remove shared-target process fixture");
}

#[test]
#[ignore = "subprocess worker selected explicitly by the parent shared-target test"]
fn shared_incremental_process_worker() {
    let root = PathBuf::from(
        std::env::var_os(WORKER_ROOT_ENV).expect("worker root environment must be set"),
    );
    let worker = std::env::var(WORKER_ID_ENV).expect("worker identity environment must be set");
    fs::write(root.join(format!("ready-{worker}")), b"ready").expect("mark worker ready");
    wait_for_path(&root.join("go"));

    let workspace = workspace_root_for(env!("CARGO_MANIFEST_DIR"));
    let real_cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join(format!("cache-{worker}")),
        &[PERF_PROBE_PACKAGE],
        "debug",
    )
    .with_shared_incremental_target(root.join("shared-target"))
    .with_cargo_program(root.join("cargo-wrapper.sh"))
    .with_extra_env_os([
        (OsString::from("REAL_CARGO"), real_cargo),
        (
            OsString::from("IC_TESTKIT_ACTIVE_BUILD"),
            root.join("active-build").into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_OVERLAP_FILE"),
            root.join("overlap").into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_BUILD_LOG"),
            root.join("builds").into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_WORKER_VARIANT"),
            OsString::from(worker),
        ),
    ]);

    let outcome = build_wasm_canisters_cached(&spec).expect("run shared-target worker build");
    assert!(matches!(outcome, WasmBuildOutcome::Built(_)));
}

fn write_cargo_wrapper(path: &Path) {
    fs::write(
        path,
        b"#!/bin/sh\n\
if [ \"$1\" = \"build\" ]; then\n\
  if mkdir \"$IC_TESTKIT_ACTIVE_BUILD\" 2>/dev/null; then\n\
    printf '%s\\n' \"$IC_TESTKIT_WORKER_VARIANT\" >> \"$IC_TESTKIT_BUILD_LOG\"\n\
    sleep 0.25\n\
    \"$REAL_CARGO\" \"$@\"\n\
    status=$?\n\
    rmdir \"$IC_TESTKIT_ACTIVE_BUILD\"\n\
    exit $status\n\
  fi\n\
  : > \"$IC_TESTKIT_OVERLAP_FILE\"\n\
fi\n\
exec \"$REAL_CARGO\" \"$@\"\n",
    )
    .expect("write Cargo process-test wrapper");
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make Cargo wrapper executable");
}

fn spawn_worker(executable: &Path, root: &Path, worker: &str) -> std::process::Child {
    Command::new(executable)
        .args(["--ignored", "--exact", "shared_incremental_process_worker"])
        .env(WORKER_ROOT_ENV, root)
        .env(WORKER_ID_ENV, worker)
        .spawn()
        .expect("spawn shared-target process worker")
}

fn wait_for_path(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn unique_temp_directory(label: &str) -> PathBuf {
    let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ic-testkit-{label}-{}-{sequence}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("remove stale shared-target process directory");
    }
    fs::create_dir_all(&path).expect("create shared-target process directory");
    path
}
