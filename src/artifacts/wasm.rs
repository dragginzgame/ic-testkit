use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Resolve the wasm artifact path for one crate under a target directory.
#[must_use]
pub fn wasm_path(target_dir: &Path, crate_name: &str, profile_target_dir: &str) -> PathBuf {
    target_dir
        .join("wasm32-unknown-unknown")
        .join(profile_target_dir)
        .join(format!("{crate_name}.wasm"))
}

/// Check whether all requested wasm artifacts already exist.
#[must_use]
pub fn wasm_artifacts_ready(
    target_dir: &Path,
    canisters: &[&str],
    profile_target_dir: &str,
) -> bool {
    canisters
        .iter()
        .all(|name| wasm_path(target_dir, name, profile_target_dir).is_file())
}

/// Read a compiled wasm artifact for one crate.
#[must_use]
pub fn read_wasm(target_dir: &Path, crate_name: &str, profile_target_dir: &str) -> Vec<u8> {
    let path = wasm_path(target_dir, crate_name, profile_target_dir);
    fs::read(&path).unwrap_or_else(|err| panic!("failed to read {crate_name} wasm: {err}"))
}

/// Build one or more wasm canisters into the provided target directory.
pub fn build_wasm_canisters(
    workspace_root: &Path,
    target_dir: &Path,
    packages: &[&str],
    cargo_profile_args: &[&str],
    extra_env: &[(&str, &str)],
) {
    let mut cmd = cargo_command();
    cmd.current_dir(workspace_root);
    cmd.env("CARGO_TARGET_DIR", target_dir);
    cmd.args(["build", "--target", "wasm32-unknown-unknown"]);
    cmd.args(cargo_profile_args);

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    for name in packages {
        cmd.args(["-p", name]);
    }

    let output = cmd.output().expect("failed to run cargo build");
    assert!(
        output.status.success(),
        "cargo build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cargo_command() -> Command {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);

    if let Some(toolchain) = std::env::var_os("RUSTUP_TOOLCHAIN") {
        command.env("RUSTUP_TOOLCHAIN", toolchain);
    }

    command
}
