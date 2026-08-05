use std::{ffi::OsString, fs, io, path::Path};

use super::digest::{InputDigest, digest_labeled_paths, write_atomic};

const WATCHED_INPUT_STAMP_VERSION: &str = "ic-testkit-watched-input-v1";

/// Exact content digest captured across a set of watched input trees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WatchedInputSnapshot {
    digest: InputDigest,
}

impl WatchedInputSnapshot {
    /// Recursively hash the paths and contents of all watched inputs.
    ///
    /// File timestamps are deliberately excluded, so the same content produces
    /// the same digest after a Git checkout or CI cache restore.
    pub fn capture(workspace_root: &Path, watched_relative_paths: &[&str]) -> io::Result<Self> {
        let paths = watched_relative_paths
            .iter()
            .map(|relative| ((*relative).into(), workspace_root.join(relative)))
            .collect::<Vec<_>>();
        Ok(Self {
            digest: digest_labeled_paths("watched-inputs-v1", &paths, &[])?,
        })
    }

    /// Return the exact content digest of the watched inputs.
    #[must_use]
    pub const fn digest(self) -> InputDigest {
        self.digest
    }

    /// Check whether one artifact carries a matching exact-input stamp.
    ///
    /// An existing artifact without a stamp is not considered fresh. Call
    /// [`mark_artifact_fresh`](Self::mark_artifact_fresh) only after the
    /// artifact has been produced successfully from this snapshot.
    pub fn artifact_is_fresh(self, artifact_path: &Path) -> io::Result<bool> {
        let metadata = fs::metadata(artifact_path)?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Ok(false);
        }

        match fs::read_to_string(watched_input_stamp_path(artifact_path)) {
            Ok(stamp) => Ok(stamp == self.stamp_contents()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Atomically record that an existing artifact was built from this input snapshot.
    pub fn mark_artifact_fresh(self, artifact_path: &Path) -> io::Result<()> {
        let metadata = fs::metadata(artifact_path)?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot stamp missing or empty artifact: {}",
                    artifact_path.display()
                ),
            ));
        }

        write_atomic(
            &watched_input_stamp_path(artifact_path),
            self.stamp_contents().as_bytes(),
        )
    }

    fn stamp_contents(self) -> String {
        format!("{WATCHED_INPUT_STAMP_VERSION}\nsha256:{}\n", self.digest)
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

fn watched_input_stamp_path(artifact_path: &Path) -> std::path::PathBuf {
    let mut stamp_name = artifact_path
        .file_name()
        .map_or_else(|| OsString::from("artifact"), OsString::from);
    stamp_name.push(".ic-testkit-input");
    artifact_path.with_file_name(stamp_name)
}

#[cfg(test)]
mod tests {
    use super::WatchedInputSnapshot;
    use super::icp_artifact_ready_for_build;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_workspace() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ic-testkit-icp-artifact-test-{unique}-{sequence}"));
        fs::create_dir_all(path.join(".icp/local/canisters/counter"))
            .expect("create temp workspace");
        path
    }

    #[test]
    fn icp_artifact_ready_requires_matching_content_stamp() {
        let workspace_root = temp_workspace();
        let artifact_relative_path = ".icp/local/canisters/counter/counter.wasm.gz";
        let artifact_path = workspace_root.join(artifact_relative_path);
        fs::write(workspace_root.join("Cargo.toml"), "workspace").expect("write watched input");
        fs::write(&artifact_path, b"wasm").expect("write artifact");

        assert!(!icp_artifact_ready_for_build(
            &workspace_root,
            artifact_relative_path,
            &["Cargo.toml"],
        ));

        let snapshot = WatchedInputSnapshot::capture(&workspace_root, &["Cargo.toml"])
            .expect("capture exact watched inputs");
        snapshot
            .mark_artifact_fresh(&artifact_path)
            .expect("stamp artifact inputs");
        assert!(icp_artifact_ready_for_build(
            &workspace_root,
            artifact_relative_path,
            &["Cargo.toml"],
        ));

        fs::write(workspace_root.join("Cargo.toml"), "changed").expect("update watched input");
        assert!(!icp_artifact_ready_for_build(
            &workspace_root,
            artifact_relative_path,
            &["Cargo.toml"],
        ));

        let changed = WatchedInputSnapshot::capture(&workspace_root, &["Cargo.toml"])
            .expect("capture changed watched inputs");
        assert_ne!(snapshot.digest(), changed.digest());

        let _ = fs::remove_dir_all(workspace_root);
    }

    #[test]
    fn watched_input_digest_ignores_checkout_root_and_input_order() {
        let first_root = temp_workspace();
        let second_root = temp_workspace();
        for root in [&first_root, &second_root] {
            fs::create_dir_all(root.join("src")).expect("create watched source directory");
            fs::write(root.join("Cargo.toml"), "[workspace]").expect("write manifest input");
            fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 7 }")
                .expect("write source input");
        }

        let first = WatchedInputSnapshot::capture(&first_root, &["Cargo.toml", "src"])
            .expect("capture first checkout");
        let second = WatchedInputSnapshot::capture(&second_root, &["src", "Cargo.toml"])
            .expect("capture second checkout");
        assert_eq!(first.digest(), second.digest());

        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
    }
}
