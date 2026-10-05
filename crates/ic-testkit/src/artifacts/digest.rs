use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{self, Read as _, Write as _},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(unix)]
use std::os::unix::{ffi::OsStrExt as _, fs::MetadataExt as _};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt as _;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct AtomicCopyErrorContext {
    source_path: PathBuf,
    destination_path: PathBuf,
    source: io::Error,
}

impl std::fmt::Display for AtomicCopyErrorContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to atomically copy {} to {}: {}",
            self.source_path.display(),
            self.destination_path.display(),
            self.source
        )
    }
}

impl std::error::Error for AtomicCopyErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// SHA-256 digest of one deterministic artifact-input set.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputDigest([u8; 32]);

impl InputDigest {
    /// Borrow the raw SHA-256 bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Render the digest as lowercase hexadecimal.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut hex = String::with_capacity(64);
        write!(hex, "{self}").expect("writing to a String cannot fail");
        hex
    }
}

impl std::fmt::Display for InputDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(super) struct InputHasher(Sha256);

impl InputHasher {
    pub(super) fn new(domain: &str) -> Self {
        let mut hasher = Self(Sha256::new());
        hasher.field("domain", domain.as_bytes());
        hasher
    }

    pub(super) fn field(&mut self, label: &str, value: &[u8]) {
        self.field_header(
            label,
            u64::try_from(value.len()).expect("input value length must fit in u64"),
        );
        self.0.update(value);
    }

    fn field_header(&mut self, label: &str, value_len: u64) {
        self.0.update(
            u64::try_from(label.len())
                .expect("input label length must fit in u64")
                .to_le_bytes(),
        );
        self.0.update(label.as_bytes());
        self.0.update(value_len.to_le_bytes());
    }

    fn file_field(&mut self, label: &str, path: &Path) -> io::Result<u64> {
        let mut file = File::open(path)?;
        let expected_len = file.metadata()?.len();
        self.field_header(label, expected_len);

        let mut actual_len = 0_u64;
        // Small sources need only their declared length; large artifacts use bounded reads.
        // Even empty files need a nonempty read buffer to detect growth.
        let buffer_len = usize::try_from(expected_len.clamp(1, 64 * 1024))
            .expect("bounded artifact buffer length must fit in usize");
        let mut buffer = vec![0_u8; buffer_len];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            actual_len = actual_len
                .saturating_add(u64::try_from(read).expect("artifact read length must fit in u64"));
            if actual_len > expected_len {
                break;
            }
            self.0.update(&buffer[..read]);
        }
        if actual_len != expected_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "file changed size while hashing: expected {expected_len} bytes, read {actual_len}"
                ),
            ));
        }
        Ok(actual_len)
    }

    pub(super) fn finish(self) -> InputDigest {
        InputDigest(self.0.finalize().into())
    }
}

pub(super) fn digest_bytes(domain: &str, value: &[u8]) -> InputDigest {
    let mut hasher = InputHasher::new(domain);
    hasher.field("content", value);
    hasher.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FileDigest {
    pub(super) bytes: u64,
    pub(super) digest: InputDigest,
}

pub(super) fn digest_file(domain: &str, path: &Path) -> io::Result<FileDigest> {
    let mut hasher = InputHasher::new(domain);
    let bytes = hasher.file_field("content", path)?;
    Ok(FileDigest {
        bytes,
        digest: hasher.finish(),
    })
}

/// Read a UTF-8 stamp without allocating or reading an oversized sidecar in full.
/// An oversized stamp is stale; other read and decoding errors reach the caller.
pub(super) fn read_stamp_with_limit(path: &Path, maximum_len: usize) -> io::Result<Option<String>> {
    read_file_with_limit(path, maximum_len)?
        .map(|contents| {
            String::from_utf8(contents)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .transpose()
}

/// Read at most the format's maximum length plus one byte to detect oversized files.
pub(super) fn read_file_with_limit(path: &Path, maximum_len: usize) -> io::Result<Option<Vec<u8>>> {
    let mut contents = Vec::with_capacity(maximum_len + 1);
    File::open(path)?
        .take((maximum_len + 1) as u64)
        .read_to_end(&mut contents)?;
    if contents.len() > maximum_len {
        return Ok(None);
    }
    Ok(Some(contents))
}

/// Only reuse an independent, caller-owned writable destination. The caller
/// coordinates other writers and supplies a digest from a verified cache entry.
pub(super) fn destination_matches_digest(
    domain: &str,
    destination: &Path,
    expected: &FileDigest,
) -> bool {
    destination_is_reusable(destination, expected.bytes)
        && digest_file(domain, destination).is_ok_and(|actual| actual == *expected)
}

pub(super) fn destination_matches_bytes(destination: &Path, expected: &[u8]) -> bool {
    destination_is_reusable(
        destination,
        u64::try_from(expected.len()).expect("artifact byte length must fit in u64"),
    ) && read_file_with_limit(destination, expected.len())
        .is_ok_and(|actual| actual.as_deref() == Some(expected))
}

fn destination_is_reusable(destination: &Path, expected_bytes: u64) -> bool {
    #[cfg(unix)]
    {
        let Ok(metadata) = fs::symlink_metadata(destination) else {
            return false;
        };
        // SAFETY: geteuid takes no pointers and has no failure case.
        let effective_uid = unsafe { libc::geteuid() };
        // Detach links and normalize foreign-owned, restricted or executable
        // files, even when their bytes match a retained artifact.
        if !metadata.file_type().is_file()
            || metadata.nlink() != 1
            || metadata.uid() != effective_uid
            || metadata.mode() & 0o600 != 0o600
            || metadata.mode() & 0o7111 != 0
            || metadata.len() != expected_bytes
        {
            return false;
        }
        true
    }
    #[cfg(not(unix))]
    {
        // Preserve replacement where a portable single-link check is unavailable.
        let _ = (destination, expected_bytes);
        false
    }
}

pub(super) fn digest_labeled_paths<L: AsRef<Path>, P: AsRef<Path>>(
    domain: &str,
    paths: impl IntoIterator<Item = (L, P)>,
    excluded_roots: &[PathBuf],
) -> io::Result<InputDigest> {
    let mut paths = paths.into_iter().collect::<Vec<_>>();
    paths.sort_by(|(left, _), (right, _)| {
        os_bytes(left.as_ref().as_os_str()).cmp(&os_bytes(right.as_ref().as_os_str()))
    });

    let excluded_roots = excluded_roots
        .iter()
        .filter_map(|path| path.canonicalize().ok())
        .collect::<Vec<_>>();
    let mut visited_directories = BTreeSet::new();
    let mut hasher = InputHasher::new(domain);
    for (label, path) in paths {
        hash_path(
            &mut hasher,
            label.as_ref(),
            path.as_ref(),
            &excluded_roots,
            &mut visited_directories,
            true,
            None,
        )?;
    }
    Ok(hasher.finish())
}

#[derive(Default)]
pub(super) struct LabeledPathDigestCache {
    entries: Vec<LabeledPathDigestCacheEntry>,
}

struct LabeledPathDigestCacheEntry {
    domain: String,
    label: PathBuf,
    path: PathBuf,
    canonical_root: PathBuf,
    excluded_roots: Vec<PathBuf>,
    traversed_external_path: bool,
    digest: InputDigest,
}

struct HashPathTrace {
    canonical_root: PathBuf,
    traversed_external_path: bool,
}

pub(super) fn digest_labeled_paths_composable<'a>(
    domain: &str,
    paths: impl IntoIterator<Item = (&'a Path, &'a Path)>,
    excluded_roots: &[PathBuf],
    cache: &mut LabeledPathDigestCache,
) -> io::Result<InputDigest> {
    let mut paths = paths.into_iter().collect::<Vec<_>>();
    paths.sort_by(|(left, _), (right, _)| {
        os_bytes(left.as_os_str()).cmp(&os_bytes(right.as_os_str()))
    });
    let excluded_roots = excluded_roots
        .iter()
        .filter_map(|path| path.canonicalize().ok())
        .collect::<Vec<_>>();
    let mut hasher = InputHasher::new(&format!("{domain}/composable-v1"));
    for (label, path) in paths {
        let digest = cache.digest_root(domain, label, path, &excluded_roots)?;
        hasher.field("input-label", &os_bytes(label.as_os_str()));
        hasher.field("input-digest", digest.as_bytes());
    }
    Ok(hasher.finish())
}

impl LabeledPathDigestCache {
    fn digest_root(
        &mut self,
        domain: &str,
        label: &Path,
        path: &Path,
        excluded_roots: &[PathBuf],
    ) -> io::Result<InputDigest> {
        let canonical_root = path.canonicalize()?;
        if let Some(entry) = self.entries.iter().find(|entry| {
            entry.domain == domain
                && entry.label == label
                && entry.path == path
                && entry.excluded_roots.iter().eq(effective_root_exclusions(
                    &entry.canonical_root,
                    excluded_roots,
                    entry.traversed_external_path,
                ))
        }) {
            return Ok(entry.digest);
        }
        let mut hasher = InputHasher::new(&format!("{domain}/root-v1"));
        let mut trace = HashPathTrace {
            canonical_root: canonical_root.clone(),
            traversed_external_path: false,
        };
        hash_path(
            &mut hasher,
            label,
            path,
            excluded_roots,
            &mut BTreeSet::new(),
            true,
            Some(&mut trace),
        )?;
        let digest = hasher.finish();
        self.entries.push(LabeledPathDigestCacheEntry {
            domain: domain.to_owned(),
            label: label.to_owned(),
            path: path.to_owned(),
            canonical_root,
            excluded_roots: effective_root_exclusions(
                &trace.canonical_root,
                excluded_roots,
                trace.traversed_external_path,
            )
            .cloned()
            .collect(),
            traversed_external_path: trace.traversed_external_path,
            digest,
        });
        Ok(digest)
    }
}

fn effective_root_exclusions<'a>(
    canonical_root: &'a Path,
    excluded_roots: &'a [PathBuf],
    traversed_external_path: bool,
) -> impl Iterator<Item = &'a PathBuf> {
    excluded_roots.iter().filter(move |excluded| {
        traversed_external_path
            || excluded.starts_with(canonical_root)
            || canonical_root.starts_with(excluded)
    })
}

fn hash_path(
    hasher: &mut InputHasher,
    label: &Path,
    path: &Path,
    excluded_roots: &[PathBuf],
    visited_directories: &mut BTreeSet<PathBuf>,
    declared_root: bool,
    mut trace: Option<&mut HashPathTrace>,
) -> io::Result<()> {
    let context =
        |error: io::Error| io::Error::new(error.kind(), format!("{}: {error}", path.display()));
    let canonical = path.canonicalize().map_err(context)?;
    if let Some(trace) = &mut trace
        && !canonical.starts_with(&trace.canonical_root)
    {
        trace.traversed_external_path = true;
    }
    if excluded_roots
        .iter()
        .any(|excluded| canonical.starts_with(excluded))
    {
        if declared_root {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "declared input is located inside an excluded cache root: {}",
                    path.display()
                ),
            ));
        }
        return Ok(());
    }

    let metadata = fs::metadata(path).map_err(context)?;
    let label_bytes = os_bytes(label.as_os_str());
    if metadata.is_file() {
        hasher.field("file-path", &label_bytes);
        hasher.file_field("file-content", path).map_err(context)?;
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "watched input is not a regular file or directory: {}",
                path.display()
            ),
        ));
    }

    hasher.field("directory", &label_bytes);
    if !visited_directories.insert(canonical) {
        hasher.field("directory-already-visited", &label_bytes);
        return Ok(());
    }

    let mut entries = fs::read_dir(path)
        .map_err(context)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(context)?;
    entries.sort_by_cached_key(|entry| os_bytes(&entry.file_name()).into_owned());
    for entry in entries {
        hash_path(
            hasher,
            &label.join(entry.file_name()),
            &entry.path(),
            excluded_roots,
            visited_directories,
            false,
            trace.as_deref_mut(),
        )?;
    }
    Ok(())
}

pub(super) fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    write_file_atomic(path, |file| file.write_all(contents))
}

pub(super) fn copy_file_atomic(source: &Path, destination: &Path) -> io::Result<u64> {
    let result = (|| {
        let mut source_file = File::open(source)?;
        write_file_atomic(destination, |destination_file| {
            io::copy(&mut source_file, destination_file)
        })
    })();
    result.map_err(|source_error| {
        io::Error::new(
            source_error.kind(),
            AtomicCopyErrorContext {
                source_path: source.to_owned(),
                destination_path: destination.to_owned(),
                source: source_error,
            },
        )
    })
}

fn write_file_atomic<T>(
    path: &Path,
    write: impl FnOnce(&mut File) -> io::Result<T>,
) -> io::Result<T> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("atomic output path has no parent: {}", path.display()),
        )
    })?;
    fs::create_dir_all(parent)?;

    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("atomic output path has no file name: {}", path.display()),
        )
    })?;
    let temp_path = loop {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temp_name = format!(".ic-testkit-tmp-{}-{sequence}", std::process::id());
        // Keep names short and distinct from the destination, including on
        // case-insensitive filesystems. The sibling preserves atomic rename.
        if !file_name
            .as_encoded_bytes()
            .eq_ignore_ascii_case(temp_name.as_bytes())
        {
            break parent.join(temp_name);
        }
    };

    // Cleanup owns this path only after exclusive creation succeeds.
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)?;
    let result = (|| {
        let value = write(&mut file)?;
        file.sync_all()?;
        fs::rename(&temp_path, path)?;
        Ok(value)
    })();
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[cfg(unix)]
pub(super) fn os_bytes(value: &OsStr) -> Cow<'_, [u8]> {
    Cow::Borrowed(value.as_bytes())
}

#[cfg(windows)]
pub(super) fn os_bytes(value: &OsStr) -> Cow<'_, [u8]> {
    Cow::Owned(value.encode_wide().flat_map(u16::to_le_bytes).collect())
}

#[cfg(not(any(unix, windows)))]
pub(super) fn os_bytes(value: &OsStr) -> Cow<'_, [u8]> {
    Cow::Owned(value.to_string_lossy().as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::{
        LabeledPathDigestCache, copy_file_atomic, digest_bytes, digest_file,
        digest_labeled_paths_composable, write_atomic,
    };
    use crate::artifacts::test_support::unique_temp_directory;
    use std::{
        fs,
        io::{self, Write as _},
        path::PathBuf,
    };

    #[cfg(unix)]
    use super::{InputHasher, digest_labeled_paths};
    #[cfg(unix)]
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};
    #[cfg(windows)]
    use std::{ffi::OsString, os::windows::ffi::OsStringExt as _};

    #[test]
    fn digest_text_preserves_lowercase_hex_and_leading_zeroes() {
        let digest = super::InputDigest(std::array::from_fn(|index| {
            u8::try_from(index).expect("digest byte index must fit")
        }));
        let expected = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        assert_eq!(digest.to_hex(), expected);
        assert_eq!(digest.to_string(), expected);
        assert_eq!(super::InputDigest([0xff; 32]).to_string(), "ff".repeat(32));
    }

    #[test]
    #[cfg(unix)]
    fn labeled_path_digests_preserve_native_names_and_sorted_order() {
        let names: &[&[u8]] = &[
            b"\xce\xbb",
            #[cfg(target_os = "linux")]
            b"\xff",
        ];
        for &name in names {
            let root = unique_temp_directory("native-path-digest");
            let tree = root.join("tree");
            fs::create_dir_all(tree.join("nested")).unwrap();
            fs::write(tree.join(OsStr::from_bytes(name)), b"native").unwrap();
            fs::write(tree.join("nested/z"), b"last").unwrap();
            fs::write(tree.join("a"), b"first").unwrap();
            fs::write(root.join("top"), b"top").unwrap();
            let mut paths = [
                (PathBuf::from("tree"), tree),
                (PathBuf::from("aaa"), root.join("top")),
            ];

            let tree_fields = |hasher: &mut InputHasher| {
                hasher.field("directory", b"tree");
                hasher.field("file-path", b"tree/a");
                hasher.field("file-content", b"first");
                hasher.field("directory", b"tree/nested");
                hasher.field("file-path", b"tree/nested/z");
                hasher.field("file-content", b"last");
                hasher.field("file-path", &[b"tree/".as_slice(), name].concat());
                hasher.field("file-content", b"native");
            };
            let mut expected = InputHasher::new("native-path-test-v1");
            expected.field("file-path", b"aaa");
            expected.field("file-content", b"top");
            tree_fields(&mut expected);
            let expected = expected.finish();

            let mut top = InputHasher::new("native-path-test-v1/root-v1");
            top.field("file-path", b"aaa");
            top.field("file-content", b"top");
            let mut tree = InputHasher::new("native-path-test-v1/root-v1");
            tree_fields(&mut tree);
            let mut composable = InputHasher::new("native-path-test-v1/composable-v1");
            composable.field("input-label", b"aaa");
            composable.field("input-digest", top.finish().as_bytes());
            composable.field("input-label", b"tree");
            composable.field("input-digest", tree.finish().as_bytes());
            let composable = composable.finish();

            for _ in 0..2 {
                assert_eq!(
                    digest_labeled_paths(
                        "native-path-test-v1",
                        paths.iter().map(|(label, path)| (label, path)),
                        &[],
                    )
                    .unwrap(),
                    expected,
                );
                assert_eq!(
                    digest_labeled_paths_composable(
                        "native-path-test-v1",
                        paths
                            .iter()
                            .map(|(label, path)| (label.as_path(), path.as_path())),
                        &[],
                        &mut LabeledPathDigestCache::default(),
                    )
                    .unwrap(),
                    composable,
                );
                paths.reverse();
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    #[cfg(unix)]
    fn native_bytes_preserve_non_utf8_without_a_filesystem_roundtrip() {
        assert_eq!(
            super::os_bytes(OsStr::from_bytes(b"name\xff")).as_ref(),
            b"name\xff"
        );
    }

    #[test]
    #[cfg(windows)]
    fn native_names_preserve_utf16_little_endian_encoding() {
        let value = OsString::from_wide(&[0x0061, 0xd800, 0x0100]);
        assert_eq!(super::os_bytes(&value).as_ref(), &[0x61, 0, 0, 0xd8, 0, 1]);
    }

    #[test]
    fn streaming_digest_and_atomic_copy_preserve_exact_bytes() {
        let root = unique_temp_directory("streaming-digest");
        let source = root.join("source");
        let destination = root.join("destination");
        let mut contents = vec![0_u8; 192 * 1024 + 37];
        for (index, byte) in contents.iter_mut().enumerate() {
            *byte = u8::try_from(index % 251).expect("test byte must fit");
        }
        for length in [
            0,
            1,
            1024,
            16 * 1024,
            64 * 1024 - 1,
            64 * 1024,
            64 * 1024 + 1,
            contents.len(),
        ] {
            let data = &contents[..length];
            fs::write(&source, data).expect("write source");
            let streamed = digest_file("streaming-test-v1", &source).expect("digest file");
            assert_eq!(
                streamed.bytes,
                u64::try_from(length).expect("fixture length must fit in u64")
            );
            assert_eq!(streamed.digest, digest_bytes("streaming-test-v1", data));
        }

        write_atomic(&destination, b"old").expect("write original destination");
        assert_eq!(
            copy_file_atomic(&source, &destination).expect("copy source atomically"),
            u64::try_from(contents.len()).expect("fixture length must fit in u64")
        );
        assert_eq!(
            fs::read(&destination).expect("read copied destination"),
            contents
        );

        let missing = root.join("missing");
        let error = copy_file_atomic(&missing, &destination).expect_err("missing source must fail");
        let message = error.to_string();
        assert!(message.contains(&missing.display().to_string()));
        assert!(message.contains(&destination.display().to_string()));
        fs::remove_dir_all(root).expect("remove streaming-digest test directory");
    }

    #[test]
    fn atomic_creation_failure_preserves_existing_files() {
        const CHILD_ENV: &str = "IC_TESTKIT_ATOMIC_CREATION_COLLISION_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            // Isolate the temporary-name sequence from other parallel tests.
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "artifacts::digest::tests::atomic_creation_failure_preserves_existing_files",
                    "--test-threads=1",
                ])
                .env(CHILD_ENV, "1")
                .output()
                .unwrap();
            assert!(
                child.status.success(),
                "collision regression failed: {}{}",
                String::from_utf8_lossy(&child.stdout),
                String::from_utf8_lossy(&child.stderr)
            );
            return;
        }

        let root = unique_temp_directory("atomic-creation-collision");
        let destination = root.join("output");
        fs::write(&destination, b"original output").unwrap();
        let sequence = super::TEMP_FILE_SEQUENCE.load(super::Ordering::Relaxed);
        let existing = root.join(format!(".ic-testkit-tmp-{}-{sequence}", std::process::id()));
        fs::write(&existing, b"existing temporary file").unwrap();

        let error = write_atomic(&destination, b"replacement").unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&destination).unwrap(), b"original output");
        assert_eq!(fs::read(&existing).unwrap(), b"existing temporary file");

        // A subsequent acquisition gets a new name and can publish normally.
        write_atomic(&destination, b"replacement").unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"replacement");
        assert_eq!(fs::read(&existing).unwrap(), b"existing temporary file");

        // A caller may choose a destination in the temporary-name namespace.
        // It must still stay absent until publication rather than be opened directly.
        let sequence = super::TEMP_FILE_SEQUENCE.load(super::Ordering::Relaxed);
        let destination = root.join(format!(".ic-testkit-tmp-{}-{sequence}", std::process::id()));
        super::write_file_atomic(&destination, |file| {
            assert!(!destination.exists());
            std::io::Write::write_all(file, b"separate temporary file")
        })
        .unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"separate temporary file");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_publication_failures_remove_only_the_owned_temporary_file() {
        let root = unique_temp_directory("atomic-publication-failure");
        let destination = root.join("output");
        fs::write(&destination, b"original output").unwrap();
        let error = super::write_file_atomic(&destination, |file| {
            file.write_all(b"partial output")?;
            Err::<(), _>(io::Error::other("synthetic write failure"))
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "synthetic write failure");
        assert_eq!(fs::read(&destination).unwrap(), b"original output");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);

        // Rename must also leave the old destination and clean up the new file.
        fs::remove_file(&destination).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("child"), b"original child").unwrap();
        assert!(write_atomic(&destination, b"replacement").is_err());
        assert_eq!(
            fs::read(destination.join("child")).unwrap(),
            b"original child"
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn atomic_publication_supports_long_destination_names() {
        let root = unique_temp_directory("atomic-long-destination");
        let destination = root.join("a".repeat(255));
        // Establish that the destination itself is valid on this filesystem.
        fs::write(&destination, b"original output").unwrap();
        write_atomic(&destination, b"replacement").unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"replacement");

        let source = root.join("source");
        fs::write(&source, b"copied output").unwrap();
        assert_eq!(copy_file_atomic(&source, &destination).unwrap(), 13);
        assert_eq!(fs::read(&destination).unwrap(), b"copied output");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn composable_digest_reuses_roots_across_irrelevant_exclusion_changes() {
        let root = unique_temp_directory("composable-digest-cache");
        let input = root.join("input");
        fs::create_dir_all(&input).expect("create composable input");
        fs::create_dir_all(root.join("generated-a")).expect("create first generated root");
        fs::create_dir_all(root.join("generated-b")).expect("create second generated root");
        fs::write(input.join("source"), b"source").expect("write composable input");
        let paths = [(PathBuf::from("shared"), input)];
        let mut cache = LabeledPathDigestCache::default();

        let first = digest_labeled_paths_composable(
            "composable-test-v1",
            paths
                .iter()
                .map(|(label, path)| (label.as_path(), path.as_path())),
            &[root.join("generated-a")],
            &mut cache,
        )
        .expect("hash first composable input");
        let second = digest_labeled_paths_composable(
            "composable-test-v1",
            paths
                .iter()
                .map(|(label, path)| (label.as_path(), path.as_path())),
            &[root.join("generated-b")],
            &mut cache,
        )
        .expect("reuse composable input root");

        assert_eq!(first, second);
        assert_eq!(cache.entries.len(), 1);
        fs::remove_dir_all(root).expect("remove composable digest fixture");
    }

    #[test]
    fn composable_digest_rehashes_changed_descendant_exclusions_and_rejects_ancestors() {
        let root = unique_temp_directory("composable-relevant-exclusions");
        let input = root.join("input");
        let generated = input.join("generated");
        fs::create_dir_all(&generated).unwrap();
        fs::write(input.join("source"), b"source").unwrap();
        fs::write(generated.join("artifact"), b"generated").unwrap();
        let paths = [(PathBuf::from("input"), input.clone())];
        let digest = |exclusions: &[PathBuf], cache: &mut LabeledPathDigestCache| {
            digest_labeled_paths_composable(
                "exclusions-test-v1",
                paths
                    .iter()
                    .map(|(label, path)| (label.as_path(), path.as_path())),
                exclusions,
                cache,
            )
        };
        let mut cache = LabeledPathDigestCache::default();
        let excluded = digest(std::slice::from_ref(&generated), &mut cache).unwrap();
        let included = digest(&[], &mut cache).unwrap();
        assert_ne!(included, excluded);
        assert_eq!(
            included,
            digest(&[], &mut LabeledPathDigestCache::default()).unwrap(),
        );
        for ancestor in [&input, &root] {
            assert_eq!(
                digest(std::slice::from_ref(ancestor), &mut cache)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::InvalidInput,
            );
        }
        assert_eq!(
            digest(std::slice::from_ref(&generated), &mut cache).unwrap(),
            excluded,
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn composable_digest_tracks_exclusions_beyond_an_external_symlink() {
        let root = unique_temp_directory("composable-external-exclusions");
        let input = root.join("input");
        let external = root.join("external");
        fs::create_dir_all(&input).unwrap();
        fs::create_dir_all(external.join("first")).unwrap();
        fs::create_dir_all(external.join("second")).unwrap();
        fs::write(input.join("source"), b"source").unwrap();
        fs::write(external.join("first/file"), b"first").unwrap();
        fs::write(external.join("second/file"), b"second").unwrap();
        std::os::unix::fs::symlink(&external, input.join("linked")).unwrap();
        let paths = [(PathBuf::from("input"), input)];
        let digest = |exclusion: &PathBuf, cache: &mut LabeledPathDigestCache| {
            digest_labeled_paths_composable(
                "external-exclusions-test-v1",
                paths
                    .iter()
                    .map(|(label, path)| (label.as_path(), path.as_path())),
                std::slice::from_ref(exclusion),
                cache,
            )
        };
        let mut cache = LabeledPathDigestCache::default();
        let first = digest(&external.join("first"), &mut cache).unwrap();
        let second = digest(&external.join("second"), &mut cache).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            second,
            digest(
                &external.join("second"),
                &mut LabeledPathDigestCache::default(),
            )
            .unwrap(),
        );
        assert_eq!(digest(&external.join("first"), &mut cache).unwrap(), first);
        fs::remove_dir_all(root).unwrap();
    }
}
