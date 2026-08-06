use fs2::FileExt as _;
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::digest::write_atomic;

const CACHE_DIRECTORY_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n\
# This file is a cache directory tag created by ic-testkit.\n\
# For information about cache directory tags see https://bford.info/cachedir/\n";
pub(super) const CACHE_DIRECTORY_TAG_SIGNATURE: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n";
pub(super) const LAST_USED_FILE: &str = ".ic-testkit-last-used";
const LAST_MAINTENANCE_FILE: &str = ".ic-testkit-last-maintenance";

/// Caller-selected retention limits for content-addressed artifact entries.
///
/// Age pruning runs before size pruning. A policy without either limit scans
/// the selected cache namespace and updates its cache metadata without
/// removing entries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCachePrunePolicy {
    max_age: Option<Duration>,
    max_size_bytes: Option<u64>,
}

/// Summary of one lock-coordinated artifact-cache pruning pass.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCachePruneReport {
    entries_scanned: usize,
    entries_removed: usize,
    bytes_before: u64,
    bytes_removed: u64,
    uncommitted_directories_removed: usize,
    uncommitted_bytes_removed: u64,
}

/// Nonfatal retention attempted as part of a successful cache acquisition.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactCacheMaintenance {
    /// Configured retention completed under the cache lock.
    Pruned(ArtifactCachePruneReport),
    /// Configured retention failed after the requested artifacts were ready.
    PruneFailed {
        /// Cache error rendered without invalidating the successful acquisition.
        message: String,
    },
}

impl ArtifactCachePrunePolicy {
    /// Create a policy that records cache metadata without removing entries.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_age: None,
            max_size_bytes: None,
        }
    }

    /// Remove entries older than `max_age` before applying the size limit.
    #[must_use]
    pub const fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Remove least-recently-used entries until retained logical size is at most `bytes`.
    #[must_use]
    pub const fn with_max_size_bytes(mut self, bytes: u64) -> Self {
        self.max_size_bytes = Some(bytes);
        self
    }

    /// Configured maximum entry age, if any.
    #[must_use]
    pub const fn max_age(self) -> Option<Duration> {
        self.max_age
    }

    /// Configured maximum logical cache size in bytes, if any.
    #[must_use]
    pub const fn max_size_bytes(self) -> Option<u64> {
        self.max_size_bytes
    }

    pub(super) fn maintenance_identity(self) -> String {
        format!(
            "age={:?};size={:?}",
            self.max_age.map(|duration| duration.as_nanos()),
            self.max_size_bytes
        )
    }
}

impl ArtifactCachePruneReport {
    /// Number of content-addressed directories considered for pruning.
    #[must_use]
    pub const fn entries_scanned(self) -> usize {
        self.entries_scanned
    }

    /// Number of content-addressed directories removed.
    #[must_use]
    pub const fn entries_removed(self) -> usize {
        self.entries_removed
    }

    /// Number of content-addressed directories retained.
    #[must_use]
    pub const fn entries_retained(self) -> usize {
        self.entries_scanned.saturating_sub(self.entries_removed)
    }

    /// Logical bytes occupied by scanned entries before pruning.
    #[must_use]
    pub const fn bytes_before(self) -> u64 {
        self.bytes_before
    }

    /// Logical bytes removed by pruning.
    #[must_use]
    pub const fn bytes_removed(self) -> u64 {
        self.bytes_removed
    }

    /// Logical bytes occupied by retained entries after pruning.
    #[must_use]
    pub const fn bytes_retained(self) -> u64 {
        self.bytes_before.saturating_sub(self.bytes_removed)
    }

    /// Abandoned transaction directories removed outside the committed-entry totals.
    #[must_use]
    pub const fn uncommitted_directories_removed(self) -> usize {
        self.uncommitted_directories_removed
    }

    /// Logical bytes removed from abandoned transaction directories.
    #[must_use]
    pub const fn uncommitted_bytes_removed(self) -> u64 {
        self.uncommitted_bytes_removed
    }

    pub(super) const fn record_uncommitted_removal(&mut self, bytes: u64) {
        self.uncommitted_directories_removed += 1;
        self.uncommitted_bytes_removed = self.uncommitted_bytes_removed.saturating_add(bytes);
    }
}

impl ArtifactCacheMaintenance {
    /// Successful pruning report, or `None` when maintenance failed.
    #[must_use]
    pub const fn prune_report(&self) -> Option<ArtifactCachePruneReport> {
        match self {
            Self::Pruned(report) => Some(*report),
            Self::PruneFailed { .. } => None,
        }
    }

    /// Rendered maintenance failure, or `None` when pruning succeeded.
    #[must_use]
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Pruned(_) => None,
            Self::PruneFailed { message } => Some(message),
        }
    }
}

#[derive(Debug)]
pub(super) struct CacheFsError {
    pub(super) operation: &'static str,
    pub(super) path: PathBuf,
    pub(super) source: io::Error,
}

pub(super) fn ensure_cache_directory_tag(cache_root: &Path) -> Result<(), CacheFsError> {
    let path = cache_root.join("CACHEDIR.TAG");
    if fs::read_to_string(&path)
        .is_ok_and(|contents| contents.starts_with(CACHE_DIRECTORY_TAG_SIGNATURE))
    {
        return Ok(());
    }
    write_atomic(&path, CACHE_DIRECTORY_TAG.as_bytes()).map_err(|source| CacheFsError {
        operation: "write cache directory tag",
        path,
        source,
    })
}

pub(super) fn lock_cache_file(path: &Path) -> Result<(File, Duration), CacheFsError> {
    let file = open_cache_lock_file(path)?;
    let started = Instant::now();
    file.lock_exclusive().map_err(|source| CacheFsError {
        operation: "lock cache",
        path: path.to_owned(),
        source,
    })?;
    Ok((file, started.elapsed()))
}

pub(super) fn try_lock_cache_file(path: &Path) -> Result<Option<File>, CacheFsError> {
    let file = open_cache_lock_file(path)?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(file)),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(source) => Err(CacheFsError {
            operation: "try lock cache",
            path: path.to_owned(),
            source,
        }),
    }
}

fn open_cache_lock_file(path: &Path) -> Result<File, CacheFsError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CacheFsError {
            operation: "create cache lock directory",
            path: parent.to_owned(),
            source,
        })?;
    }
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
        .map_err(|source| CacheFsError {
            operation: "open cache lock",
            path: path.to_owned(),
            source,
        })
}

pub(super) fn record_cache_entry_use(path: &Path) -> Result<(), CacheFsError> {
    write_last_used(path, SystemTime::now())
}

pub(super) fn cache_maintenance_due(
    path: &Path,
    minimum_interval: Option<Duration>,
    maintenance_identity: &str,
) -> Result<bool, CacheFsError> {
    let Some(minimum_interval) = minimum_interval else {
        return Ok(true);
    };
    let marker = path.join(LAST_MAINTENANCE_FILE);
    let contents = match fs::read_to_string(&marker) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
        Err(source) => {
            return Err(CacheFsError {
                operation: "read cache maintenance time",
                path: marker,
                source,
            });
        }
    };
    let mut lines = contents.lines();
    let Some(last_maintenance) = lines.next().and_then(decode_system_time) else {
        return Ok(true);
    };
    if lines.next() != Some(maintenance_identity) {
        return Ok(true);
    }
    Ok(match SystemTime::now().duration_since(last_maintenance) {
        Ok(elapsed) => elapsed >= minimum_interval,
        Err(_) => true,
    })
}

pub(super) fn record_cache_maintenance(
    path: &Path,
    maintenance_identity: &str,
) -> Result<(), CacheFsError> {
    fs::create_dir_all(path).map_err(|source| CacheFsError {
        operation: "create cache maintenance directory",
        path: path.to_owned(),
        source,
    })?;
    let marker = path.join(LAST_MAINTENANCE_FILE);
    let elapsed = encode_system_time(&marker, SystemTime::now())?;
    let contents = format!("{}\n{maintenance_identity}\n", elapsed.as_nanos());
    write_atomic(&marker, contents.as_bytes()).map_err(|source| CacheFsError {
        operation: "record cache maintenance time",
        path: marker,
        source,
    })
}

pub(super) fn perform_scheduled_cache_maintenance(
    path: &Path,
    minimum_interval: Option<Duration>,
    maintenance_identity: &str,
    maintenance: impl FnOnce() -> Result<ArtifactCachePruneReport, String>,
) -> (Option<ArtifactCacheMaintenance>, Option<Duration>) {
    let started = Instant::now();
    match cache_maintenance_due(path, minimum_interval, maintenance_identity) {
        Ok(false) => return (None, Some(started.elapsed())),
        Ok(true) => {}
        Err(error) => {
            return (
                Some(ArtifactCacheMaintenance::PruneFailed {
                    message: error.to_string(),
                }),
                Some(started.elapsed()),
            );
        }
    }

    let result = maintenance();
    let marker = record_cache_maintenance(path, maintenance_identity);
    let outcome = match (result, marker) {
        (Ok(report), Ok(())) => ArtifactCacheMaintenance::Pruned(report),
        (Err(message), Ok(())) => ArtifactCacheMaintenance::PruneFailed { message },
        (Ok(_), Err(error)) => ArtifactCacheMaintenance::PruneFailed {
            message: error.to_string(),
        },
        (Err(message), Err(marker)) => ArtifactCacheMaintenance::PruneFailed {
            message: format!(
                "{message}; additionally failed to record the maintenance attempt: {marker}"
            ),
        },
    };
    (Some(outcome), Some(started.elapsed()))
}

pub(super) fn write_last_used(path: &Path, last_used: SystemTime) -> Result<(), CacheFsError> {
    let marker = path.join(LAST_USED_FILE);
    write_system_time(&marker, last_used, "record cache use time")
}

fn write_system_time(
    path: &Path,
    timestamp: SystemTime,
    operation: &'static str,
) -> Result<(), CacheFsError> {
    let elapsed = encode_system_time(path, timestamp)?;
    write_atomic(path, elapsed.as_nanos().to_string().as_bytes()).map_err(|source| CacheFsError {
        operation,
        path: path.to_owned(),
        source,
    })
}

fn encode_system_time(path: &Path, timestamp: SystemTime) -> Result<Duration, CacheFsError> {
    timestamp
        .duration_since(UNIX_EPOCH)
        .map_err(|source| CacheFsError {
            operation: "encode cache time",
            path: path.to_owned(),
            source: io::Error::new(io::ErrorKind::InvalidInput, source),
        })
}

fn decode_system_time(contents: &str) -> Option<SystemTime> {
    let nanoseconds = contents.parse::<u128>().ok()?;
    let seconds = u64::try_from(nanoseconds / 1_000_000_000).ok()?;
    let subsecond_nanos = (nanoseconds % 1_000_000_000) as u32;
    UNIX_EPOCH.checked_add(Duration::new(seconds, subsecond_nanos))
}

pub(super) fn prune_direct_child_directories(
    cache_root: &Path,
    policy: ArtifactCachePrunePolicy,
    protected_entry: Option<&Path>,
    is_eligible: impl Fn(&Path) -> bool,
) -> Result<ArtifactCachePruneReport, CacheFsError> {
    let mut entries = cache_entries(cache_root, is_eligible)?;
    let bytes_before = entries
        .iter()
        .fold(0_u64, |total, entry| total.saturating_add(entry.bytes));
    let mut report = ArtifactCachePruneReport {
        entries_scanned: entries.len(),
        entries_removed: 0,
        bytes_before,
        bytes_removed: 0,
        uncommitted_directories_removed: 0,
        uncommitted_bytes_removed: 0,
    };
    let now = SystemTime::now();

    if let Some(max_age) = policy.max_age() {
        for entry in &mut entries {
            let age = now.duration_since(entry.last_used).unwrap_or_default();
            if protected_entry != Some(entry.path.as_path()) && age > max_age {
                remove_cache_entry(entry, &mut report)?;
            }
        }
    }

    if let Some(max_size_bytes) = policy.max_size_bytes() {
        entries.sort_by(|left, right| {
            left.last_used
                .cmp(&right.last_used)
                .then_with(|| left.path.cmp(&right.path))
        });
        for entry in &mut entries {
            if report.bytes_retained() <= max_size_bytes {
                break;
            }
            if protected_entry == Some(entry.path.as_path()) {
                continue;
            }
            remove_cache_entry(entry, &mut report)?;
        }
    }

    Ok(report)
}

pub(super) fn directory_logical_size(path: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    let mut pending = vec![path.to_owned()];
    while let Some(current) = pending.pop() {
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.is_dir() {
            for entry in fs::read_dir(&current)? {
                pending.push(entry?.path());
            }
        } else {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

pub(super) fn is_sha256_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let bytes = name.as_encoded_bytes();
        bytes.len() == 64 && bytes.iter().all(u8::is_ascii_hexdigit)
    })
}

pub(super) fn remove_path_if_present(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

struct CacheEntry {
    path: PathBuf,
    bytes: u64,
    last_used: SystemTime,
    removed: bool,
}

impl std::fmt::Display for CacheFsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to {} at {}: {}",
            self.operation,
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for CacheFsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn cache_entries(
    cache_root: &Path,
    is_eligible: impl Fn(&Path) -> bool,
) -> Result<Vec<CacheEntry>, CacheFsError> {
    let read_dir = match fs::read_dir(cache_root) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(CacheFsError {
                operation: "read cache directory",
                path: cache_root.to_owned(),
                source,
            });
        }
    };
    let mut entries = Vec::new();
    for directory_entry in read_dir {
        let directory_entry = directory_entry.map_err(|source| CacheFsError {
            operation: "read cache entry",
            path: cache_root.to_owned(),
            source,
        })?;
        let path = directory_entry.path();
        let file_type = directory_entry.file_type().map_err(|source| CacheFsError {
            operation: "inspect cache entry",
            path: path.clone(),
            source,
        })?;
        if !file_type.is_dir() || !is_eligible(&path) {
            continue;
        }
        let bytes = directory_logical_size(&path).map_err(|source| CacheFsError {
            operation: "measure cache entry",
            path: path.clone(),
            source,
        })?;
        let last_used = cache_entry_last_used(&path).map_err(|source| CacheFsError {
            operation: "read cache use time",
            path: path.clone(),
            source,
        })?;
        entries.push(CacheEntry {
            path,
            bytes,
            last_used,
            removed: false,
        });
    }
    Ok(entries)
}

pub(super) fn cache_entry_last_used(path: &Path) -> io::Result<SystemTime> {
    let marker = path.join(LAST_USED_FILE);
    if let Ok(contents) = fs::read_to_string(&marker)
        && let Some(timestamp) = decode_system_time(&contents)
    {
        return Ok(timestamp);
    }
    fs::metadata(path)?.modified()
}

fn remove_cache_entry(
    entry: &mut CacheEntry,
    report: &mut ArtifactCachePruneReport,
) -> Result<(), CacheFsError> {
    if entry.removed {
        return Ok(());
    }
    remove_path_if_present(&entry.path).map_err(|source| CacheFsError {
        operation: "prune cache entry",
        path: entry.path.clone(),
        source,
    })?;
    entry.removed = true;
    report.entries_removed += 1;
    report.bytes_removed = report.bytes_removed.saturating_add(entry.bytes);
    Ok(())
}
