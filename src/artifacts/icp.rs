use std::{fs, io, path::Path, time::SystemTime};

///
/// WatchedInputSnapshot
///

#[derive(Clone, Copy, Debug)]
pub struct WatchedInputSnapshot {
    newest_input_mtime: SystemTime,
}

impl WatchedInputSnapshot {
    /// Capture the newest modification time across all watched inputs once.
    pub fn capture(workspace_root: &Path, watched_relative_paths: &[&str]) -> io::Result<Self> {
        Ok(Self {
            newest_input_mtime: newest_watched_input_mtime(workspace_root, watched_relative_paths)?,
        })
    }

    /// Check whether one artifact is newer than the captured watched inputs.
    pub fn artifact_is_fresh(self, artifact_path: &Path) -> io::Result<bool> {
        let artifact_mtime = fs::metadata(artifact_path)?.modified()?;
        Ok(self.newest_input_mtime <= artifact_mtime)
    }
}

/// Check whether an ICP artifact exists, is nonempty, and is fresh against watched inputs.
#[must_use]
pub fn icp_artifact_ready_for_build(
    workspace_root: &Path,
    artifact_relative_path: &str,
    watched_relative_paths: &[&str],
) -> bool {
    let Ok(watched_inputs) = WatchedInputSnapshot::capture(workspace_root, watched_relative_paths)
    else {
        return false;
    };

    icp_artifact_ready_with_snapshot(workspace_root, artifact_relative_path, watched_inputs)
}

/// Check one ICP artifact against one already-captured watched-input snapshot.
#[must_use]
pub fn icp_artifact_ready_with_snapshot(
    workspace_root: &Path,
    artifact_relative_path: &str,
    watched_inputs: WatchedInputSnapshot,
) -> bool {
    let artifact_path = workspace_root.join(artifact_relative_path);

    match fs::metadata(&artifact_path) {
        Ok(meta) if meta.is_file() && meta.len() > 0 => watched_inputs
            .artifact_is_fresh(&artifact_path)
            .unwrap_or(false),
        _ => false,
    }
}

// Walk watched files and directories and return the newest modification time.
fn newest_watched_input_mtime(
    workspace_root: &Path,
    watched_relative_paths: &[&str],
) -> io::Result<SystemTime> {
    let mut newest = SystemTime::UNIX_EPOCH;

    for relative in watched_relative_paths {
        let path = workspace_root.join(relative);
        newest = newest.max(newest_path_mtime(&path)?);
    }

    Ok(newest)
}

// Recursively compute the newest modification time under one watched path.
fn newest_path_mtime(path: &Path) -> io::Result<SystemTime> {
    let metadata = fs::metadata(path)?;
    let mut newest = metadata.modified()?;

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            newest = newest.max(newest_path_mtime(&entry.path())?);
        }
    }

    Ok(newest)
}

#[cfg(test)]
mod tests {
    use super::icp_artifact_ready_for_build;
    use std::{
        fs,
        path::PathBuf,
        thread::sleep,
        time::Duration,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_workspace() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ic-testkit-icp-artifact-test-{unique}"));
        fs::create_dir_all(path.join(".icp/local/canisters/counter"))
            .expect("create temp workspace");
        path
    }

    #[test]
    fn icp_artifact_ready_requires_fresh_nonempty_artifact() {
        let workspace_root = temp_workspace();
        let artifact_relative_path = ".icp/local/canisters/counter/counter.wasm.gz";
        let artifact_path = workspace_root.join(artifact_relative_path);
        fs::write(workspace_root.join("Cargo.toml"), "workspace").expect("write watched input");
        sleep(Duration::from_millis(20));
        fs::write(&artifact_path, b"wasm").expect("write artifact");

        assert!(icp_artifact_ready_for_build(
            &workspace_root,
            artifact_relative_path,
            &["Cargo.toml"],
        ));

        sleep(Duration::from_millis(20));
        fs::write(workspace_root.join("Cargo.toml"), "changed").expect("update watched input");
        assert!(!icp_artifact_ready_for_build(
            &workspace_root,
            artifact_relative_path,
            &["Cargo.toml"],
        ));

        let _ = fs::remove_dir_all(workspace_root);
    }
}
