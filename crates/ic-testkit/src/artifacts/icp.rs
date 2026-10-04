use std::{ffi::OsString, fs, io, path::Path};

use super::digest::{InputDigest, digest_labeled_paths, read_stamp_with_limit, write_atomic};

const WATCHED_INPUT_STAMP_VERSION: &str = "ic-testkit-watched-input-v1";

/// Exact content digest captured across a set of watched input trees.
///
/// This lightweight freshness helper records input identity only. It does not
/// lock producers, validate output content, or retain artifact paths. Use
/// [`super::ArtifactCacheSpec`] for transactional publication and retained outputs.
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
            .map(|relative| (Path::new(relative), workspace_root.join(relative)));
        Ok(Self {
            digest: digest_labeled_paths("watched-inputs-v1", paths, &[])?,
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
    /// Output bytes are not hashed; a replaced nonempty artifact can still
    /// carry the same matching input stamp.
    /// Stamp reads are bounded by the expected stamp length plus one byte;
    /// oversized stamps are stale. I/O errors and invalid UTF-8 in stamps within
    /// that size limit are reported to the caller.
    pub fn artifact_is_fresh(self, artifact_path: &Path) -> io::Result<bool> {
        let metadata = fs::metadata(artifact_path)?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Ok(false);
        }

        let expected = self.stamp_contents();
        match read_stamp_with_limit(&watched_input_stamp_path(artifact_path), expected.len()) {
            Ok(stamp) => Ok(stamp.is_some_and(|contents| contents == expected)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Atomically record that an existing artifact was built from this input snapshot.
    ///
    /// The caller must coordinate producers and verify that inputs have not
    /// changed during the build before stamping. This checks only that the
    /// artifact is a nonempty regular file, not that its bytes match the build.
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

    watched_inputs
        .artifact_is_fresh(&artifact_path)
        .unwrap_or(false)
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

    #[test]
    fn artifact_freshness_rejects_malformed_and_oversized_stamps() {
        let root = temp_workspace();
        let artifact = root.join("artifact.wasm");
        fs::write(root.join("Cargo.toml"), "workspace").expect("write watched input");
        fs::write(&artifact, b"wasm").expect("write artifact");
        let snapshot =
            WatchedInputSnapshot::capture(&root, &["Cargo.toml"]).expect("capture watched inputs");
        let stamp_path = super::watched_input_stamp_path(&artifact);
        let expected = snapshot.stamp_contents();

        assert!(!snapshot.artifact_is_fresh(&artifact).unwrap());
        for contents in [
            String::new(),
            expected[..expected.len() - 1].to_owned(),
            expected.replacen("sha256:", "sha257:", 1),
            format!("{expected}\n"),
        ] {
            fs::write(&stamp_path, contents).expect("write malformed stamp");
            assert!(!snapshot.artifact_is_fresh(&artifact).unwrap());
        }

        fs::write(&stamp_path, [0xff]).expect("write invalid UTF-8 stamp");
        assert_eq!(
            snapshot.artifact_is_fresh(&artifact).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData,
        );

        // A sparse oversized sidecar must not be allocated or read in full.
        fs::write(&stamp_path, &expected).expect("write matching prefix");
        fs::OpenOptions::new()
            .write(true)
            .open(&stamp_path)
            .expect("open oversized stamp")
            .set_len(1024 * 1024 * 1024)
            .expect("extend oversized stamp");
        assert!(!snapshot.artifact_is_fresh(&artifact).unwrap());

        snapshot.mark_artifact_fresh(&artifact).unwrap();
        assert!(snapshot.artifact_is_fresh(&artifact).unwrap());
        fs::remove_file(&stamp_path).expect("remove stamp");
        fs::create_dir(&stamp_path).expect("replace stamp with unreadable directory");
        assert!(snapshot.artifact_is_fresh(&artifact).is_err());
        assert!(!icp_artifact_ready_for_build(
            &root,
            "artifact.wasm",
            &["Cargo.toml"],
        ));
        let _ = fs::remove_dir_all(root);
    }
}
