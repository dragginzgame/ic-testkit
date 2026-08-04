use std::path::{Path, PathBuf};

/// Resolve the conventional workspace root from a crate manifest directory.
///
/// A manifest directly below `<workspace>/crates` resolves to `<workspace>`;
/// other paths are returned unchanged.
#[must_use]
pub fn workspace_root_for(crate_manifest_dir: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(crate_manifest_dir);
    if manifest_dir.parent().and_then(Path::file_name) == Some(std::ffi::OsStr::new("crates")) {
        return manifest_dir
            .parent()
            .and_then(Path::parent)
            .map(PathBuf::from)
            .expect("workspace root");
    }

    manifest_dir
}

/// Return `<workspace>/target/<name>` for isolated host-side test artifacts.
#[must_use]
pub fn test_target_dir(workspace_root: &Path, name: &str) -> PathBuf {
    workspace_root.join("target").join(name)
}
