use std::{
    fs,
    path::{Path, PathBuf},
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
