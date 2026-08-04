use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Resolve one crate's Wasm artifact under a caller-selected Cargo target directory.
#[must_use]
pub fn wasm_path(target_dir: &Path, crate_name: &str, profile_target_dir: &str) -> PathBuf {
    target_dir
        .join("wasm32-unknown-unknown")
        .join(profile_target_dir)
        .join(format!("{crate_name}.wasm"))
}

/// Check whether every requested Wasm artifact is a regular file.
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

/// Read a compiled Wasm artifact for one crate.
///
/// # Panics
///
/// Panics with the crate name when the artifact cannot be read.
#[must_use]
pub fn read_wasm(target_dir: &Path, crate_name: &str, profile_target_dir: &str) -> Vec<u8> {
    let path = wasm_path(target_dir, crate_name, profile_target_dir);
    fs::read(&path).unwrap_or_else(|err| panic!("failed to read {crate_name} wasm: {err}"))
}

/// Build one or more Wasm canisters into the provided target directory.
///
/// `cargo_profile_args` accepts Cargo flags such as `--release`. `extra_env`
/// applies only to the child Cargo process and does not mutate the current
/// process environment.
///
/// # Panics
///
/// Panics when Cargo cannot be launched or returns a failing status.
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
