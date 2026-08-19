use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};
use toml::Value as TomlValue;

use crate::timing::saturating_add_optional_duration;

use super::{
    cache_fs::{
        ArtifactCacheMaintenance, ArtifactCachePrunePolicy, ArtifactCachePruneReport, CacheFsError,
        cache_entry_last_used, cache_maintenance_due, directory_logical_size,
        ensure_cache_directory_tag as ensure_cache_tag, is_sha256_directory, lock_cache_file,
        lock_cache_file_with_wait_observer, perform_scheduled_cache_maintenance,
        prune_direct_child_directories, record_cache_entry_use as record_entry_use,
        record_cache_maintenance, remove_path_if_present,
    },
    digest::{
        InputDigest, InputHasher, LabeledPathDigestCache, copy_file_atomic, digest_bytes,
        digest_file, digest_labeled_paths_composable, os_bytes, write_atomic,
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
/// The selected package graph, sources, semantic workspace projection, Cargo
/// configuration, Rust toolchain files, target, profile arguments, explicit
/// child environment, selected inherited environment, and additional watched
/// inputs contribute to the build fingerprint. The complete workspace
/// manifest and lockfile remain conservative mutation-validation inputs.
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
    prune_policy: Option<ArtifactCachePrunePolicy>,
    prune_interval: Option<Duration>,
    shared_incremental_maintenance_config: Option<SharedIncrementalTargetMaintenanceConfig>,
}

/// Failure handling for integrated shared incremental-target maintenance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SharedIncrementalTargetMaintenanceFailureMode {
    /// Fail the Wasm acquisition when scheduled maintenance fails.
    #[default]
    Strict,
    /// Preserve the acquisition and attach a structured failed-maintenance outcome.
    BestEffort,
}

/// Scheduled shared incremental-target maintenance attached to a Wasm acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetMaintenanceConfig {
    policy: SharedIncrementalTargetPrunePolicy,
    minimum_interval: Duration,
    failure_mode: SharedIncrementalTargetMaintenanceFailureMode,
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
    exact_cache_path: PathBuf,
    artifacts: Vec<PathBuf>,
    timings: WasmBuildTimings,
    maintenance: Option<ArtifactCacheMaintenance>,
    shared_incremental_maintenance: Option<SharedIncrementalTargetMaintenanceOutcome>,
}

/// Timings for cache coordination, input resolution, and Cargo execution.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

/// Primary phase in which one cacheable Wasm acquisition failed.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildFailurePhase {
    /// The caller supplied an invalid build specification.
    Specification,
    /// Waiting for the exact-cache lock or preparing its directory.
    ExactCacheCoordination,
    /// Reading Cargo or rustc identity.
    ToolIdentity,
    /// Running or decoding Cargo metadata.
    CargoMetadata,
    /// Discovering selected source and configuration inputs.
    InputDiscovery,
    /// Hashing selected and conservative input contents.
    ContentHashing,
    /// Waiting for or preparing a shared incremental target.
    SharedTargetCoordination,
    /// Applying configured shared-target maintenance.
    SharedTargetMaintenance,
    /// Executing Cargo for the selected Wasm packages.
    CargoBuild,
    /// Validating, copying, stamping, or materializing Wasm artifacts.
    ArtifactPublication,
    /// Applying exact-cache retention after a successful acquisition.
    ExactCacheMaintenance,
    /// Removing an incomplete exact-cache entry after failure.
    Cleanup,
}

/// Partial phase timings retained when a Wasm acquisition fails.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmBuildFailureTimings {
    exact_cache_coordination: Duration,
    shared_target_coordination: Option<Duration>,
    input_resolution: WasmInputResolutionTimings,
    shared_target_maintenance: Option<Duration>,
    cargo_build: Option<Duration>,
    artifact_publication: Option<Duration>,
    exact_cache_maintenance: Option<Duration>,
    cleanup: Option<Duration>,
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
    validation_digest: InputDigest,
    inputs: Vec<CargoBuildInput>,
    exclusions: Vec<PathBuf>,
    timings: WasmInputResolutionTimings,
}

pub(super) struct WasmBuildBatchInputResolver<'a, 'session> {
    specs: &'a [WasmBuildSpec],
    groups: Vec<BatchResolutionGroup>,
    group_by_index: Vec<usize>,
    resolved: Vec<Option<Result<ResolvedCargoBuildInputs, WasmBuildError>>>,
    session: Option<&'session mut WasmBuildSessionState>,
    snapshot: Option<&'session WasmBuildInputSnapshotState>,
    metrics: WasmBuildBatchInputMetrics,
}

pub(super) struct WasmBuildSessionState {
    snapshots: Vec<(WasmBuildSpec, ResolvedCargoBuildInputs)>,
    digest_cache: LabeledPathDigestCache,
    snapshot_reuses: usize,
    invalidated: bool,
}

pub(super) struct WasmBuildInputSnapshotState {
    snapshots: Vec<(WasmBuildSpec, ResolvedCargoBuildInputs)>,
    preparation_metrics: WasmBuildBatchInputMetrics,
    preparation_timings: WasmInputResolutionTimings,
    reader_reuses: AtomicUsize,
    invalidation: Arc<RwLock<bool>>,
}

pub(super) struct WasmBuildBatchAttempt {
    pub(super) result: Result<WasmBuildOutcome, WasmBuildError>,
    pub(super) failure_phase: Option<WasmBuildFailurePhase>,
    pub(super) failure_timings: Option<WasmBuildFailureTimings>,
}

impl WasmBuildBatchAttempt {
    pub(super) fn invalid_spec(error: WasmBuildError, total: Duration) -> Self {
        Self {
            result: Err(error),
            failure_phase: Some(WasmBuildFailurePhase::Specification),
            failure_timings: Some(WasmBuildFailureTimings {
                total,
                ..WasmBuildFailureTimings::default()
            }),
        }
    }
}

struct BatchResolutionGroup {
    indexes: Vec<usize>,
}

struct ResolvedLocalInputs {
    validation_inputs: Vec<(PathBuf, PathBuf)>,
    fingerprint: LocalInputFingerprint,
}

enum LocalInputFingerprint {
    Conservative,
    Projected {
        inputs: Vec<(PathBuf, PathBuf)>,
        workspace: InputDigest,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct WasmBuildBatchInputMetrics {
    pub(super) runs: usize,
    pub(super) reuses: usize,
    pub(super) session_reuses: usize,
    pub(super) prepared_reuses: usize,
}

#[derive(Eq, PartialEq)]
struct BatchResolutionKey {
    workspace_root: PathBuf,
    cargo_program: OsString,
    rustc_program: OsString,
    metadata_arguments: Vec<OsString>,
    environment: BTreeMap<OsString, Option<OsString>>,
}

/// Lock-coordinated disk-usage observation for a caller-owned shared Cargo target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetInspection {
    target_dir: PathBuf,
    logical_size_bytes: u64,
    last_used: SystemTime,
    lock_wait: Duration,
}

/// Whole-target retention limits for caller-owned shared Cargo state.
///
/// Unlike immutable fingerprint entries, a shared Cargo target has no safe
/// per-entry LRU boundary. When either configured limit is exceeded,
/// maintenance clears every other target child while preserving
/// `ic-testkit`'s coordination metadata and the target root. Callers must not
/// colocate unrelated data that needs to survive a clear.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SharedIncrementalTargetPrunePolicy {
    max_age: Option<Duration>,
    max_size_bytes: Option<u64>,
}

/// Result of explicit shared Cargo target maintenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedIncrementalTargetMaintenance {
    target_dir: PathBuf,
    logical_size_bytes_before: u64,
    logical_size_bytes_after: u64,
    last_used_before: SystemTime,
    cleared: bool,
    lock_wait: Duration,
    maintenance: Duration,
}

/// Result of interval-limited shared Cargo target maintenance.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedIncrementalTargetMaintenanceOutcome {
    /// The configured shared target does not exist, so nothing was created or inspected.
    Missing {
        /// Configured target path. A missing path cannot necessarily be canonicalized.
        target_dir: PathBuf,
    },
    /// A successful matching maintenance pass is still inside the requested interval.
    Skipped {
        /// Canonical shared Cargo target directory.
        target_dir: PathBuf,
        /// Time spent waiting for another process using the shared target.
        lock_wait: Duration,
        /// Time spent checking the small cross-process schedule marker.
        schedule_check: Duration,
    },
    /// Retention was evaluated under the shared-target lock.
    Performed {
        /// Completed retention report.
        maintenance: SharedIncrementalTargetMaintenance,
        /// Time spent checking the small cross-process schedule marker.
        schedule_check: Duration,
    },
    /// Integrated best-effort maintenance failed without invalidating the Wasm acquisition.
    Failed {
        /// Canonical shared Cargo target directory.
        target_dir: PathBuf,
        /// Time spent waiting for another process using the shared target.
        lock_wait: Duration,
        /// Rendered maintenance failure retained for diagnostics.
        message: String,
    },
}

/// Observation settings for one cacheable Wasm build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasmBuildProgressConfig {
    heartbeat_interval: Option<Duration>,
    emit_cargo_output: bool,
}

/// Raw child-process stream attached to a Cargo progress event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildOutputStream {
    /// Cargo standard output.
    Stdout,
    /// Cargo standard error.
    Stderr,
}

/// Final cache state reported by a successful observed build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildProgressOutcome {
    /// Cargo ran and exact artifacts were published.
    Built,
    /// Exact artifacts were reused without Cargo.
    Reused,
}

/// Potentially long phase of one observed Wasm-cache acquisition.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBuildProgressPhase {
    /// Waiting for exclusive ownership of the exact artifact cache.
    ExactCacheLock,
    /// Reading the Cargo executable identity.
    CargoIdentity,
    /// Reading the Rust compiler identity.
    RustcIdentity,
    /// Resolving Cargo's package graph.
    CargoMetadata,
    /// Discovering local source and configuration inputs.
    InputDiscovery,
    /// Hashing exact source and configuration contents.
    ContentHashing,
    /// Waiting for exclusive ownership of a shared incremental Cargo target.
    SharedTargetLock,
    /// Inspecting or clearing a shared incremental Cargo target.
    SharedTargetMaintenance,
    /// Compiling the selected Wasm packages.
    CargoBuild,
    /// Validating, copying, hashing, or stamping exact artifacts.
    ArtifactPublication,
    /// Applying retention to immutable exact-cache entries.
    ExactCacheMaintenance,
}

/// Structured progress emitted by an observed cacheable Wasm build.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmBuildProgressEvent {
    /// One build/cache acquisition started.
    Started,
    /// One exact Cargo input-resolution pass completed.
    InputsResolved {
        /// Complete exact build fingerprint.
        fingerprint: InputDigest,
        /// Semantic selected-source/configuration digest.
        input_digest: InputDigest,
        /// Time spent on this resolution pass.
        elapsed: Duration,
    },
    /// No reusable exact entry existed for this fingerprint.
    CacheMiss {
        /// Missing exact fingerprint.
        fingerprint: InputDigest,
    },
    /// Exact artifacts were found and materialized when necessary.
    CacheHit {
        /// Reused exact fingerprint.
        fingerprint: InputDigest,
    },
    /// The build is about to wait for a caller-owned shared Cargo target.
    SharedTargetLockStarted {
        /// Shared target selected by the build specification.
        target_dir: PathBuf,
    },
    /// Exclusive shared-target ownership was acquired.
    SharedTargetLockAcquired {
        /// Canonical shared target directory.
        target_dir: PathBuf,
        /// Time spent waiting for another process.
        wait: Duration,
    },
    /// Scheduled shared-target retention is about to be evaluated under lock.
    SharedTargetMaintenanceStarted {
        /// Canonical shared target selected by the build specification.
        target_dir: PathBuf,
    },
    /// Scheduled shared-target retention completed or was skipped.
    SharedTargetMaintenanceFinished {
        /// Structured retention result attached to the successful acquisition.
        outcome: SharedIncrementalTargetMaintenanceOutcome,
    },
    /// Cargo compilation started.
    CargoStarted {
        /// Cargo target receiving compilation state.
        target_dir: PathBuf,
    },
    /// One raw Cargo output chunk was read without lossy UTF-8 conversion.
    CargoOutput {
        /// Child-process stream that produced the bytes.
        stream: WasmBuildOutputStream,
        /// Raw output bytes in per-stream read order.
        bytes: Vec<u8>,
    },
    /// The current acquisition phase remained active without another event.
    Heartbeat {
        /// Phase that is still making or waiting for progress.
        phase: WasmBuildProgressPhase,
        /// Time elapsed since this phase started.
        elapsed: Duration,
    },
    /// Cargo exited and all captured output was drained.
    CargoFinished {
        /// Whether Cargo reported success.
        success: bool,
        /// Portable exit code when the platform exposes one.
        code: Option<i32>,
        /// Complete Cargo execution duration.
        elapsed: Duration,
    },
    /// The complete cacheable build operation succeeded.
    Finished {
        /// Whether Cargo ran or an exact entry was reused.
        outcome: WasmBuildProgressOutcome,
        /// Exact fingerprint selected by the operation.
        fingerprint: InputDigest,
        /// Total operation duration.
        elapsed: Duration,
    },
}

impl Default for WasmBuildProgressConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Some(Duration::from_secs(10)),
            emit_cargo_output: true,
        }
    }
}

impl WasmBuildProgressConfig {
    /// Observe acquisition progress and emit a heartbeat at least every ten quiet seconds.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Select the maximum quiet interval between phase-aware heartbeat events.
    ///
    /// A zero interval is rejected before any build work begins.
    #[must_use]
    pub const fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    /// Disable time-based heartbeats while retaining phase and output events.
    #[must_use]
    pub const fn without_heartbeats(mut self) -> Self {
        self.heartbeat_interval = None;
        self
    }

    /// Select whether raw Cargo stdout/stderr chunks are forwarded.
    ///
    /// Output is always captured for structured build failures.
    #[must_use]
    pub const fn with_cargo_output(mut self, emit: bool) -> Self {
        self.emit_cargo_output = emit;
        self
    }

    /// Configured heartbeat interval, or `None` when disabled.
    #[must_use]
    pub const fn heartbeat_interval(self) -> Option<Duration> {
        self.heartbeat_interval
    }

    /// Whether raw Cargo output chunks are emitted to the observer.
    #[must_use]
    pub const fn emits_cargo_output(self) -> bool {
        self.emit_cargo_output
    }
}

struct ProgressReporter<'a> {
    config: WasmBuildProgressConfig,
    observer: Option<&'a mut dyn FnMut(WasmBuildProgressEvent)>,
    last_event: Instant,
    failure_phase: Option<WasmBuildFailurePhase>,
    failure_timings: WasmBuildFailureTimings,
}

impl ProgressReporter<'_> {
    fn silent() -> Self {
        Self {
            config: WasmBuildProgressConfig {
                heartbeat_interval: None,
                emit_cargo_output: false,
            },
            observer: None,
            last_event: Instant::now(),
            failure_phase: None,
            failure_timings: WasmBuildFailureTimings::default(),
        }
    }

    fn observed(
        config: WasmBuildProgressConfig,
        observer: &'_ mut dyn FnMut(WasmBuildProgressEvent),
    ) -> ProgressReporter<'_> {
        ProgressReporter {
            config,
            observer: Some(observer),
            last_event: Instant::now(),
            failure_phase: None,
            failure_timings: WasmBuildFailureTimings::default(),
        }
    }

    fn emit(&mut self, event: WasmBuildProgressEvent) {
        if let Some(observer) = &mut self.observer {
            observer(event);
            self.last_event = Instant::now();
        }
    }

    const fn is_observed(&self) -> bool {
        self.observer.is_some()
    }

    fn heartbeat_due_in(&self) -> Option<Duration> {
        self.config
            .heartbeat_interval
            .map(|interval| interval.saturating_sub(self.last_event.elapsed()))
    }

    fn emit_heartbeat(&mut self, phase: WasmBuildProgressPhase, elapsed: Duration) {
        self.emit(WasmBuildProgressEvent::Heartbeat { phase, elapsed });
    }

    fn emit_heartbeat_if_due(&mut self, phase: WasmBuildProgressPhase, elapsed: Duration) {
        if self.heartbeat_due_in() == Some(Duration::ZERO) {
            self.emit_heartbeat(phase, elapsed);
        }
    }

    fn run_phase<T, F>(&mut self, phase: WasmBuildProgressPhase, operation: F) -> T
    where
        T: Send,
        F: FnOnce() -> T + Send,
    {
        let started = Instant::now();
        self.begin_phase(progress_failure_phase(phase));
        let result = if !self.is_observed() || self.config.heartbeat_interval.is_none() {
            operation()
        } else {
            thread::scope(|scope| {
                let (finished, completion) = mpsc::sync_channel(0);
                let worker = scope.spawn(move || {
                    let result = operation();
                    let _ = finished.send(());
                    result
                });
                loop {
                    let wait = self
                        .heartbeat_due_in()
                        .expect("observed phase must have a heartbeat interval");
                    match completion.recv_timeout(wait) {
                        Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                            return worker
                                .join()
                                .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            self.emit_heartbeat(phase, started.elapsed());
                        }
                    }
                }
            })
        };
        self.record_phase(progress_failure_phase(phase), started.elapsed());
        result
    }

    const fn begin_phase(&mut self, phase: WasmBuildFailurePhase) {
        self.failure_phase = Some(phase);
    }

    fn record_phase(&mut self, phase: WasmBuildFailurePhase, elapsed: Duration) {
        self.failure_phase = Some(phase);
        let timings = &mut self.failure_timings;
        match phase {
            WasmBuildFailurePhase::Specification => {}
            WasmBuildFailurePhase::ExactCacheCoordination => {
                timings.exact_cache_coordination =
                    timings.exact_cache_coordination.saturating_add(elapsed);
            }
            WasmBuildFailurePhase::ToolIdentity => {
                timings.input_resolution.tool_identity = timings
                    .input_resolution
                    .tool_identity
                    .saturating_add(elapsed);
                timings.input_resolution.total =
                    timings.input_resolution.total.saturating_add(elapsed);
            }
            WasmBuildFailurePhase::CargoMetadata => {
                timings.input_resolution.cargo_metadata = timings
                    .input_resolution
                    .cargo_metadata
                    .saturating_add(elapsed);
                timings.input_resolution.total =
                    timings.input_resolution.total.saturating_add(elapsed);
            }
            WasmBuildFailurePhase::InputDiscovery => {
                timings.input_resolution.input_discovery = timings
                    .input_resolution
                    .input_discovery
                    .saturating_add(elapsed);
                timings.input_resolution.total =
                    timings.input_resolution.total.saturating_add(elapsed);
            }
            WasmBuildFailurePhase::ContentHashing => {
                timings.input_resolution.content_hashing = timings
                    .input_resolution
                    .content_hashing
                    .saturating_add(elapsed);
                timings.input_resolution.total =
                    timings.input_resolution.total.saturating_add(elapsed);
            }
            WasmBuildFailurePhase::SharedTargetCoordination => {
                timings.shared_target_coordination = Some(
                    timings
                        .shared_target_coordination
                        .unwrap_or_default()
                        .saturating_add(elapsed),
                );
            }
            WasmBuildFailurePhase::SharedTargetMaintenance => {
                timings.shared_target_maintenance = Some(
                    timings
                        .shared_target_maintenance
                        .unwrap_or_default()
                        .saturating_add(elapsed),
                );
            }
            WasmBuildFailurePhase::CargoBuild => {
                timings.cargo_build = Some(
                    timings
                        .cargo_build
                        .unwrap_or_default()
                        .saturating_add(elapsed),
                );
            }
            WasmBuildFailurePhase::ArtifactPublication => {
                timings.artifact_publication = Some(
                    timings
                        .artifact_publication
                        .unwrap_or_default()
                        .saturating_add(elapsed),
                );
            }
            WasmBuildFailurePhase::ExactCacheMaintenance => {
                timings.exact_cache_maintenance = Some(
                    timings
                        .exact_cache_maintenance
                        .unwrap_or_default()
                        .saturating_add(elapsed),
                );
            }
            WasmBuildFailurePhase::Cleanup => {
                timings.cleanup = Some(timings.cleanup.unwrap_or_default().saturating_add(elapsed));
            }
        }
    }

    fn failure_details(
        &self,
        error: &WasmBuildError,
        total: Duration,
    ) -> (WasmBuildFailurePhase, WasmBuildFailureTimings) {
        let phase = self
            .failure_phase
            .unwrap_or_else(|| classify_unobserved_failure(error));
        let mut timings = self.failure_timings;
        timings.total = total;
        (phase, timings)
    }
}

const fn progress_failure_phase(phase: WasmBuildProgressPhase) -> WasmBuildFailurePhase {
    match phase {
        WasmBuildProgressPhase::ExactCacheLock => WasmBuildFailurePhase::ExactCacheCoordination,
        WasmBuildProgressPhase::CargoIdentity | WasmBuildProgressPhase::RustcIdentity => {
            WasmBuildFailurePhase::ToolIdentity
        }
        WasmBuildProgressPhase::CargoMetadata => WasmBuildFailurePhase::CargoMetadata,
        WasmBuildProgressPhase::InputDiscovery => WasmBuildFailurePhase::InputDiscovery,
        WasmBuildProgressPhase::ContentHashing => WasmBuildFailurePhase::ContentHashing,
        WasmBuildProgressPhase::SharedTargetLock => WasmBuildFailurePhase::SharedTargetCoordination,
        WasmBuildProgressPhase::SharedTargetMaintenance => {
            WasmBuildFailurePhase::SharedTargetMaintenance
        }
        WasmBuildProgressPhase::CargoBuild => WasmBuildFailurePhase::CargoBuild,
        WasmBuildProgressPhase::ArtifactPublication => WasmBuildFailurePhase::ArtifactPublication,
        WasmBuildProgressPhase::ExactCacheMaintenance => {
            WasmBuildFailurePhase::ExactCacheMaintenance
        }
    }
}

const fn classify_unobserved_failure(error: &WasmBuildError) -> WasmBuildFailurePhase {
    match error {
        WasmBuildError::InvalidSpec { .. } => WasmBuildFailurePhase::Specification,
        WasmBuildError::CommandSpawn { phase, .. }
        | WasmBuildError::CommandFailed { phase, .. } => match phase {
            WasmBuildPhase::CargoIdentity | WasmBuildPhase::RustcIdentity => {
                WasmBuildFailurePhase::ToolIdentity
            }
            WasmBuildPhase::CargoMetadata => WasmBuildFailurePhase::CargoMetadata,
            WasmBuildPhase::CargoBuild => WasmBuildFailurePhase::CargoBuild,
        },
        WasmBuildError::InvalidMetadata { .. } => WasmBuildFailurePhase::CargoMetadata,
        WasmBuildError::InvalidCargoConfiguration { .. } => WasmBuildFailurePhase::InputDiscovery,
        WasmBuildError::MissingArtifacts { .. } => WasmBuildFailurePhase::ArtifactPublication,
        WasmBuildError::InputsChangedDuringBuild { .. } => WasmBuildFailurePhase::ContentHashing,
        WasmBuildError::PreparedInputSnapshotInvalidated => {
            WasmBuildFailurePhase::ArtifactPublication
        }
        WasmBuildError::FailedBuildCleanup { .. } => WasmBuildFailurePhase::Cleanup,
        WasmBuildError::Io { .. } => WasmBuildFailurePhase::ExactCacheCoordination,
    }
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
    /// A discovered Cargo configuration could not be interpreted exactly.
    InvalidCargoConfiguration { path: PathBuf, message: String },
    /// Cargo succeeded without producing every declared Wasm artifact.
    MissingArtifacts { paths: Vec<PathBuf> },
    /// Declared inputs changed while Cargo was building.
    InputsChangedDuringBuild {
        before: InputDigest,
        after: InputDigest,
    },
    /// Another concurrent reader invalidated the prepared input snapshot before publication.
    PreparedInputSnapshotInvalidated,
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
            shared_incremental_maintenance_config: None,
        }
    }

    /// Set Cargo profile and feature arguments used for the build and fingerprint.
    #[must_use]
    pub fn with_cargo_profile_args<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cargo_profile_args = arguments
            .into_iter()
            .map(|argument| argument.as_ref().to_owned())
            .collect();
        self
    }

    /// Set deterministic OS-native child-process environment overrides.
    #[must_use]
    pub fn with_extra_env<I, K, V>(mut self, environment: I) -> Self
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

    /// Add ambient environment names whose current values affect the build.
    ///
    /// Common Rust and Cargo toolchain variables are included automatically.
    /// Callers must declare application-specific variables read by build scripts.
    #[must_use]
    pub fn with_inherited_env<I, S>(mut self, names: I) -> Self
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
    pub fn with_additional_inputs<I, P>(mut self, paths: I) -> Self
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

    /// Schedule caller-owned shared-target retention as part of acquisition.
    ///
    /// This option requires [`Self::with_shared_incremental_target`]. Every
    /// acquisition coordinates through that target, including an exact hit,
    /// so a missing target can be created and receive its first schedule
    /// marker immediately. Matching recent passes only check the marker; due
    /// passes reuse the acquisition's exact Cargo input resolution before
    /// evaluating retention. The structured result is attached to the build
    /// record and emitted through observed progress. Maintenance failures fail
    /// the acquisition and do not record a successful schedule marker.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance_at_most_every(
        mut self,
        policy: SharedIncrementalTargetPrunePolicy,
        minimum_interval: Duration,
    ) -> Self {
        self.shared_incremental_maintenance_config = Some(
            SharedIncrementalTargetMaintenanceConfig::new(policy, minimum_interval),
        );
        self
    }

    /// Attach an explicit shared-target maintenance configuration.
    ///
    /// This is the configurable counterpart to
    /// [`Self::with_shared_incremental_target_maintenance_at_most_every`] and
    /// supports strict or best-effort failure handling.
    #[must_use]
    pub const fn with_shared_incremental_target_maintenance(
        mut self,
        config: SharedIncrementalTargetMaintenanceConfig,
    ) -> Self {
        self.shared_incremental_maintenance_config = Some(config);
        self
    }

    /// Apply cache retention under the build operation's existing process lock.
    ///
    /// Maintenance is best-effort: its structured result is attached to the
    /// successful build record and cannot turn ready artifacts into a build
    /// failure. The active fingerprint is protected from this pruning pass.
    #[must_use]
    pub const fn with_prune_policy(mut self, policy: ArtifactCachePrunePolicy) -> Self {
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
        policy: ArtifactCachePrunePolicy,
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

    /// Exact-entry retention policy attached to this specification, when configured.
    #[must_use]
    pub const fn prune_policy(&self) -> Option<ArtifactCachePrunePolicy> {
        self.prune_policy
    }

    /// Minimum interval between exact-entry retention attempts, when scheduled.
    #[must_use]
    pub const fn prune_interval(&self) -> Option<Duration> {
        self.prune_interval
    }

    /// Shared incremental-target maintenance attached to this specification.
    #[must_use]
    pub const fn shared_incremental_target_maintenance(
        &self,
    ) -> Option<SharedIncrementalTargetMaintenanceConfig> {
        self.shared_incremental_maintenance_config
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

    /// Semantic digest of selected package sources and configuration inputs.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Immutable content-addressed cache directory for this exact build.
    ///
    /// The directory is selected by the build fingerprint and contains the
    /// cached Wasm artifacts and their stamps. Callers can persist this path
    /// in CI without depending on `ic-testkit`'s private target layout.
    #[must_use]
    pub fn exact_cache_path(&self) -> &Path {
        &self.exact_cache_path
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
    pub const fn maintenance(&self) -> Option<&ArtifactCacheMaintenance> {
        self.maintenance.as_ref()
    }

    /// Scheduled caller-owned shared-target maintenance, when configured.
    #[must_use]
    pub const fn shared_incremental_maintenance(
        &self,
    ) -> Option<&SharedIncrementalTargetMaintenanceOutcome> {
        self.shared_incremental_maintenance.as_ref()
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

    /// Detailed tool, metadata, discovery, and hashing timings.
    #[must_use]
    pub const fn input_resolution(self) -> WasmInputResolutionTimings {
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

    pub(super) const fn saturating_add(self, other: Self) -> Self {
        let mut input_resolution = self.input_resolution;
        input_resolution.include(other.input_resolution);
        Self {
            lock_wait: self.lock_wait.saturating_add(other.lock_wait),
            shared_incremental_lock_wait: saturating_add_optional_duration(
                self.shared_incremental_lock_wait,
                other.shared_incremental_lock_wait,
            ),
            input_resolution,
            cargo_build: saturating_add_optional_duration(self.cargo_build, other.cargo_build),
            cache_maintenance: saturating_add_optional_duration(
                self.cache_maintenance,
                other.cache_maintenance,
            ),
            total: self.total.saturating_add(other.total),
        }
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

impl WasmBuildFailureTimings {
    /// Time spent coordinating the exact artifact cache before failure.
    #[must_use]
    pub const fn exact_cache_coordination(self) -> Duration {
        self.exact_cache_coordination
    }

    /// Time spent coordinating a shared incremental target, when reached.
    #[must_use]
    pub const fn shared_target_coordination(self) -> Option<Duration> {
        self.shared_target_coordination
    }

    /// Partial Cargo/rustc identity, metadata, discovery, and hashing timings.
    #[must_use]
    pub const fn input_resolution(self) -> WasmInputResolutionTimings {
        self.input_resolution
    }

    /// Time spent on shared-target maintenance, when reached.
    #[must_use]
    pub const fn shared_target_maintenance(self) -> Option<Duration> {
        self.shared_target_maintenance
    }

    /// Time spent executing Cargo, including an unsuccessful execution.
    #[must_use]
    pub const fn cargo_build(self) -> Option<Duration> {
        self.cargo_build
    }

    /// Time spent validating or publishing artifacts, when reached.
    #[must_use]
    pub const fn artifact_publication(self) -> Option<Duration> {
        self.artifact_publication
    }

    /// Time spent on exact-cache maintenance, when reached.
    #[must_use]
    pub const fn exact_cache_maintenance(self) -> Option<Duration> {
        self.exact_cache_maintenance
    }

    /// Explicit incomplete-entry cleanup time, when failure required it.
    #[must_use]
    pub const fn cleanup(self) -> Option<Duration> {
        self.cleanup
    }

    /// Complete failed acquisition wall time.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
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

    /// Semantic digest of selected Cargo sources and workspace configuration.
    ///
    /// Unlike [`Self::validation_digest`], this may remain unchanged after an
    /// unrelated host-only workspace manifest or lockfile update.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Conservative digest of every raw source and configuration input.
    ///
    /// This digest is used for mutation guards. It may change while
    /// [`Self::input_digest`] and [`Self::fingerprint`] remain unchanged.
    #[must_use]
    pub const fn validation_digest(&self) -> InputDigest {
        self.validation_digest
    }

    /// Stable logical labels and conservative resolved validation paths.
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
        self.current_validation_digest()
            .map(|current| current == self.validation_digest)
    }

    pub(super) fn current_validation_digest(&self) -> Result<InputDigest, WasmBuildError> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| (input.label.clone(), input.path.clone()))
            .collect::<Vec<_>>();
        digest_labeled_paths_composable(
            "wasm-source-inputs-v1",
            &inputs,
            &self.exclusions,
            &mut LabeledPathDigestCache::default(),
        )
        .map_err(|source| WasmBuildError::Io {
            operation: "rehash resolved Cargo build inputs",
            path: self
                .inputs
                .first()
                .map_or_else(PathBuf::new, |input| input.path.clone()),
            source,
        })
    }
}

impl WasmBuildSessionState {
    pub(super) fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            digest_cache: LabeledPathDigestCache::default(),
            snapshot_reuses: 0,
            invalidated: false,
        }
    }

    pub(super) const fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub(super) const fn snapshot_reuses(&self) -> usize {
        self.snapshot_reuses
    }

    pub(super) const fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    fn reuse(&mut self, spec: &WasmBuildSpec) -> Option<ResolvedCargoBuildInputs> {
        let (_, snapshot) = self
            .snapshots
            .iter()
            .find(|(candidate, _)| candidate == spec)?;
        self.snapshot_reuses = self.snapshot_reuses.saturating_add(1);
        let mut snapshot = snapshot.clone();
        snapshot.timings = WasmInputResolutionTimings::default();
        Some(snapshot)
    }

    fn remember(&mut self, spec: &WasmBuildSpec, resolved: &ResolvedCargoBuildInputs) {
        if self
            .snapshots
            .iter()
            .any(|(candidate, _)| candidate == spec)
        {
            return;
        }
        let mut snapshot = resolved.clone();
        snapshot.timings = WasmInputResolutionTimings::default();
        self.snapshots.push((spec.clone(), snapshot));
    }

    fn invalidate(&mut self) {
        self.snapshots.clear();
        self.digest_cache = LabeledPathDigestCache::default();
        self.invalidated = true;
    }
}

impl WasmBuildInputSnapshotState {
    pub(super) fn prepare(specs: &[WasmBuildSpec]) -> Result<Self, WasmBuildError> {
        for spec in specs {
            validate_spec(spec)?;
        }
        let mut resolver = WasmBuildBatchInputResolver::new(specs);
        let mut snapshots = Vec::with_capacity(specs.len());
        let mut preparation_timings = WasmInputResolutionTimings::default();
        let mut progress = ProgressReporter::silent();
        for (index, spec) in specs.iter().enumerate() {
            let mut prepared_input = resolver.resolve(index, &mut progress)?;
            preparation_timings.include(prepared_input.timings);
            prepared_input.timings = WasmInputResolutionTimings::default();
            snapshots.push((spec.clone(), prepared_input));
        }
        Ok(Self {
            snapshots,
            preparation_metrics: resolver.metrics(),
            preparation_timings,
            reader_reuses: AtomicUsize::new(0),
            invalidation: Arc::new(RwLock::new(false)),
        })
    }

    pub(super) fn contains(&self, spec: &WasmBuildSpec) -> bool {
        self.snapshots
            .iter()
            .any(|(candidate, _)| candidate == spec)
    }

    fn reuse(&self, spec: &WasmBuildSpec) -> Option<ResolvedCargoBuildInputs> {
        let (_, snapshot) = self
            .snapshots
            .iter()
            .find(|(candidate, _)| candidate == spec)?;
        let _ = self
            .reader_reuses
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_add(1))
            });
        Some(snapshot.clone())
    }

    pub(super) const fn specification_count(&self) -> usize {
        self.snapshots.len()
    }

    pub(super) const fn preparation_metrics(&self) -> WasmBuildBatchInputMetrics {
        self.preparation_metrics
    }

    pub(super) const fn preparation_timings(&self) -> WasmInputResolutionTimings {
        self.preparation_timings
    }

    pub(super) fn reader_reuses(&self) -> usize {
        self.reader_reuses.load(Ordering::Relaxed)
    }

    pub(super) fn is_invalidated(&self) -> bool {
        *self
            .invalidation
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn invalidate(&self) {
        *self
            .invalidation
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    }

    fn invalidation(&self) -> Arc<RwLock<bool>> {
        Arc::clone(&self.invalidation)
    }
}

impl<'a, 'session> WasmBuildBatchInputResolver<'a, 'session> {
    pub(super) fn new(specs: &'a [WasmBuildSpec]) -> Self {
        Self::create(specs, None, None)
    }

    pub(super) fn with_session(
        specs: &'a [WasmBuildSpec],
        session: &'session mut WasmBuildSessionState,
    ) -> Self {
        Self::create(specs, Some(session), None)
    }

    pub(super) fn with_snapshot(
        specs: &'a [WasmBuildSpec],
        snapshot: &'session WasmBuildInputSnapshotState,
    ) -> Self {
        Self::create(specs, None, Some(snapshot))
    }

    fn create(
        specs: &'a [WasmBuildSpec],
        mut session: Option<&'session mut WasmBuildSessionState>,
        snapshot: Option<&'session WasmBuildInputSnapshotState>,
    ) -> Self {
        let mut keys = Vec::<BatchResolutionKey>::new();
        let mut groups = Vec::<BatchResolutionGroup>::new();
        let mut group_by_index = Vec::with_capacity(specs.len());
        for (index, spec) in specs.iter().enumerate() {
            let key = BatchResolutionKey::for_spec(spec);
            let group = keys
                .iter()
                .position(|candidate| *candidate == key)
                .unwrap_or_else(|| {
                    keys.push(key);
                    groups.push(BatchResolutionGroup {
                        indexes: Vec::new(),
                    });
                    groups.len() - 1
                });
            groups[group].indexes.push(index);
            group_by_index.push(group);
        }
        let mut metrics = WasmBuildBatchInputMetrics::default();
        let resolved = specs
            .iter()
            .map(|spec| {
                let session_reused = session
                    .as_deref_mut()
                    .and_then(|session| session.reuse(spec));
                if session_reused.is_some() {
                    metrics.session_reuses = metrics.session_reuses.saturating_add(1);
                    return session_reused.map(Ok);
                }
                if let Some(snapshot) = snapshot {
                    let reused = snapshot
                        .reuse(spec)
                        .expect("prepared input snapshot must contain every reader specification");
                    metrics.prepared_reuses = metrics.prepared_reuses.saturating_add(1);
                    return Some(Ok(reused));
                }
                None
            })
            .collect();
        Self {
            specs,
            groups,
            group_by_index,
            resolved,
            session,
            snapshot,
            metrics,
        }
    }

    pub(super) const fn metrics(&self) -> WasmBuildBatchInputMetrics {
        self.metrics
    }

    pub(super) fn invalidate_source_lease(&mut self) {
        if let Some(session) = self.session.as_deref_mut() {
            session.invalidate();
            // Every unresolved entry was captured before the detected source race,
            // including entries resolved only for this batch. Force later entries
            // through fresh discovery instead of consuming a now-stale snapshot.
            for resolved in &mut self.resolved {
                *resolved = None;
            }
            self.session = None;
        }
        if let Some(snapshot) = self.snapshot {
            snapshot.invalidate();
        }
    }

    pub(super) const fn assumes_sources_immutable(&self) -> bool {
        self.session.is_some() || self.snapshot.is_some()
    }

    pub(super) fn prepared_invalidation(&self) -> Option<Arc<RwLock<bool>>> {
        self.snapshot.map(WasmBuildInputSnapshotState::invalidation)
    }

    fn resolve(
        &mut self,
        index: usize,
        progress: &mut ProgressReporter<'_>,
    ) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
        if self.resolved[index].is_none() {
            self.resolve_group(index, progress)?;
        }
        self.resolved[index]
            .take()
            .expect("resolved batch input must be populated")
    }

    fn resolve_group(
        &mut self,
        active_index: usize,
        progress: &mut ProgressReporter<'_>,
    ) -> Result<(), WasmBuildError> {
        let total_started = Instant::now();
        let indexes = self.groups[self.group_by_index[active_index]]
            .indexes
            .clone();
        let active = &self.specs[active_index];

        let (cargo_identity, rustc_identity, tool_identity) =
            resolve_batch_tool_identity(active, progress)?;

        let metadata_started = Instant::now();
        let metadata = progress.run_phase(WasmBuildProgressPhase::CargoMetadata, || {
            cargo_metadata(active)
        })?;
        let cargo_metadata = metadata_started.elapsed();

        let (discovered, input_discovery) =
            self.discover_group_inputs(indexes, &metadata, progress);

        let hashing_started = Instant::now();
        let mut batch_digest_cache = LabeledPathDigestCache::default();
        let digest_cache = self
            .session
            .as_deref_mut()
            .map_or(&mut batch_digest_cache, |session| &mut session.digest_cache);
        let workspace_root = active.workspace_root.clone();
        let resolved_inputs = progress.run_phase(WasmBuildProgressPhase::ContentHashing, || {
            discovered
                .into_iter()
                .map(|(index, inputs, exclusions)| {
                    let (input_digest, validation_digest) = digest_resolved_local_inputs(
                        &inputs,
                        &exclusions,
                        digest_cache,
                        &workspace_root,
                        "hash batched Wasm build inputs",
                        "hash batched semantic Wasm build inputs",
                    )?;
                    Ok::<_, WasmBuildError>((
                        index,
                        inputs.validation_inputs,
                        exclusions,
                        input_digest,
                        validation_digest,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
        let content_hashing = hashing_started.elapsed();
        let timings = WasmInputResolutionTimings {
            tool_identity,
            cargo_metadata,
            input_discovery,
            content_hashing,
            total: total_started.elapsed(),
        };
        let resolved_count = resolved_inputs.len();
        if resolved_count > 0 {
            self.metrics.runs += 1;
            self.metrics.reuses += resolved_count.saturating_sub(1);
        }
        let timing_index = resolved_inputs
            .iter()
            .any(|(index, ..)| *index == active_index)
            .then_some(active_index)
            .or_else(|| resolved_inputs.first().map(|(index, ..)| *index));
        for (index, inputs, exclusions, input_digest, validation_digest) in resolved_inputs {
            let spec = &self.specs[index];
            let resolved = ResolvedCargoBuildInputs {
                fingerprint: finish_build_fingerprint(
                    spec,
                    &cargo_identity,
                    &rustc_identity,
                    input_digest,
                ),
                input_digest,
                validation_digest,
                inputs: inputs
                    .into_iter()
                    .map(|(label, path)| CargoBuildInput { label, path })
                    .collect(),
                exclusions,
                timings: if Some(index) == timing_index {
                    timings
                } else {
                    WasmInputResolutionTimings::default()
                },
            };
            if let Some(session) = self.session.as_deref_mut() {
                session.remember(spec, &resolved);
            }
            self.resolved[index] = Some(Ok(resolved));
        }
        Ok(())
    }

    fn discover_group_inputs(
        &mut self,
        indexes: Vec<usize>,
        metadata: &Value,
        progress: &mut ProgressReporter<'_>,
    ) -> (Vec<(usize, ResolvedLocalInputs, Vec<PathBuf>)>, Duration) {
        let started = Instant::now();
        let pending = indexes
            .into_iter()
            .filter(|index| {
                self.resolved[*index].is_none() && validate_spec(&self.specs[*index]).is_ok()
            })
            .collect::<Vec<_>>();
        let results = progress.run_phase(WasmBuildProgressPhase::InputDiscovery, || {
            pending
                .into_iter()
                .map(|index| {
                    let spec = &self.specs[index];
                    let result = (|| {
                        let inputs = resolve_local_inputs(spec, metadata)?;
                        validate_shared_incremental_target_boundary(
                            spec,
                            &inputs.validation_inputs,
                        )?;
                        let exclusions = source_exclusions(spec, &inputs.validation_inputs);
                        Ok::<_, WasmBuildError>((inputs, exclusions))
                    })();
                    (index, result)
                })
                .collect::<Vec<_>>()
        });
        let mut discovered = Vec::new();
        for (index, result) in results {
            match result {
                Ok((inputs, exclusions)) => discovered.push((index, inputs, exclusions)),
                Err(error) => self.resolved[index] = Some(Err(error)),
            }
        }
        (discovered, started.elapsed())
    }
}

fn resolve_batch_tool_identity(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<(Vec<u8>, Vec<u8>, Duration), WasmBuildError> {
    let started = Instant::now();
    let cargo_identity = progress.run_phase(WasmBuildProgressPhase::CargoIdentity, || {
        command_identity(
            spec,
            WasmBuildPhase::CargoIdentity,
            &spec.cargo_program,
            &["--version", "--verbose"],
        )
    })?;
    let rustc_program = spec
        .extra_env
        .get(OsStr::new("RUSTC"))
        .unwrap_or(&spec.rustc_program);
    let rustc_identity = progress.run_phase(WasmBuildProgressPhase::RustcIdentity, || {
        command_identity(spec, WasmBuildPhase::RustcIdentity, rustc_program, &["-vV"])
    })?;
    Ok((cargo_identity, rustc_identity, started.elapsed()))
}

impl BatchResolutionKey {
    fn for_spec(spec: &WasmBuildSpec) -> Self {
        Self {
            workspace_root: spec.workspace_root.clone(),
            cargo_program: spec.cargo_program.clone(),
            rustc_program: spec
                .extra_env
                .get(OsStr::new("RUSTC"))
                .unwrap_or(&spec.rustc_program)
                .clone(),
            metadata_arguments: metadata_arguments(&spec.cargo_profile_args),
            environment: effective_environment(spec),
        }
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

impl SharedIncrementalTargetPrunePolicy {
    /// Create an explicit policy without a clearing threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_age: None,
            max_size_bytes: None,
        }
    }

    /// Clear shared Cargo state when its recorded use is older than `max_age`.
    #[must_use]
    pub const fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Clear shared Cargo state when its logical size exceeds `bytes`.
    #[must_use]
    pub const fn with_max_size_bytes(mut self, bytes: u64) -> Self {
        self.max_size_bytes = Some(bytes);
        self
    }

    /// Configured maximum time since recorded build use.
    #[must_use]
    pub const fn max_age(self) -> Option<Duration> {
        self.max_age
    }

    /// Configured maximum logical target size.
    #[must_use]
    pub const fn max_size_bytes(self) -> Option<u64> {
        self.max_size_bytes
    }

    fn maintenance_identity(self) -> String {
        format!(
            "age={:?};size={:?}",
            self.max_age.map(|duration| duration.as_nanos()),
            self.max_size_bytes
        )
    }
}

impl SharedIncrementalTargetMaintenanceConfig {
    /// Schedule one strict retention pass at most once per interval.
    #[must_use]
    pub const fn new(
        policy: SharedIncrementalTargetPrunePolicy,
        minimum_interval: Duration,
    ) -> Self {
        Self {
            policy,
            minimum_interval,
            failure_mode: SharedIncrementalTargetMaintenanceFailureMode::Strict,
        }
    }

    /// Select whether an integrated maintenance failure fails the acquisition.
    #[must_use]
    pub const fn with_failure_mode(
        mut self,
        failure_mode: SharedIncrementalTargetMaintenanceFailureMode,
    ) -> Self {
        self.failure_mode = failure_mode;
        self
    }

    /// Configured whole-target retention policy.
    #[must_use]
    pub const fn policy(self) -> SharedIncrementalTargetPrunePolicy {
        self.policy
    }

    /// Minimum interval between successful matching maintenance passes.
    #[must_use]
    pub const fn minimum_interval(self) -> Duration {
        self.minimum_interval
    }

    /// Configured maintenance failure handling.
    #[must_use]
    pub const fn failure_mode(self) -> SharedIncrementalTargetMaintenanceFailureMode {
        self.failure_mode
    }
}

impl SharedIncrementalTargetMaintenance {
    /// Canonical shared Cargo target directory maintained under lock.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    /// Logical bytes observed before applying the policy.
    #[must_use]
    pub const fn logical_size_bytes_before(&self) -> u64 {
        self.logical_size_bytes_before
    }

    /// Logical bytes retained after applying the policy.
    #[must_use]
    pub const fn logical_size_bytes_after(&self) -> u64 {
        self.logical_size_bytes_after
    }

    /// Most recent build use observed before applying the policy.
    #[must_use]
    pub const fn last_used_before(&self) -> SystemTime {
        self.last_used_before
    }

    /// Whether a configured limit caused the mutable target contents to be cleared.
    #[must_use]
    pub const fn was_cleared(&self) -> bool {
        self.cleared
    }

    /// Time spent waiting for another process using the shared target.
    #[must_use]
    pub const fn lock_wait(&self) -> Duration {
        self.lock_wait
    }

    /// Time spent measuring and, when required, clearing the target.
    #[must_use]
    pub const fn maintenance(&self) -> Duration {
        self.maintenance
    }
}

impl std::fmt::Display for SharedIncrementalTargetMaintenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "target={} action={} bytes={}=>{} lock={:?} maintenance={:?}",
            self.target_dir.display(),
            if self.cleared { "cleared" } else { "retained" },
            self.logical_size_bytes_before,
            self.logical_size_bytes_after,
            self.lock_wait,
            self.maintenance,
        )
    }
}

impl SharedIncrementalTargetMaintenanceOutcome {
    /// Configured or canonical target associated with this result.
    #[must_use]
    pub fn target_dir(&self) -> &Path {
        match self {
            Self::Missing { target_dir }
            | Self::Skipped { target_dir, .. }
            | Self::Failed { target_dir, .. } => target_dir,
            Self::Performed { maintenance, .. } => maintenance.target_dir(),
        }
    }

    /// Completed maintenance report, when retention was evaluated.
    #[must_use]
    pub const fn maintenance(&self) -> Option<&SharedIncrementalTargetMaintenance> {
        match self {
            Self::Performed { maintenance, .. } => Some(maintenance),
            Self::Missing { .. } | Self::Skipped { .. } | Self::Failed { .. } => None,
        }
    }

    /// Whether retention was evaluated during this call.
    #[must_use]
    pub const fn was_performed(&self) -> bool {
        matches!(self, Self::Performed { .. })
    }

    /// Time spent waiting for another process, when the target existed.
    #[must_use]
    pub const fn lock_wait(&self) -> Option<Duration> {
        match self {
            Self::Missing { .. } => None,
            Self::Skipped { lock_wait, .. } | Self::Failed { lock_wait, .. } => Some(*lock_wait),
            Self::Performed { maintenance, .. } => Some(maintenance.lock_wait()),
        }
    }

    /// Time spent checking the schedule marker, when the target existed.
    #[must_use]
    pub const fn schedule_check(&self) -> Option<Duration> {
        match self {
            Self::Missing { .. } | Self::Failed { .. } => None,
            Self::Skipped { schedule_check, .. } | Self::Performed { schedule_check, .. } => {
                Some(*schedule_check)
            }
        }
    }

    /// Rendered integrated maintenance failure, when best-effort handling preserved acquisition.
    #[must_use]
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Failed { message, .. } => Some(message),
            Self::Missing { .. } | Self::Skipped { .. } | Self::Performed { .. } => None,
        }
    }
}

impl std::fmt::Display for SharedIncrementalTargetMaintenanceOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { target_dir } => {
                write!(formatter, "target={} action=missing", target_dir.display())
            }
            Self::Skipped {
                target_dir,
                lock_wait,
                schedule_check,
            } => write!(
                formatter,
                "target={} action=skipped lock={lock_wait:?} schedule={schedule_check:?}",
                target_dir.display(),
            ),
            Self::Performed {
                maintenance,
                schedule_check,
            } => write!(formatter, "{maintenance} schedule={schedule_check:?}"),
            Self::Failed {
                target_dir,
                lock_wait,
                message,
            } => write!(
                formatter,
                "target={} action=failed lock={lock_wait:?} error={message}",
                target_dir.display(),
            ),
        }
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
        )?;
        if let Some(maintenance) = self.record().shared_incremental_maintenance() {
            write!(formatter, " shared_maintenance=({maintenance})")?;
        }
        Ok(())
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
    if !shared_incremental_target_exists(spec, "inspect shared incremental Cargo target")? {
        return Ok(None);
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

/// Apply explicit whole-target retention to caller-owned shared Cargo state.
///
/// Returns `None` without creating anything when the target does not exist.
/// Policy evaluation and any clearing occur under the same cross-process lock
/// used by shared-incremental builds. The target root, `CACHEDIR.TAG`, and
/// `.ic-testkit` lock metadata are preserved, so another process cannot enter
/// through a replacement lock while maintenance is active.
/// Every other target child is removed when a limit is exceeded; unrelated
/// data that must survive must not be colocated there. Exact Cargo input
/// resolution first rejects targets overlapping source or configuration.
///
/// This function is never called automatically by exact Wasm acquisitions.
/// Consumers retain ownership of when mutable incremental state may be lost.
pub fn maintain_shared_incremental_target(
    spec: &WasmBuildSpec,
    policy: SharedIncrementalTargetPrunePolicy,
) -> Result<Option<SharedIncrementalTargetMaintenance>, WasmBuildError> {
    if !shared_incremental_target_exists(
        spec,
        "inspect shared incremental Cargo target before maintenance",
    )? {
        return Ok(None);
    }

    // Reuse the exact build resolver so destructive maintenance cannot act on
    // a target that overlaps Cargo sources, configuration, or additional
    // inputs. The target itself is excluded as generated state during hashing.
    let _ = resolve_cargo_build_inputs(spec)?;
    let (_lock, lock_wait, canonical) = lock_shared_incremental_target(spec)?;
    maintain_shared_incremental_target_locked(&canonical, policy, lock_wait).map(Some)
}

/// Apply whole-target retention at most once per interval across processes.
///
/// The schedule marker is checked under the same lock used by shared Cargo
/// builds. A matching successful pass inside `minimum_interval` returns
/// [`SharedIncrementalTargetMaintenanceOutcome::Skipped`] without resolving
/// Cargo inputs or traversing the target. Missing targets are not created.
/// Changing the policy makes maintenance immediately due, and a zero interval
/// always evaluates retention.
///
/// Due maintenance performs exact Cargo input resolution before inspecting or
/// clearing the target. Failures are returned and are not recorded as a
/// successful pass, so an unsafe configuration cannot be hidden by the
/// schedule.
pub fn maintain_shared_incremental_target_at_most_every(
    spec: &WasmBuildSpec,
    policy: SharedIncrementalTargetPrunePolicy,
    minimum_interval: Duration,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    if !shared_incremental_target_exists(
        spec,
        "inspect shared incremental Cargo target before scheduled maintenance",
    )? {
        return Ok(SharedIncrementalTargetMaintenanceOutcome::Missing { target_dir });
    }

    let (_lock, lock_wait, canonical) = lock_shared_incremental_target(spec)?;
    let schedule = schedule_shared_incremental_target_maintenance(
        &canonical,
        policy,
        minimum_interval,
        lock_wait,
    )?;
    let schedule = match schedule {
        SharedIncrementalTargetMaintenanceSchedule::Skipped(outcome) => return Ok(outcome),
        SharedIncrementalTargetMaintenanceSchedule::Due(due) => due,
    };

    // Keep the schedule decision and maintenance in one critical section so
    // concurrent test binaries cannot all perform the same expensive scan.
    let _ = resolve_cargo_build_inputs(spec)?;
    perform_due_shared_incremental_target_maintenance(&canonical, policy, lock_wait, schedule)
}

enum SharedIncrementalTargetMaintenanceSchedule {
    Skipped(SharedIncrementalTargetMaintenanceOutcome),
    Due(DueSharedIncrementalTargetMaintenance),
}

struct DueSharedIncrementalTargetMaintenance {
    schedule_root: PathBuf,
    maintenance_identity: String,
    schedule_check: Duration,
}

fn schedule_shared_incremental_target_maintenance(
    canonical: &Path,
    policy: SharedIncrementalTargetPrunePolicy,
    minimum_interval: Duration,
    lock_wait: Duration,
) -> Result<SharedIncrementalTargetMaintenanceSchedule, WasmBuildError> {
    let schedule_root = canonical.join(".ic-testkit");
    let maintenance_identity = policy.maintenance_identity();
    let schedule_started = Instant::now();
    let due = cache_maintenance_due(
        &schedule_root,
        Some(minimum_interval),
        &maintenance_identity,
    )
    .map_err(wasm_cache_fs_error)?;
    let schedule_check = schedule_started.elapsed();
    if !due {
        return Ok(SharedIncrementalTargetMaintenanceSchedule::Skipped(
            SharedIncrementalTargetMaintenanceOutcome::Skipped {
                target_dir: canonical.to_owned(),
                lock_wait,
                schedule_check,
            },
        ));
    }
    Ok(SharedIncrementalTargetMaintenanceSchedule::Due(
        DueSharedIncrementalTargetMaintenance {
            schedule_root,
            maintenance_identity,
            schedule_check,
        },
    ))
}

fn perform_due_shared_incremental_target_maintenance(
    canonical: &Path,
    policy: SharedIncrementalTargetPrunePolicy,
    lock_wait: Duration,
    due: DueSharedIncrementalTargetMaintenance,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    let DueSharedIncrementalTargetMaintenance {
        schedule_root,
        maintenance_identity,
        schedule_check,
    } = due;
    let maintenance = maintain_shared_incremental_target_locked(canonical, policy, lock_wait)?;
    record_cache_maintenance(&schedule_root, &maintenance_identity).map_err(wasm_cache_fs_error)?;
    Ok(SharedIncrementalTargetMaintenanceOutcome::Performed {
        maintenance,
        schedule_check,
    })
}

fn maintain_shared_incremental_target_locked(
    canonical: &Path,
    policy: SharedIncrementalTargetPrunePolicy,
    lock_wait: Duration,
) -> Result<SharedIncrementalTargetMaintenance, WasmBuildError> {
    let started = Instant::now();
    let logical_size_bytes_before =
        directory_logical_size(canonical).map_err(|source| WasmBuildError::Io {
            operation: "measure shared incremental Cargo target before maintenance",
            path: canonical.to_owned(),
            source,
        })?;
    let last_used_before =
        cache_entry_last_used(canonical).map_err(|source| WasmBuildError::Io {
            operation: "read shared incremental Cargo target use time before maintenance",
            path: canonical.to_owned(),
            source,
        })?;
    let expired = policy.max_age.is_some_and(|max_age| {
        SystemTime::now()
            .duration_since(last_used_before)
            .is_ok_and(|age| age > max_age)
    });
    let oversized = policy
        .max_size_bytes
        .is_some_and(|max_size_bytes| logical_size_bytes_before > max_size_bytes);
    let cleared = expired || oversized;
    if cleared {
        clear_shared_incremental_target_contents(canonical)?;
        record_cache_entry_use(canonical)?;
    }
    let logical_size_bytes_after = if cleared {
        directory_logical_size(canonical).map_err(|source| WasmBuildError::Io {
            operation: "measure shared incremental Cargo target after maintenance",
            path: canonical.to_owned(),
            source,
        })?
    } else {
        logical_size_bytes_before
    };
    Ok(SharedIncrementalTargetMaintenance {
        target_dir: canonical.to_owned(),
        logical_size_bytes_before,
        logical_size_bytes_after,
        last_used_before,
        cleared,
        lock_wait,
        maintenance: started.elapsed(),
    })
}

fn clear_shared_incremental_target_contents(target_dir: &Path) -> Result<(), WasmBuildError> {
    let entries = fs::read_dir(target_dir).map_err(|source| WasmBuildError::Io {
        operation: "read shared incremental Cargo target for maintenance",
        path: target_dir.to_owned(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| WasmBuildError::Io {
                operation: "read shared incremental Cargo target entry for maintenance",
                path: target_dir.to_owned(),
                source,
            })?
            .path();
        let preserved = path
            .file_name()
            .is_some_and(|name| name == ".ic-testkit" || name == "CACHEDIR.TAG");
        if !preserved {
            remove_path_if_present(&path).map_err(|source| WasmBuildError::Io {
                operation: "clear shared incremental Cargo target entry",
                path,
                source,
            })?;
        }
    }
    Ok(())
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
    build_wasm_canisters_cached_internal(spec, &mut ProgressReporter::silent(), None)
}

pub(super) fn build_wasm_canisters_cached_in_batch(
    spec: &WasmBuildSpec,
    index: usize,
    resolver: &mut WasmBuildBatchInputResolver<'_, '_>,
) -> WasmBuildBatchAttempt {
    let started = Instant::now();
    let mut progress = ProgressReporter::silent();
    let result = build_wasm_canisters_cached_internal(spec, &mut progress, Some((resolver, index)));
    if result
        .as_ref()
        .is_err_and(WasmBuildError::indicates_input_change)
    {
        resolver.invalidate_source_lease();
    }
    batch_attempt(result, &progress, started.elapsed())
}

/// Build or reuse one exact Wasm set while streaming structured progress.
///
/// Cargo output remains captured for [`WasmBuildError::CommandFailed`] and is
/// additionally forwarded as raw chunks when enabled. Potentially long input
/// resolution, lock waits, maintenance, Cargo, and publication phases emit
/// periodic heartbeats, so a legitimate acquisition need not appear stalled.
/// Observer panics propagate after joining active phase work, terminating the
/// Cargo child when applicable, and preserving normal cleanup.
pub fn build_wasm_canisters_cached_with_progress<F>(
    spec: &WasmBuildSpec,
    config: WasmBuildProgressConfig,
    mut observer: F,
) -> Result<WasmBuildOutcome, WasmBuildError>
where
    F: FnMut(WasmBuildProgressEvent),
{
    if config.heartbeat_interval == Some(Duration::ZERO) {
        return Err(WasmBuildError::InvalidSpec {
            message: "Wasm build progress heartbeat interval must be greater than zero".to_owned(),
        });
    }
    build_wasm_canisters_cached_internal(
        spec,
        &mut ProgressReporter::observed(config, &mut observer),
        None,
    )
}

pub(super) fn build_wasm_canisters_cached_in_batch_with_progress<F>(
    spec: &WasmBuildSpec,
    index: usize,
    resolver: &mut WasmBuildBatchInputResolver<'_, '_>,
    config: WasmBuildProgressConfig,
    mut observer: F,
) -> WasmBuildBatchAttempt
where
    F: FnMut(WasmBuildProgressEvent),
{
    if config.heartbeat_interval == Some(Duration::ZERO) {
        return WasmBuildBatchAttempt::invalid_spec(
            WasmBuildError::InvalidSpec {
                message: "Wasm build progress heartbeat interval must be greater than zero"
                    .to_owned(),
            },
            Duration::ZERO,
        );
    }
    let started = Instant::now();
    let mut progress = ProgressReporter::observed(config, &mut observer);
    let result = build_wasm_canisters_cached_internal(spec, &mut progress, Some((resolver, index)));
    if result
        .as_ref()
        .is_err_and(WasmBuildError::indicates_input_change)
    {
        resolver.invalidate_source_lease();
    }
    batch_attempt(result, &progress, started.elapsed())
}

fn batch_attempt(
    result: Result<WasmBuildOutcome, WasmBuildError>,
    progress: &ProgressReporter<'_>,
    total: Duration,
) -> WasmBuildBatchAttempt {
    let (failure_phase, failure_timings) = result.as_ref().err().map_or((None, None), |error| {
        let (phase, timings) = progress.failure_details(error, total);
        (Some(phase), Some(timings))
    });
    WasmBuildBatchAttempt {
        result,
        failure_phase,
        failure_timings,
    }
}

fn batch_source_assumptions(
    batch_resolution: Option<&(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
) -> (bool, Option<Arc<RwLock<bool>>>) {
    batch_resolution.map_or((false, None), |(resolver, _)| {
        (
            resolver.assumes_sources_immutable(),
            resolver.prepared_invalidation(),
        )
    })
}

fn build_wasm_canisters_cached_internal(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
    mut batch_resolution: Option<(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let total_started = Instant::now();
    validate_spec(spec)?;
    let (assumes_sources_immutable, prepared_invalidation) =
        batch_source_assumptions(batch_resolution.as_ref());
    progress.emit(WasmBuildProgressEvent::Started);
    if spec.shared_incremental_maintenance_config.is_some() {
        let outcome = build_wasm_canisters_cached_with_scheduled_shared_maintenance(
            spec,
            total_started,
            progress,
            batch_resolution.take(),
        )?;
        emit_finished_progress(&outcome, progress);
        return Ok(outcome);
    }
    let (cache_lock, first_lock_wait) =
        lock_wasm_build_cache_with_progress(&spec.target_dir, progress)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    let resolved = resolve_initial_inputs(spec, batch_resolution.take(), progress)?;
    let isolated_acquisition =
        SharedIncrementalAcquisitionContext::isolated(prepared_invalidation.clone());
    if let Some(outcome) = try_reuse_wasm_artifacts(
        spec,
        &resolved,
        first_lock_wait,
        &isolated_acquisition,
        total_started,
        progress,
    )? {
        emit_finished_progress(&outcome, progress);
        return Ok(outcome);
    }
    progress.emit(WasmBuildProgressEvent::CacheMiss {
        fingerprint: resolved.fingerprint,
    });

    let outcome = match &spec.cache_mode {
        WasmBuildCacheMode::Isolated => {
            let cache_entry = cache_entry_directory(spec, resolved.fingerprint);
            build_wasm_cache_miss(
                spec,
                resolved,
                first_lock_wait,
                isolated_acquisition,
                cache_entry,
                total_started,
                progress,
            )
        }
        WasmBuildCacheMode::SharedIncremental { .. } => {
            drop(cache_lock);
            build_wasm_with_shared_incremental(
                spec,
                resolved,
                first_lock_wait,
                assumes_sources_immutable,
                prepared_invalidation,
                total_started,
                progress,
            )
        }
    }?;
    emit_finished_progress(&outcome, progress);
    Ok(outcome)
}

fn build_wasm_with_shared_incremental(
    spec: &WasmBuildSpec,
    resolved: ResolvedCargoBuildInputs,
    first_lock_wait: Duration,
    assumes_sources_immutable: bool,
    prepared_invalidation: Option<Arc<RwLock<bool>>>,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let configured_target = shared_incremental_target(spec)
        .expect("shared cache mode must resolve a shared Cargo target");
    progress.emit(WasmBuildProgressEvent::SharedTargetLockStarted {
        target_dir: configured_target,
    });
    let (shared_lock, shared_lock_wait, shared_target) =
        lock_shared_incremental_target_with_progress(spec, progress)?;
    progress.emit(WasmBuildProgressEvent::SharedTargetLockAcquired {
        target_dir: shared_target.clone(),
        wait: shared_lock_wait,
    });
    let (_cache_lock, second_lock_wait) =
        lock_wasm_build_cache_with_progress(&spec.target_dir, progress)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    let current = if assumes_sources_immutable {
        resolved
    } else {
        let mut current = resolve_inputs_with_progress(spec, progress)?;
        current.timings.include(resolved.timings);
        current
    };
    let lock_wait = first_lock_wait.saturating_add(second_lock_wait);
    let shared_incremental =
        SharedIncrementalAcquisitionContext::shared(shared_lock_wait, None, prepared_invalidation);
    if let Some(outcome) = try_reuse_wasm_artifacts(
        spec,
        &current,
        lock_wait,
        &shared_incremental,
        total_started,
        progress,
    )? {
        return Ok(outcome);
    }

    let outcome = build_wasm_cache_miss(
        spec,
        current,
        lock_wait,
        shared_incremental,
        shared_target,
        total_started,
        progress,
    );
    drop(shared_lock);
    outcome
}

fn build_wasm_canisters_cached_with_scheduled_shared_maintenance(
    spec: &WasmBuildSpec,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
    batch_resolution: Option<(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let (_, prepared_invalidation) = batch_source_assumptions(batch_resolution.as_ref());
    let configured_target = shared_incremental_target(spec)
        .expect("validated scheduled maintenance must have a shared Cargo target");
    progress.emit(WasmBuildProgressEvent::SharedTargetLockStarted {
        target_dir: configured_target,
    });
    let (_shared_lock, shared_lock_wait, shared_target) =
        lock_shared_incremental_target_with_progress(spec, progress)?;
    progress.emit(WasmBuildProgressEvent::SharedTargetLockAcquired {
        target_dir: shared_target.clone(),
        wait: shared_lock_wait,
    });
    let (_cache_lock, lock_wait) = lock_wasm_build_cache_with_progress(&spec.target_dir, progress)?;
    ensure_cache_directory_tag(&spec.target_dir)?;

    // Resolution under both locks proves the target boundary once for the
    // scheduled retention pass and the following exact-cache acquisition.
    let resolved = resolve_initial_inputs(spec, batch_resolution, progress)?;
    let shared_maintenance = perform_configured_shared_incremental_target_maintenance(
        spec,
        &shared_target,
        shared_lock_wait,
        progress,
    )?;
    let shared_incremental = SharedIncrementalAcquisitionContext::shared(
        shared_lock_wait,
        Some(shared_maintenance),
        prepared_invalidation,
    );
    if let Some(outcome) = try_reuse_wasm_artifacts(
        spec,
        &resolved,
        lock_wait,
        &shared_incremental,
        total_started,
        progress,
    )? {
        return Ok(outcome);
    }
    progress.emit(WasmBuildProgressEvent::CacheMiss {
        fingerprint: resolved.fingerprint,
    });
    build_wasm_cache_miss(
        spec,
        resolved,
        lock_wait,
        shared_incremental,
        shared_target,
        total_started,
        progress,
    )
}

fn perform_configured_shared_incremental_target_maintenance(
    spec: &WasmBuildSpec,
    shared_target: &Path,
    lock_wait: Duration,
    progress: &mut ProgressReporter<'_>,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    let config = spec
        .shared_incremental_maintenance_config
        .expect("configured shared-target maintenance must have settings");
    progress.emit(WasmBuildProgressEvent::SharedTargetMaintenanceStarted {
        target_dir: shared_target.to_owned(),
    });
    let result = progress.run_phase(WasmBuildProgressPhase::SharedTargetMaintenance, || {
        let schedule = schedule_shared_incremental_target_maintenance(
            shared_target,
            config.policy,
            config.minimum_interval,
            lock_wait,
        )?;
        match schedule {
            SharedIncrementalTargetMaintenanceSchedule::Skipped(outcome) => Ok(outcome),
            SharedIncrementalTargetMaintenanceSchedule::Due(due) => {
                perform_due_shared_incremental_target_maintenance(
                    shared_target,
                    config.policy,
                    lock_wait,
                    due,
                )
            }
        }
    });
    let outcome = integrated_shared_maintenance_result(config, shared_target, lock_wait, result)?;
    progress.emit(WasmBuildProgressEvent::SharedTargetMaintenanceFinished {
        outcome: outcome.clone(),
    });
    Ok(outcome)
}

fn integrated_shared_maintenance_result(
    config: SharedIncrementalTargetMaintenanceConfig,
    shared_target: &Path,
    lock_wait: Duration,
    result: Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError>,
) -> Result<SharedIncrementalTargetMaintenanceOutcome, WasmBuildError> {
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error)
            if config.failure_mode == SharedIncrementalTargetMaintenanceFailureMode::BestEffort =>
        {
            Ok(SharedIncrementalTargetMaintenanceOutcome::Failed {
                target_dir: shared_target.to_owned(),
                lock_wait,
                message: error.to_string(),
            })
        }
        Err(error) => Err(error),
    }
}

fn resolve_inputs_with_progress(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let resolved = build_fingerprint_with_progress(spec, progress)?;
    progress.emit(WasmBuildProgressEvent::InputsResolved {
        fingerprint: resolved.fingerprint,
        input_digest: resolved.input_digest,
        elapsed: resolved.timings.total,
    });
    Ok(resolved)
}

fn resolve_initial_inputs(
    spec: &WasmBuildSpec,
    batch_resolution: Option<(&mut WasmBuildBatchInputResolver<'_, '_>, usize)>,
    progress: &mut ProgressReporter<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let resolved = if let Some((resolver, index)) = batch_resolution {
        resolver.resolve(index, progress)?
    } else {
        build_fingerprint_with_progress(spec, progress)?
    };
    progress.emit(WasmBuildProgressEvent::InputsResolved {
        fingerprint: resolved.fingerprint,
        input_digest: resolved.input_digest,
        elapsed: resolved.timings.total,
    });
    Ok(resolved)
}

fn emit_finished_progress(outcome: &WasmBuildOutcome, progress: &mut ProgressReporter<'_>) {
    let state = if outcome.is_reused() {
        progress.emit(WasmBuildProgressEvent::CacheHit {
            fingerprint: outcome.record().fingerprint,
        });
        WasmBuildProgressOutcome::Reused
    } else {
        WasmBuildProgressOutcome::Built
    };
    progress.emit(WasmBuildProgressEvent::Finished {
        outcome: state,
        fingerprint: outcome.record().fingerprint,
        elapsed: outcome.record().timings.total,
    });
}

#[derive(Clone, Debug, Default)]
struct SharedIncrementalAcquisitionContext {
    lock_wait: Option<Duration>,
    maintenance: Option<SharedIncrementalTargetMaintenanceOutcome>,
    prepared_invalidation: Option<Arc<RwLock<bool>>>,
}

impl SharedIncrementalAcquisitionContext {
    fn isolated(prepared_invalidation: Option<Arc<RwLock<bool>>>) -> Self {
        Self {
            prepared_invalidation,
            ..Self::default()
        }
    }

    const fn shared(
        lock_wait: Duration,
        maintenance: Option<SharedIncrementalTargetMaintenanceOutcome>,
        prepared_invalidation: Option<Arc<RwLock<bool>>>,
    ) -> Self {
        Self {
            lock_wait: Some(lock_wait),
            maintenance,
            prepared_invalidation,
        }
    }

    fn lock_prepared_publication(
        &self,
    ) -> Result<Option<std::sync::RwLockReadGuard<'_, bool>>, WasmBuildError> {
        let guard = self.prepared_invalidation.as_deref().map(|invalidation| {
            invalidation
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        if guard.as_deref().is_some_and(|invalidated| *invalidated) {
            return Err(WasmBuildError::PreparedInputSnapshotInvalidated);
        }
        Ok(guard)
    }
}

fn try_reuse_wasm_artifacts(
    spec: &WasmBuildSpec,
    resolved: &ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental: &SharedIncrementalAcquisitionContext,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
) -> Result<Option<WasmBuildOutcome>, WasmBuildError> {
    let _publication_guard = shared_incremental.lock_prepared_publication()?;
    let fingerprint = resolved.fingerprint;
    let artifacts = expected_artifacts(spec, &spec.target_dir);
    let cache_entry = cache_entry_directory(spec, fingerprint);
    let artifacts_match = progress.run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
        artifact_set_matches(&artifacts, fingerprint)
    });
    if artifacts_match {
        ensure_exact_cache_entry(spec, &artifacts, &cache_entry, fingerprint, progress)?;
        return Ok(Some(WasmBuildOutcome::Reused(complete_build_record(
            spec,
            BuildRecordInput {
                fingerprint,
                input_digest: resolved.input_digest,
                artifacts,
                lock_wait,
                shared_incremental: shared_incremental.clone(),
                input_resolution: resolved.timings,
                cargo_build: None,
                active_entry: &cache_entry,
            },
            total_started,
            progress,
        ))));
    }

    let cached_artifacts = expected_artifacts(spec, &cache_entry);
    let cached_artifacts_match = progress
        .run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
            artifact_set_matches(&cached_artifacts, fingerprint)
        });
    if !cached_artifacts_match {
        return Ok(None);
    }
    progress.run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
        materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
        record_cache_entry_use(&cache_entry)
    })?;
    Ok(Some(WasmBuildOutcome::Reused(complete_build_record(
        spec,
        BuildRecordInput {
            fingerprint,
            input_digest: resolved.input_digest,
            artifacts,
            lock_wait,
            shared_incremental: shared_incremental.clone(),
            input_resolution: resolved.timings,
            cargo_build: None,
            active_entry: &cache_entry,
        },
        total_started,
        progress,
    ))))
}

fn ensure_exact_cache_entry(
    spec: &WasmBuildSpec,
    artifacts: &[PathBuf],
    cache_entry: &Path,
    fingerprint: InputDigest,
    progress: &mut ProgressReporter<'_>,
) -> Result<(), WasmBuildError> {
    let cached_artifacts = expected_artifacts(spec, cache_entry);
    let entry_is_current =
        progress.run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
            if artifact_set_matches(&cached_artifacts, fingerprint) {
                record_cache_entry_use(cache_entry)?;
                Ok::<_, WasmBuildError>(true)
            } else {
                Ok(false)
            }
        })?;
    if entry_is_current {
        return Ok(());
    }
    progress.run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
        remove_directory_if_present(cache_entry)?;
        create_dir_all(
            cache_entry,
            "create content-addressed Cargo target directory",
        )
    })?;
    let incomplete = IncompleteBuildDirectory::new(cache_entry.to_owned());
    let result = progress.run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
        copy_wasm_artifacts(artifacts, &cached_artifacts)?;
        publish_artifact_stamps(&cached_artifacts, fingerprint)?;
        record_cache_entry_use(cache_entry)
    });
    match result {
        Ok(()) => {
            incomplete.preserve();
            Ok(())
        }
        Err(build_error) => Err(cleanup_failed_fingerprint_build(
            build_error,
            incomplete,
            progress,
        )),
    }
}

fn build_wasm_cache_miss(
    spec: &WasmBuildSpec,
    resolved: ResolvedCargoBuildInputs,
    lock_wait: Duration,
    shared_incremental: SharedIncrementalAcquisitionContext,
    cargo_target_dir: PathBuf,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    let fingerprint = resolved.fingerprint;
    let mut input_resolution = resolved.timings;
    let artifacts = expected_artifacts(spec, &spec.target_dir);
    let cache_entry = cache_entry_directory(spec, fingerprint);
    let preparation_started = Instant::now();
    progress.begin_phase(WasmBuildFailurePhase::ArtifactPublication);
    let preparation_result = (|| {
        remove_directory_if_present(&cache_entry)?;
        create_dir_all(
            &cache_entry,
            "create content-addressed Cargo target directory",
        )
    })();
    progress.record_phase(
        WasmBuildFailurePhase::ArtifactPublication,
        preparation_started.elapsed(),
    );
    preparation_result?;
    let incomplete_directory = IncompleteBuildDirectory::new(cache_entry.clone());
    let build_result = (|| {
        if matches!(
            spec.cache_mode,
            WasmBuildCacheMode::SharedIncremental { .. }
        ) {
            record_cache_entry_use(&cargo_target_dir)?;
        }
        let build_started = Instant::now();
        progress.begin_phase(WasmBuildFailurePhase::CargoBuild);
        let cargo_result = run_cargo_build(spec, &cargo_target_dir, progress);
        let cargo_build = build_started.elapsed();
        progress.record_phase(WasmBuildFailurePhase::CargoBuild, cargo_build);
        cargo_result?;
        let built_artifacts = expected_artifacts(spec, &cargo_target_dir);
        let validation_started = Instant::now();
        progress.begin_phase(WasmBuildFailurePhase::ArtifactPublication);
        let missing = missing_artifacts(&built_artifacts);
        progress.record_phase(
            WasmBuildFailurePhase::ArtifactPublication,
            validation_started.elapsed(),
        );
        if !missing.is_empty() {
            return Err(WasmBuildError::MissingArtifacts { paths: missing });
        }

        let verified = resolve_inputs_with_progress(spec, progress)?;
        input_resolution.include(verified.timings);
        if resolved.validation_digest != verified.validation_digest {
            return Err(WasmBuildError::InputsChangedDuringBuild {
                before: resolved.validation_digest,
                after: verified.validation_digest,
            });
        }
        if fingerprint != verified.fingerprint {
            return Err(WasmBuildError::InputsChangedDuringBuild {
                before: fingerprint,
                after: verified.fingerprint,
            });
        }

        // Publication is the prepared snapshot's linearization boundary. A
        // reader that reaches it first may finish publishing; invalidation
        // takes the write side of this lock and therefore precedes every later
        // reader without racing a successful stamp into existence.
        let publication_guard = shared_incremental.lock_prepared_publication()?;

        let cached_artifacts = expected_artifacts(spec, &cache_entry);
        progress.run_phase(WasmBuildProgressPhase::ArtifactPublication, || {
            if cargo_target_dir != cache_entry {
                copy_wasm_artifacts(&built_artifacts, &cached_artifacts)?;
            }
            publish_artifact_stamps(&cached_artifacts, fingerprint)?;
            materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
            record_cache_entry_use(&cache_entry)
        })?;
        drop(publication_guard);

        Ok(WasmBuildOutcome::Built(complete_build_record(
            spec,
            BuildRecordInput {
                fingerprint,
                input_digest: resolved.input_digest,
                artifacts,
                lock_wait,
                shared_incremental,
                input_resolution,
                cargo_build: Some(cargo_build),
                active_entry: &cache_entry,
            },
            total_started,
            progress,
        )))
    })();
    finish_fingerprint_build(build_result, incomplete_directory, progress)
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
    policy: ArtifactCachePrunePolicy,
) -> Result<ArtifactCachePruneReport, WasmBuildError> {
    let (_lock_file, _) = lock_wasm_build_cache(target_dir)?;
    ensure_cache_directory_tag(target_dir)?;

    prune_wasm_build_cache_locked(target_dir, policy, None)
}

struct BuildRecordInput<'a> {
    fingerprint: InputDigest,
    input_digest: InputDigest,
    artifacts: Vec<PathBuf>,
    lock_wait: Duration,
    shared_incremental: SharedIncrementalAcquisitionContext,
    input_resolution: WasmInputResolutionTimings,
    cargo_build: Option<Duration>,
    active_entry: &'a Path,
}

fn complete_build_record(
    spec: &WasmBuildSpec,
    input: BuildRecordInput<'_>,
    total_started: Instant,
    progress: &mut ProgressReporter<'_>,
) -> WasmBuildRecord {
    let (maintenance, cache_maintenance) = spec.prune_policy.map_or((None, None), |policy| {
        progress.run_phase(WasmBuildProgressPhase::ExactCacheMaintenance, || {
            let cache_root = spec.target_dir.join(".ic-testkit/wasm-targets");
            let identity = policy.maintenance_identity();
            perform_scheduled_cache_maintenance(&cache_root, spec.prune_interval, &identity, || {
                prune_wasm_build_cache_locked(&spec.target_dir, policy, Some(input.active_entry))
                    .map_err(|error| error.to_string())
            })
        })
    });
    WasmBuildRecord {
        fingerprint: input.fingerprint,
        input_digest: input.input_digest,
        exact_cache_path: input.active_entry.to_owned(),
        artifacts: input.artifacts,
        timings: WasmBuildTimings {
            lock_wait: input.lock_wait,
            shared_incremental_lock_wait: input.shared_incremental.lock_wait,
            input_resolution: input.input_resolution,
            cargo_build: input.cargo_build,
            cache_maintenance,
            total: total_started.elapsed(),
        },
        maintenance,
        shared_incremental_maintenance: input.shared_incremental.maintenance,
    }
}

fn prune_wasm_build_cache_locked(
    target_dir: &Path,
    policy: ArtifactCachePrunePolicy,
    protected_entry: Option<&Path>,
) -> Result<ArtifactCachePruneReport, WasmBuildError> {
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
    progress: &mut ProgressReporter<'_>,
) -> Result<WasmBuildOutcome, WasmBuildError> {
    match result {
        Ok(outcome) => {
            incomplete_directory.preserve();
            Ok(outcome)
        }
        Err(build_error) => Err(cleanup_failed_fingerprint_build(
            build_error,
            incomplete_directory,
            progress,
        )),
    }
}

fn cleanup_failed_fingerprint_build(
    build_error: WasmBuildError,
    incomplete_directory: IncompleteBuildDirectory,
    progress: &mut ProgressReporter<'_>,
) -> WasmBuildError {
    let path = incomplete_directory.path.clone();
    let primary_phase = progress.failure_phase;
    let cleanup_started = Instant::now();
    let cleanup = incomplete_directory.cleanup();
    progress.record_phase(WasmBuildFailurePhase::Cleanup, cleanup_started.elapsed());
    match cleanup {
        Ok(()) => {
            progress.failure_phase = primary_phase;
            build_error
        }
        Err(source) => WasmBuildError::FailedBuildCleanup {
            build_error: Box::new(build_error),
            path,
            source,
        },
    }
}

fn lock_wasm_build_cache(target_dir: &Path) -> Result<(File, Duration), WasmBuildError> {
    create_dir_all(target_dir, "create Cargo target directory")?;
    let lock_path = target_dir.join(".ic-testkit/wasm-build.lock");
    lock_cache_file(&lock_path).map_err(wasm_cache_fs_error)
}

fn lock_wasm_build_cache_with_progress(
    target_dir: &Path,
    progress: &mut ProgressReporter<'_>,
) -> Result<(File, Duration), WasmBuildError> {
    progress.begin_phase(WasmBuildFailurePhase::ExactCacheCoordination);
    create_dir_all(target_dir, "create Cargo target directory")?;
    let lock_path = target_dir.join(".ic-testkit/wasm-build.lock");
    lock_cache_file_with_progress(&lock_path, WasmBuildProgressPhase::ExactCacheLock, progress)
}

fn lock_shared_incremental_target(
    spec: &WasmBuildSpec,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    lock_shared_incremental_target_internal(spec, None)
}

fn lock_shared_incremental_target_with_progress(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    lock_shared_incremental_target_internal(spec, Some(progress))
}

fn lock_shared_incremental_target_internal(
    spec: &WasmBuildSpec,
    mut progress: Option<&mut ProgressReporter<'_>>,
) -> Result<(File, Duration, PathBuf), WasmBuildError> {
    if let Some(progress) = progress.as_deref_mut() {
        progress.begin_phase(WasmBuildFailurePhase::SharedTargetCoordination);
    }
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
    let (lock, wait) = if let Some(progress) = progress {
        lock_cache_file_with_progress(
            &lock_path,
            WasmBuildProgressPhase::SharedTargetLock,
            progress,
        )?
    } else {
        lock_cache_file(&lock_path).map_err(wasm_cache_fs_error)?
    };
    Ok((lock, wait, canonical))
}

fn lock_cache_file_with_progress(
    lock_path: &Path,
    phase: WasmBuildProgressPhase,
    progress: &mut ProgressReporter<'_>,
) -> Result<(File, Duration), WasmBuildError> {
    let failure_phase = progress_failure_phase(phase);
    let started = Instant::now();
    progress.begin_phase(failure_phase);
    let result = if !progress.is_observed() || progress.config.heartbeat_interval.is_none() {
        lock_cache_file(lock_path).map_err(wasm_cache_fs_error)
    } else {
        let heartbeat_interval = progress
            .config
            .heartbeat_interval
            .expect("observed cache lock must have a heartbeat interval");
        lock_cache_file_with_wait_observer(lock_path, heartbeat_interval, |elapsed| {
            progress.emit_heartbeat_if_due(phase, elapsed);
        })
        .map_err(wasm_cache_fs_error)
    };
    progress.record_phase(failure_phase, started.elapsed());
    result
}

fn ensure_cache_directory_tag(target_dir: &Path) -> Result<(), WasmBuildError> {
    ensure_cache_tag(target_dir).map_err(wasm_cache_fs_error)
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
    if spec.shared_incremental_maintenance_config.is_some()
        && !matches!(
            spec.cache_mode,
            WasmBuildCacheMode::SharedIncremental { .. }
        )
    {
        return Err(WasmBuildError::InvalidSpec {
            message:
                "scheduled shared-target maintenance requires a shared incremental Cargo target"
                    .to_owned(),
        });
    }
    Ok(())
}

fn build_fingerprint(spec: &WasmBuildSpec) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    build_fingerprint_with_progress(spec, &mut ProgressReporter::silent())
}

fn build_fingerprint_with_progress(
    spec: &WasmBuildSpec,
    progress: &mut ProgressReporter<'_>,
) -> Result<ResolvedCargoBuildInputs, WasmBuildError> {
    let total_started = Instant::now();
    let tool_started = Instant::now();
    let cargo_identity = progress.run_phase(WasmBuildProgressPhase::CargoIdentity, || {
        command_identity(
            spec,
            WasmBuildPhase::CargoIdentity,
            &spec.cargo_program,
            &["--version", "--verbose"],
        )
    })?;
    let rustc_program = spec
        .extra_env
        .get(OsStr::new("RUSTC"))
        .unwrap_or(&spec.rustc_program);
    let rustc_identity = progress.run_phase(WasmBuildProgressPhase::RustcIdentity, || {
        command_identity(spec, WasmBuildPhase::RustcIdentity, rustc_program, &["-vV"])
    })?;
    let tool_identity = tool_started.elapsed();

    let metadata_started = Instant::now();
    let metadata = progress.run_phase(WasmBuildProgressPhase::CargoMetadata, || {
        cargo_metadata(spec)
    })?;
    let cargo_metadata = metadata_started.elapsed();

    let discovery_started = Instant::now();
    let (inputs, exclusions) =
        progress.run_phase(WasmBuildProgressPhase::InputDiscovery, || {
            let inputs = resolve_local_inputs(spec, &metadata)?;
            validate_shared_incremental_target_boundary(spec, &inputs.validation_inputs)?;
            let exclusions = source_exclusions(spec, &inputs.validation_inputs);
            Ok::<_, WasmBuildError>((inputs, exclusions))
        })?;
    let input_discovery = discovery_started.elapsed();

    let hashing_started = Instant::now();
    let (input_digest, validation_digest) =
        progress.run_phase(WasmBuildProgressPhase::ContentHashing, || {
            let mut cache = LabeledPathDigestCache::default();
            digest_resolved_local_inputs(
                &inputs,
                &exclusions,
                &mut cache,
                &spec.workspace_root,
                "hash Wasm build inputs",
                "hash semantic Wasm build inputs",
            )
        })?;
    let content_hashing = hashing_started.elapsed();

    let fingerprint =
        finish_build_fingerprint(spec, &cargo_identity, &rustc_identity, input_digest);
    Ok(ResolvedCargoBuildInputs {
        fingerprint,
        input_digest,
        validation_digest,
        inputs: inputs
            .validation_inputs
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

fn finish_build_fingerprint(
    spec: &WasmBuildSpec,
    cargo_identity: &[u8],
    rustc_identity: &[u8],
    input_digest: InputDigest,
) -> InputDigest {
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
    hasher.field("cargo-identity", cargo_identity);
    hasher.field("rustc-identity", rustc_identity);
    hasher.field("source-input-digest", input_digest.as_bytes());
    hasher.finish()
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
    source: Option<String>,
    semantic_fields: Vec<(&'static str, Option<String>)>,
}

const SEMANTIC_PACKAGE_FIELDS: &[&str] = &[
    "authors",
    "default_run",
    "description",
    "documentation",
    "edition",
    "homepage",
    "license",
    "license_file",
    "links",
    "metadata",
    "name",
    "readme",
    "repository",
    "rust_version",
    "version",
];

struct LockedPackageIdentity {
    name: String,
    version: String,
    source: String,
    checksum: Option<String>,
}

fn resolve_local_inputs(
    spec: &WasmBuildSpec,
    metadata: &Value,
) -> Result<ResolvedLocalInputs, WasmBuildError> {
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
    let projection = semantic_workspace_projection(metadata, &packages, &closure, &workspace_root)?;
    let mut validation_inputs = workspace_configuration_inputs(spec, &workspace_root)?;
    append_package_inputs(&mut validation_inputs, &packages, closure, &workspace_root)?;
    append_additional_inputs(&mut validation_inputs, spec, &workspace_root);
    let fingerprint = projection.map_or(LocalInputFingerprint::Conservative, |workspace| {
        LocalInputFingerprint::Projected {
            inputs: validation_inputs
                .iter()
                .filter(|(label, _)| !is_broad_workspace_input(label))
                .cloned()
                .collect(),
            workspace,
        }
    });
    Ok(ResolvedLocalInputs {
        validation_inputs,
        fingerprint,
    })
}

fn metadata_packages(metadata: &Value) -> Result<HashMap<String, MetadataPackage>, WasmBuildError> {
    let packages_value = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata has no package array"))?;
    let mut packages = HashMap::new();
    for value in packages_value {
        let source = optional_string(value, "source")?;
        let package = MetadataPackage {
            id: required_string(value, "id")?,
            name: required_string(value, "name")?,
            version: required_string(value, "version")?,
            manifest_path: PathBuf::from(required_string(value, "manifest_path")?),
            is_local: value.get("source").is_some_and(Value::is_null),
            source,
            semantic_fields: SEMANTIC_PACKAGE_FIELDS
                .iter()
                .map(|field| (*field, value.get(*field).map(Value::to_string)))
                .collect(),
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

fn semantic_workspace_projection(
    metadata: &Value,
    packages: &HashMap<String, MetadataPackage>,
    closure: &BTreeSet<String>,
    workspace_root: &Path,
) -> Result<Option<InputDigest>, WasmBuildError> {
    // A workspace-root or external local package cannot be separated from the
    // broad root safely; `None` keeps the complete-input fingerprint.
    let locked_packages = locked_package_identities(workspace_root)?;
    let mut identities = HashMap::new();
    for id in closure {
        let package = packages
            .get(id)
            .ok_or_else(|| invalid_metadata(&format!("resolved package `{id}` is missing")))?;
        let Some(identity) = semantic_package_identity(package, workspace_root, &locked_packages)
        else {
            return Ok(None);
        };
        identities.insert(id.as_str(), identity);
    }

    let nodes = metadata
        .pointer("/resolve/nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata has no resolved dependency nodes"))?;
    let nodes_by_id = nodes
        .iter()
        .map(|node| Ok((required_string(node, "id")?, node)))
        .collect::<Result<HashMap<_, _>, WasmBuildError>>()?;
    let mut projected_packages = closure
        .iter()
        .map(|id| {
            let package = packages
                .get(id)
                .expect("selected package closure was validated above");
            let identity = identities[id.as_str()];
            let node = nodes_by_id.get(id).copied().ok_or_else(|| {
                invalid_metadata(&format!("resolved package `{id}` has no dependency node"))
            })?;
            let projection = semantic_package_projection(package, node, &identities)?;
            Ok::<_, WasmBuildError>((identity, projection))
        })
        .collect::<Result<Vec<_>, _>>()?;
    projected_packages.sort_by_key(|(identity, _)| *identity);

    let root_manifest = workspace_root.join("Cargo.toml");
    let root_contents =
        fs::read_to_string(&root_manifest).map_err(|source| WasmBuildError::Io {
            operation: "read workspace manifest for semantic projection",
            path: root_manifest.clone(),
            source,
        })?;
    let root = toml::from_str::<TomlValue>(&root_contents).map_err(|error| {
        invalid_metadata(&format!(
            "workspace manifest could not be projected as TOML: {error}"
        ))
    })?;

    let mut hasher = InputHasher::new("wasm-semantic-workspace-projection-v1");
    for (identity, projection) in projected_packages {
        hasher.field("package-identity", identity.as_bytes());
        hasher.field("package-projection", projection.as_bytes());
    }
    hash_toml_setting(&mut hasher, "cargo-features", root.get("cargo-features"));
    hash_toml_setting(&mut hasher, "profile", root.get("profile"));
    let workspace = root.get("workspace").and_then(TomlValue::as_table);
    hash_toml_setting(
        &mut hasher,
        "workspace-resolver",
        workspace.and_then(|table| table.get("resolver")),
    );
    hash_toml_setting(
        &mut hasher,
        "workspace-lints",
        workspace.and_then(|table| table.get("lints")),
    );
    Ok(Some(hasher.finish()))
}

fn locked_package_identities(
    workspace_root: &Path,
) -> Result<Vec<LockedPackageIdentity>, WasmBuildError> {
    let lockfile = workspace_root.join("Cargo.lock");
    let contents = match fs::read_to_string(&lockfile) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(WasmBuildError::Io {
                operation: "read Cargo lockfile for semantic projection",
                path: lockfile,
                source,
            });
        }
    };
    let lock = toml::from_str::<TomlValue>(&contents).map_err(|error| {
        invalid_metadata(&format!(
            "Cargo lockfile could not be projected as TOML: {error}"
        ))
    })?;
    let Some(packages) = lock.get("package").and_then(TomlValue::as_array) else {
        return Ok(Vec::new());
    };
    packages
        .iter()
        .filter_map(|package| {
            let Some(table) = package.as_table() else {
                return Some(Err(invalid_metadata(
                    "Cargo lockfile package entry is not a table",
                )));
            };
            let source = table.get("source")?.as_str().map(str::to_owned);
            Some(
                source
                    .ok_or_else(|| {
                        invalid_metadata("Cargo lockfile package source is not a string")
                    })
                    .and_then(|source| {
                        Ok(LockedPackageIdentity {
                            name: required_toml_string(table, "name", "Cargo lockfile package")?,
                            version: required_toml_string(
                                table,
                                "version",
                                "Cargo lockfile package",
                            )?,
                            source,
                            checksum: optional_toml_string(
                                table,
                                "checksum",
                                "Cargo lockfile package",
                            )?,
                        })
                    }),
            )
        })
        .collect()
}

fn required_toml_string(
    table: &toml::Table,
    field: &str,
    context: &str,
) -> Result<String, WasmBuildError> {
    table
        .get(field)
        .and_then(TomlValue::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid_metadata(&format!("{context} `{field}` is missing or not a string")))
}

fn optional_toml_string(
    table: &toml::Table,
    field: &str,
    context: &str,
) -> Result<Option<String>, WasmBuildError> {
    match table.get(field) {
        None => Ok(None),
        Some(TomlValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(invalid_metadata(&format!(
            "{context} `{field}` is not a string"
        ))),
    }
}

fn semantic_package_identity(
    package: &MetadataPackage,
    workspace_root: &Path,
    locked_packages: &[LockedPackageIdentity],
) -> Option<InputDigest> {
    let mut hasher = InputHasher::new("wasm-semantic-package-identity-v1");
    hasher.field("name", package.name.as_bytes());
    hasher.field("version", package.version.as_bytes());
    if package.is_local {
        let manifest = package.manifest_path.strip_prefix(workspace_root).ok()?;
        let package_root = package.manifest_path.parent()?;
        if package_root == workspace_root {
            return None;
        }
        hasher.field("local-manifest", &os_bytes(manifest.as_os_str()));
    } else {
        let metadata_source = package.source.as_deref()?;
        let locked = locked_packages.iter().find(|locked| {
            locked.name == package.name
                && locked.version == package.version
                && locked.source == metadata_source
        })?;
        match locked.source.as_str() {
            source if source.starts_with("registry+") && locked.checksum.is_some() => {}
            source if source.starts_with("git+") && source.contains('#') => {}
            _ => return None,
        }
        hasher.field("external-package-id", package.id.as_bytes());
        hasher.field("external-source", locked.source.as_bytes());
        hasher.field(
            "external-checksum",
            locked.checksum.as_deref().unwrap_or_default().as_bytes(),
        );
    }
    Some(hasher.finish())
}

fn semantic_package_projection(
    package: &MetadataPackage,
    node: &Value,
    identities: &HashMap<&str, InputDigest>,
) -> Result<InputDigest, WasmBuildError> {
    // These are the effective package values Cargo can expose to compilation
    // through CARGO_PKG_* variables. Local manifests and external checksums
    // cover the remaining package definition.
    let mut hasher = InputHasher::new("wasm-semantic-package-projection-v1");
    for (field, value) in &package.semantic_fields {
        hasher.field("package-field-name", field.as_bytes());
        match value {
            Some(value) => hasher.field("package-field-value", value.as_bytes()),
            None => hasher.field("package-field-missing", b""),
        }
    }

    let mut features = node
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata dependency node has no features array"))?
        .iter()
        .map(|feature| {
            feature.as_str().map(str::to_owned).ok_or_else(|| {
                invalid_metadata("Cargo metadata dependency feature is not a string")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    features.sort();
    for feature in features {
        hasher.field("enabled-feature", feature.as_bytes());
    }

    let mut dependencies = node
        .get("deps")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_metadata("Cargo metadata dependency node has no deps array"))?
        .iter()
        .map(|dependency| {
            let name = required_string(dependency, "name")?;
            let package_id = required_string(dependency, "pkg")?;
            let identity = identities
                .get(package_id.as_str())
                .copied()
                .ok_or_else(|| {
                    invalid_metadata(&format!(
                        "dependency `{package_id}` is outside the selected package closure"
                    ))
                })?;
            let kinds = dependency
                .get("dep_kinds")
                .ok_or_else(|| invalid_metadata("Cargo metadata dependency has no kind array"))?
                .to_string();
            Ok::<_, WasmBuildError>((name, identity, kinds))
        })
        .collect::<Result<Vec<_>, _>>()?;
    dependencies.sort();
    for (name, identity, kinds) in dependencies {
        hasher.field("dependency-name", name.as_bytes());
        hasher.field("dependency-identity", identity.as_bytes());
        hasher.field("dependency-kinds", kinds.as_bytes());
    }
    Ok(hasher.finish())
}

fn hash_toml_setting(hasher: &mut InputHasher, label: &str, value: Option<&TomlValue>) {
    hasher.field("workspace-setting-name", label.as_bytes());
    match value {
        Some(value) => hasher.field("workspace-setting-value", value.to_string().as_bytes()),
        None => hasher.field("workspace-setting-missing", b""),
    }
}

fn is_broad_workspace_input(label: &Path) -> bool {
    label == Path::new("workspace/Cargo.toml") || label == Path::new("workspace/Cargo.lock")
}

fn digest_resolved_local_inputs(
    inputs: &ResolvedLocalInputs,
    exclusions: &[PathBuf],
    cache: &mut LabeledPathDigestCache,
    error_path: &Path,
    validation_operation: &'static str,
    semantic_operation: &'static str,
) -> Result<(InputDigest, InputDigest), WasmBuildError> {
    let validation_digest = digest_labeled_paths_composable(
        "wasm-source-inputs-v1",
        &inputs.validation_inputs,
        exclusions,
        cache,
    )
    .map_err(|source| WasmBuildError::Io {
        operation: validation_operation,
        path: error_path.to_owned(),
        source,
    })?;
    let input_digest = semantic_input_digest(inputs, validation_digest, exclusions, cache)
        .map_err(|source| WasmBuildError::Io {
            operation: semantic_operation,
            path: error_path.to_owned(),
            source,
        })?;
    Ok((input_digest, validation_digest))
}

fn semantic_input_digest(
    inputs: &ResolvedLocalInputs,
    validation_digest: InputDigest,
    exclusions: &[PathBuf],
    cache: &mut LabeledPathDigestCache,
) -> io::Result<InputDigest> {
    let LocalInputFingerprint::Projected {
        inputs: fingerprint_inputs,
        workspace,
    } = &inputs.fingerprint
    else {
        return Ok(validation_digest);
    };
    let path_digest = digest_labeled_paths_composable(
        "wasm-source-inputs-v1",
        fingerprint_inputs,
        exclusions,
        cache,
    )?;
    let mut hasher = InputHasher::new("wasm-semantic-source-inputs-v1");
    hasher.field("path-input-digest", path_digest.as_bytes());
    hasher.field("workspace-projection", workspace.as_bytes());
    Ok(hasher.finish())
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
    let resolved_inputs = inputs
        .iter()
        .map(|(_, input)| {
            let canonical = input.canonicalize().map_err(|source| WasmBuildError::Io {
                operation: "resolve Cargo input boundary",
                path: input.clone(),
                source,
            })?;
            let metadata = fs::metadata(&canonical).map_err(|source| WasmBuildError::Io {
                operation: "inspect Cargo input boundary",
                path: canonical.clone(),
                source,
            })?;
            Ok((canonical, metadata.is_dir()))
        })
        .collect::<Result<Vec<_>, WasmBuildError>>()?;
    let safe_generated_roots = std::iter::once(spec.target_dir.clone())
        .chain(std::iter::once(spec.workspace_root.join("target")))
        .chain(
            inputs
                .iter()
                .filter(|(_, path)| path.is_dir())
                .map(|(_, path)| path.join("target")),
        )
        .filter_map(|path| canonicalize_allow_missing(&path).ok())
        .filter(|root| {
            !resolved_inputs
                .iter()
                .any(|(input, _is_directory)| input.starts_with(root))
        })
        .collect::<Vec<_>>();
    if safe_generated_roots
        .iter()
        .any(|root| shared_target.starts_with(root))
    {
        return Ok(());
    }

    for (input, is_directory) in resolved_inputs {
        if shared_target == input
            || (is_directory && shared_target.starts_with(&input))
            || input.starts_with(&shared_target)
        {
            return Err(WasmBuildError::InvalidSpec {
                message: format!(
                    "shared incremental target {} must not overlap exact Cargo inputs unless it is inside a generated target directory",
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

fn shared_incremental_target_exists(
    spec: &WasmBuildSpec,
    operation: &'static str,
) -> Result<bool, WasmBuildError> {
    let target_dir =
        shared_incremental_target(spec).ok_or_else(|| WasmBuildError::InvalidSpec {
            message: "shared incremental target is not configured".to_owned(),
        })?;
    match fs::symlink_metadata(&target_dir) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(WasmBuildError::InvalidSpec {
            message: format!(
                "shared incremental Cargo target {} must be a directory",
                target_dir.display()
            ),
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(WasmBuildError::Io {
            operation,
            path: target_dir,
            source,
        }),
    }
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

fn run_cargo_build(
    spec: &WasmBuildSpec,
    build_target_dir: &Path,
    progress: &mut ProgressReporter<'_>,
) -> Result<(), WasmBuildError> {
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

    if !progress.is_observed() {
        let output = command
            .output()
            .map_err(|source| WasmBuildError::CommandSpawn {
                phase: WasmBuildPhase::CargoBuild,
                program: spec.cargo_program.clone(),
                source,
            })?;
        return ensure_command_success(WasmBuildPhase::CargoBuild, output).map(|_| ());
    }

    run_observed_cargo_build(spec, build_target_dir, command, progress)
}

fn run_observed_cargo_build(
    spec: &WasmBuildSpec,
    build_target_dir: &Path,
    mut command: Command,
    progress: &mut ProgressReporter<'_>,
) -> Result<(), WasmBuildError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let started = Instant::now();
    let child = command
        .spawn()
        .map_err(|source| WasmBuildError::CommandSpawn {
            phase: WasmBuildPhase::CargoBuild,
            program: spec.cargo_program.clone(),
            source,
        })?;
    let mut child = ObservedChild::new(child);
    progress.emit(WasmBuildProgressEvent::CargoStarted {
        target_dir: build_target_dir.to_owned(),
    });

    let stdout = child
        .child_mut()
        .stdout
        .take()
        .expect("Cargo stdout must be piped");
    let stderr = child
        .child_mut()
        .stderr
        .take()
        .expect("Cargo stderr must be piped");
    let (sender, chunks) = mpsc::channel();
    let stdout_sender = sender.clone();
    let stdout_reader = thread::spawn(move || {
        read_process_output(stdout, WasmBuildOutputStream::Stdout, stdout_sender)
    });
    let stderr_reader =
        thread::spawn(move || read_process_output(stderr, WasmBuildOutputStream::Stderr, sender));

    let captured = capture_observed_cargo_output(chunks, progress, started);

    let status = child.wait().map_err(|source| WasmBuildError::Io {
        operation: "wait for observed cargo build",
        path: PathBuf::from(&spec.cargo_program),
        source,
    })?;
    join_output_reader(
        stdout_reader,
        "read observed cargo stdout",
        &spec.cargo_program,
    )?;
    join_output_reader(
        stderr_reader,
        "read observed cargo stderr",
        &spec.cargo_program,
    )?;
    let elapsed = started.elapsed();
    progress.emit(WasmBuildProgressEvent::CargoFinished {
        success: status.success(),
        code: status.code(),
        elapsed,
    });

    ensure_command_success(
        WasmBuildPhase::CargoBuild,
        Output {
            status,
            stdout: captured.stdout,
            stderr: captured.stderr,
        },
    )
    .map(|_| ())
}

struct CapturedProcessOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn capture_observed_cargo_output(
    chunks: mpsc::Receiver<ProcessOutputChunk>,
    progress: &mut ProgressReporter<'_>,
    started: Instant,
) -> CapturedProcessOutput {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    loop {
        let message = match progress.heartbeat_due_in() {
            Some(wait) => match chunks.recv_timeout(wait) {
                Ok(chunk) => Some(chunk),
                Err(RecvTimeoutError::Timeout) => {
                    progress.emit_heartbeat(WasmBuildProgressPhase::CargoBuild, started.elapsed());
                    None
                }
                Err(RecvTimeoutError::Disconnected) => break,
            },
            None => match chunks.recv() {
                Ok(chunk) => Some(chunk),
                Err(_) => break,
            },
        };
        let Some(chunk) = message else {
            continue;
        };
        match chunk.stream {
            WasmBuildOutputStream::Stdout => stdout.extend_from_slice(&chunk.bytes),
            WasmBuildOutputStream::Stderr => stderr.extend_from_slice(&chunk.bytes),
        }
        if progress.config.emit_cargo_output {
            progress.emit(WasmBuildProgressEvent::CargoOutput {
                stream: chunk.stream,
                bytes: chunk.bytes,
            });
        }
    }
    CapturedProcessOutput { stdout, stderr }
}

#[derive(Debug)]
struct ProcessOutputChunk {
    stream: WasmBuildOutputStream,
    bytes: Vec<u8>,
}

fn read_process_output<R: io::Read>(
    mut reader: R,
    stream: WasmBuildOutputStream,
    sender: mpsc::Sender<ProcessOutputChunk>,
) -> io::Result<()> {
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok(());
        }
        if sender
            .send(ProcessOutputChunk {
                stream,
                bytes: buffer[..count].to_vec(),
            })
            .is_err()
        {
            return Ok(());
        }
    }
}

fn join_output_reader(
    reader: thread::JoinHandle<io::Result<()>>,
    operation: &'static str,
    cargo_program: &OsStr,
) -> Result<(), WasmBuildError> {
    let result = reader.join().map_err(|_| WasmBuildError::Io {
        operation,
        path: PathBuf::from(cargo_program),
        source: io::Error::other("Cargo output reader panicked"),
    })?;
    result.map_err(|source| WasmBuildError::Io {
        operation,
        path: PathBuf::from(cargo_program),
        source,
    })
}

struct ObservedChild(Option<Child>);

impl ObservedChild {
    const fn new(child: Child) -> Self {
        Self(Some(child))
    }

    const fn child_mut(&mut self) -> &mut Child {
        self.0.as_mut().expect("observed child must be present")
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child_mut().wait()?;
        self.0.take();
        Ok(status)
    }
}

impl Drop for ObservedChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
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

fn optional_string(value: &Value, field: &str) -> Result<Option<String>, WasmBuildError> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(invalid_metadata(&format!(
            "Cargo metadata field `{field}` is not a string or null"
        ))),
    }
}

fn invalid_metadata(message: &str) -> WasmBuildError {
    WasmBuildError::InvalidMetadata {
        message: message.to_owned(),
    }
}

impl WasmBuildError {
    fn indicates_input_change(&self) -> bool {
        match self {
            Self::InputsChangedDuringBuild { .. } => true,
            Self::FailedBuildCleanup { build_error, .. } => build_error.indicates_input_change(),
            _ => false,
        }
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

impl std::fmt::Display for WasmBuildProgressPhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ExactCacheLock => "exact cache lock",
            Self::CargoIdentity => "Cargo identity",
            Self::RustcIdentity => "Rust compiler identity",
            Self::CargoMetadata => "Cargo metadata",
            Self::InputDiscovery => "input discovery",
            Self::ContentHashing => "content hashing",
            Self::SharedTargetLock => "shared target lock",
            Self::SharedTargetMaintenance => "shared target maintenance",
            Self::CargoBuild => "Cargo build",
            Self::ArtifactPublication => "artifact publication",
            Self::ExactCacheMaintenance => "exact cache maintenance",
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
            Self::PreparedInputSnapshotInvalidated => formatter.write_str(
                "the prepared Wasm input snapshot was invalidated before artifact publication",
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
mod tests;
