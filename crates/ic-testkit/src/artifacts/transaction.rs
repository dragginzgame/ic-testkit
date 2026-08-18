use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use crate::timing::saturating_add_optional_duration;

use super::{
    cache_fs::{
        ArtifactCacheMaintenance, ArtifactCachePrunePolicy, ArtifactCachePruneReport, CacheFsError,
        LAST_USED_FILE, directory_logical_size, ensure_cache_directory_tag, is_sha256_directory,
        lock_cache_file, perform_scheduled_cache_maintenance, prune_direct_child_directories,
        record_cache_entry_use, remove_path_if_present, try_lock_cache_file,
    },
    digest::{
        InputDigest, InputHasher, copy_file_atomic, digest_bytes, digest_file,
        digest_labeled_paths, os_bytes, write_atomic,
    },
    wasm_cache::{
        ResolvedCargoBuildInputs, WasmBuildError, WasmBuildSpec, resolve_cargo_build_inputs,
    },
};

const ARTIFACT_CACHE_FORMAT: &str = "ic-testkit-artifact-set-v1";
const MANIFEST_FILE: &str = "manifest.ic-testkit";
const MAX_PREPARATION_RETRIES: usize = 3;
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Built-in validation applied to one transactional artifact output.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArtifactOutputValidation {
    /// Require a regular file; an empty file is valid.
    RegularFile,
    /// Require a nonempty regular file.
    #[default]
    NonEmptyFile,
}

/// Complete caller-owned description of one transactional artifact set.
///
/// Declared inputs and tools must not be located inside `cache_root`. Output
/// destinations must remain outside it, resolve to distinct paths, and not
/// overlap a declared input or tool. Cargo-derived cache/output paths must
/// likewise remain outside resolved inputs unless covered by their exact
/// generated-state exclusions.
#[derive(Clone, Eq, PartialEq)]
pub struct ArtifactCacheSpec {
    cache_root: PathBuf,
    namespace: String,
    recipe_id: String,
    coordination_scope: String,
    inputs: Vec<LabeledPath>,
    tools: Vec<LabeledPath>,
    arguments: Vec<OsString>,
    environment: BTreeMap<OsString, Option<OsString>>,
    identities: Vec<LabeledIdentity>,
    cargo_build_inputs: Vec<CargoBuildInputSet>,
    outputs: Vec<OutputSpec>,
    prune_policy: Option<ArtifactCachePrunePolicy>,
    prune_interval: Option<Duration>,
}

/// Result of preparing a transactional artifact-set acquisition.
pub enum ArtifactCachePreparation {
    /// A complete verified entry was materialized without running the caller's build.
    Reused(ArtifactCacheRecord),
    /// The caller must populate and commit a transaction-owned staging directory.
    Build(ArtifactBuildTransaction),
}

/// Whether an artifact transaction published or reused its exact output set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactCacheOutcome {
    /// The caller populated staging and the complete result was published.
    Built(ArtifactCacheRecord),
    /// A complete existing entry was verified and materialized.
    Reused(ArtifactCacheRecord),
}

/// Details shared by built and reused transactional artifact outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactCacheRecord {
    key: InputDigest,
    input_digest: InputDigest,
    artifacts: Vec<ArtifactCacheArtifact>,
    timings: ArtifactCacheTimings,
    maintenance: Option<ArtifactCacheMaintenance>,
}

/// One logical artifact materialized at its caller-selected destination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactCacheArtifact {
    name: String,
    path: PathBuf,
}

/// Phase timings for one transactional artifact-cache acquisition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCacheTimings {
    coordination_lock_wait: Duration,
    content_lock_wait: Duration,
    namespace_lock_wait: Duration,
    input_capture: Duration,
    cache_lookup: Duration,
    caller_build: Option<Duration>,
    output_validation: Duration,
    publication: Duration,
    materialization: Duration,
    maintenance: Option<Duration>,
    total: Duration,
}

/// Owned miss transaction holding the recipe and content-key process locks.
pub struct ArtifactBuildTransaction {
    spec: Box<ArtifactCacheSpec>,
    resolved: ResolvedKey,
    staging_directory: PathBuf,
    entry_directory: PathBuf,
    namespace_directory: PathBuf,
    _coordination_lock: File,
    _content_lock: File,
    timings: ArtifactCacheTimings,
    total_started: Instant,
    caller_build_started: Instant,
    staging_armed: bool,
}

/// Structured failure from transactional artifact caching.
#[non_exhaustive]
#[derive(Debug)]
pub enum ArtifactCacheError {
    /// The caller supplied an incomplete, ambiguous, or unsafe specification.
    InvalidSpec { message: String },
    /// A filesystem operation failed.
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    /// Inputs repeatedly changed while preparing a stable cache acquisition.
    InputsChangedDuringPreparation {
        before: InputDigest,
        after: InputDigest,
    },
    /// Inputs changed while the caller owned a build transaction.
    InputsChangedDuringBuild {
        before: InputDigest,
        after: InputDigest,
    },
    /// A resolved Cargo source/configuration set changed after it was captured.
    CargoBuildInputsChanged {
        /// Caller-selected logical input-set label.
        label: String,
        /// Exact Cargo fingerprint or source digest captured by the resolver.
        before: InputDigest,
        /// Exact fingerprint or source digest observed during revalidation.
        after: InputDigest,
    },
    /// Rehashing a resolved Cargo input set failed.
    CargoBuildInputRevalidation {
        /// Caller-selected logical input-set label.
        label: String,
        /// Underlying exact Cargo input error.
        source: WasmBuildError,
    },
    /// One or more declared staged outputs were missing or failed validation.
    InvalidOutputs { outputs: Vec<(String, PathBuf)> },
    /// A logical output name was not declared by the transaction specification.
    UnknownOutput { name: String },
    /// Transaction failure cleanup also failed.
    FailedTransactionCleanup {
        transaction_error: Box<Self>,
        path: PathBuf,
        source: io::Error,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LabeledPath {
    label: String,
    path: PathBuf,
}

#[derive(Clone, Eq, PartialEq)]
struct LabeledIdentity {
    label: String,
    value: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq)]
struct CargoBuildInputSet {
    label: String,
    build_spec: WasmBuildSpec,
    resolved: ResolvedCargoBuildInputs,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutputSpec {
    name: String,
    destination: PathBuf,
    validation: ArtifactOutputValidation,
}

#[derive(Clone, Copy)]
struct ResolvedKey {
    key: InputDigest,
    input_digest: InputDigest,
}

struct ArtifactInfo {
    bytes: u64,
    digest: InputDigest,
}

impl ArtifactCacheSpec {
    /// Describe one external artifact recipe using non-secret stable identifiers.
    ///
    /// The namespace selects an independent content store. `recipe_id` must be
    /// bumped whenever undeclared pipeline semantics change. The default
    /// coordination scope is the namespace.
    #[must_use]
    pub fn new(cache_root: &Path, namespace: &str, recipe_id: &str) -> Self {
        Self {
            cache_root: cache_root.to_owned(),
            namespace: namespace.to_owned(),
            recipe_id: recipe_id.to_owned(),
            coordination_scope: namespace.to_owned(),
            inputs: Vec::new(),
            tools: Vec::new(),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            identities: Vec::new(),
            cargo_build_inputs: Vec::new(),
            outputs: Vec::new(),
            prune_policy: None,
            prune_interval: None,
        }
    }

    /// Serialize recipes that share caller-owned mutable external build state.
    #[must_use]
    pub fn with_coordination_scope(mut self, coordination_scope: &str) -> Self {
        coordination_scope.clone_into(&mut self.coordination_scope);
        self
    }

    /// Add one exact input file or directory outside the cache root under a stable label.
    #[must_use]
    pub fn with_input(mut self, label: &str, path: &Path) -> Self {
        self.inputs.push(LabeledPath {
            label: label.to_owned(),
            path: path.to_owned(),
        });
        self
    }

    /// Add one exact executable or tool path outside the cache root under a stable label.
    #[must_use]
    pub fn with_tool(mut self, label: &str, path: &Path) -> Self {
        self.tools.push(LabeledPath {
            label: label.to_owned(),
            path: path.to_owned(),
        });
        self
    }

    /// Set OS-native ordered command arguments that contribute to the content key.
    #[must_use]
    pub fn with_arguments<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.arguments = arguments
            .into_iter()
            .map(|argument| argument.as_ref().to_owned())
            .collect();
        self
    }

    /// Set OS-native environment values that contribute to the content key.
    #[must_use]
    pub fn with_environment<I, K, V>(mut self, environment: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        self.environment.extend(
            environment
                .into_iter()
                .map(|(name, value)| (name.into(), Some(value.into()))),
        );
        self
    }

    /// Record OS-native environment names whose unset state contributes to the content key.
    #[must_use]
    pub fn with_unset_environment<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.environment
            .extend(names.into_iter().map(|name| (name.into(), None)));
        self
    }

    /// Add opaque non-secret identity bytes under a stable logical label.
    #[must_use]
    pub fn with_identity_bytes(mut self, label: &str, value: &[u8]) -> Self {
        self.identities.push(LabeledIdentity {
            label: label.to_owned(),
            value: value.to_vec(),
        });
        self
    }

    /// Add one exact Cargo input snapshot as transactional cache identity and guard.
    ///
    /// The resolved fingerprint keys toolchain, arguments, environment and the
    /// dependency closure. Its Cargo-aware source/configuration paths and
    /// exclusions are rehashed automatically during preparation, cache-hit
    /// materialization and transaction commit.
    #[must_use]
    pub fn with_cargo_build_inputs(
        mut self,
        label: &str,
        build_spec: &WasmBuildSpec,
        resolved: &ResolvedCargoBuildInputs,
    ) -> Self {
        self.cargo_build_inputs.push(CargoBuildInputSet {
            label: label.to_owned(),
            build_spec: build_spec.clone(),
            resolved: resolved.clone(),
        });
        self
    }

    /// Declare one nonempty regular-file output and its nonoverlapping public destination.
    #[must_use]
    pub fn with_output(self, name: &str, destination: &Path) -> Self {
        self.with_output_validation(name, destination, ArtifactOutputValidation::NonEmptyFile)
    }

    /// Declare one output, nonoverlapping public destination, and validation policy.
    #[must_use]
    pub fn with_output_validation(
        mut self,
        name: &str,
        destination: &Path,
        validation: ArtifactOutputValidation,
    ) -> Self {
        self.outputs.push(OutputSpec {
            name: name.to_owned(),
            destination: destination.to_owned(),
            validation,
        });
        self.outputs
            .sort_by(|left, right| left.name.cmp(&right.name));
        self
    }

    /// Apply best-effort retention while protecting the acquired entry.
    #[must_use]
    pub const fn with_prune_policy(mut self, policy: ArtifactCachePrunePolicy) -> Self {
        self.prune_policy = Some(policy);
        self.prune_interval = None;
        self
    }

    /// Apply retention at most once per `minimum_interval` for this namespace.
    ///
    /// The small due-marker check still occurs under the namespace lock.
    /// Successful acquisitions skip the directory scan until the interval has
    /// elapsed. The interval applies to attempted maintenance, including a
    /// nonfatal failed attempt. A zero interval is equivalent to
    /// [`Self::with_prune_policy`].
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

    /// Caller-selected root containing cache data and coordination locks.
    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    /// Stable caller-owned content-store namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Stable caller-owned external recipe identity.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        &self.recipe_id
    }
}

impl std::fmt::Debug for ArtifactCacheSpec {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ArtifactCacheSpec")
            .field("cache_root", &self.cache_root)
            .field("namespace", &self.namespace)
            .field("recipe_id", &self.recipe_id)
            .field("coordination_scope", &self.coordination_scope)
            .field("inputs", &self.inputs)
            .field("tools", &self.tools)
            .field("argument_count", &self.arguments.len())
            .field(
                "environment_names",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .field(
                "identity_labels",
                &self
                    .identities
                    .iter()
                    .map(|identity| identity.label.as_str())
                    .collect::<Vec<_>>(),
            )
            .field(
                "cargo_build_input_labels",
                &self
                    .cargo_build_inputs
                    .iter()
                    .map(|input| input.label.as_str())
                    .collect::<Vec<_>>(),
            )
            .field("outputs", &self.outputs)
            .field("prune_policy", &self.prune_policy)
            .field("prune_interval", &self.prune_interval)
            .finish()
    }
}

impl ArtifactCachePreparation {
    /// Return the already materialized record, or `None` for a cache miss transaction.
    #[must_use]
    pub const fn reused_record(&self) -> Option<&ArtifactCacheRecord> {
        match self {
            Self::Reused(record) => Some(record),
            Self::Build(_) => None,
        }
    }
}

impl ArtifactCacheOutcome {
    /// Read the common acquisition record.
    #[must_use]
    pub const fn record(&self) -> &ArtifactCacheRecord {
        match self {
            Self::Built(record) | Self::Reused(record) => record,
        }
    }

    /// Report whether a complete matching artifact set was reused.
    #[must_use]
    pub const fn is_reused(&self) -> bool {
        matches!(self, Self::Reused(_))
    }
}

impl ArtifactCacheRecord {
    /// Exact content key selecting the immutable cache entry.
    #[must_use]
    pub const fn key(&self) -> InputDigest {
        self.key
    }

    /// Exact digest of declared input and tool paths.
    #[must_use]
    pub const fn input_digest(&self) -> InputDigest {
        self.input_digest
    }

    /// Logical artifacts and their materialized caller destinations.
    #[must_use]
    pub fn artifacts(&self) -> &[ArtifactCacheArtifact] {
        &self.artifacts
    }

    /// Phase timings for this acquisition.
    #[must_use]
    pub const fn timings(&self) -> ArtifactCacheTimings {
        self.timings
    }

    /// Best-effort retention attempted during this acquisition, when configured.
    #[must_use]
    pub const fn maintenance(&self) -> Option<&ArtifactCacheMaintenance> {
        self.maintenance.as_ref()
    }
}

impl ArtifactCacheArtifact {
    /// Stable logical output name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Caller-selected materialized destination.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArtifactCacheTimings {
    /// Time waiting for the caller-selected coordination scope.
    #[must_use]
    pub const fn coordination_lock_wait(self) -> Duration {
        self.coordination_lock_wait
    }

    /// Time waiting for exact content-key ownership.
    #[must_use]
    pub const fn content_lock_wait(self) -> Duration {
        self.content_lock_wait
    }

    /// Time waiting for short namespace mutations.
    #[must_use]
    pub const fn namespace_lock_wait(self) -> Duration {
        self.namespace_lock_wait
    }

    /// Time spent hashing and verifying declared inputs.
    #[must_use]
    pub const fn input_capture(self) -> Duration {
        self.input_capture
    }

    /// Time spent validating or rejecting a committed cache entry.
    #[must_use]
    pub const fn cache_lookup(self) -> Duration {
        self.cache_lookup
    }

    /// Time the caller held a miss transaction before committing it.
    #[must_use]
    pub const fn caller_build(self) -> Option<Duration> {
        self.caller_build
    }

    /// Time spent validating staged output files.
    #[must_use]
    pub const fn output_validation(self) -> Duration {
        self.output_validation
    }

    /// Time spent publishing an immutable content entry.
    #[must_use]
    pub const fn publication(self) -> Duration {
        self.publication
    }

    /// Time spent materializing caller-facing output destinations.
    #[must_use]
    pub const fn materialization(self) -> Duration {
        self.materialization
    }

    /// Time spent on configured best-effort retention.
    #[must_use]
    pub const fn maintenance(self) -> Option<Duration> {
        self.maintenance
    }

    /// Complete acquisition duration.
    #[must_use]
    pub const fn total(self) -> Duration {
        self.total
    }

    pub(super) const fn saturating_add(self, other: Self) -> Self {
        Self {
            coordination_lock_wait: self
                .coordination_lock_wait
                .saturating_add(other.coordination_lock_wait),
            content_lock_wait: self
                .content_lock_wait
                .saturating_add(other.content_lock_wait),
            namespace_lock_wait: self
                .namespace_lock_wait
                .saturating_add(other.namespace_lock_wait),
            input_capture: self.input_capture.saturating_add(other.input_capture),
            cache_lookup: self.cache_lookup.saturating_add(other.cache_lookup),
            caller_build: saturating_add_optional_duration(self.caller_build, other.caller_build),
            output_validation: self
                .output_validation
                .saturating_add(other.output_validation),
            publication: self.publication.saturating_add(other.publication),
            materialization: self.materialization.saturating_add(other.materialization),
            maintenance: saturating_add_optional_duration(self.maintenance, other.maintenance),
            total: self.total.saturating_add(other.total),
        }
    }
}

impl std::fmt::Display for ArtifactCacheTimings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "total={:?} coordination_lock={:?} content_lock={:?} namespace_lock={:?} inputs={:?} lookup={:?} build={:?} validation={:?} publication={:?} materialization={:?} maintenance={:?}",
            self.total,
            self.coordination_lock_wait,
            self.content_lock_wait,
            self.namespace_lock_wait,
            self.input_capture,
            self.cache_lookup,
            self.caller_build,
            self.output_validation,
            self.publication,
            self.materialization,
            self.maintenance,
        )
    }
}

impl std::fmt::Display for ArtifactCacheOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = if self.is_reused() { "reused" } else { "built" };
        write!(
            formatter,
            "{state} key={} artifacts={} {}",
            self.record().key,
            self.record().artifacts.len(),
            self.record().timings,
        )
    }
}

impl ArtifactBuildTransaction {
    /// Transaction-owned directory in which the caller may run its build.
    ///
    /// Commit accepts only the cache-created `outputs` child at this root.
    /// Callers must remove or relocate logs and other temporary root children.
    #[must_use]
    pub fn staging_directory(&self) -> &Path {
        &self.staging_directory
    }

    /// Checked staging destination for one declared logical output.
    pub fn output_path(&self, name: &str) -> Result<PathBuf, ArtifactCacheError> {
        self.output_index(name)
            .map(|index| staged_output_path(&self.staging_directory, index))
            .ok_or_else(|| ArtifactCacheError::UnknownOutput {
                name: name.to_owned(),
            })
    }

    /// Copy a fixed external output into its transaction-owned staging path.
    pub fn import_output(&self, name: &str, source: &Path) -> Result<(), ArtifactCacheError> {
        let destination = self.output_path(name)?;
        copy_file_atomic(source, &destination).map_err(|source_error| ArtifactCacheError::Io {
            operation: "import artifact output into staging",
            path: destination,
            source: source_error,
        })?;
        Ok(())
    }

    /// Validate the exact staging schema and atomically publish the complete output set.
    pub fn commit(mut self) -> Result<ArtifactCacheOutcome, ArtifactCacheError> {
        let result = self.commit_inner();
        match result {
            Ok(outcome) => Ok(outcome),
            Err(transaction_error) if self.staging_armed => {
                let path = self.staging_directory.clone();
                match remove_path_if_present(&path) {
                    Ok(()) => {
                        self.staging_armed = false;
                        Err(transaction_error)
                    }
                    Err(source) => Err(ArtifactCacheError::FailedTransactionCleanup {
                        transaction_error: Box::new(transaction_error),
                        path,
                        source,
                    }),
                }
            }
            Err(transaction_error) => Err(transaction_error),
        }
    }

    /// Abandon the transaction and synchronously remove its staging directory.
    pub fn abort(mut self) -> Result<(), ArtifactCacheError> {
        remove_path_if_present(&self.staging_directory).map_err(|source| {
            ArtifactCacheError::Io {
                operation: "abort artifact cache transaction",
                path: self.staging_directory.clone(),
                source,
            }
        })?;
        self.staging_armed = false;
        Ok(())
    }

    fn output_index(&self, name: &str) -> Option<usize> {
        self.spec
            .outputs
            .iter()
            .position(|output| output.name == name)
    }

    fn commit_inner(&mut self) -> Result<ArtifactCacheOutcome, ArtifactCacheError> {
        self.timings.caller_build = Some(self.caller_build_started.elapsed());

        let validation_started = Instant::now();
        let output_info = inspect_complete_output_set(&self.spec, &self.staging_directory)?;
        self.timings.output_validation = validation_started.elapsed();

        let capture_started = Instant::now();
        revalidate_cargo_build_input_fingerprints(&self.spec)?;
        let verified = resolve_key(&self.spec)?;
        self.timings.input_capture = self
            .timings
            .input_capture
            .saturating_add(capture_started.elapsed());
        if verified.input_digest != self.resolved.input_digest {
            return Err(ArtifactCacheError::InputsChangedDuringBuild {
                before: self.resolved.input_digest,
                after: verified.input_digest,
            });
        }

        let publication_started = Instant::now();
        let manifest = manifest_contents(self.resolved.key, &self.spec, &output_info);
        write_atomic(
            &self.staging_directory.join(MANIFEST_FILE),
            manifest.as_bytes(),
        )
        .map_err(|source| ArtifactCacheError::Io {
            operation: "write artifact cache manifest",
            path: self.staging_directory.join(MANIFEST_FILE),
            source,
        })?;
        let namespace_lock_path = namespace_lock_path(&self.spec);
        let (_namespace_lock, namespace_wait) =
            lock_cache_file(&namespace_lock_path).map_err(artifact_cache_fs_error)?;
        self.timings.namespace_lock_wait = self
            .timings
            .namespace_lock_wait
            .saturating_add(namespace_wait);
        remove_path_if_present(&self.entry_directory).map_err(|source| ArtifactCacheError::Io {
            operation: "remove conflicting artifact cache entry",
            path: self.entry_directory.clone(),
            source,
        })?;
        fs::rename(&self.staging_directory, &self.entry_directory).map_err(|source| {
            ArtifactCacheError::Io {
                operation: "publish artifact cache entry",
                path: self.entry_directory.clone(),
                source,
            }
        })?;
        self.staging_armed = false;
        self.timings.publication = publication_started.elapsed();

        let materialization_started = Instant::now();
        materialize_outputs(&self.spec, &self.entry_directory)?;
        self.timings.materialization = materialization_started.elapsed();
        record_cache_entry_use(&self.entry_directory).map_err(artifact_cache_fs_error)?;
        let (maintenance, maintenance_timing) = perform_maintenance_locked(
            &self.spec,
            &self.namespace_directory,
            &self.entry_directory,
        );
        self.timings.maintenance = maintenance_timing;
        self.timings.total = self.total_started.elapsed();

        Ok(ArtifactCacheOutcome::Built(cache_record(
            &self.spec,
            self.resolved,
            self.timings,
            maintenance,
        )))
    }
}

impl Drop for ArtifactBuildTransaction {
    fn drop(&mut self) {
        if self.staging_armed {
            let _ = remove_path_if_present(&self.staging_directory);
        }
    }
}

/// Verify or begin one exact external artifact-set transaction.
pub fn prepare_artifact_cache(
    spec: &ArtifactCacheSpec,
) -> Result<ArtifactCachePreparation, ArtifactCacheError> {
    let total_started = Instant::now();
    validate_spec(spec)?;
    validate_filesystem_boundaries(spec)?;
    let namespace_directory = initialize_cache(spec)?;

    let coordination_lock_path = coordination_lock_path(spec);
    let (coordination_lock, coordination_wait) =
        lock_cache_file(&coordination_lock_path).map_err(artifact_cache_fs_error)?;
    let mut timings = ArtifactCacheTimings {
        coordination_lock_wait: coordination_wait,
        ..ArtifactCacheTimings::default()
    };
    let initial_cargo_started = Instant::now();
    revalidate_cargo_build_input_fingerprints(spec)?;
    timings.input_capture = initial_cargo_started.elapsed();
    let mut last_change = None;

    for _ in 0..MAX_PREPARATION_RETRIES {
        let capture_started = Instant::now();
        let resolved = resolve_key(spec)?;
        timings.input_capture = timings
            .input_capture
            .saturating_add(capture_started.elapsed());

        let content_lock_path = content_lock_path(spec, resolved.key);
        let (content_lock, content_wait) =
            lock_cache_file(&content_lock_path).map_err(artifact_cache_fs_error)?;
        timings.content_lock_wait = timings.content_lock_wait.saturating_add(content_wait);

        let verification_started = Instant::now();
        let verified = resolve_key(spec)?;
        timings.input_capture = timings
            .input_capture
            .saturating_add(verification_started.elapsed());
        if resolved.input_digest != verified.input_digest {
            last_change = Some((resolved.input_digest, verified.input_digest));
            drop(content_lock);
            continue;
        }

        let entry_directory = entry_directory(&namespace_directory, resolved.key);
        let namespace_lock_path = namespace_lock_path(spec);
        let (namespace_lock, namespace_wait) =
            lock_cache_file(&namespace_lock_path).map_err(artifact_cache_fs_error)?;
        timings.namespace_lock_wait = timings.namespace_lock_wait.saturating_add(namespace_wait);
        let lookup_started = Instant::now();
        let reusable = cache_entry_is_valid(spec, resolved.key, &entry_directory)?;
        timings.cache_lookup = timings
            .cache_lookup
            .saturating_add(lookup_started.elapsed());

        if reusable {
            let materialization_started = Instant::now();
            materialize_outputs(spec, &entry_directory)?;
            timings.materialization = timings
                .materialization
                .saturating_add(materialization_started.elapsed());
            let after_started = Instant::now();
            revalidate_cargo_build_input_fingerprints(spec)?;
            let after = resolve_key(spec)?;
            timings.input_capture = timings
                .input_capture
                .saturating_add(after_started.elapsed());
            if after.input_digest != resolved.input_digest {
                last_change = Some((resolved.input_digest, after.input_digest));
                drop(namespace_lock);
                drop(content_lock);
                continue;
            }
            record_cache_entry_use(&entry_directory).map_err(artifact_cache_fs_error)?;
            let (maintenance, maintenance_timing) =
                perform_maintenance_locked(spec, &namespace_directory, &entry_directory);
            timings.maintenance = maintenance_timing;
            timings.total = total_started.elapsed();
            return Ok(ArtifactCachePreparation::Reused(cache_record(
                spec,
                resolved,
                timings,
                maintenance,
            )));
        }

        remove_path_if_present(&entry_directory).map_err(|source| ArtifactCacheError::Io {
            operation: "remove invalid artifact cache entry",
            path: entry_directory.clone(),
            source,
        })?;
        let staging_directory = create_staging_directory(&namespace_directory, resolved.key)?;
        drop(namespace_lock);
        return Ok(ArtifactCachePreparation::Build(ArtifactBuildTransaction {
            spec: Box::new(spec.clone()),
            resolved,
            staging_directory,
            entry_directory,
            namespace_directory,
            _coordination_lock: coordination_lock,
            _content_lock: content_lock,
            timings,
            total_started,
            caller_build_started: Instant::now(),
            staging_armed: true,
        }));
    }

    let (before, after) = last_change.expect("preparation retries require a recorded input change");
    Err(ArtifactCacheError::InputsChangedDuringPreparation { before, after })
}

fn revalidate_cargo_build_input_fingerprints(
    spec: &ArtifactCacheSpec,
) -> Result<(), ArtifactCacheError> {
    for cargo_input in &spec.cargo_build_inputs {
        let current = resolve_cargo_build_inputs(&cargo_input.build_spec).map_err(|source| {
            ArtifactCacheError::CargoBuildInputRevalidation {
                label: cargo_input.label.clone(),
                source,
            }
        })?;
        let before = cargo_input.resolved.fingerprint();
        let after = current.fingerprint();
        if after != before {
            return Err(ArtifactCacheError::CargoBuildInputsChanged {
                label: cargo_input.label.clone(),
                before,
                after,
            });
        }
    }
    Ok(())
}

/// Prune one transactional artifact namespace without acquiring an artifact.
///
/// This strict maintenance entry point returns cache errors directly. Build
/// acquisitions configured with [`ArtifactCacheSpec::with_prune_policy`] use
/// the same implementation but attach maintenance as a nonfatal record.
/// Abandoned staging is removed only when its content-key lock is currently
/// unowned and is reported separately from committed-entry retention.
pub fn prune_artifact_cache(
    cache_root: &Path,
    namespace: &str,
    policy: ArtifactCachePrunePolicy,
) -> Result<ArtifactCachePruneReport, ArtifactCacheError> {
    validate_identifier("namespace", namespace)?;
    if cache_root.as_os_str().is_empty() {
        return invalid_spec("cache root must not be empty");
    }
    ensure_cache_directory_tag(cache_root).map_err(artifact_cache_fs_error)?;
    let namespace_directory = namespace_directory_for(cache_root, namespace);
    let lock_path = namespace_lock_path_for(cache_root, namespace);
    let (_lock, _) = lock_cache_file(&lock_path).map_err(artifact_cache_fs_error)?;
    prune_artifact_namespace_locked(cache_root, &namespace_directory, policy, None)
}

fn initialize_cache(spec: &ArtifactCacheSpec) -> Result<PathBuf, ArtifactCacheError> {
    ensure_cache_directory_tag(&spec.cache_root).map_err(artifact_cache_fs_error)?;
    let namespace = namespace_directory(spec);
    fs::create_dir_all(entries_directory(&namespace)).map_err(|source| ArtifactCacheError::Io {
        operation: "create artifact cache namespace",
        path: namespace.clone(),
        source,
    })?;
    Ok(namespace)
}

fn validate_spec(spec: &ArtifactCacheSpec) -> Result<(), ArtifactCacheError> {
    validate_identifier("namespace", &spec.namespace)?;
    validate_identifier("recipe identity", &spec.recipe_id)?;
    validate_identifier("coordination scope", &spec.coordination_scope)?;
    if spec.cache_root.as_os_str().is_empty() {
        return invalid_spec("cache root must not be empty");
    }
    if spec.outputs.is_empty() {
        return invalid_spec("at least one artifact output is required");
    }

    let mut labels = BTreeSet::new();
    for input in &spec.inputs {
        validate_path_label("input", &input.label)?;
        if !labels.insert(format!("input/{}", input.label)) {
            return invalid_spec(&format!("duplicate input label `{}`", input.label));
        }
    }
    for tool in &spec.tools {
        validate_path_label("tool", &tool.label)?;
        if !labels.insert(format!("tool/{}", tool.label)) {
            return invalid_spec(&format!("duplicate tool label `{}`", tool.label));
        }
    }
    let mut identity_labels = BTreeSet::new();
    for identity in &spec.identities {
        validate_label("identity", &identity.label)?;
        if !identity_labels.insert(&identity.label) {
            return invalid_spec(&format!("duplicate identity label `{}`", identity.label));
        }
    }
    let mut cargo_input_labels = BTreeSet::new();
    for cargo_inputs in &spec.cargo_build_inputs {
        validate_label("Cargo build input", &cargo_inputs.label)?;
        if !cargo_input_labels.insert(&cargo_inputs.label) {
            return invalid_spec(&format!(
                "duplicate Cargo build input label `{}`",
                cargo_inputs.label
            ));
        }
    }
    if spec
        .environment
        .keys()
        .any(|name| name.as_os_str().is_empty())
    {
        return invalid_spec("environment names must not be empty");
    }
    let mut output_names = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    for output in &spec.outputs {
        validate_output_name(&output.name)?;
        if !output_names.insert(&output.name) {
            return invalid_spec(&format!("duplicate output name `{}`", output.name));
        }
        if output.destination.as_os_str().is_empty() {
            return invalid_spec(&format!(
                "output `{}` destination must not be empty",
                output.name
            ));
        }
        if !destinations.insert(&output.destination) {
            return invalid_spec(&format!(
                "output `{}` shares a destination with another output",
                output.name
            ));
        }
    }
    Ok(())
}

fn validate_filesystem_boundaries(spec: &ArtifactCacheSpec) -> Result<(), ArtifactCacheError> {
    let cache_root =
        canonicalize_allow_missing(&spec.cache_root).map_err(|source| ArtifactCacheError::Io {
            operation: "resolve artifact cache root",
            path: spec.cache_root.clone(),
            source,
        })?;
    let mut declared_paths = Vec::with_capacity(spec.inputs.len() + spec.tools.len());
    for (kind, labeled_paths) in [("input", &spec.inputs), ("tool", &spec.tools)] {
        for labeled in labeled_paths {
            let canonical =
                canonicalize_path(&labeled.path, "canonicalize declared artifact cache path")?;
            if canonical.starts_with(&cache_root) {
                return invalid_spec(&format!(
                    "{kind} `{}` must not be located inside the artifact cache root",
                    labeled.label
                ));
            }
            let is_directory = fs::metadata(&canonical)
                .map_err(|source| ArtifactCacheError::Io {
                    operation: "inspect declared artifact cache path",
                    path: canonical.clone(),
                    source,
                })?
                .is_dir();
            declared_paths.push((kind, labeled.label.as_str(), canonical, is_directory));
        }
    }
    for cargo_inputs in &spec.cargo_build_inputs {
        if resolved_cargo_inputs_watch_path(&cargo_inputs.resolved, &cache_root)? {
            return invalid_spec(&format!(
                "artifact cache root must be outside resolved Cargo build inputs `{}` or inside one of their generated-state exclusions",
                cargo_inputs.label
            ));
        }
    }

    let mut destinations = BTreeSet::new();
    for output in &spec.outputs {
        let destination = canonicalize_allow_missing(&output.destination).map_err(|source| {
            ArtifactCacheError::Io {
                operation: "resolve artifact output destination",
                path: output.destination.clone(),
                source,
            }
        })?;
        if destination.starts_with(&cache_root) {
            return invalid_spec(&format!(
                "output `{}` destination must be outside the artifact cache root",
                output.name
            ));
        }
        if !destinations.insert(destination.clone()) {
            return invalid_spec(&format!(
                "output `{}` resolves to the same destination as another output",
                output.name
            ));
        }
        for (kind, label, declared, is_directory) in &declared_paths {
            if destination == *declared || (*is_directory && destination.starts_with(declared)) {
                return invalid_spec(&format!(
                    "output `{}` destination overlaps declared {kind} `{label}`",
                    output.name
                ));
            }
        }
        for cargo_inputs in &spec.cargo_build_inputs {
            if resolved_cargo_inputs_watch_path(&cargo_inputs.resolved, &destination)? {
                return invalid_spec(&format!(
                    "output `{}` destination must be outside resolved Cargo build inputs `{}` or inside one of their generated-state exclusions",
                    output.name, cargo_inputs.label
                ));
            }
        }
        match fs::metadata(&destination) {
            Ok(metadata) if metadata.is_dir() => {
                return invalid_spec(&format!(
                    "output `{}` destination must not be an existing directory",
                    output.name
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(ArtifactCacheError::Io {
                    operation: "inspect artifact output destination",
                    path: destination,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn resolved_cargo_inputs_watch_path(
    resolved: &ResolvedCargoBuildInputs,
    candidate: &Path,
) -> Result<bool, ArtifactCacheError> {
    for exclusion in resolved.exclusions() {
        let exclusion =
            canonicalize_allow_missing(exclusion).map_err(|source| ArtifactCacheError::Io {
                operation: "resolve Cargo build input exclusion",
                path: exclusion.clone(),
                source,
            })?;
        if candidate.starts_with(exclusion) {
            return Ok(false);
        }
    }

    for input in resolved.inputs() {
        let path = canonicalize_path(input.path(), "canonicalize resolved Cargo build input")?;
        let is_directory = fs::metadata(&path)
            .map_err(|source| ArtifactCacheError::Io {
                operation: "inspect resolved Cargo build input",
                path: path.clone(),
                source,
            })?
            .is_dir();
        if candidate == path || (is_directory && candidate.starts_with(path)) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn canonicalize_path(path: &Path, operation: &'static str) -> Result<PathBuf, ArtifactCacheError> {
    path.canonicalize()
        .map_err(|source| ArtifactCacheError::Io {
            operation,
            path: path.to_owned(),
            source,
        })
}

fn canonicalize_allow_missing(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut resolved = PathBuf::new();
    let mut missing_depth = 0_usize;
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                let candidate = resolved.join(component.as_os_str());
                if missing_depth == 0 && matches!(component, Component::Normal(_)) {
                    match candidate.canonicalize() {
                        Ok(canonical) => resolved = canonical,
                        Err(error) if error.kind() == io::ErrorKind::NotFound => {
                            resolved = candidate;
                            missing_depth = 1;
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    resolved = candidate;
                    if matches!(component, Component::Normal(_)) && missing_depth > 0 {
                        missing_depth += 1;
                    }
                }
            }
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
                missing_depth = missing_depth.saturating_sub(1);
            }
        }
    }
    Ok(resolved)
}

fn validate_identifier(kind: &str, value: &str) -> Result<(), ArtifactCacheError> {
    if value.is_empty() {
        return invalid_spec(&format!("{kind} must not be empty"));
    }
    if value.len() > 256 {
        return invalid_spec(&format!("{kind} must not exceed 256 bytes"));
    }
    Ok(())
}

fn validate_label(kind: &str, value: &str) -> Result<(), ArtifactCacheError> {
    if value.is_empty() {
        return invalid_spec(&format!("{kind} label must not be empty"));
    }
    if value.len() > 256 {
        return invalid_spec(&format!("{kind} label must not exceed 256 bytes"));
    }
    Ok(())
}

fn validate_path_label(kind: &str, value: &str) -> Result<(), ArtifactCacheError> {
    validate_label(kind, value)?;
    if value
        .bytes()
        .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/')))
        || value
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return invalid_spec(&format!(
            "{kind} label `{value}` must be a portable relative logical path"
        ));
    }
    Ok(())
}

fn validate_output_name(name: &str) -> Result<(), ArtifactCacheError> {
    if name.is_empty() || name.len() > 128 {
        return invalid_spec("output names must contain 1 to 128 bytes");
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return invalid_spec(&format!(
            "output name `{name}` must use only ASCII letters, digits, dot, dash, or underscore"
        ));
    }
    if matches!(name, "." | "..") {
        return invalid_spec("output names must not be dot path components");
    }
    Ok(())
}

fn invalid_spec<T>(message: &str) -> Result<T, ArtifactCacheError> {
    Err(ArtifactCacheError::InvalidSpec {
        message: message.to_owned(),
    })
}

fn resolve_key(spec: &ArtifactCacheSpec) -> Result<ResolvedKey, ArtifactCacheError> {
    let paths = spec
        .inputs
        .iter()
        .map(|input| {
            (
                PathBuf::from("input").join(&input.label),
                input.path.clone(),
            )
        })
        .chain(
            spec.tools
                .iter()
                .map(|tool| (PathBuf::from("tool").join(&tool.label), tool.path.clone())),
        )
        .collect::<Vec<_>>();
    let declared_input_digest = digest_labeled_paths(
        "artifact-set-inputs-v1",
        &paths,
        std::slice::from_ref(&spec.cache_root),
    )
    .map_err(|source| ArtifactCacheError::Io {
        operation: "hash artifact cache inputs",
        path: spec.cache_root.clone(),
        source,
    })?;
    let input_digest = if spec.cargo_build_inputs.is_empty() {
        declared_input_digest
    } else {
        let mut cargo_inputs = spec.cargo_build_inputs.iter().collect::<Vec<_>>();
        cargo_inputs.sort_by(|left, right| left.label.cmp(&right.label));
        let mut inputs = InputHasher::new("artifact-set-inputs-with-cargo-v1");
        inputs.field("declared-input-digest", declared_input_digest.as_bytes());
        for cargo_input in cargo_inputs {
            let current = cargo_input
                .resolved
                .current_input_digest()
                .map_err(|source| ArtifactCacheError::CargoBuildInputRevalidation {
                    label: cargo_input.label.clone(),
                    source,
                })?;
            let before = cargo_input.resolved.input_digest();
            if current != before {
                return Err(ArtifactCacheError::CargoBuildInputsChanged {
                    label: cargo_input.label.clone(),
                    before,
                    after: current,
                });
            }
            inputs.field("cargo-input-label", cargo_input.label.as_bytes());
            inputs.field(
                "cargo-build-fingerprint",
                cargo_input.resolved.fingerprint().as_bytes(),
            );
            inputs.field("cargo-input-digest", current.as_bytes());
        }
        inputs.finish()
    };

    let mut hasher = InputHasher::new(ARTIFACT_CACHE_FORMAT);
    hasher.field("namespace", spec.namespace.as_bytes());
    hasher.field("recipe-id", spec.recipe_id.as_bytes());
    hasher.field("input-digest", input_digest.as_bytes());
    for argument in &spec.arguments {
        hasher.field("argument", &os_bytes(argument));
    }
    for (name, value) in &spec.environment {
        hasher.field("environment-name", &os_bytes(name));
        match value {
            Some(value) => hasher.field("environment-value", &os_bytes(value)),
            None => hasher.field("environment-unset", b""),
        }
    }
    let mut identities = spec.identities.iter().collect::<Vec<_>>();
    identities.sort_by(|left, right| left.label.cmp(&right.label));
    for identity in identities {
        hasher.field("identity-label", identity.label.as_bytes());
        hasher.field("identity-value", &identity.value);
    }
    for output in &spec.outputs {
        hasher.field("output-name", output.name.as_bytes());
        hasher.field(
            "output-validation",
            output.validation.cache_token().as_bytes(),
        );
    }
    Ok(ResolvedKey {
        key: hasher.finish(),
        input_digest,
    })
}

fn cache_entry_is_valid(
    spec: &ArtifactCacheSpec,
    key: InputDigest,
    entry: &Path,
) -> Result<bool, ArtifactCacheError> {
    let entry_metadata = match fs::symlink_metadata(entry) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(ArtifactCacheError::Io {
                operation: "inspect artifact cache entry",
                path: entry.to_owned(),
                source,
            });
        }
    };
    if !entry_metadata.file_type().is_dir() || !cache_entry_root_is_valid(entry)? {
        return Ok(false);
    }
    let manifest_path = entry.join(MANIFEST_FILE);
    let manifest_metadata = match fs::symlink_metadata(&manifest_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(ArtifactCacheError::Io {
                operation: "inspect artifact cache manifest",
                path: manifest_path,
                source,
            });
        }
    };
    if !manifest_metadata.file_type().is_file() {
        return Ok(false);
    }
    let manifest = fs::read(&manifest_path).map_err(|source| ArtifactCacheError::Io {
        operation: "read artifact cache manifest",
        path: manifest_path,
        source,
    })?;
    let Some(output_info) = inspect_cached_output_set(spec, entry)? else {
        return Ok(false);
    };
    Ok(manifest == manifest_contents(key, spec, &output_info).as_bytes())
}

fn inspect_complete_output_set(
    spec: &ArtifactCacheSpec,
    root: &Path,
) -> Result<Vec<ArtifactInfo>, ArtifactCacheError> {
    let mut info = Vec::new();
    let mut invalid = Vec::new();
    let output_directory = root.join("outputs");
    if !is_plain_directory(
        &output_directory,
        "inspect artifact staging output directory",
    )? {
        invalid.push(("<outputs>".to_owned(), output_directory));
        return Err(ArtifactCacheError::InvalidOutputs { outputs: invalid });
    }
    let outputs = &spec.outputs;
    for (index, output) in outputs.iter().enumerate() {
        let path = staged_output_path(root, index);
        match inspect_artifact(&path, output.validation) {
            Ok(Some(artifact)) => info.push(artifact),
            Ok(None) => invalid.push((output.name.clone(), path)),
            Err(source) => {
                return Err(ArtifactCacheError::Io {
                    operation: "inspect staged artifact output",
                    path,
                    source,
                });
            }
        }
    }
    invalid.extend(
        undeclared_output_paths(root, outputs.len())?
            .into_iter()
            .map(|path| ("<undeclared>".to_owned(), path)),
    );
    invalid.extend(
        undeclared_child_paths(
            root,
            &BTreeSet::from([OsString::from("outputs")]),
            "read artifact staging directory",
        )?
        .into_iter()
        .map(|path| ("<undeclared>".to_owned(), path)),
    );
    if invalid.is_empty() {
        Ok(info)
    } else {
        Err(ArtifactCacheError::InvalidOutputs { outputs: invalid })
    }
}

fn inspect_cached_output_set(
    spec: &ArtifactCacheSpec,
    root: &Path,
) -> Result<Option<Vec<ArtifactInfo>>, ArtifactCacheError> {
    let mut info = Vec::new();
    let outputs = &spec.outputs;
    for (index, output) in outputs.iter().enumerate() {
        let path = staged_output_path(root, index);
        match inspect_artifact(&path, output.validation) {
            Ok(Some(artifact)) => info.push(artifact),
            Ok(None) => return Ok(None),
            Err(source) => {
                return Err(ArtifactCacheError::Io {
                    operation: "inspect cached artifact output",
                    path,
                    source,
                });
            }
        }
    }
    if !undeclared_output_paths(root, outputs.len())?.is_empty() {
        return Ok(None);
    }
    Ok(Some(info))
}

fn undeclared_output_paths(
    root: &Path,
    output_count: usize,
) -> Result<Vec<PathBuf>, ArtifactCacheError> {
    let output_directory = root.join("outputs");
    let expected = (0..output_count)
        .map(format_output_index)
        .map(OsString::from)
        .collect::<BTreeSet<_>>();
    undeclared_child_paths(
        &output_directory,
        &expected,
        "read artifact output directory",
    )
}

fn undeclared_child_paths(
    directory: &Path,
    expected: &BTreeSet<OsString>,
    operation: &'static str,
) -> Result<Vec<PathBuf>, ArtifactCacheError> {
    let entries = fs::read_dir(directory).map_err(|source| ArtifactCacheError::Io {
        operation,
        path: directory.to_owned(),
        source,
    })?;
    let mut undeclared = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ArtifactCacheError::Io {
            operation,
            path: directory.to_owned(),
            source,
        })?;
        if !expected.contains(&entry.file_name()) {
            undeclared.push(entry.path());
        }
    }
    Ok(undeclared)
}

fn cache_entry_root_is_valid(root: &Path) -> Result<bool, ArtifactCacheError> {
    let expected = BTreeSet::from([
        OsString::from("outputs"),
        OsString::from(MANIFEST_FILE),
        OsString::from(LAST_USED_FILE),
    ]);
    if !undeclared_child_paths(root, &expected, "read artifact cache entry")?.is_empty() {
        return Ok(false);
    }
    if !is_plain_directory(
        &root.join("outputs"),
        "inspect artifact cache output directory",
    )? {
        return Ok(false);
    }
    let last_used = root.join(LAST_USED_FILE);
    match fs::symlink_metadata(&last_used) {
        Ok(metadata) => Ok(metadata.file_type().is_file()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(source) => Err(ArtifactCacheError::Io {
            operation: "inspect artifact cache use marker",
            path: last_used,
            source,
        }),
    }
}

fn is_plain_directory(path: &Path, operation: &'static str) -> Result<bool, ArtifactCacheError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(ArtifactCacheError::Io {
            operation,
            path: path.to_owned(),
            source,
        }),
    }
}

fn inspect_artifact(
    path: &Path,
    validation: ArtifactOutputValidation,
) -> io::Result<Option<ArtifactInfo>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() {
        return Ok(None);
    }
    if validation == ArtifactOutputValidation::NonEmptyFile && metadata.len() == 0 {
        return Ok(None);
    }
    let (bytes, digest) = digest_file("artifact-set-output-v1", path)?;
    Ok(Some(ArtifactInfo { bytes, digest }))
}

fn manifest_contents(
    key: InputDigest,
    spec: &ArtifactCacheSpec,
    output_info: &[ArtifactInfo],
) -> String {
    let mut manifest = format!("{ARTIFACT_CACHE_FORMAT}\nkey:{key}\n");
    for ((index, output), info) in spec.outputs.iter().enumerate().zip(output_info) {
        use std::fmt::Write as _;
        writeln!(
            manifest,
            "output:{index}:{}:{}:{}:{}",
            output.name,
            output.validation.cache_token(),
            info.bytes,
            info.digest,
        )
        .expect("writing an artifact manifest to a String cannot fail");
    }
    manifest
}

fn materialize_outputs(spec: &ArtifactCacheSpec, entry: &Path) -> Result<(), ArtifactCacheError> {
    for (index, output) in spec.outputs.iter().enumerate() {
        let cached = staged_output_path(entry, index);
        copy_file_atomic(&cached, &output.destination).map_err(|source| {
            ArtifactCacheError::Io {
                operation: "materialize artifact output",
                path: output.destination.clone(),
                source,
            }
        })?;
    }
    Ok(())
}

fn perform_maintenance_locked(
    spec: &ArtifactCacheSpec,
    namespace: &Path,
    protected_entry: &Path,
) -> (Option<ArtifactCacheMaintenance>, Option<Duration>) {
    spec.prune_policy.map_or((None, None), |policy| {
        let identity = policy.maintenance_identity();
        perform_scheduled_cache_maintenance(namespace, spec.prune_interval, &identity, || {
            prune_artifact_namespace_locked(
                &spec.cache_root,
                namespace,
                policy,
                Some(protected_entry),
            )
            .map_err(|error| error.to_string())
        })
    })
}

fn prune_artifact_namespace_locked(
    cache_root: &Path,
    namespace: &Path,
    policy: ArtifactCachePrunePolicy,
    protected_entry: Option<&Path>,
) -> Result<ArtifactCachePruneReport, ArtifactCacheError> {
    let mut report = prune_direct_child_directories(
        &entries_directory(namespace),
        policy,
        protected_entry,
        is_sha256_directory,
    )
    .map_err(artifact_cache_fs_error)?;
    remove_abandoned_staging(cache_root, namespace, &mut report)?;
    Ok(report)
}

fn remove_abandoned_staging(
    cache_root: &Path,
    namespace: &Path,
    report: &mut ArtifactCachePruneReport,
) -> Result<(), ArtifactCacheError> {
    let staging_root = namespace.join("staging");
    let entries = match fs::read_dir(&staging_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(ArtifactCacheError::Io {
                operation: "read artifact staging root during pruning",
                path: staging_root,
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| ArtifactCacheError::Io {
            operation: "read artifact staging entry during pruning",
            path: staging_root.clone(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| ArtifactCacheError::Io {
            operation: "inspect artifact staging entry during pruning",
            path: entry.path(),
            source,
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let Some(key) = staging_content_key(&file_name) else {
            continue;
        };
        let lock_path = content_lock_path_for_key(cache_root, key);
        let Some(_content_lock) =
            try_lock_cache_file(&lock_path).map_err(artifact_cache_fs_error)?
        else {
            continue;
        };
        let path = entry.path();
        let bytes = directory_logical_size(&path).map_err(|source| ArtifactCacheError::Io {
            operation: "measure abandoned artifact staging directory",
            path: path.clone(),
            source,
        })?;
        remove_path_if_present(&path).map_err(|source| ArtifactCacheError::Io {
            operation: "remove abandoned artifact staging directory during pruning",
            path,
            source,
        })?;
        report.record_uncommitted_removal(bytes);
    }
    Ok(())
}

fn staging_content_key(name: &OsStr) -> Option<&str> {
    let name = name.to_str()?;
    let (key, suffix) = name.split_once('-')?;
    (!suffix.is_empty() && key.len() == 64 && key.as_bytes().iter().all(u8::is_ascii_hexdigit))
        .then_some(key)
}

fn cache_record(
    spec: &ArtifactCacheSpec,
    resolved: ResolvedKey,
    timings: ArtifactCacheTimings,
    maintenance: Option<ArtifactCacheMaintenance>,
) -> ArtifactCacheRecord {
    ArtifactCacheRecord {
        key: resolved.key,
        input_digest: resolved.input_digest,
        artifacts: spec
            .outputs
            .iter()
            .map(|output| ArtifactCacheArtifact {
                name: output.name.clone(),
                path: output.destination.clone(),
            })
            .collect(),
        timings,
        maintenance,
    }
}

fn create_staging_directory(
    namespace: &Path,
    key: InputDigest,
) -> Result<PathBuf, ArtifactCacheError> {
    let staging_root = namespace.join("staging");
    fs::create_dir_all(&staging_root).map_err(|source| ArtifactCacheError::Io {
        operation: "create artifact staging root",
        path: staging_root.clone(),
        source,
    })?;
    remove_same_key_staging(&staging_root, key)?;
    let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let staging = staging_root.join(format!("{key}-{}-{sequence}", std::process::id()));
    fs::create_dir_all(staging.join("outputs")).map_err(|source| ArtifactCacheError::Io {
        operation: "create artifact transaction staging directory",
        path: staging.clone(),
        source,
    })?;
    Ok(staging)
}

fn remove_same_key_staging(root: &Path, key: InputDigest) -> Result<(), ArtifactCacheError> {
    let prefix = format!("{key}-");
    let entries = fs::read_dir(root).map_err(|source| ArtifactCacheError::Io {
        operation: "read artifact staging root",
        path: root.to_owned(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ArtifactCacheError::Io {
            operation: "read artifact staging entry",
            path: root.to_owned(),
            source,
        })?;
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            remove_path_if_present(&entry.path()).map_err(|source| ArtifactCacheError::Io {
                operation: "remove abandoned artifact staging directory",
                path: entry.path(),
                source,
            })?;
        }
    }
    Ok(())
}

fn staged_output_path(root: &Path, index: usize) -> PathBuf {
    root.join("outputs").join(format_output_index(index))
}

fn format_output_index(index: usize) -> String {
    format!("{index:04}.artifact")
}

fn namespace_directory(spec: &ArtifactCacheSpec) -> PathBuf {
    namespace_directory_for(&spec.cache_root, &spec.namespace)
}

fn namespace_directory_for(cache_root: &Path, namespace: &str) -> PathBuf {
    cache_root
        .join(".ic-testkit/artifact-sets/namespaces")
        .join(identifier_digest("artifact-cache-namespace-v1", namespace))
}

fn entries_directory(namespace: &Path) -> PathBuf {
    namespace.join("entries")
}

fn entry_directory(namespace: &Path, key: InputDigest) -> PathBuf {
    entries_directory(namespace).join(key.to_hex())
}

fn coordination_lock_path(spec: &ArtifactCacheSpec) -> PathBuf {
    spec.cache_root
        .join(".ic-testkit/artifact-sets/locks/coordination")
        .join(format!(
            "{}.lock",
            identifier_digest("artifact-cache-coordination-v1", &spec.coordination_scope,)
        ))
}

fn content_lock_path(spec: &ArtifactCacheSpec, key: InputDigest) -> PathBuf {
    content_lock_path_for_key(&spec.cache_root, &key.to_hex())
}

fn content_lock_path_for_key(cache_root: &Path, key: &str) -> PathBuf {
    cache_root
        .join(".ic-testkit/artifact-sets/locks/content")
        .join(format!("{key}.lock"))
}

fn namespace_lock_path(spec: &ArtifactCacheSpec) -> PathBuf {
    namespace_lock_path_for(&spec.cache_root, &spec.namespace)
}

fn namespace_lock_path_for(cache_root: &Path, namespace: &str) -> PathBuf {
    cache_root
        .join(".ic-testkit/artifact-sets/locks/namespaces")
        .join(format!(
            "{}.lock",
            identifier_digest("artifact-cache-namespace-v1", namespace)
        ))
}

fn identifier_digest(domain: &str, identifier: &str) -> String {
    digest_bytes(domain, identifier.as_bytes()).to_hex()
}

fn artifact_cache_fs_error(error: CacheFsError) -> ArtifactCacheError {
    ArtifactCacheError::Io {
        operation: error.operation,
        path: error.path,
        source: error.source,
    }
}

impl ArtifactOutputValidation {
    const fn cache_token(self) -> &'static str {
        match self {
            Self::RegularFile => "regular-file",
            Self::NonEmptyFile => "nonempty-file",
        }
    }
}

impl std::fmt::Display for ArtifactCacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSpec { message } => {
                write!(formatter, "invalid artifact cache spec: {message}")
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
            Self::InputsChangedDuringPreparation { before, after } => write!(
                formatter,
                "artifact inputs repeatedly changed during cache preparation: {before} -> {after}",
            ),
            Self::InputsChangedDuringBuild { before, after } => write!(
                formatter,
                "artifact inputs changed while the caller was building: {before} -> {after}",
            ),
            Self::CargoBuildInputsChanged {
                label,
                before,
                after,
            } => write!(
                formatter,
                "resolved Cargo build inputs `{label}` changed: {before} -> {after}",
            ),
            Self::CargoBuildInputRevalidation { label, source } => write!(
                formatter,
                "failed to revalidate resolved Cargo build inputs `{label}`: {source}",
            ),
            Self::InvalidOutputs { outputs } => write!(
                formatter,
                "artifact transaction has missing or invalid outputs: {}",
                outputs
                    .iter()
                    .map(|(name, path)| format!("{name} ({})", path.display()))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Self::UnknownOutput { name } => {
                write!(
                    formatter,
                    "artifact transaction has no output named `{name}`"
                )
            }
            Self::FailedTransactionCleanup {
                transaction_error,
                path,
                source,
            } => write!(
                formatter,
                "artifact transaction failed ({transaction_error}) and staging cleanup at {} also failed: {source}",
                path.display(),
            ),
        }
    }
}

impl std::error::Error for ArtifactCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::FailedTransactionCleanup { source, .. } => Some(source),
            Self::CargoBuildInputRevalidation { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "transaction/tests.rs"]
mod tests;
