use fs2::FileExt as _;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::{
    digest::{
        InputDigest, InputHasher, digest_bytes, digest_labeled_paths, os_bytes, write_atomic,
    },
    wasm::wasm_path,
};

const CACHE_FORMAT_VERSION: &str = "ic-testkit-wasm-build-v1";
const DEFAULT_TARGET: &str = "wasm32-unknown-unknown";
const CACHE_DIRECTORY_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n\
# This file is a cache directory tag created by ic-testkit.\n\
# For information about cache directory tags see https://bford.info/cachedir/\n";
const CACHE_DIRECTORY_TAG_SIGNATURE: &str = "Signature: 8a477f597d28d172789f06886806bc55\n";
const LAST_USED_FILE: &str = ".ic-testkit-last-used";
const AUTOMATIC_ENVIRONMENT: &[&str] = &[
    "CARGO_BUILD_RUSTC",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTFLAGS",
    "RUSTUP_TOOLCHAIN",
];

/// Complete caller-owned description of one cacheable Cargo Wasm build.
///
/// The package dependency closure, workspace manifest, lockfile, Cargo
/// configuration, Rust toolchain files, target, profile arguments, explicit
/// child environment, selected inherited environment, and additional watched
/// inputs all contribute to the build fingerprint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBuildSpec {
    workspace_root: PathBuf,
    target_dir: PathBuf,
    packages: Vec<String>,
    profile_target_dir: String,
    cargo_profile_args: Vec<OsString>,
    extra_env: BTreeMap<OsString, OsString>,
    inherited_env: BTreeSet<OsString>,
    additional_inputs: Vec<PathBuf>,
    target: String,
    cargo_program: OsString,
    rustc_program: OsString,
}

/// Whether a cacheable Wasm build ran Cargo or reused exact matching artifacts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildOutcome {
    /// Cargo ran and a new successful stamp was published.
    Built(WasmBuildRecord),
    /// Existing artifacts and their content-addressed stamp matched exactly.
    Reused(WasmBuildRecord),
}

/// Details shared by built and reused Wasm outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmBuildRecord {
    fingerprint: InputDigest,
    input_digest: InputDigest,
    artifacts: Vec<PathBuf>,
    timings: WasmBuildTimings,
}

/// Timings for cache coordination, input resolution, and Cargo execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmBuildTimings {
    lock_wait: Duration,
    input_resolution: Duration,
    cargo_build: Option<Duration>,
    total: Duration,
}

/// Caller-selected retention limits for content-addressed Cargo target directories.
///
/// Age pruning runs before size pruning. A policy without either limit scans
/// the cache and writes its cache-directory tag without removing entries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildCachePrunePolicy {
    max_age: Option<Duration>,
    max_size_bytes: Option<u64>,
}

/// Summary of one lock-coordinated Wasm build-cache pruning pass.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildCachePruneReport {
    entries_scanned: usize,
    entries_removed: usize,
    bytes_before: u64,
    bytes_removed: u64,
}

/// External phase associated with a cacheable Wasm build failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildPhase {
    /// Resolving Cargo's package graph.
    CargoMetadata,
    /// Reading the Cargo executable identity.
    CargoIdentity,
    /// Reading the Rust compiler identity.
    RustcIdentity,
    /// Compiling the selected Wasm packages.
    CargoBuild,
}

/// Structured failure from a cacheable Wasm build.
#[non_exhaustive]
#[derive(Debug)]
pub enum WasmBuildError {
    /// The caller supplied an incomplete or inconsistent specification.
    InvalidSpec { message: String },
    /// A filesystem operation failed.
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    /// An external command could not be launched.
    CommandSpawn {
        phase: WasmBuildPhase,
        program: OsString,
        source: io::Error,
    },
    /// An external command completed unsuccessfully.
    CommandFailed {
        phase: WasmBuildPhase,
        status: ExitStatus,
        stdout: String,
        stderr: String,
    },
    /// Cargo metadata did not contain the expected package graph.
    InvalidMetadata { message: String },
    /// Cargo succeeded without producing every declared Wasm artifact.
    MissingArtifacts { paths: Vec<PathBuf> },
    /// Declared inputs changed while Cargo was building.
    InputsChangedDuringBuild {
        before: InputDigest,
        after: InputDigest,
    },
    /// A build failed and its incomplete fingerprint directory could not be removed.
    FailedBuildCleanup {
        build_error: Box<Self>,
        path: PathBuf,
        source: io::Error,
    },
}

impl WasmBuildSpec {
    /// Describe one Cargo build targeting `wasm32-unknown-unknown`.
    ///
    /// `profile_target_dir` is Cargo's output subdirectory, such as `debug`,
    /// `release`, or the name supplied to `--profile`.
    #[must_use]
    pub fn new(
        workspace_root: &Path,
        target_dir: &Path,
        packages: &[&str],
        profile_target_dir: &str,
    ) -> Self {
        Self {
            workspace_root: workspace_root.to_owned(),
            target_dir: target_dir.to_owned(),
            packages: packages
                .iter()
                .map(|package| (*package).to_owned())
                .collect(),
            profile_target_dir: profile_target_dir.to_owned(),
            cargo_profile_args: Vec::new(),
            extra_env: BTreeMap::new(),
            inherited_env: BTreeSet::new(),
            additional_inputs: Vec::new(),
            target: DEFAULT_TARGET.to_owned(),
            cargo_program: std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()),
            rustc_program: std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()),
        }
    }

    /// Set Cargo profile and feature arguments used for both the build and fingerprint.
    #[must_use]
    pub fn with_cargo_profile_args(mut self, arguments: &[&str]) -> Self {
        self.cargo_profile_args = arguments.iter().map(OsString::from).collect();
        self
    }

    /// Set deterministic child-process environment overrides.
    #[must_use]
    pub fn with_extra_env(mut self, environment: &[(&str, &str)]) -> Self {
        self.extra_env = environment
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .collect();
        self
    }

    /// Add ambient environment variables whose current values affect the build.
    ///
    /// Common Rust and Cargo toolchain variables are included automatically.
    /// Callers must declare application-specific variables read by build scripts.
    #[must_use]
    pub fn with_inherited_env(mut self, names: &[&str]) -> Self {
        self.inherited_env.extend(names.iter().map(OsString::from));
        self
    }

    /// Add files or directories not discoverable through Cargo's local dependency graph.
    ///
    /// Relative paths are resolved from the workspace root. Use this for build
    /// script configuration, generated schemas, or other externally read inputs.
    #[must_use]
    pub fn with_additional_inputs(mut self, paths: &[&str]) -> Self {
        self.additional_inputs
            .extend(paths.iter().map(PathBuf::from));
        self
    }

    /// Override the Cargo compilation target.
    #[must_use]
    pub fn with_target(mut self, target: &str) -> Self {
        target.clone_into(&mut self.target);
        self
    }

    /// Override the Cargo executable used by metadata, identity, and build commands.
    #[must_use]
    pub fn with_cargo_program(mut self, program: impl Into<OsString>) -> Self {
        self.cargo_program = program.into();
        self
    }

    /// Override the Rust compiler executable used to fingerprint the toolchain.
    #[must_use]
    pub fn with_rustc_program(mut self, program: impl Into<OsString>) -> Self {
        self.rustc_program = program.into();
        self
    }

    /// Workspace containing the selected Cargo packages.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Cargo target directory containing artifacts, lock, and stamps.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Selected Cargo package names.
    #[must_use]
    pub fn packages(&self) -> &[String] {
        &self.packages
    }
}

impl WasmBuildOutcome {
    /// Read the common build record.
    #[must_use]
    pub const fn record(&self) -> &WasmBuildRecord {
        match self {
            Self::Built(record) | Self::Reused(record) => record,
        }
    }

    /// Report whether exact matching artifacts were reused.
    #[must_use]
    pub const fn is_reused(&self) -> bool {
        matches!(self, Self::Reused(_))
    }
}

impl WasmBuildRecord {
    /// Exact build fingerprint used by the atomic cache stamp.
    #[must_use]
    pub const fn fingerprint(&self) -> InputDigest {
        self.fingerprint
    }

    /// Exact digest of package sources, lockfile, and configuration inputs.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Expected Wasm artifacts produced or reused by the build.
    #[must_use]
    pub fn artifacts(&self) -> &[PathBuf] {
        &self.artifacts
    }

    /// Phase timings captured by the cacheable build operation.
    #[must_use]
    pub const fn timings(&self) -> WasmBuildTimings {
        self.timings
    }
}

impl WasmBuildTimings {
    /// Time spent waiting for the output-directory process lock.
    #[must_use]
    pub const fn lock_wait(self) -> Duration {
        self.lock_wait
    }

    /// Time spent resolving toolchain identity, Cargo metadata, and exact inputs.
    #[must_use]
    pub const fn input_resolution(self) -> Duration {
        self.input_resolution
    }

    /// Time spent in `cargo build`, or `None` for a cache hit.
    #[must_use]
    pub const fn cargo_build(self) -> Option<Duration> {
        self.cargo_build
    }

    /// Total operation duration, including lock coordination.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl WasmBuildCachePrunePolicy {
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
}

impl WasmBuildCachePruneReport {
    /// Number of fingerprint directories considered for pruning.
    #[must_use]
    pub const fn entries_scanned(self) -> usize {
        self.entries_scanned
    }

    /// Number of fingerprint directories removed.
    #[must_use]
    pub const fn entries_removed(self) -> usize {
        self.entries_removed
    }

    /// Number of fingerprint directories retained.
    #[must_use]
    pub const fn entries_retained(self) -> usize {
        self.entries_scanned - self.entries_removed
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
        self.bytes_before - self.bytes_removed
    }
}

/// Build or reuse one exact set of Cargo Wasm artifacts.
///
/// The operation takes an exclusive process lock scoped to `target_dir`, then
/// fingerprints all declared inputs. A cache hit requires both a matching
/// atomic stamp and every expected nonempty Wasm output. Failed or interrupted
/// builds never publish a successful stamp.
pub fn build_wasm_canisters_cached(
    spec: &WasmBuildSpec,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let total_started = Instant::now();
    validate_spec(spec)?;
    let (_lock_file, lock_wait) = lock_wasm_build_cache(&spec.target_dir)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    let input_started = Instant::now();
    let resolved = build_fingerprint(spec)?;
    let mut input_resolution = input_started.elapsed();
    let fingerprint = resolved.fingerprint;
    let artifacts = expected_artifacts(spec, &spec.target_dir);
    let build_target_dir = spec
        .target_dir
        .join(".ic-testkit/wasm-targets")
        .join(fingerprint.to_hex());

    if artifact_set_matches(&artifacts, fingerprint) {
        record_cache_entry_use_if_present(&build_target_dir)?;
        return Ok(WasmBuildOutcome::Reused(WasmBuildRecord {
            fingerprint,
            input_digest: resolved.input_digest,
            artifacts,
            timings: WasmBuildTimings {
                lock_wait,
                input_resolution,
                cargo_build: None,
                total: total_started.elapsed(),
            },
        }));
    }

    let cached_artifacts = expected_artifacts(spec, &build_target_dir);
    if artifact_set_matches(&cached_artifacts, fingerprint) {
        materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
        record_cache_entry_use(&build_target_dir)?;
        return Ok(WasmBuildOutcome::Reused(WasmBuildRecord {
            fingerprint,
            input_digest: resolved.input_digest,
            artifacts,
            timings: WasmBuildTimings {
                lock_wait,
                input_resolution,
                cargo_build: None,
                total: total_started.elapsed(),
            },
        }));
    }

    remove_directory_if_present(&build_target_dir)?;
    create_dir_all(
        &build_target_dir,
        "create content-addressed Cargo target directory",
    )?;
    let incomplete_directory = IncompleteBuildDirectory::new(build_target_dir.clone());
    let build_result = (|| {
        let build_started = Instant::now();
        run_cargo_build(spec, &build_target_dir)?;
        let cargo_build = build_started.elapsed();
        let missing = missing_artifacts(&cached_artifacts);
        if !missing.is_empty() {
            return Err(WasmBuildError::MissingArtifacts { paths: missing });
        }

        let verification_started = Instant::now();
        let verified = build_fingerprint(spec)?;
        input_resolution += verification_started.elapsed();
        if fingerprint != verified.fingerprint {
            return Err(WasmBuildError::InputsChangedDuringBuild {
                before: fingerprint,
                after: verified.fingerprint,
            });
        }

        publish_artifact_stamps(&cached_artifacts, fingerprint)?;
        materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
        record_cache_entry_use(&build_target_dir)?;

        Ok(WasmBuildOutcome::Built(WasmBuildRecord {
            fingerprint,
            input_digest: resolved.input_digest,
            artifacts,
            timings: WasmBuildTimings {
                lock_wait,
                input_resolution,
                cargo_build: Some(cargo_build),
                total: total_started.elapsed(),
            },
        }))
    })();
    finish_fingerprint_build(build_result, incomplete_directory)
}

/// Prune fingerprint-specific Cargo target directories under `target_dir`.
///
/// Pruning uses the same exclusive process lock as builds. Entries older than
/// the configured age are removed first, then least-recently-used entries are
/// removed until the configured logical byte limit is met. Only direct child
/// directories with SHA-256 fingerprint names are eligible; caller-facing
/// artifacts and unrelated target contents are never removed.
pub fn prune_wasm_build_cache(
    target_dir: &Path,
    policy: WasmBuildCachePrunePolicy,
) -> Result<WasmBuildCachePruneReport, WasmBuildError> {
    let (_lock_file, _) = lock_wasm_build_cache(target_dir)?;
    ensure_cache_directory_tag(target_dir)?;

    let cache_root = target_dir.join(".ic-testkit/wasm-targets");
    let mut entries = cache_entries(&cache_root)?;
    let bytes_before = entries
        .iter()
        .fold(0_u64, |total, entry| total.saturating_add(entry.bytes));
    let entries_scanned = entries.len();
    let now = SystemTime::now();
    let mut report = WasmBuildCachePruneReport {
        entries_scanned,
        entries_removed: 0,
        bytes_before,
        bytes_removed: 0,
    };

    if let Some(max_age) = policy.max_age {
        for entry in &mut entries {
            let age = now.duration_since(entry.last_used).unwrap_or_default();
            if age > max_age {
                remove_cache_entry(entry, &mut report)?;
            }
        }
    }

    if let Some(max_size_bytes) = policy.max_size_bytes {
        entries.sort_by(|left, right| {
            left.last_used
                .cmp(&right.last_used)
                .then_with(|| left.path.cmp(&right.path))
        });
        for entry in &mut entries {
            if report.bytes_retained() <= max_size_bytes {
                break;
            }
            remove_cache_entry(entry, &mut report)?;
        }
    }

    Ok(report)
}

struct CacheEntry {
    path: PathBuf,
    bytes: u64,
    last_used: SystemTime,
    removed: bool,
}

struct IncompleteBuildDirectory {
    path: PathBuf,
    armed: bool,
}

impl IncompleteBuildDirectory {
    const fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn preserve(mut self) {
        self.armed = false;
    }

    fn cleanup(mut self) -> io::Result<()> {
        let result = remove_dir_all_if_present(&self.path);
        if result.is_ok() {
            self.armed = false;
        }
        result
    }
}

impl Drop for IncompleteBuildDirectory {
    fn drop(&mut self) {
        if self.armed {
            let _ = remove_dir_all_if_present(&self.path);
        }
    }
}

fn finish_fingerprint_build(
    result: Result<WasmBuildOutcome, WasmBuildError>,
    incomplete_directory: IncompleteBuildDirectory,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    match result {
        Ok(outcome) => {
            incomplete_directory.preserve();
            Ok(outcome)
        }
        Err(build_error) => {
            let path = incomplete_directory.path.clone();
            match incomplete_directory.cleanup() {
                Ok(()) => Err(build_error),
                Err(source) => Err(WasmBuildError::FailedBuildCleanup {
                    build_error: Box::new(build_error),
                    path,
                    source,
                }),
            }
        }
    }
}

fn lock_wasm_build_cache(target_dir: &Path) -> Result<(File, Duration), WasmBuildError> {
    create_dir_all(target_dir, "create Cargo target directory")?;
    let lock_path = target_dir.join(".ic-testkit/wasm-build.lock");
    let lock_file = open_lock_file(&lock_path)?;
    let lock_started = Instant::now();
    lock_file
        .lock_exclusive()
        .map_err(|source| WasmBuildError::Io {
            operation: "lock Wasm build cache",
            path: lock_path,
            source,
        })?;
    Ok((lock_file, lock_started.elapsed()))
}

fn ensure_cache_directory_tag(target_dir: &Path) -> Result<(), WasmBuildError> {
    let path = target_dir.join("CACHEDIR.TAG");
    if fs::read_to_string(&path)
        .is_ok_and(|contents| contents.starts_with(CACHE_DIRECTORY_TAG_SIGNATURE))
    {
        return Ok(());
    }
    write_atomic(&path, CACHE_DIRECTORY_TAG.as_bytes()).map_err(|source| WasmBuildError::Io {
        operation: "write Cargo cache directory tag",
        path,
        source,
    })
}

fn record_cache_entry_use_if_present(path: &Path) -> Result<(), WasmBuildError> {
    if path.is_dir() {
        record_cache_entry_use(path)?;
    }
    Ok(())
}

fn record_cache_entry_use(path: &Path) -> Result<(), WasmBuildError> {
    write_last_used(path, SystemTime::now())
}

fn write_last_used(path: &Path, last_used: SystemTime) -> Result<(), WasmBuildError> {
    let elapsed = last_used
        .duration_since(UNIX_EPOCH)
        .map_err(|source| WasmBuildError::Io {
            operation: "record Wasm build cache use time",
            path: path.join(LAST_USED_FILE),
            source: io::Error::new(io::ErrorKind::InvalidInput, source),
        })?;
    let timestamp = elapsed.as_nanos().to_string();
    let marker = path.join(LAST_USED_FILE);
    write_atomic(&marker, timestamp.as_bytes()).map_err(|source| WasmBuildError::Io {
        operation: "record Wasm build cache use time",
        path: marker,
        source,
    })
}

fn cache_entries(cache_root: &Path) -> Result<Vec<CacheEntry>, WasmBuildError> {
    let read_dir = match fs::read_dir(cache_root) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(WasmBuildError::Io {
                operation: "read Wasm build cache directory",
                path: cache_root.to_owned(),
                source,
            });
        }
    };
    let mut entries = Vec::new();
    for directory_entry in read_dir {
        let directory_entry = directory_entry.map_err(|source| WasmBuildError::Io {
            operation: "read Wasm build cache entry",
            path: cache_root.to_owned(),
            source,
        })?;
        let path = directory_entry.path();
        let file_type = directory_entry
            .file_type()
            .map_err(|source| WasmBuildError::Io {
                operation: "inspect Wasm build cache entry",
                path: path.clone(),
                source,
            })?;
        if !file_type.is_dir() || !is_fingerprint_directory(&path) {
            continue;
        }
        let bytes = directory_logical_size(&path).map_err(|source| WasmBuildError::Io {
            operation: "measure Wasm build cache entry",
            path: path.clone(),
            source,
        })?;
        let last_used = cache_entry_last_used(&path).map_err(|source| WasmBuildError::Io {
            operation: "read Wasm build cache use time",
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

fn is_fingerprint_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let bytes = name.as_encoded_bytes();
        bytes.len() == 64 && bytes.iter().all(u8::is_ascii_hexdigit)
    })
}

fn directory_logical_size(path: &Path) -> io::Result<u64> {
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

fn cache_entry_last_used(path: &Path) -> io::Result<SystemTime> {
    let marker = path.join(LAST_USED_FILE);
    if let Ok(contents) = fs::read_to_string(&marker)
        && let Ok(nanoseconds) = contents.parse::<u128>()
    {
        let seconds = nanoseconds / 1_000_000_000;
        let subsecond_nanos = (nanoseconds % 1_000_000_000) as u32;
        if let Ok(seconds) = u64::try_from(seconds)
            && let Some(timestamp) = UNIX_EPOCH.checked_add(Duration::new(seconds, subsecond_nanos))
        {
            return Ok(timestamp);
        }
    }
    fs::metadata(path)?.modified()
}

fn remove_cache_entry(
    entry: &mut CacheEntry,
    report: &mut WasmBuildCachePruneReport,
) -> Result<(), WasmBuildError> {
    if entry.removed {
        return Ok(());
    }
    remove_dir_all_if_present(&entry.path).map_err(|source| WasmBuildError::Io {
        operation: "prune Wasm build cache entry",
        path: entry.path.clone(),
        source,
    })?;
    entry.removed = true;
    report.entries_removed += 1;
    report.bytes_removed = report.bytes_removed.saturating_add(entry.bytes);
    Ok(())
}

fn validate_spec(spec: &WasmBuildSpec) -> Result<(), WasmBuildError> {
    if spec.packages.is_empty() {
        return Err(WasmBuildError::InvalidSpec {
            message: "at least one Cargo package is required".to_owned(),
        });
    }
    if spec.profile_target_dir.is_empty() {
        return Err(WasmBuildError::InvalidSpec {
            message: "Cargo profile target directory must not be empty".to_owned(),
        });
    }
    if spec.target.is_empty() {
        return Err(WasmBuildError::InvalidSpec {
            message: "Cargo compilation target must not be empty".to_owned(),
        });
    }
    Ok(())
}

struct ResolvedFingerprint {
    fingerprint: InputDigest,
    input_digest: InputDigest,
}

fn build_fingerprint(spec: &WasmBuildSpec) -> Result<ResolvedFingerprint, WasmBuildError> {
    let cargo_identity = command_identity(
        spec,
        WasmBuildPhase::CargoIdentity,
        &spec.cargo_program,
        &["--version", "--verbose"],
    )?;
    let rustc_program = spec
        .extra_env
        .get(OsStr::new("RUSTC"))
        .unwrap_or(&spec.rustc_program);
    let rustc_identity =
        command_identity(spec, WasmBuildPhase::RustcIdentity, rustc_program, &["-vV"])?;
    let metadata = cargo_metadata(spec)?;
    let inputs = resolve_local_inputs(spec, &metadata)?;
    let exclusions = source_exclusions(spec, &inputs);
    let input_digest = digest_labeled_paths("wasm-source-inputs-v1", &inputs, &exclusions)
        .map_err(|source| WasmBuildError::Io {
            operation: "hash Wasm build inputs",
            path: spec.workspace_root.clone(),
            source,
        })?;

    let mut hasher = InputHasher::new(CACHE_FORMAT_VERSION);
    let mut packages = spec.packages.clone();
    packages.sort();
    packages.dedup();
    for package in packages {
        hasher.field("package", package.as_bytes());
    }
    hasher.field("target", spec.target.as_bytes());
    hasher.field("profile-target-dir", spec.profile_target_dir.as_bytes());
    for argument in &spec.cargo_profile_args {
        hasher.field("cargo-argument", &os_bytes(argument));
    }
    for (key, value) in effective_environment(spec) {
        hasher.field("environment-key", &os_bytes(&key));
        if let Some(value) = value {
            hasher.field("environment-value", &os_bytes(&value));
        } else {
            hasher.field("environment-unset", b"");
        }
    }
    hasher.field("cargo-identity", &cargo_identity);
    hasher.field("rustc-identity", &rustc_identity);
    hasher.field("source-input-digest", input_digest.as_bytes());
    Ok(ResolvedFingerprint {
        fingerprint: hasher.finish(),
        input_digest,
    })
}

fn command_identity(
    spec: &WasmBuildSpec,
    phase: WasmBuildPhase,
    program: &OsStr,
    arguments: &[&str],
) -> Result<Vec<u8>, WasmBuildError> {
    let mut command = Command::new(program);
    command.current_dir(&spec.workspace_root).args(arguments);
    apply_command_environment(&mut command, spec);
    let output = command
        .output()
        .map_err(|source| WasmBuildError::CommandSpawn {
            phase,
            program: program.to_owned(),
            source,
        })?;
    ensure_command_success(phase, output).map(|output| {
        let mut identity = output.stdout;
        identity.extend_from_slice(&output.stderr);
        identity
    })
}

fn cargo_metadata(spec: &WasmBuildSpec) -> Result<Value, WasmBuildError> {
    let mut command = Command::new(&spec.cargo_program);
    command
        .current_dir(&spec.workspace_root)
        .args(["metadata", "--format-version", "1"]);
    for argument in metadata_arguments(&spec.cargo_profile_args) {
        command.arg(argument);
    }
    apply_command_environment(&mut command, spec);
    let output = command
        .output()
        .map_err(|source| WasmBuildError::CommandSpawn {
            phase: WasmBuildPhase::CargoMetadata,
            program: spec.cargo_program.clone(),
            source,
        })?;
    let output = ensure_command_success(WasmBuildPhase::CargoMetadata, output)?;
    serde_json::from_slice(&output.stdout).map_err(|error| WasmBuildError::InvalidMetadata {
        message: format!("Cargo metadata was not valid JSON: {error}"),
    })
}

fn metadata_arguments(arguments: &[OsString]) -> Vec<OsString> {
    let mut selected = Vec::new();
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        let argument_text = argument.to_string_lossy();
        match argument_text.as_ref() {
            "--all-features" | "--no-default-features" | "--locked" | "--offline" | "--frozen" => {
                selected.push(argument.clone());
            }
            "--features" | "-F" | "--filter-platform" => {
                selected.push(argument.clone());
                if let Some(value) = arguments.next() {
                    selected.push(value.clone());
                }
            }
            _ if argument_text.starts_with("--features=")
                || argument_text.starts_with("--filter-platform=") =>
            {
                selected.push(argument.clone());
            }
            _ => {}
        }
    }
    selected
}

#[derive(Clone)]
struct MetadataPackage {
    id: String,
    name: String,
    version: String,
    manifest_path: PathBuf,
    is_local: bool,
}

fn resolve_local_inputs(
    spec: &WasmBuildSpec,
    metadata: &Value,
) -> Result<Vec<(PathBuf, PathBuf)>, WasmBuildError> {
    let packages = metadata_packages(metadata)?;
    let mut selected_ids = selected_package_ids(spec, metadata, &packages)?;
    let dependencies = metadata_dependencies(metadata)?;
    let mut closure = BTreeSet::new();
    while let Some(id) = selected_ids.pop_front() {
        if !closure.insert(id.clone()) {
            continue;
        }
        if let Some(deps) = dependencies.get(&id) {
            selected_ids.extend(deps.iter().cloned());
        }
    }

    let workspace_root = metadata
        .get("workspace_root")
        .and_then(Value::as_str)
        .map_or_else(|| spec.workspace_root.clone(), PathBuf::from);
    let mut inputs = workspace_configuration_inputs(&workspace_root);
    append_package_inputs(&mut inputs, &packages, closure, &workspace_root)?;
    append_additional_inputs(&mut inputs, spec, &workspace_root);
    Ok(inputs)
}

fn metadata_packages(metadata: &Value) -> Result<HashMap<String, MetadataPackage>, WasmBuildError> {
    let packages_value = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata has no package array"))?;
    let mut packages = HashMap::new();
    for value in packages_value {
        let package = MetadataPackage {
            id: required_string(value, "id")?,
            name: required_string(value, "name")?,
            version: required_string(value, "version")?,
            manifest_path: PathBuf::from(required_string(value, "manifest_path")?),
            is_local: value.get("source").is_some_and(Value::is_null),
        };
        packages.insert(package.id.clone(), package);
    }
    Ok(packages)
}

fn selected_package_ids(
    spec: &WasmBuildSpec,
    metadata: &Value,
    packages: &HashMap<String, MetadataPackage>,
) -> Result<VecDeque<String>, WasmBuildError> {
    let workspace_members = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata has no workspace member array"))?
        .iter()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let mut selected_ids = VecDeque::new();
    for requested in &spec.packages {
        let matches = packages
            .values()
            .filter(|package| {
                package.name == *requested && workspace_members.contains(package.id.as_str())
            })
            .map(|package| package.id.clone())
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [id] => selected_ids.push_back(id.clone()),
            [] => {
                return Err(WasmBuildError::InvalidSpec {
                    message: format!("Cargo workspace contains no package named `{requested}`"),
                });
            }
            _ => {
                return Err(WasmBuildError::InvalidSpec {
                    message: format!("Cargo workspace package name `{requested}` is ambiguous"),
                });
            }
        }
    }
    Ok(selected_ids)
}

fn metadata_dependencies(metadata: &Value) -> Result<HashMap<String, Vec<String>>, WasmBuildError> {
    let mut dependencies = HashMap::<String, Vec<String>>::new();
    let nodes = metadata
        .pointer("/resolve/nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata has no resolved dependency nodes"))?;
    for node in nodes {
        let id = required_string(node, "id")?;
        let deps = node
            .get("deps")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_metadata("Cargo metadata dependency node has no deps array"))?
            .iter()
            .map(|dependency| required_string(dependency, "pkg"))
            .collect::<Result<Vec<_>, _>>()?;
        dependencies.insert(id, deps);
    }
    Ok(dependencies)
}

fn workspace_configuration_inputs(workspace_root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut inputs = Vec::new();
    add_if_present(
        &mut inputs,
        "workspace/Cargo.toml",
        workspace_root.join("Cargo.toml"),
    );
    add_if_present(
        &mut inputs,
        "workspace/Cargo.lock",
        workspace_root.join("Cargo.lock"),
    );
    add_if_present(
        &mut inputs,
        "workspace/.cargo/config.toml",
        workspace_root.join(".cargo/config.toml"),
    );
    add_if_present(
        &mut inputs,
        "workspace/.cargo/config",
        workspace_root.join(".cargo/config"),
    );
    add_if_present(
        &mut inputs,
        "workspace/rust-toolchain.toml",
        workspace_root.join("rust-toolchain.toml"),
    );
    add_if_present(
        &mut inputs,
        "workspace/rust-toolchain",
        workspace_root.join("rust-toolchain"),
    );
    inputs
}

fn append_package_inputs(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    packages: &HashMap<String, MetadataPackage>,
    closure: BTreeSet<String>,
    workspace_root: &Path,
) -> Result<(), WasmBuildError> {
    for id in closure {
        let Some(package) = packages.get(&id) else {
            return Err(invalid_metadata(&format!(
                "resolved package `{id}` is missing"
            )));
        };
        if !package.is_local {
            continue;
        }
        let root = package.manifest_path.parent().ok_or_else(|| {
            invalid_metadata(&format!(
                "package `{}` manifest has no parent",
                package.name
            ))
        })?;
        let relative_manifest = package
            .manifest_path
            .strip_prefix(workspace_root)
            .unwrap_or(&package.manifest_path);
        let label = PathBuf::from(format!("package/{}@{}", package.name, package.version))
            .join(relative_manifest.parent().unwrap_or_else(|| Path::new(".")));
        inputs.push((label, root.to_owned()));
    }
    Ok(())
}

fn append_additional_inputs(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    spec: &WasmBuildSpec,
    workspace_root: &Path,
) {
    for additional in &spec.additional_inputs {
        let path = if additional.is_absolute() {
            additional.clone()
        } else {
            workspace_root.join(additional)
        };
        inputs.push((PathBuf::from("additional").join(additional), path));
    }
}

fn source_exclusions(spec: &WasmBuildSpec, inputs: &[(PathBuf, PathBuf)]) -> Vec<PathBuf> {
    let mut exclusions = vec![
        spec.target_dir.clone(),
        spec.workspace_root.join("target"),
        spec.workspace_root.join(".git"),
    ];
    for (_, path) in inputs {
        if path.is_dir() {
            exclusions.push(path.join("target"));
            exclusions.push(path.join(".git"));
        }
    }
    exclusions
}

fn effective_environment(spec: &WasmBuildSpec) -> BTreeMap<OsString, Option<OsString>> {
    let mut names = spec.inherited_env.clone();
    names.extend(AUTOMATIC_ENVIRONMENT.iter().map(OsString::from));
    let mut environment = names
        .into_iter()
        .map(|name| {
            let value = std::env::var_os(&name);
            (name, value)
        })
        .collect::<BTreeMap<_, _>>();
    for (key, value) in &spec.extra_env {
        environment.insert(key.clone(), Some(value.clone()));
    }
    environment
}

fn apply_command_environment(command: &mut Command, spec: &WasmBuildSpec) {
    for (key, value) in &spec.extra_env {
        command.env(key, value);
    }
}

fn run_cargo_build(spec: &WasmBuildSpec, build_target_dir: &Path) -> Result<(), WasmBuildError> {
    let mut command = Command::new(&spec.cargo_program);
    command
        .current_dir(&spec.workspace_root)
        .env("CARGO_TARGET_DIR", build_target_dir)
        .args(["build", "--target", &spec.target])
        .args(&spec.cargo_profile_args);
    apply_command_environment(&mut command, spec);
    for package in &spec.packages {
        command.args(["-p", package]);
    }

    let output = command
        .output()
        .map_err(|source| WasmBuildError::CommandSpawn {
            phase: WasmBuildPhase::CargoBuild,
            program: spec.cargo_program.clone(),
            source,
        })?;
    ensure_command_success(WasmBuildPhase::CargoBuild, output).map(|_| ())
}

fn ensure_command_success(phase: WasmBuildPhase, output: Output) -> Result<Output, WasmBuildError> {
    if output.status.success() {
        return Ok(output);
    }
    Err(WasmBuildError::CommandFailed {
        phase,
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn expected_artifacts(spec: &WasmBuildSpec, target_dir: &Path) -> Vec<PathBuf> {
    let mut packages = spec.packages.iter().map(String::as_str).collect::<Vec<_>>();
    packages.sort_unstable();
    packages.dedup();
    packages
        .into_iter()
        .map(|package| {
            if spec.target == DEFAULT_TARGET {
                wasm_path(target_dir, package, &spec.profile_target_dir)
            } else {
                target_dir
                    .join(&spec.target)
                    .join(&spec.profile_target_dir)
                    .join(format!("{package}.wasm"))
            }
        })
        .collect()
}

fn artifact_set_matches(artifacts: &[PathBuf], fingerprint: InputDigest) -> bool {
    artifacts.iter().all(|path| {
        fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
            && cache_stamp_matches(path, fingerprint)
    })
}

fn missing_artifacts(artifacts: &[PathBuf]) -> Vec<PathBuf> {
    artifacts
        .iter()
        .filter(|path| {
            fs::metadata(path).map_or(true, |metadata| !metadata.is_file() || metadata.len() == 0)
        })
        .cloned()
        .collect()
}

fn cache_stamp_matches(artifact: &Path, fingerprint: InputDigest) -> bool {
    let stamp_path = artifact_stamp_path(artifact);
    let Ok(expected) = artifact_stamp_contents(artifact, fingerprint) else {
        return false;
    };
    fs::read_to_string(stamp_path).is_ok_and(|stamp| stamp == expected)
}

fn artifact_stamp_path(artifact: &Path) -> PathBuf {
    let mut name = artifact
        .file_name()
        .map_or_else(|| OsString::from("artifact"), OsString::from);
    name.push(".ic-testkit-build");
    artifact.with_file_name(name)
}

fn artifact_stamp_contents(artifact: &Path, fingerprint: InputDigest) -> io::Result<String> {
    let artifact_digest = digest_bytes("wasm-artifact-v1", &fs::read(artifact)?);
    Ok(format!(
        "{CACHE_FORMAT_VERSION}\nbuild-sha256:{fingerprint}\nartifact-sha256:{artifact_digest}\n"
    ))
}

fn publish_artifact_stamps(
    artifacts: &[PathBuf],
    fingerprint: InputDigest,
) -> Result<(), WasmBuildError> {
    for artifact in artifacts {
        let stamp_path = artifact_stamp_path(artifact);
        let stamp = artifact_stamp_contents(artifact, fingerprint).map_err(|source| {
            WasmBuildError::Io {
                operation: "hash built Wasm artifact",
                path: artifact.clone(),
                source,
            }
        })?;
        write_atomic(&stamp_path, stamp.as_bytes()).map_err(|source| WasmBuildError::Io {
            operation: "publish Wasm build stamp",
            path: stamp_path,
            source,
        })?;
    }
    Ok(())
}

fn materialize_artifacts(
    cached_artifacts: &[PathBuf],
    artifacts: &[PathBuf],
    fingerprint: InputDigest,
) -> Result<(), WasmBuildError> {
    for (cached, artifact) in cached_artifacts.iter().zip(artifacts) {
        let contents = fs::read(cached).map_err(|source| WasmBuildError::Io {
            operation: "read content-addressed Wasm artifact",
            path: cached.clone(),
            source,
        })?;
        write_atomic(artifact, &contents).map_err(|source| WasmBuildError::Io {
            operation: "publish Wasm artifact",
            path: artifact.clone(),
            source,
        })?;
    }
    publish_artifact_stamps(artifacts, fingerprint)
}

fn remove_directory_if_present(path: &Path) -> Result<(), WasmBuildError> {
    remove_dir_all_if_present(path).map_err(|source| WasmBuildError::Io {
        operation: "remove incomplete content-addressed Cargo target directory",
        path: path.to_owned(),
        source,
    })
}

fn remove_dir_all_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn open_lock_file(path: &Path) -> Result<File, WasmBuildError> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent, "create Wasm build lock directory")?;
    }
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
        .map_err(|source| WasmBuildError::Io {
            operation: "open Wasm build lock",
            path: path.to_owned(),
            source,
        })
}

fn create_dir_all(path: &Path, operation: &'static str) -> Result<(), WasmBuildError> {
    fs::create_dir_all(path).map_err(|source| WasmBuildError::Io {
        operation,
        path: path.to_owned(),
        source,
    })
}

fn add_if_present(inputs: &mut Vec<(PathBuf, PathBuf)>, label: &str, path: PathBuf) {
    if path.exists() {
        inputs.push((PathBuf::from(label), path));
    }
}

fn required_string(value: &Value, field: &str) -> Result<String, WasmBuildError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid_metadata(&format!("Cargo metadata field `{field}` is missing")))
}

fn invalid_metadata(message: &str) -> WasmBuildError {
    WasmBuildError::InvalidMetadata {
        message: message.to_owned(),
    }
}

impl std::fmt::Display for WasmBuildPhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::CargoMetadata => "cargo metadata",
            Self::CargoIdentity => "Cargo identity",
            Self::RustcIdentity => "Rust compiler identity",
            Self::CargoBuild => "cargo build",
        })
    }
}

impl std::fmt::Display for WasmBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSpec { message } => {
                write!(formatter, "invalid Wasm build spec: {message}")
            }
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::CommandSpawn {
                phase,
                program,
                source,
            } => write!(
                formatter,
                "failed to launch {phase} using `{}`: {source}",
                program.to_string_lossy(),
            ),
            Self::CommandFailed {
                phase,
                status,
                stdout,
                stderr,
            } => write!(
                formatter,
                "{phase} failed with {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            ),
            Self::InvalidMetadata { message } => {
                write!(formatter, "invalid Cargo metadata: {message}")
            }
            Self::MissingArtifacts { paths } => write!(
                formatter,
                "cargo build succeeded without producing: {}",
                paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Self::InputsChangedDuringBuild { before, after } => write!(
                formatter,
                "Wasm build inputs changed while Cargo was running: {before} -> {after}",
            ),
            Self::FailedBuildCleanup {
                build_error,
                path,
                source,
            } => write!(
                formatter,
                "Wasm build failed ({build_error}) and its incomplete target directory at {} could not be removed: {source}",
                path.display(),
            ),
        }
    }
}

impl std::error::Error for WasmBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. }
            | Self::CommandSpawn { source, .. }
            | Self::FailedBuildCleanup { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CACHE_DIRECTORY_TAG_SIGNATURE, IncompleteBuildDirectory, WasmBuildCachePrunePolicy,
        WasmBuildError, WasmBuildOutcome, WasmBuildSpec, directory_logical_size,
        ensure_cache_directory_tag, finish_fingerprint_build, metadata_arguments,
        prune_wasm_build_cache, validate_spec, write_last_used,
    };
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn metadata_receives_only_resolution_arguments() {
        let arguments = [
            OsString::from("--profile"),
            OsString::from("fast"),
            OsString::from("--locked"),
            OsString::from("--features=alpha,beta"),
        ];
        assert_eq!(
            metadata_arguments(&arguments),
            [
                OsString::from("--locked"),
                OsString::from("--features=alpha,beta"),
            ]
        );
    }

    #[test]
    fn build_spec_requires_at_least_one_package() {
        let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &[], "debug");
        assert!(matches!(
            validate_spec(&spec),
            Err(WasmBuildError::InvalidSpec { .. })
        ));
    }

    #[test]
    fn cache_directory_tag_is_created_at_target_root() {
        let target_dir = unique_temp_directory("cache-directory-tag");
        fs::write(target_dir.join("CACHEDIR.TAG"), "not a cache tag")
            .expect("write invalid cache tag");

        ensure_cache_directory_tag(&target_dir).expect("write valid cache tag");

        let contents =
            fs::read_to_string(target_dir.join("CACHEDIR.TAG")).expect("read cache directory tag");
        assert!(contents.starts_with(CACHE_DIRECTORY_TAG_SIGNATURE));
        fs::remove_dir_all(target_dir).expect("remove tag test directory");
    }

    #[test]
    fn failed_build_removes_its_incomplete_fingerprint_directory() {
        let target_dir = unique_temp_directory("failed-build-cleanup");
        let fingerprint_dir = target_dir.join("a".repeat(64));
        fs::create_dir_all(&fingerprint_dir).expect("create incomplete target directory");
        fs::write(fingerprint_dir.join("partial-output"), b"partial")
            .expect("write incomplete output");
        let failure: Result<WasmBuildOutcome, WasmBuildError> = Err(WasmBuildError::InvalidSpec {
            message: "synthetic build failure".to_owned(),
        });

        let result = finish_fingerprint_build(
            failure,
            IncompleteBuildDirectory::new(fingerprint_dir.clone()),
        );

        assert!(matches!(result, Err(WasmBuildError::InvalidSpec { .. })));
        assert!(!fingerprint_dir.exists());
        fs::remove_dir_all(target_dir).expect("remove cleanup test directory");
    }

    #[test]
    fn age_pruning_removes_only_stale_fingerprint_directories() {
        let target_dir = unique_temp_directory("age-pruning");
        let cache_root = target_dir.join(".ic-testkit/wasm-targets");
        let old = create_cache_entry(&cache_root, 'a', 10, UNIX_EPOCH + Duration::from_secs(1));
        let current = create_cache_entry(&cache_root, 'b', 10, SystemTime::now());
        let unrelated = cache_root.join("not-a-fingerprint");
        fs::create_dir_all(&unrelated).expect("create unrelated directory");

        let report = prune_wasm_build_cache(
            &target_dir,
            WasmBuildCachePrunePolicy::new().with_max_age(Duration::from_secs(60)),
        )
        .expect("prune old cache entry");

        assert_eq!(report.entries_scanned(), 2);
        assert_eq!(report.entries_removed(), 1);
        assert_eq!(report.entries_retained(), 1);
        assert!(!old.exists());
        assert!(current.exists());
        assert!(unrelated.exists());
        assert!(target_dir.join("CACHEDIR.TAG").is_file());
        fs::remove_dir_all(target_dir).expect("remove age-pruning test directory");
    }

    #[test]
    fn size_pruning_removes_least_recently_used_entries_first() {
        let target_dir = unique_temp_directory("size-pruning");
        let cache_root = target_dir.join(".ic-testkit/wasm-targets");
        let oldest = create_cache_entry(&cache_root, 'a', 10, UNIX_EPOCH + Duration::from_secs(1));
        let middle = create_cache_entry(&cache_root, 'b', 10, UNIX_EPOCH + Duration::from_secs(2));
        let newest = create_cache_entry(&cache_root, 'c', 10, UNIX_EPOCH + Duration::from_secs(3));
        let newest_bytes = directory_logical_size(&newest).expect("measure newest entry");

        let report = prune_wasm_build_cache(
            &target_dir,
            WasmBuildCachePrunePolicy::new().with_max_size_bytes(newest_bytes),
        )
        .expect("prune cache to size");

        assert_eq!(report.entries_scanned(), 3);
        assert_eq!(report.entries_removed(), 2);
        assert_eq!(report.entries_retained(), 1);
        assert!(report.bytes_retained() <= newest_bytes);
        assert!(!oldest.exists());
        assert!(!middle.exists());
        assert!(newest.exists());
        fs::remove_dir_all(target_dir).expect("remove size-pruning test directory");
    }

    fn create_cache_entry(
        cache_root: &Path,
        fingerprint_digit: char,
        payload_bytes: usize,
        last_used: SystemTime,
    ) -> PathBuf {
        let path = cache_root.join(fingerprint_digit.to_string().repeat(64));
        fs::create_dir_all(&path).expect("create cache entry");
        fs::write(path.join("payload"), vec![0; payload_bytes]).expect("write cache payload");
        write_last_used(&path, last_used).expect("write cache use time");
        path
    }

    fn unique_temp_directory(label: &str) -> PathBuf {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ic-testkit-{label}-{}-{sequence}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale test directory");
        }
        fs::create_dir_all(&path).expect("create test directory");
        path
    }
}
