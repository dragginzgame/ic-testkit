use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fmt::Write as _,
    fs::{self, OpenOptions},
    io::{self, Write as _},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
        for byte in self.0 {
            write!(hex, "{byte:02x}").expect("writing to a String cannot fail");
        }
        hex
    }
}

impl std::fmt::Display for InputDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_hex())
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
        self.0.update(
            u64::try_from(label.len())
                .expect("input label length must fit in u64")
                .to_le_bytes(),
        );
        self.0.update(label.as_bytes());
        self.0.update(
            u64::try_from(value.len())
                .expect("input value length must fit in u64")
                .to_le_bytes(),
        );
        self.0.update(value);
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

pub(super) fn digest_labeled_paths(
    domain: &str,
    paths: &[(PathBuf, PathBuf)],
    excluded_roots: &[PathBuf],
) -> io::Result<InputDigest> {
    let mut paths = paths.to_vec();
    paths.sort_by(|(left, _), (right, _)| {
        os_bytes(left.as_os_str()).cmp(&os_bytes(right.as_os_str()))
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
            &label,
            &path,
            &excluded_roots,
            &mut visited_directories,
        )?;
    }
    Ok(hasher.finish())
}

fn hash_path(
    hasher: &mut InputHasher,
    label: &Path,
    path: &Path,
    excluded_roots: &[PathBuf],
    visited_directories: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    let canonical = path.canonicalize()?;
    if excluded_roots
        .iter()
        .any(|excluded| canonical.starts_with(excluded))
    {
        return Ok(());
    }

    let metadata = fs::metadata(path)?;
    let label_bytes = os_bytes(label.as_os_str());
    if metadata.is_file() {
        hasher.field("file-path", &label_bytes);
        hasher.field("file-content", &fs::read(path)?);
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

    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| os_bytes(&entry.file_name()));
    for entry in entries {
        hash_path(
            hasher,
            &label.join(entry.file_name()),
            &entry.path(),
            excluded_roots,
            visited_directories,
        )?;
    }
    Ok(())
}

pub(super) fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
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
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut temp_name = file_name.to_os_string();
    temp_name.push(format!(".tmp-{}-{sequence}", std::process::id()));
    let temp_path = parent.join(temp_name);

    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        fs::rename(&temp_path, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[cfg(unix)]
pub(super) fn os_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    value.as_bytes().to_vec()
}

#[cfg(windows)]
pub(super) fn os_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt as _;
    value
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>()
}

#[cfg(not(any(unix, windows)))]
pub(super) fn os_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}
