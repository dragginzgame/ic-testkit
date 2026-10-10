use ic_host_process::tool::{ExecutionFailure, OutputLimit, OutputLimits, capture_command};
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
/// Both discovery streams must fit in 64 KiB; oversized output is refused
/// before parsing. No elapsed-time deadline is imposed.
///
/// # Errors
/// Returns filesystem, Cargo invocation or invalid Cargo response errors.
pub fn workspace_root_for(crate_manifest_dir: impl AsRef<Path>) -> io::Result<PathBuf> {
    let manifest = crate_manifest_dir
        .as_ref()
        .canonicalize()?
        .join("Cargo.toml");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args([
            "--offline",
            "locate-project",
            "--workspace",
            "--message-format=json",
        ])
        .arg("--manifest-path")
        .arg(manifest);
    locate_workspace(&mut command)
}

fn locate_workspace(command: &mut Command) -> io::Result<PathBuf> {
    let output = capture_command(
        command,
        OutputLimits {
            stdout: OutputLimit::Terminate(64 * 1024),
            stderr: OutputLimit::Terminate(64 * 1024),
            timeout: None,
        },
    )
    .and_then(ic_host_process::tool::ExecutionEvidence::require_complete)
    .map_err(|error| {
        if let Some(execution) = error.execution_error()
            && matches!(execution.failure, ExecutionFailure::ExitStatus)
            && execution.cleanup.is_none()
        {
            return io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Cargo workspace discovery failed ({}): {}",
                    execution
                        .evidence
                        .status
                        .expect("failed command has a status"),
                    String::from_utf8_lossy(&execution.evidence.stderr).trim()
                ),
            );
        }
        let kind = match error.execution_error().map(|execution| &execution.failure) {
            Some(ExecutionFailure::Io { source, .. }) => source.kind(),
            Some(ExecutionFailure::ExitStatus) => io::ErrorKind::InvalidInput,
            _ => io::ErrorKind::Other,
        };
        io::Error::new(kind, error)
    })?;
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use ic_host_process::tool::ToolError;
    use std::fs;

    #[test]
    fn discovery_refuses_oversized_complete_json_and_preserves_invocation_errors() {
        let root = crate::artifacts::test_support::unique_temp_directory("discovery-capture");
        let mut response = br#"{"root":"/tmp/Cargo.toml"}"#.to_vec();
        response.resize(64 * 1024, b' ');
        fs::write(root.join("response"), &response).unwrap();
        let mut command = Command::new("/bin/sh");
        command.current_dir(&root).args(["-c", "cat response"]);
        assert_eq!(locate_workspace(&mut command).unwrap(), Path::new("/tmp"));
        response.push(b' ');
        fs::write(root.join("response"), &response).unwrap();
        let error = locate_workspace(&mut command).unwrap_err();
        assert!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<ToolError>()
                .is_some()
        );
        let mut failed = Command::new("/bin/sh");
        failed.args(["-c", "printf refused >&2; exit 23"]);
        let error = locate_workspace(&mut failed).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("refused"));
        let mut missing = Command::new(root.join("absent"));
        let error = locate_workspace(&mut missing).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(
            error
                .get_ref()
                .unwrap()
                .downcast_ref::<ToolError>()
                .is_some()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
