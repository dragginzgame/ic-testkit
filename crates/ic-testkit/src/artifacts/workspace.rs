use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
};

/// Ask Cargo for the workspace owning a crate manifest directory.
///
/// Cargo owns membership, explicit `package.workspace`, exclusions and nested
/// independent workspaces. This performs an offline `locate-project`, without
/// resolving dependencies or installing a toolchain. It never guesses from the
/// directory layout or falls back to the supplied directory.
///
/// # Errors
/// Returns filesystem, Cargo invocation or invalid Cargo response errors.
pub fn workspace_root_for(crate_manifest_dir: impl AsRef<Path>) -> io::Result<PathBuf> {
    let manifest = crate_manifest_dir
        .as_ref()
        .canonicalize()?
        .join("Cargo.toml");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args([
            "--offline",
            "locate-project",
            "--workspace",
            "--message-format=json",
        ])
        .arg("--manifest-path")
        .arg(manifest)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Cargo workspace discovery failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let manifest = response
        .get("root")
        .and_then(serde_json::Value::as_str)
        .map(Path::new)
        .filter(|path| {
            path.is_absolute() && path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml"))
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Cargo returned no absolute workspace manifest",
            )
        })?;
    manifest.parent().map(Path::to_owned).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Cargo workspace manifest has no parent",
        )
    })
}

/// Return `<workspace>/target/<name>` for isolated host-side test artifacts.
#[must_use]
pub fn test_target_dir(workspace_root: &Path, name: &str) -> PathBuf {
    workspace_root.join("target").join(name)
}
