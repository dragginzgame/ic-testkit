use std::{
    fs,
    path::{Path, PathBuf},
};

use super::wasm_cache::{WasmBuildSpec, build_wasm_canisters_cached};

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
    let profile_target_dir = profile_target_dir(cargo_profile_args);
    let spec = WasmBuildSpec::new(workspace_root, target_dir, packages, &profile_target_dir)
        .with_cargo_profile_args(cargo_profile_args)
        .with_extra_env(extra_env);
    build_wasm_canisters_cached(&spec)
        .unwrap_or_else(|error| panic!("cargo Wasm build failed: {error}"));
}

fn profile_target_dir(cargo_profile_args: &[&str]) -> String {
    let mut profile = "debug";
    let mut args = cargo_profile_args.iter().copied();
    while let Some(argument) = args.next() {
        match argument {
            "--release" => profile = "release",
            "--profile" => {
                if let Some(value) = args.next() {
                    profile = value;
                }
            }
            _ => {
                if let Some(value) = argument.strip_prefix("--profile=") {
                    profile = value;
                }
            }
        }
    }
    profile.to_owned()
}

#[cfg(test)]
mod tests {
    use super::profile_target_dir;

    #[test]
    fn profile_target_directory_follows_cargo_arguments() {
        assert_eq!(profile_target_dir(&[]), "debug");
        assert_eq!(profile_target_dir(&["--release"]), "release");
        assert_eq!(profile_target_dir(&["--profile", "fast"]), "fast");
        assert_eq!(profile_target_dir(&["--profile=small"]), "small");
    }
}
