use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
    time::{Duration, Instant, SystemTime},
};
use toml::Value as TomlValue;

use super::{
    cache_fs::{
        ArtifactCacheMaintenance, ArtifactCachePrunePolicy, ArtifactCachePruneReport, CacheFsError,
        cache_entry_last_used, directory_logical_size,
        ensure_cache_directory_tag as ensure_cache_tag, is_sha256_directory, lock_cache_file,
        perform_scheduled_cache_maintenance, prune_direct_child_directories,
        record_cache_entry_use as record_entry_use, remove_path_if_present,
    },
    digest::{
        InputDigest, InputHasher, copy_file_atomic, digest_bytes, digest_file,
        digest_labeled_paths, os_bytes, write_atomic,
    },
    wasm::wasm_path,
};

const CACHE_FORMAT_VERSION: &str = "ic-testkit-wasm-build-v1";
const DEFAULT_TARGET: &str = "wasm32-unknown-unknown";
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
    cache_mode: WasmBuildCacheMode,
    prune_policy: Option<WasmBuildCachePrunePolicy>,
    prune_interval: Option<Duration>,
}

/// Cargo-target ownership mode for one exact cached Wasm build.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildCacheMode {
    /// Build each exact fingerprint in its own content-addressed Cargo target.
    Isolated,
    /// Build misses in caller-owned shared Cargo incremental state, then cache final Wasm files.
    SharedIncremental {
        /// Mutable Cargo target directory shared across source fingerprints.
        target_dir: PathBuf,
    },
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
    maintenance: Option<WasmBuildCacheMaintenance>,
}

/// Timings for cache coordination, input resolution, and Cargo execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmBuildTimings {
    lock_wait: Duration,
    shared_incremental_lock_wait: Option<Duration>,
    input_resolution: WasmInputResolutionTimings,
    cargo_build: Option<Duration>,
    cache_maintenance: Option<Duration>,
    total: Duration,
}

/// Detailed timings for exact Wasm build-input resolution.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmInputResolutionTimings {
    tool_identity: Duration,
    cargo_metadata: Duration,
    input_discovery: Duration,
    content_hashing: Duration,
    total: Duration,
}

/// One exact local Cargo source or configuration input under a stable logical label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoBuildInput {
    label: PathBuf,
    path: PathBuf,
}

/// Resolved exact inputs and identity for one [`WasmBuildSpec`].
///
/// The snapshot can be resolved again after an external operation to detect
/// source, configuration, toolchain, argument, or environment changes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedCargoBuildInputs {
    fingerprint: InputDigest,
    input_digest: InputDigest,
    inputs: Vec<CargoBuildInput>,
    exclusions: Vec<PathBuf>,
    timings: WasmInputResolutionTimings,
}

/// Lock-coordinated disk-usage observation for a caller-owned shared Cargo target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetInspection {
    target_dir: PathBuf,
    logical_size_bytes: u64,
    last_used: SystemTime,
    lock_wait: Duration,
}

/// Wasm-cache compatibility name for generic artifact-cache retention limits.
pub type WasmBuildCachePrunePolicy = ArtifactCachePrunePolicy;

/// Wasm-cache compatibility name for a generic artifact-cache pruning report.
pub type WasmBuildCachePruneReport = ArtifactCachePruneReport;

/// Wasm-cache compatibility name for generic nonfatal cache maintenance.
pub type WasmBuildCacheMaintenance = ArtifactCacheMaintenance;

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
    /// A discovered Cargo configuration could not be interpreted exactly.
    InvalidCargoConfiguration { path: PathBuf, message: String },
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
            cache_mode: WasmBuildCacheMode::Isolated,
            prune_policy: None,
            prune_interval: None,
        }
    }

    /// Set Cargo profile and feature arguments used for both the build and fingerprint.
    #[must_use]
    pub fn with_cargo_profile_args(mut self, arguments: &[&str]) -> Self {
        self.cargo_profile_args = arguments.iter().map(OsString::from).collect();
        self
    }

    /// Set OS-native Cargo profile and feature arguments used for the build and fingerprint.
    #[must_use]
    pub fn with_cargo_profile_args_os<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.cargo_profile_args = arguments.into_iter().map(Into::into).collect();
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

    /// Set OS-native deterministic child-process environment overrides.
    #[must_use]
    pub fn with_extra_env_os<I, K, V>(mut self, environment: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        self.extra_env = environment
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
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

    /// Add OS-native ambient environment names whose current values affect the build.
    #[must_use]
    pub fn with_inherited_env_os<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.inherited_env.extend(names.into_iter().map(Into::into));
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

    /// Add path-native files or directories outside Cargo's local dependency graph.
    #[must_use]
    pub fn with_additional_input_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.additional_inputs
            .extend(paths.into_iter().map(Into::into));
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

    /// Build cache misses in one caller-owned shared Cargo incremental target.
    ///
    /// Exact final Wasm artifacts still live in the content-addressed cache.
    /// The shared target is coordinated across processes but is never pruned
    /// or removed by `ic-testkit` after a failed build.
    #[must_use]
    pub fn with_shared_incremental_target(mut self, target_dir: impl Into<PathBuf>) -> Self {
        self.cache_mode = WasmBuildCacheMode::SharedIncremental {
            target_dir: target_dir.into(),
        };
        self
    }

    /// Apply cache retention under the build operation's existing process lock.
    ///
    /// Maintenance is best-effort: its structured result is attached to the
    /// successful build record and cannot turn ready artifacts into a build
    /// failure. The active fingerprint is protected from this pruning pass.
    #[must_use]
    pub const fn with_prune_policy(mut self, policy: WasmBuildCachePrunePolicy) -> Self {
        self.prune_policy = Some(policy);
        self.prune_interval = None;
        self
    }

    /// Apply exact-entry retention at most once per `minimum_interval`.
    ///
    /// The active fingerprint remains protected. A zero interval is equivalent
    /// to [`Self::with_prune_policy`]. The interval covers attempted
    /// maintenance, including a nonfatal failed attempt. This schedule never
    /// owns or scans a caller-owned shared incremental Cargo target.
    #[must_use]
    pub const fn with_prune_policy_at_most_every(
        mut self,
        policy: WasmBuildCachePrunePolicy,
        minimum_interval: Duration,
    ) -> Self {
        self.prune_policy = Some(policy);
        self.prune_interval = Some(minimum_interval);
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

    /// Cargo-target ownership mode used for cache misses.
    #[must_use]
    pub const fn cache_mode(&self) -> &WasmBuildCacheMode {
        &self.cache_mode
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

    /// Cache maintenance attempted under the build lock, when configured.
    #[must_use]
    pub const fn maintenance(&self) -> Option<&WasmBuildCacheMaintenance> {
        self.maintenance.as_ref()
    }
}

impl WasmBuildTimings {
    /// Time spent waiting for the output-directory process lock.
    #[must_use]
    pub const fn lock_wait(self) -> Duration {
        self.lock_wait
    }

    /// Time spent waiting for a shared incremental-target lock, when configured.
    #[must_use]
    pub const fn shared_incremental_lock_wait(self) -> Option<Duration> {
        self.shared_incremental_lock_wait
    }

    /// Time spent resolving toolchain identity, Cargo metadata, and exact inputs.
    #[must_use]
    pub const fn input_resolution(self) -> Duration {
        self.input_resolution.total
    }

    /// Detailed tool, metadata, discovery, and hashing timings.
    #[must_use]
    pub const fn input_resolution_detail(self) -> WasmInputResolutionTimings {
        self.input_resolution
    }

    /// Time spent in `cargo build`, or `None` for a cache hit.
    #[must_use]
    pub const fn cargo_build(self) -> Option<Duration> {
        self.cargo_build
    }

    /// Time spent on configured best-effort cache maintenance.
    #[must_use]
    pub const fn cache_maintenance(self) -> Option<Duration> {
        self.cache_maintenance
    }

    /// Total operation duration, including lock coordination.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }
}

impl WasmInputResolutionTimings {
    /// Time spent reading Cargo and rustc identities.
    #[must_use]
    pub const fn tool_identity(self) -> Duration {
        self.tool_identity
    }

    /// Time spent running and decoding `cargo metadata`.
    #[must_use]
    pub const fn cargo_metadata(self) -> Duration {
        self.cargo_metadata
    }

    /// Time spent resolving packages, configuration, and watched paths.
    #[must_use]
    pub const fn input_discovery(self) -> Duration {
        self.input_discovery
    }

    /// Time spent reading and hashing exact input contents.
    #[must_use]
    pub const fn content_hashing(self) -> Duration {
        self.content_hashing
    }

    /// Complete input-resolution duration.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }

    const fn include(&mut self, other: Self) {
        self.tool_identity = self.tool_identity.saturating_add(other.tool_identity);
        self.cargo_metadata = self.cargo_metadata.saturating_add(other.cargo_metadata);
        self.input_discovery = self.input_discovery.saturating_add(other.input_discovery);
        self.content_hashing = self.content_hashing.saturating_add(other.content_hashing);
        self.total = self.total.saturating_add(other.total);
    }
}

impl CargoBuildInput {
    /// Stable checkout-independent label used while hashing this input.
    #[must_use]
    pub fn label(&self) -> &Path {
        &self.label
    }

    /// Resolved file or directory read by the Cargo build.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ResolvedCargoBuildInputs {
    /// Exact build fingerprint including Cargo inputs, tools, arguments, and environment.
    #[must_use]
    pub const fn fingerprint(&self) -> InputDigest {
        self.fingerprint
    }

    /// Exact digest of local source and configuration contents.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Stable logical labels and resolved local input paths.
    #[must_use]
    pub fn inputs(&self) -> &[CargoBuildInput] {
        &self.inputs
    }

    /// Generated-state roots excluded while recursively hashing local inputs.
    ///
    /// These exclusions are derived by `ic-testkit`; callers cannot add
    /// arbitrary exclusions through this snapshot.
    #[must_use]
    pub fn exclusions(&self) -> &[PathBuf] {
        &self.exclusions
    }

    /// Timings for tool identity, metadata, discovery, and content hashing.
    #[must_use]
    pub const fn timings(&self) -> WasmInputResolutionTimings {
        self.timings
    }

    /// Resolve `spec` again and report whether its exact identity is unchanged.
    pub fn is_current(&self, spec: &WasmBuildSpec) -> Result<bool, WasmBuildError> {
        resolve_cargo_build_inputs(spec).map(|current| current.fingerprint == self.fingerprint)
    }

    /// Rehash the already discovered Cargo source/configuration set.
    ///
    /// This is cheaper than rerunning Cargo metadata and is intended for
    /// before/after guards around external artifact transformations. Resolve a
    /// new snapshot to observe tool, argument, environment, or dependency-graph
    /// identity changes between separate acquisitions.
    pub fn is_content_current(&self) -> Result<bool, WasmBuildError> {
        self.current_input_digest()
            .map(|current| current == self.input_digest)
    }

    pub(super) fn current_input_digest(&self) -> Result<InputDigest, WasmBuildError> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| (input.label.clone(), input.path.clone()))
            .collect::<Vec<_>>();
        digest_labeled_paths("wasm-source-inputs-v1", &inputs, &self.exclusions).map_err(|source| {
            WasmBuildError::Io {
                operation: "rehash resolved Cargo build inputs",
                path: self
                    .inputs
                    .first()
                    .map_or_else(PathBuf::new, |input| input.path.clone()),
                source,
            }
        })
    }
}

impl SharedIncrementalTargetInspection {
    /// Canonical shared Cargo target directory that was inspected.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Logical bytes currently occupied by the complete shared target.
    #[must_use]
    pub const fn logical_size_bytes(&self) -> u64 {
        self.logical_size_bytes
    }

    /// Most recent build use recorded by `ic-testkit`, or the directory mtime for older targets.
    #[must_use]
    pub const fn last_used(&self) -> SystemTime {
        self.last_used
    }

    /// Time spent waiting for another process using the shared target.
    #[must_use]
    pub const fn lock_wait(&self) -> Duration {
        self.lock_wait
    }
}

impl std::fmt::Display for WasmBuildTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} lock={:?} shared_lock={:?} inputs={:?} cargo={:?} maintenance={:?}",
            self.total,
            self.lock_wait,
            self.shared_incremental_lock_wait,
            self.input_resolution.total,
            self.cargo_build,
            self.cache_maintenance,
        )
    }
}

impl std::fmt::Display for WasmBuildOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = if self.is_reused() { "reused" } else { "built" };
        write!(
            formatter,
            "{state} fingerprint={} artifacts={} {}",
            self.record().fingerprint,
            self.record().artifacts.len(),
            self.record().timings,
        )
    }
}

/// Resolve the exact Cargo source, configuration, toolchain, argument, and environment identity.
///
/// This performs the same resolution used before and after cached Wasm builds
/// without running `cargo build`.
pub fn resolve_cargo_build_inputs(
    spec: &WasmBuildSpec,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    validate_spec(spec)?;
    build_fingerprint(spec)
}

/// Inspect one configured shared Cargo target under its build coordination lock.
///
/// Returns `None` without creating anything when the caller-owned target does
/// not exist. This operation never removes Cargo state.
pub fn inspect_shared_incremental_target(
    spec: &WasmBuildSpec,
) -> Result<Option<SharedIncrementalTargetInspection>, WasmBuildError> {
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    match fs::symlink_metadata(&target_dir) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            return Err(WasmBuildError::InvalidSpec {
                message: format!(
                    "shared incremental Cargo target {} must be a directory",
                    target_dir.display()
                ),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(WasmBuildError::Io {
                operation: "inspect shared incremental Cargo target",
                path: target_dir,
                source,
            });
        }
    }

    let (_lock, lock_wait, canonical) = lock_shared_incremental_target(spec)?;
    let logical_size_bytes =
        directory_logical_size(&canonical).map_err(|source| WasmBuildError::Io {
            operation: "measure shared incremental Cargo target",
            path: canonical.clone(),
            source,
        })?;
    let last_used = cache_entry_last_used(&canonical).map_err(|source| WasmBuildError::Io {
        operation: "read shared incremental Cargo target use time",
        path: canonical.clone(),
        source,
    })?;
    Ok(Some(SharedIncrementalTargetInspection {
        target_dir: canonical,
        logical_size_bytes,
        last_used,
        lock_wait,
    }))
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
    let (cache_lock, first_lock_wait) = lock_wasm_build_cache(&spec.target_dir)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    let resolved = build_fingerprint(spec)?;
    if let Some(outcome) =
        try_reuse_wasm_artifacts(spec, &resolved, first_lock_wait, None, total_started)?
    {
        return Ok(outcome);
    }

    match &spec.cache_mode {
        WasmBuildCacheMode::Isolated => {
            let cache_entry = cache_entry_directory(spec, resolved.fingerprint);
            build_wasm_cache_miss(
                spec,
                resolved,
                first_lock_wait,
                None,
                cache_entry,
                total_started,
            )
        }
        WasmBuildCacheMode::SharedIncremental { .. } => {
            drop(cache_lock);
            let (shared_lock, shared_lock_wait, shared_target) =
                lock_shared_incremental_target(spec)?;
            let (_cache_lock, second_lock_wait) = lock_wasm_build_cache(&spec.target_dir)?;
            ensure_cache_directory_tag(&spec.target_dir)?;

            let mut current = build_fingerprint(spec)?;
            current.timings.include(resolved.timings);
            let lock_wait = first_lock_wait.saturating_add(second_lock_wait);
            if let Some(outcome) = try_reuse_wasm_artifacts(
                spec,
                &current,
                lock_wait,
                Some(shared_lock_wait),
                total_started,
            )? {
                return Ok(outcome);
            }

            let outcome = build_wasm_cache_miss(
                spec,
                current,
                lock_wait,
                Some(shared_lock_wait),
                shared_target,
                total_started,
            );
            drop(shared_lock);
            outcome
        }
    }
}

fn try_reuse_wasm_artifacts(
    spec: &WasmBuildSpec,
    resolved: &ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental_lock_wait: Option<Duration>,
    total_started: Instant,
) -> Result<Option<WasmBuildOutcome>, WasmBuildError> {
    let fingerprint = resolved.fingerprint;
    let artifacts = expected_artifacts(spec, &spec.target_dir);
    let cache_entry = cache_entry_directory(spec, fingerprint);
    if artifact_set_matches(&artifacts, fingerprint) {
        record_cache_entry_use_if_present(&cache_entry)?;
        return Ok(Some(WasmBuildOutcome::Reused(complete_build_record(
            spec,
            BuildRecordInput {
                fingerprint,
                input_digest: resolved.input_digest,
                artifacts,
                lock_wait,
                shared_incremental_lock_wait,
                input_resolution: resolved.timings,
                cargo_build: None,
                active_entry: &cache_entry,
            },
            total_started,
        ))));
    }

    let cached_artifacts = expected_artifacts(spec, &cache_entry);
    if !artifact_set_matches(&cached_artifacts, fingerprint) {
        return Ok(None);
    }
    materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
    record_cache_entry_use(&cache_entry)?;
    Ok(Some(WasmBuildOutcome::Reused(complete_build_record(
        spec,
        BuildRecordInput {
            fingerprint,
            input_digest: resolved.input_digest,
            artifacts,
            lock_wait,
            shared_incremental_lock_wait,
            input_resolution: resolved.timings,
            cargo_build: None,
            active_entry: &cache_entry,
        },
        total_started,
    ))))
}

fn build_wasm_cache_miss(
    spec: &WasmBuildSpec,
    resolved: ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental_lock_wait: Option<Duration>,
    cargo_target_dir: PathBuf,
    total_started: Instant,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let fingerprint = resolved.fingerprint;
    let mut input_resolution = resolved.timings;
    let artifacts = expected_artifacts(spec, &spec.target_dir);
    let cache_entry = cache_entry_directory(spec, fingerprint);
    remove_directory_if_present(&cache_entry)?;
    create_dir_all(
        &cache_entry,
        "create content-addressed Cargo target directory",
    )?;
    let incomplete_directory = IncompleteBuildDirectory::new(cache_entry.clone());
    let build_result = (|| {
        if matches!(
            spec.cache_mode,
            WasmBuildCacheMode::SharedIncremental { .. }
        ) {
            record_cache_entry_use(&cargo_target_dir)?;
        }
        let build_started = Instant::now();
        run_cargo_build(spec, &cargo_target_dir)?;
        let cargo_build = build_started.elapsed();
        let built_artifacts = expected_artifacts(spec, &cargo_target_dir);
        let missing = missing_artifacts(&built_artifacts);
        if !missing.is_empty() {
            return Err(WasmBuildError::MissingArtifacts { paths: missing });
        }

        let verified = build_fingerprint(spec)?;
        input_resolution.include(verified.timings);
        if fingerprint != verified.fingerprint {
            return Err(WasmBuildError::InputsChangedDuringBuild {
                before: fingerprint,
                after: verified.fingerprint,
            });
        }

        let cached_artifacts = expected_artifacts(spec, &cache_entry);
        if cargo_target_dir != cache_entry {
            copy_wasm_artifacts(&built_artifacts, &cached_artifacts)?;
        }
        publish_artifact_stamps(&cached_artifacts, fingerprint)?;
        materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
        record_cache_entry_use(&cache_entry)?;

        Ok(WasmBuildOutcome::Built(complete_build_record(
            spec,
            BuildRecordInput {
                fingerprint,
                input_digest: resolved.input_digest,
                artifacts,
                lock_wait,
                shared_incremental_lock_wait,
                input_resolution,
                cargo_build: Some(cargo_build),
                active_entry: &cache_entry,
            },
            total_started,
        )))
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

    prune_wasm_build_cache_locked(target_dir, policy, None)
}

struct BuildRecordInput<'a> {
    fingerprint: InputDigest,
    input_digest: InputDigest,
    artifacts: Vec<PathBuf>,
    lock_wait: Duration,
    shared_incremental_lock_wait: Option<Duration>,
    input_resolution: WasmInputResolutionTimings,
    cargo_build: Option<Duration>,
    active_entry: &'a Path,
}

fn complete_build_record(
    spec: &WasmBuildSpec,
    input: BuildRecordInput<'_>,
    total_started: Instant,
) -> WasmBuildRecord {
    let (maintenance, cache_maintenance) = spec.prune_policy.map_or((None, None), |policy| {
        let cache_root = spec.target_dir.join(".ic-testkit/wasm-targets");
        let identity = policy.maintenance_identity();
        perform_scheduled_cache_maintenance(&cache_root, spec.prune_interval, &identity, || {
            prune_wasm_build_cache_locked(&spec.target_dir, policy, Some(input.active_entry))
                .map_err(|error| error.to_string())
        })
    });
    WasmBuildRecord {
        fingerprint: input.fingerprint,
        input_digest: input.input_digest,
        artifacts: input.artifacts,
        timings: WasmBuildTimings {
            lock_wait: input.lock_wait,
            shared_incremental_lock_wait: input.shared_incremental_lock_wait,
            input_resolution: input.input_resolution,
            cargo_build: input.cargo_build,
            cache_maintenance,
            total: total_started.elapsed(),
        },
        maintenance,
    }
}

fn prune_wasm_build_cache_locked(
    target_dir: &Path,
    policy: WasmBuildCachePrunePolicy,
    protected_entry: Option<&Path>,
) -> Result<WasmBuildCachePruneReport, WasmBuildError> {
    let cache_root = target_dir.join(".ic-testkit/wasm-targets");
    prune_direct_child_directories(&cache_root, policy, protected_entry, is_sha256_directory)
        .map_err(wasm_cache_fs_error)
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
        let result = remove_path_if_present(&self.path);
        if result.is_ok() {
            self.armed = false;
        }
        result
    }
}

impl Drop for IncompleteBuildDirectory {
    fn drop(&mut self) {
        if self.armed {
            let _ = remove_path_if_present(&self.path);
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
    lock_cache_file(&lock_path).map_err(wasm_cache_fs_error)
}

fn lock_shared_incremental_target(
    spec: &WasmBuildSpec,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    create_dir_all(
        &target_dir,
        "create shared incremental Cargo target directory",
    )?;
    ensure_cache_tag(&target_dir).map_err(wasm_cache_fs_error)?;
    let canonical = target_dir
        .canonicalize()
        .map_err(|source| WasmBuildError::Io {
            operation: "resolve shared incremental Cargo target directory",
            path: target_dir.clone(),
            source,
        })?;
    let lock_path = canonical.join(".ic-testkit/wasm-incremental.lock");
    let (lock, wait) = lock_cache_file(&lock_path).map_err(wasm_cache_fs_error)?;
    Ok((lock, wait, canonical))
}

fn ensure_cache_directory_tag(target_dir: &Path) -> Result<(), WasmBuildError> {
    ensure_cache_tag(target_dir).map_err(wasm_cache_fs_error)
}

fn record_cache_entry_use_if_present(path: &Path) -> Result<(), WasmBuildError> {
    if path.is_dir() {
        record_cache_entry_use(path)?;
    }
    Ok(())
}

fn record_cache_entry_use(path: &Path) -> Result<(), WasmBuildError> {
    record_entry_use(path).map_err(wasm_cache_fs_error)
}

fn wasm_cache_fs_error(error: CacheFsError) -> WasmBuildError {
    WasmBuildError::Io {
        operation: error.operation,
        path: error.path,
        source: error.source,
    }
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
    if matches!(
        &spec.cache_mode,
        WasmBuildCacheMode::SharedIncremental { target_dir } if target_dir.as_os_str().is_empty()
    ) {
        return Err(WasmBuildError::InvalidSpec {
            message: "shared incremental Cargo target directory must not be empty".to_owned(),
        });
    }
    Ok(())
}

fn build_fingerprint(spec: &WasmBuildSpec) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let total_started = Instant::now();
    let tool_started = Instant::now();
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
    let tool_identity = tool_started.elapsed();

    let metadata_started = Instant::now();
    let metadata = cargo_metadata(spec)?;
    let cargo_metadata = metadata_started.elapsed();

    let discovery_started = Instant::now();
    let inputs = resolve_local_inputs(spec, &metadata)?;
    validate_shared_incremental_target_boundary(spec, &inputs)?;
    let exclusions = source_exclusions(spec, &inputs);
    let input_discovery = discovery_started.elapsed();

    let hashing_started = Instant::now();
    let input_digest = digest_labeled_paths("wasm-source-inputs-v1", &inputs, &exclusions)
        .map_err(|source| WasmBuildError::Io {
            operation: "hash Wasm build inputs",
            path: spec.workspace_root.clone(),
            source,
        })?;
    let content_hashing = hashing_started.elapsed();

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
    Ok(ResolvedCargoBuildInputs {
        fingerprint: hasher.finish(),
        input_digest,
        inputs: inputs
            .into_iter()
            .map(|(label, path)| CargoBuildInput { label, path })
            .collect(),
        exclusions,
        timings: WasmInputResolutionTimings {
            tool_identity,
            cargo_metadata,
            input_discovery,
            content_hashing,
            total: total_started.elapsed(),
        },
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
    let mut inputs = workspace_configuration_inputs(spec, &workspace_root)?;
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

fn workspace_configuration_inputs(
    spec: &WasmBuildSpec,
    workspace_root: &Path,
) -> Result<Vec<(PathBuf, PathBuf)>, WasmBuildError> {
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
        "workspace/rust-toolchain.toml",
        workspace_root.join("rust-toolchain.toml"),
    );
    add_if_present(
        &mut inputs,
        "workspace/rust-toolchain",
        workspace_root.join("rust-toolchain"),
    );
    append_cargo_configuration_inputs(&mut inputs, spec, workspace_root)?;
    Ok(inputs)
}

fn append_cargo_configuration_inputs(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    spec: &WasmBuildSpec,
    workspace_root: &Path,
) -> Result<(), WasmBuildError> {
    let invocation_root =
        spec.workspace_root
            .canonicalize()
            .map_err(|source| WasmBuildError::Io {
                operation: "resolve Cargo invocation directory",
                path: spec.workspace_root.clone(),
                source,
            })?;
    let canonical_workspace =
        workspace_root
            .canonicalize()
            .map_err(|source| WasmBuildError::Io {
                operation: "resolve Cargo workspace directory",
                path: workspace_root.to_owned(),
                source,
            })?;

    let mut roots = invocation_root
        .ancestors()
        .filter_map(|directory| effective_cargo_config(&directory.join(".cargo")))
        .collect::<Vec<_>>();
    if let Some(cargo_home) = effective_cargo_home(spec, &invocation_root)
        && let Some(config) = effective_cargo_config(&cargo_home)
    {
        roots.push(config);
    }

    let mut visited = BTreeSet::new();
    for config in roots {
        append_cargo_configuration_tree(
            inputs,
            &config,
            &canonical_workspace,
            &mut visited,
            false,
        )?;
    }
    Ok(())
}

fn effective_cargo_config(directory: &Path) -> Option<PathBuf> {
    let extensionless = directory.join("config");
    if extensionless.exists() {
        return Some(extensionless);
    }
    let toml = directory.join("config.toml");
    toml.exists().then_some(toml)
}

fn effective_cargo_home(spec: &WasmBuildSpec, invocation_root: &Path) -> Option<PathBuf> {
    if let Some(cargo_home) = command_environment_value(spec, "CARGO_HOME") {
        let cargo_home = PathBuf::from(cargo_home);
        return Some(if cargo_home.is_absolute() {
            cargo_home
        } else {
            invocation_root.join(cargo_home)
        });
    }

    default_home_directory(spec).map(|home| {
        let home = if home.is_absolute() {
            home
        } else {
            invocation_root.join(home)
        };
        home.join(".cargo")
    })
}

#[cfg(windows)]
fn default_home_directory(spec: &WasmBuildSpec) -> Option<PathBuf> {
    command_environment_value(spec, "USERPROFILE")
        .or_else(|| command_environment_value(spec, "HOME"))
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn default_home_directory(spec: &WasmBuildSpec) -> Option<PathBuf> {
    command_environment_value(spec, "HOME").map(PathBuf::from)
}

fn command_environment_value(spec: &WasmBuildSpec, name: &str) -> Option<OsString> {
    spec.extra_env
        .get(OsStr::new(name))
        .cloned()
        .or_else(|| std::env::var_os(name))
}

fn append_cargo_configuration_tree(
    inputs: &mut Vec<(PathBuf, PathBuf)>,
    config: &Path,
    workspace_root: &Path,
    visited: &mut BTreeSet<PathBuf>,
    optional: bool,
) -> Result<(), WasmBuildError> {
    let canonical = match config.canonicalize() {
        Ok(canonical) => canonical,
        Err(error) if optional && error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(WasmBuildError::Io {
                operation: "resolve Cargo configuration",
                path: config.to_owned(),
                source,
            });
        }
    };
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }

    let contents = fs::read_to_string(&canonical).map_err(|source| WasmBuildError::Io {
        operation: "read Cargo configuration",
        path: canonical.clone(),
        source,
    })?;
    let configuration = toml::from_str::<TomlValue>(&contents).map_err(|error| {
        WasmBuildError::InvalidCargoConfiguration {
            path: canonical.clone(),
            message: error.to_string(),
        }
    })?;
    inputs.push((
        cargo_configuration_label(&canonical, workspace_root),
        canonical.clone(),
    ));

    let Some(include) = configuration.get("include") else {
        return Ok(());
    };
    let parent = canonical
        .parent()
        .ok_or_else(|| WasmBuildError::InvalidCargoConfiguration {
            path: canonical.clone(),
            message: "configuration path has no parent directory".to_owned(),
        })?;
    for (included, optional) in cargo_configuration_includes(include, &canonical)? {
        let included = if included.is_absolute() {
            included
        } else {
            parent.join(included)
        };
        append_cargo_configuration_tree(inputs, &included, workspace_root, visited, optional)?;
    }
    Ok(())
}

fn cargo_configuration_includes(
    include: &TomlValue,
    config: &Path,
) -> Result<Vec<(PathBuf, bool)>, WasmBuildError> {
    let values = match include {
        TomlValue::Array(values) => values.as_slice(),
        value => std::slice::from_ref(value),
    };
    values
        .iter()
        .map(|value| match value {
            TomlValue::String(path) => Ok((PathBuf::from(path), false)),
            TomlValue::Table(table) => {
                let path = table
                    .get("path")
                    .and_then(TomlValue::as_str)
                    .ok_or_else(|| {
                        invalid_cargo_configuration(
                            config,
                            "Cargo configuration include table requires a string `path`",
                        )
                    })?;
                let optional = table
                    .get("optional")
                    .map(|value| {
                        value.as_bool().ok_or_else(|| {
                            invalid_cargo_configuration(
                                config,
                                "Cargo configuration include `optional` must be a boolean",
                            )
                        })
                    })
                    .transpose()?
                    .unwrap_or(false);
                Ok((PathBuf::from(path), optional))
            }
            _ => Err(invalid_cargo_configuration(
                config,
                "Cargo configuration `include` must contain paths or include tables",
            )),
        })
        .collect()
}

fn cargo_configuration_label(config: &Path, workspace_root: &Path) -> PathBuf {
    if let Ok(relative) = config.strip_prefix(workspace_root) {
        return PathBuf::from("cargo-config/workspace").join(relative);
    }
    let location = digest_bytes("cargo-config-location-v1", &os_bytes(config.as_os_str()));
    PathBuf::from("cargo-config/external").join(location.to_hex())
}

fn invalid_cargo_configuration(path: &Path, message: &str) -> WasmBuildError {
    WasmBuildError::InvalidCargoConfiguration {
        path: path.to_owned(),
        message: message.to_owned(),
    }
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
    if let Some(shared_target) = shared_incremental_target(spec) {
        exclusions.push(shared_target);
    }
    for (_, path) in inputs {
        if path.is_dir() {
            exclusions.push(path.join("target"));
            exclusions.push(path.join(".git"));
        }
    }
    exclusions
}

fn validate_shared_incremental_target_boundary(
    spec: &WasmBuildSpec,
    inputs: &[(PathBuf, PathBuf)],
) -> Result<(), WasmBuildError> {
    let Some(shared_target) = shared_incremental_target(spec) else {
        return Ok(());
    };
    let shared_target =
        canonicalize_allow_missing(&shared_target).map_err(|source| WasmBuildError::Io {
            operation: "resolve shared incremental Cargo target boundary",
            path: shared_target.clone(),
            source,
        })?;
    let safe_generated_roots = std::iter::once(spec.target_dir.clone())
        .chain(std::iter::once(spec.workspace_root.join("target")))
        .chain(
            inputs
                .iter()
                .filter(|(_, path)| path.is_dir())
                .map(|(_, path)| path.join("target")),
        )
        .filter_map(|path| canonicalize_allow_missing(&path).ok())
        .collect::<Vec<_>>();
    if safe_generated_roots
        .iter()
        .any(|root| shared_target.starts_with(root))
    {
        return Ok(());
    }

    for (_, input) in inputs {
        let input = input.canonicalize().map_err(|source| WasmBuildError::Io {
            operation: "resolve Cargo input boundary",
            path: input.clone(),
            source,
        })?;
        let metadata = fs::metadata(&input).map_err(|source| WasmBuildError::Io {
            operation: "inspect Cargo input boundary",
            path: input.clone(),
            source,
        })?;
        if shared_target == input || (metadata.is_dir() && shared_target.starts_with(&input)) {
            return Err(WasmBuildError::InvalidSpec {
                message: format!(
                    "shared incremental target {} must be outside exact Cargo inputs or inside a generated target directory",
                    shared_target.display()
                ),
            });
        }
    }
    Ok(())
}

fn canonicalize_allow_missing(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut unresolved = Vec::<OsString>::new();
    let mut existing = absolute.as_path();
    loop {
        match existing.canonicalize() {
            Ok(mut canonical) => {
                for component in unresolved.into_iter().rev() {
                    canonical.push(component);
                }
                return Ok(canonical);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(name) = existing.file_name() else {
                    return Err(error);
                };
                unresolved.push(name.to_owned());
                existing = existing.parent().ok_or(error)?;
            }
            Err(error) => return Err(error),
        }
    }
}

fn shared_incremental_target(spec: &WasmBuildSpec) -> Option<PathBuf> {
    let WasmBuildCacheMode::SharedIncremental { target_dir } = &spec.cache_mode else {
        return None;
    };
    Some(if target_dir.is_absolute() {
        target_dir.clone()
    } else {
        spec.workspace_root.join(target_dir)
    })
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

fn cache_entry_directory(spec: &WasmBuildSpec, fingerprint: InputDigest) -> PathBuf {
    spec.target_dir
        .join(".ic-testkit/wasm-targets")
        .join(fingerprint.to_hex())
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
    let (_, artifact_digest) = digest_file("wasm-artifact-v1", artifact)?;
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
        copy_file_atomic(cached, artifact).map_err(|source| WasmBuildError::Io {
            operation: "publish Wasm artifact",
            path: artifact.clone(),
            source,
        })?;
    }
    publish_artifact_stamps(artifacts, fingerprint)
}

fn copy_wasm_artifacts(
    source_artifacts: &[PathBuf],
    cached_artifacts: &[PathBuf],
) -> Result<(), WasmBuildError> {
    for (source, cached) in source_artifacts.iter().zip(cached_artifacts) {
        copy_file_atomic(source, cached).map_err(|source_error| WasmBuildError::Io {
            operation: "cache shared-incremental Wasm artifact",
            path: cached.clone(),
            source: source_error,
        })?;
    }
    Ok(())
}

fn remove_directory_if_present(path: &Path) -> Result<(), WasmBuildError> {
    remove_path_if_present(path).map_err(|source| WasmBuildError::Io {
        operation: "remove incomplete content-addressed Cargo target directory",
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
            Self::InvalidCargoConfiguration { path, message } => write!(
                formatter,
                "invalid Cargo configuration at {}: {message}",
                path.display(),
            ),
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
        IncompleteBuildDirectory, WasmBuildCachePrunePolicy, WasmBuildError, WasmBuildOutcome,
        WasmBuildSpec, append_cargo_configuration_inputs, ensure_cache_directory_tag,
        finish_fingerprint_build, inspect_shared_incremental_target, metadata_arguments,
        prune_wasm_build_cache, prune_wasm_build_cache_locked, resolve_cargo_build_inputs,
        validate_spec,
    };
    use crate::artifacts::cache_fs::{
        CACHE_DIRECTORY_TAG_SIGNATURE, directory_logical_size, write_last_used,
    };
    use crate::artifacts::test_support::unique_temp_directory;
    use std::{
        collections::BTreeSet,
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

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
    fn os_native_builders_preserve_dynamic_values() {
        let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &["fixture"], "debug")
            .with_cargo_profile_args_os([OsString::from("--locked")])
            .with_extra_env_os([(OsString::from("MODE"), OsString::from("exact"))])
            .with_inherited_env_os([OsString::from("RUSTFLAGS")])
            .with_additional_input_paths([PathBuf::from("schema")]);

        assert_eq!(spec.cargo_profile_args, [OsString::from("--locked")]);
        assert_eq!(
            spec.extra_env.get(&OsString::from("MODE")),
            Some(&OsString::from("exact"))
        );
        assert!(spec.inherited_env.contains(&OsString::from("RUSTFLAGS")));
        assert_eq!(spec.additional_inputs, [PathBuf::from("schema")]);
    }

    #[test]
    fn public_cargo_input_snapshot_detects_local_source_changes() {
        let root = unique_temp_directory("resolved-cargo-inputs");
        let package = root.join("fixture");
        fs::create_dir_all(package.join("src")).expect("create Cargo input fixture");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
        )
        .expect("write fixture workspace manifest");
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )
        .expect("write fixture package manifest");
        fs::write(package.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n")
            .expect("write fixture source");
        let spec = WasmBuildSpec::new(&root, &root.join("target"), &["fixture"], "debug");

        let snapshot = resolve_cargo_build_inputs(&spec).expect("resolve Cargo input snapshot");
        assert!(
            snapshot
                .is_current(&spec)
                .expect("revalidate unchanged inputs")
        );
        assert!(
            snapshot
                .inputs()
                .iter()
                .any(|input| input.path() == package)
        );
        assert!(
            snapshot
                .is_content_current()
                .expect("rehash unchanged resolved inputs")
        );

        fs::write(package.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n")
            .expect("change fixture source");
        assert!(!snapshot.is_current(&spec).expect("detect changed input"));
        assert!(
            !snapshot
                .is_content_current()
                .expect("rehash changed resolved inputs")
        );

        let unsafe_target =
            spec.with_shared_incremental_target(package.join("src/generated-target"));
        assert!(matches!(
            resolve_cargo_build_inputs(&unsafe_target),
            Err(WasmBuildError::InvalidSpec { .. })
        ));
        fs::remove_dir_all(root).expect("remove Cargo input fixture");
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
    fn shared_target_inspection_is_explicit_and_does_not_create_a_missing_target() {
        let root = unique_temp_directory("shared-target-inspection");
        let target = root.join("missing-shared-target");
        let isolated = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug");
        assert!(matches!(
            inspect_shared_incremental_target(&isolated),
            Err(WasmBuildError::InvalidSpec { .. })
        ));

        let shared = isolated.with_shared_incremental_target(&target);
        assert!(
            inspect_shared_incremental_target(&shared)
                .expect("inspect missing shared target")
                .is_none()
        );
        assert!(!target.exists());
        fs::remove_dir_all(root).expect("remove shared-target inspection fixture");
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

    #[test]
    fn in_build_pruning_protects_the_active_fingerprint() {
        let target_dir = unique_temp_directory("protected-pruning");
        let cache_root = target_dir.join(".ic-testkit/wasm-targets");
        let stale = create_cache_entry(&cache_root, 'a', 10, UNIX_EPOCH + Duration::from_secs(1));
        let active = create_cache_entry(&cache_root, 'b', 10, UNIX_EPOCH + Duration::from_secs(2));

        let report = prune_wasm_build_cache_locked(
            &target_dir,
            WasmBuildCachePrunePolicy::new()
                .with_max_age(Duration::ZERO)
                .with_max_size_bytes(0),
            Some(&active),
        )
        .expect("prune while protecting active cache entry");

        assert_eq!(report.entries_scanned(), 2);
        assert_eq!(report.entries_removed(), 1);
        assert!(!stale.exists());
        assert!(active.exists());
        assert!(report.bytes_retained() > 0);
        fs::remove_dir_all(target_dir).expect("remove protected-pruning test directory");
    }

    #[test]
    fn cargo_configuration_discovery_matches_cargo_search_and_include_rules() {
        let root = unique_temp_directory("cargo-configuration-discovery");
        let workspace = root.join("workspace");
        let workspace_cargo = workspace.join(".cargo");
        let ancestor_cargo = root.join(".cargo");
        let cargo_home = root.join("cargo-home");
        fs::create_dir_all(&workspace_cargo).expect("create workspace Cargo directory");
        fs::create_dir_all(&ancestor_cargo).expect("create ancestor Cargo directory");
        fs::create_dir_all(&cargo_home).expect("create Cargo home");

        fs::write(
            workspace_cargo.join("config"),
            "include = [\"included.toml\", { path = \"missing.toml\", optional = true }]\n",
        )
        .expect("write effective workspace Cargo config");
        fs::write(
            workspace_cargo.join("config.toml"),
            "[build]\ntarget-dir = \"ignored-by-cargo\"\n",
        )
        .expect("write shadowed workspace Cargo config");
        fs::write(
            workspace_cargo.join("included.toml"),
            "include = \"nested.toml\"\n",
        )
        .expect("write included Cargo config");
        fs::write(
            workspace_cargo.join("nested.toml"),
            "[build]\nincremental = false\n",
        )
        .expect("write nested Cargo config");
        fs::write(
            ancestor_cargo.join("config.toml"),
            "[net]\noffline = true\n",
        )
        .expect("write ancestor Cargo config");
        fs::write(cargo_home.join("config"), "[term]\nquiet = true\n")
            .expect("write Cargo-home config");

        let cargo_home_text = cargo_home.to_str().expect("temporary path is UTF-8");
        let spec = WasmBuildSpec::new(&workspace, &root.join("target"), &["fixture"], "debug")
            .with_extra_env(&[("CARGO_HOME", cargo_home_text)]);
        let mut inputs = Vec::new();
        append_cargo_configuration_inputs(&mut inputs, &spec, &workspace)
            .expect("discover effective Cargo configuration");
        let paths = inputs
            .into_iter()
            .map(|(_, path)| path)
            .collect::<BTreeSet<_>>();

        assert!(paths.contains(&workspace_cargo.join("config").canonicalize().unwrap()));
        assert!(
            paths.contains(
                &workspace_cargo
                    .join("included.toml")
                    .canonicalize()
                    .unwrap()
            )
        );
        assert!(paths.contains(&workspace_cargo.join("nested.toml").canonicalize().unwrap()));
        assert!(paths.contains(&ancestor_cargo.join("config.toml").canonicalize().unwrap()));
        assert!(paths.contains(&cargo_home.join("config").canonicalize().unwrap()));
        assert!(!paths.contains(&workspace_cargo.join("config.toml").canonicalize().unwrap()));
        assert_eq!(paths.len(), 5);
        fs::remove_dir_all(root).expect("remove Cargo-configuration test directory");
    }

    #[test]
    fn required_cargo_configuration_include_is_an_exact_input() {
        let root = unique_temp_directory("required-cargo-configuration-include");
        let workspace = root.join("workspace");
        let cargo_dir = workspace.join(".cargo");
        fs::create_dir_all(&cargo_dir).expect("create workspace Cargo directory");
        fs::write(
            cargo_dir.join("config.toml"),
            "include = \"missing.toml\"\n",
        )
        .expect("write Cargo config");
        let isolated_home = root.join("isolated-cargo-home");
        let isolated_home_text = isolated_home.to_str().expect("temporary path is UTF-8");
        let spec = WasmBuildSpec::new(&workspace, &root.join("target"), &["fixture"], "debug")
            .with_extra_env(&[("CARGO_HOME", isolated_home_text)]);

        let error = append_cargo_configuration_inputs(&mut Vec::new(), &spec, &workspace)
            .expect_err("required missing include must fail input discovery");

        assert!(matches!(error, WasmBuildError::Io { .. }));
        fs::remove_dir_all(root).expect("remove required-include test directory");
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
}
