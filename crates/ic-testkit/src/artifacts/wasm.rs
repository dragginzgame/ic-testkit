use std::path::{Path, PathBuf};

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
/// The caller supplies the maximum accepted artifact size. Reads and allocation
/// are bounded by the shared host library; symbolic links remain allowed in
/// caller-controlled target directories. This reads bytes without validating
/// the Wasm module; use [`ic_host_artifacts::wasm::inspect`] for structural facts.
///
/// # Errors
/// Returns the shared filesystem, non-regular-file, allocation or size-limit error.
pub fn read_wasm(
    target_dir: &Path,
    crate_name: &str,
    profile_target_dir: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, ic_host_artifacts::artifact::ArtifactError> {
    let path = wasm_path(target_dir, crate_name, profile_target_dir);
    ic_host_fs::read::read_file(&path, max_bytes)
}

#[cfg(test)]
mod tests {
    use super::{read_wasm, wasm_path};
    use crate::artifacts::test_support::unique_temp_directory;
    use ic_host_artifacts::artifact::ArtifactError;
    use std::fs;

    #[test]
    fn compiled_artifact_reads_use_selected_profile_and_explicit_limit() {
        let root = unique_temp_directory("bounded-wasm");
        let path = wasm_path(&root, "canister", "custom");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = b"\0asm\x01\0\0\0";
        fs::write(&path, bytes).unwrap();
        assert_eq!(
            read_wasm(&root, "canister", "custom", bytes.len()).unwrap(),
            bytes
        );
        assert!(matches!(
            read_wasm(&root, "canister", "custom", bytes.len() - 1),
            Err(ArtifactError::LimitExceeded { .. })
        ));
        assert!(matches!(
            read_wasm(&root, "canister", "other", bytes.len()),
            Err(ArtifactError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound
        ));
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(matches!(
            read_wasm(&root, "canister", "custom", bytes.len()),
            Err(ArtifactError::NotRegularFile)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
