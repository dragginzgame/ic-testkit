use ic_testkit::artifacts::{test_target_dir, wasm_artifacts_ready, wasm_path, workspace_root_for};
use std::{fs, path::PathBuf};

// Verify wasm artifact paths stay aligned with Cargo wasm target layout.
#[test]
fn wasm_path_uses_profile_target_directory() {
    let target_dir = PathBuf::from("/tmp/ic-testkit-target");

    assert_eq!(
        wasm_path(&target_dir, "runtime_probe", "custom-profile"),
        target_dir
            .join("wasm32-unknown-unknown")
            .join("custom-profile")
            .join("runtime_probe.wasm")
    );
}

// Verify readiness checks require every requested canister artifact.
#[test]
fn wasm_artifacts_ready_requires_all_artifacts() {
    let root = unique_temp_dir("ic-testkit-artifacts");
    let target_dir = root.join("target");
    let first = wasm_path(&target_dir, "alpha", "debug");
    let second = wasm_path(&target_dir, "beta", "debug");

    fs::create_dir_all(first.parent().expect("wasm parent")).expect("create wasm dir");
    fs::write(&first, b"alpha").expect("write first wasm");

    assert!(!wasm_artifacts_ready(
        &target_dir,
        &["alpha", "beta"],
        "debug"
    ));

    fs::write(&second, b"beta").expect("write second wasm");
    assert!(wasm_artifacts_ready(
        &target_dir,
        &["alpha", "beta"],
        "debug"
    ));

    fs::remove_dir_all(root).expect("clean temp dir");
}

// Verify workspace and target helpers derive stable host-side paths.
#[test]
fn workspace_helpers_resolve_expected_paths() {
    let manifest_dir = "/workspace/crates/ic-testkit";
    let workspace_root = workspace_root_for(manifest_dir);

    assert_eq!(workspace_root, PathBuf::from("/workspace"));
    assert_eq!(
        workspace_root_for("/workspace/ic-testkit"),
        PathBuf::from("/workspace/ic-testkit")
    );
    assert_eq!(
        test_target_dir(&workspace_root, "pic-wasm"),
        PathBuf::from("/workspace/target/pic-wasm")
    );
}

// Build a unique temp directory path for filesystem-only artifact tests.
fn unique_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale temp dir");
    }
    root
}
