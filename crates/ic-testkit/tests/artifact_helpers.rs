mod support;

use ic_testkit::artifacts::{test_target_dir, wasm_artifacts_ready, wasm_path, workspace_root_for};
use std::{fs, path::PathBuf};
use support::unique_temp_directory as unique_temp_dir;

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
    let workspace_root = workspace_root_for(env!("CARGO_MANIFEST_DIR")).unwrap();
    assert_eq!(
        workspace_root.join("Cargo.toml"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Cargo.toml")
    );
    assert_eq!(
        test_target_dir(&workspace_root, "pic-wasm"),
        workspace_root.join("target/pic-wasm")
    );
}

#[test]
fn workspace_discovery_obeys_cargo_membership_and_independent_roots() {
    let root = unique_temp_dir("cargo-workspace-discovery");
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"apps/demo/worker\", \"external\"]\nexclude = [\"excluded\", \"testing\"]\nresolver = \"2\"\n").unwrap();
    for (name, relative, extra) in [
        ("worker", "apps/demo/worker", ""),
        ("external", "external", "workspace = \"..\"\n"),
        ("excluded", "excluded", ""),
        ("independent", "testing", "\n[workspace]\n"),
        ("unlisted", "unlisted", ""),
    ] {
        let package = root.join(relative);
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(package.join("src/lib.rs"), "").unwrap();
        fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\n{extra}"),
        )
        .unwrap();
    }
    let canonical = root.canonicalize().unwrap();
    assert_eq!(
        workspace_root_for(root.join("apps/demo/worker")).unwrap(),
        canonical
    );
    assert_eq!(
        workspace_root_for(root.join("external")).unwrap(),
        canonical
    );
    for independent in ["excluded", "testing"] {
        assert_eq!(
            workspace_root_for(root.join(independent)).unwrap(),
            canonical.join(independent)
        );
    }
    assert!(workspace_root_for(root.join("unlisted")).is_err());
    assert!(workspace_root_for(root.join("missing")).is_err());
    assert!(!root.join("Cargo.lock").exists());
    assert!(!root.join("target").exists());
    fs::remove_dir_all(root).unwrap();
}
