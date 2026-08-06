use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use super::{
    cache_fs::{
        ArtifactCacheMaintenance, ArtifactCachePrunePolicy, ArtifactCachePruneReport, CacheFsError,
        ensure_cache_directory_tag, lock_cache_file, prune_direct_child_directories,
        record_cache_entry_use,
    },
    digest::{
        InputDigest, InputHasher, digest_bytes, digest_labeled_paths, os_bytes, write_atomic,
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
    outputs: Vec<OutputSpec>,
    prune_policy: Option<ArtifactCachePrunePolicy>,
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
            outputs: Vec::new(),
            prune_policy: None,
        }
    }

    /// Serialize recipes that share caller-owned mutable external build state.
    #[must_use]
    pub fn with_coordination_scope(mut self, coordination_scope: &str) -> Self {
        coordination_scope.clone_into(&mut self.coordination_scope);
        self
    }

    /// Add one exact input file or directory under a stable logical label.
    #[must_use]
    pub fn with_input(mut self, label: &str, path: &Path) -> Self {
        self.inputs.push(LabeledPath {
            label: label.to_owned(),
            path: path.to_owned(),
        });
        self
    }

    /// Add one exact executable or other tool file under a stable logical label.
    #[must_use]
    pub fn with_tool(mut self, label: &str, path: &Path) -> Self {
        self.tools.push(LabeledPath {
            label: label.to_owned(),
            path: path.to_owned(),
        });
        self
    }

    /// Set ordered command arguments that contribute to the content key.
    #[must_use]
    pub fn with_arguments(mut self, arguments: &[&str]) -> Self {
        self.arguments = arguments.iter().map(OsString::from).collect();
        self
    }

    /// Set environment values that contribute to the content key.
    #[must_use]
    pub fn with_environment(mut self, environment: &[(&str, &str)]) -> Self {
        self.environment.extend(
            environment
                .iter()
                .map(|(name, value)| (OsString::from(name), Some(OsString::from(value)))),
        );
        self
    }

    /// Record environment variables whose unset state contributes to the content key.
    #[must_use]
    pub fn with_unset_environment(mut self, names: &[&str]) -> Self {
        self.environment
            .extend(names.iter().map(|name| (OsString::from(name), None)));
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

    /// Declare one nonempty regular-file output and its public destination.
    #[must_use]
    pub fn with_output(self, name: &str, destination: &Path) -> Self {
        self.with_output_validation(name, destination, ArtifactOutputValidation::NonEmptyFile)
    }

    /// Declare one output, public destination, and built-in validation policy.
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
        self
    }

    /// Apply best-effort retention while protecting the acquired entry.
    #[must_use]
    pub const fn with_prune_policy(mut self, policy: ArtifactCachePrunePolicy) -> Self {
        self.prune_policy = Some(policy);
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
            .field("outputs", &self.outputs)
            .field("prune_policy", &self.prune_policy)
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
}

impl ArtifactBuildTransaction {
    /// Transaction-owned directory in which the caller may run its build.
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
        let contents = fs::read(source).map_err(|source_error| ArtifactCacheError::Io {
            operation: "read external artifact output",
            path: source.to_owned(),
            source: source_error,
        })?;
        write_atomic(&destination, &contents).map_err(|source_error| ArtifactCacheError::Io {
            operation: "import artifact output into staging",
            path: destination,
            source: source_error,
        })
    }

    /// Validate and atomically publish the complete staged output set.
    pub fn commit(mut self) -> Result<ArtifactCacheOutcome, ArtifactCacheError> {
        let result = self.commit_inner();
        match result {
            Ok(outcome) => Ok(outcome),
            Err(transaction_error) if self.staging_armed => {
                let path = self.staging_directory.clone();
                match remove_dir_all_if_present(&path) {
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
        remove_dir_all_if_present(&self.staging_directory).map_err(|source| {
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
        sorted_outputs(&self.spec)
            .iter()
            .position(|output| output.name == name)
    }

    fn commit_inner(&mut self) -> Result<ArtifactCacheOutcome, ArtifactCacheError> {
        self.timings.caller_build = Some(self.caller_build_started.elapsed());

        let validation_started = Instant::now();
        let output_info = inspect_complete_output_set(&self.spec, &self.staging_directory)?;
        self.timings.output_validation = validation_started.elapsed();

        let capture_started = Instant::now();
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
        remove_dir_all_if_present(&self.entry_directory).map_err(|source| {
            ArtifactCacheError::Io {
                operation: "remove conflicting artifact cache entry",
                path: self.entry_directory.clone(),
                source,
            }
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
        let (maintenance, maintenance_timing) =
            perform_maintenance(&self.spec, &self.namespace_directory, &self.entry_directory);
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
            let _ = remove_dir_all_if_present(&self.staging_directory);
        }
    }
}

/// Verify or begin one exact external artifact-set transaction.
pub fn prepare_artifact_cache(
    spec: &ArtifactCacheSpec,
) -> Result<ArtifactCachePreparation, ArtifactCacheError> {
    let total_started = Instant::now();
    validate_spec(spec)?;
    let namespace_directory = initialize_cache(spec)?;

    let coordination_lock_path = coordination_lock_path(spec);
    let (coordination_lock, coordination_wait) =
        lock_cache_file(&coordination_lock_path).map_err(artifact_cache_fs_error)?;
    let mut timings = ArtifactCacheTimings {
        coordination_lock_wait: coordination_wait,
        ..ArtifactCacheTimings::default()
    };
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
                perform_maintenance(spec, &namespace_directory, &entry_directory);
            timings.maintenance = maintenance_timing;
            timings.total = total_started.elapsed();
            return Ok(ArtifactCachePreparation::Reused(cache_record(
                spec,
                resolved,
                timings,
                maintenance,
            )));
        }

        remove_dir_all_if_present(&entry_directory).map_err(|source| ArtifactCacheError::Io {
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

/// Prune one transactional artifact namespace without acquiring an artifact.
///
/// This strict maintenance entry point returns cache errors directly. Build
/// acquisitions configured with [`ArtifactCacheSpec::with_prune_policy`] use
/// the same implementation but attach maintenance as a nonfatal record.
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
    let namespace_digest = identifier_digest("artifact-cache-namespace-v1", namespace);
    let namespace_directory = cache_root
        .join(".ic-testkit/artifact-sets/namespaces")
        .join(&namespace_digest);
    let lock_path = cache_root
        .join(".ic-testkit/artifact-sets/locks/namespaces")
        .join(format!("{namespace_digest}.lock"));
    let (_lock, _) = lock_cache_file(&lock_path).map_err(artifact_cache_fs_error)?;
    prune_direct_child_directories(
        &entries_directory(&namespace_directory),
        policy,
        None,
        is_content_key_directory,
    )
    .map_err(artifact_cache_fs_error)
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
    let mut paths = spec
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
    paths.shrink_to_fit();
    let input_digest = digest_labeled_paths(
        "artifact-set-inputs-v1",
        &paths,
        std::slice::from_ref(&spec.cache_root),
    )
    .map_err(|source| ArtifactCacheError::Io {
        operation: "hash artifact cache inputs",
        path: spec.cache_root.clone(),
        source,
    })?;

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
    for output in sorted_outputs(spec) {
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
    if !entry.is_dir() {
        return Ok(false);
    }
    let manifest = match fs::read_to_string(entry.join(MANIFEST_FILE)) {
        Ok(manifest) => manifest,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(ArtifactCacheError::Io {
                operation: "read artifact cache manifest",
                path: entry.join(MANIFEST_FILE),
                source,
            });
        }
    };
    let Some(output_info) = inspect_cached_output_set(spec, entry)? else {
        return Ok(false);
    };
    Ok(manifest == manifest_contents(key, spec, &output_info))
}

fn inspect_complete_output_set(
    spec: &ArtifactCacheSpec,
    root: &Path,
) -> Result<Vec<ArtifactInfo>, ArtifactCacheError> {
    let mut info = Vec::new();
    let mut invalid = Vec::new();
    let outputs = sorted_outputs(spec);
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
        undeclared_output_paths(root, outputs.len())?.map(|path| ("<undeclared>".to_owned(), path)),
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
    let outputs = sorted_outputs(spec);
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
    if undeclared_output_paths(root, outputs.len())?
        .next()
        .is_some()
    {
        return Ok(None);
    }
    Ok(Some(info))
}

fn undeclared_output_paths(
    root: &Path,
    output_count: usize,
) -> Result<impl Iterator<Item = PathBuf>, ArtifactCacheError> {
    let output_directory = root.join("outputs");
    let expected = (0..output_count)
        .map(format_output_index)
        .collect::<BTreeSet<_>>();
    let entries = fs::read_dir(&output_directory).map_err(|source| ArtifactCacheError::Io {
        operation: "read artifact output directory",
        path: output_directory.clone(),
        source,
    })?;
    let mut undeclared = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ArtifactCacheError::Io {
            operation: "read artifact output entry",
            path: output_directory.clone(),
            source,
        })?;
        if !expected.contains(&entry.file_name().to_string_lossy().into_owned()) {
            undeclared.push(entry.path());
        }
    }
    Ok(undeclared.into_iter())
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
    let contents = fs::read(path)?;
    if validation == ArtifactOutputValidation::NonEmptyFile && contents.is_empty() {
        return Ok(None);
    }
    Ok(Some(ArtifactInfo {
        bytes: u64::try_from(contents.len()).expect("artifact length must fit in u64"),
        digest: digest_bytes("artifact-set-output-v1", &contents),
    }))
}

fn manifest_contents(
    key: InputDigest,
    spec: &ArtifactCacheSpec,
    output_info: &[ArtifactInfo],
) -> String {
    let mut manifest = format!("{ARTIFACT_CACHE_FORMAT}\nkey:{key}\n");
    for ((index, output), info) in sorted_outputs(spec).iter().enumerate().zip(output_info) {
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
    for (index, output) in sorted_outputs(spec).iter().enumerate() {
        let cached = staged_output_path(entry, index);
        let contents = fs::read(&cached).map_err(|source| ArtifactCacheError::Io {
            operation: "read cached artifact output",
            path: cached,
            source,
        })?;
        write_atomic(&output.destination, &contents).map_err(|source| ArtifactCacheError::Io {
            operation: "materialize artifact output",
            path: output.destination.clone(),
            source,
        })?;
    }
    Ok(())
}

fn perform_maintenance(
    spec: &ArtifactCacheSpec,
    namespace: &Path,
    protected_entry: &Path,
) -> (Option<ArtifactCacheMaintenance>, Option<Duration>) {
    spec.prune_policy.map_or((None, None), |policy| {
        let started = Instant::now();
        let result = prune_direct_child_directories(
            &entries_directory(namespace),
            policy,
            Some(protected_entry),
            is_content_key_directory,
        );
        let maintenance = match result {
            Ok(report) => ArtifactCacheMaintenance::Pruned(report),
            Err(error) => ArtifactCacheMaintenance::PruneFailed {
                message: artifact_cache_fs_error(error).to_string(),
            },
        };
        (Some(maintenance), Some(started.elapsed()))
    })
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
        artifacts: sorted_outputs(spec)
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
            remove_dir_all_if_present(&entry.path()).map_err(|source| ArtifactCacheError::Io {
                operation: "remove abandoned artifact staging directory",
                path: entry.path(),
                source,
            })?;
        }
    }
    Ok(())
}

fn sorted_outputs(spec: &ArtifactCacheSpec) -> Vec<&OutputSpec> {
    let mut outputs = spec.outputs.iter().collect::<Vec<_>>();
    outputs.sort_by(|left, right| left.name.cmp(&right.name));
    outputs
}

fn staged_output_path(root: &Path, index: usize) -> PathBuf {
    root.join("outputs").join(format_output_index(index))
}

fn format_output_index(index: usize) -> String {
    format!("{index:04}.artifact")
}

fn namespace_directory(spec: &ArtifactCacheSpec) -> PathBuf {
    spec.cache_root
        .join(".ic-testkit/artifact-sets/namespaces")
        .join(identifier_digest(
            "artifact-cache-namespace-v1",
            &spec.namespace,
        ))
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
    spec.cache_root
        .join(".ic-testkit/artifact-sets/locks/content")
        .join(format!("{key}.lock"))
}

fn namespace_lock_path(spec: &ArtifactCacheSpec) -> PathBuf {
    spec.cache_root
        .join(".ic-testkit/artifact-sets/locks/namespaces")
        .join(format!(
            "{}.lock",
            identifier_digest("artifact-cache-namespace-v1", &spec.namespace)
        ))
}

fn identifier_digest(domain: &str, identifier: &str) -> String {
    digest_bytes(domain, identifier.as_bytes()).to_hex()
}

fn is_content_key_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let bytes = name.as_encoded_bytes();
        bytes.len() == 64 && bytes.iter().all(u8::is_ascii_hexdigit)
    })
}

fn remove_dir_all_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
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
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactCacheError, ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec,
        ArtifactOutputValidation, entry_directory, namespace_directory, prepare_artifact_cache,
        prune_artifact_cache, resolve_key,
    };
    use crate::artifacts::{ArtifactCacheMaintenance, ArtifactCachePrunePolicy};
    use std::{
        fs,
        panic::{AssertUnwindSafe, catch_unwind},
        path::{Path, PathBuf},
        sync::{
            Arc, Barrier,
            atomic::{AtomicU64, AtomicUsize, Ordering},
        },
        thread,
    };

    static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn one_output_is_built_materialized_repaired_and_reused() {
        let root = unique_temp_directory("one-output");
        let input = root.join("input.wasm");
        let destination = root.join("public/optimized.wasm");
        fs::write(&input, b"raw-wasm").expect("write input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "optimizer", "pipeline/v1")
            .with_input("raw-wasm", &input)
            .with_arguments(&["-O3", "--strip-debug"])
            .with_output("optimized.wasm", &destination);
        assert!(!destination.starts_with(spec.cache_root()));

        let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
        fs::write(
            transaction
                .output_path("optimized.wasm")
                .expect("declared output path"),
            b"optimized-wasm",
        )
        .expect("write staged output");
        let built = transaction.commit().expect("commit artifact transaction");

        assert!(matches!(built, ArtifactCacheOutcome::Built(_)));
        assert_eq!(fs::read(&destination).unwrap(), b"optimized-wasm");
        fs::write(&destination, b"tampered-public-output").expect("tamper public output");

        let reused = prepare_artifact_cache(&spec).expect("prepare exact reuse");
        let record = reused.reused_record().expect("expected exact reuse");
        assert_eq!(record.key(), built.record().key());
        assert_eq!(record.artifacts()[0].name(), "optimized.wasm");
        assert_eq!(fs::read(&destination).unwrap(), b"optimized-wasm");
        assert!(record.timings().caller_build().is_none());
        fs::remove_dir_all(root).expect("remove one-output test directory");
    }

    #[test]
    fn multi_output_commit_is_complete_and_name_order_independent() {
        let root = unique_temp_directory("multi-output");
        let input = root.join("source");
        fs::write(&input, b"source").expect("write input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "release-set", "recipe/v2")
            .with_input("source", &input)
            .with_output("role-b.wasm", &root.join("public/role-b.wasm"))
            .with_output("metadata.json", &root.join("public/metadata.json"))
            .with_output("root.wasm", &root.join("public/root.wasm"));
        let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
        for (name, contents) in [
            ("root.wasm", b"root".as_slice()),
            ("role-b.wasm", b"role-b".as_slice()),
            ("metadata.json", b"{}".as_slice()),
        ] {
            fs::write(transaction.output_path(name).unwrap(), contents)
                .expect("write staged output");
        }

        let outcome = transaction.commit().expect("commit complete output set");

        assert_eq!(
            outcome
                .record()
                .artifacts()
                .iter()
                .map(super::ArtifactCacheArtifact::name)
                .collect::<Vec<_>>(),
            ["metadata.json", "role-b.wasm", "root.wasm"],
        );
        assert!(matches!(
            prepare_artifact_cache(&spec).unwrap(),
            ArtifactCachePreparation::Reused(_)
        ));
        fs::remove_dir_all(root).expect("remove multi-output test directory");
    }

    #[test]
    fn incomplete_output_set_fails_and_removes_staging() {
        let root = unique_temp_directory("incomplete-output");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "batch", "recipe/v1")
            .with_input("input", &input)
            .with_output("first", &root.join("first"))
            .with_output("second", &root.join("second"));
        let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
        fs::write(transaction.output_path("first").unwrap(), b"first")
            .expect("write only first output");
        let staging = transaction.staging_directory().to_owned();

        let error = transaction
            .commit()
            .expect_err("partial transaction must fail");

        assert!(matches!(error, ArtifactCacheError::InvalidOutputs { .. }));
        assert!(!staging.exists());
        assert!(matches!(
            prepare_artifact_cache(&spec).unwrap(),
            ArtifactCachePreparation::Build(_)
        ));
        fs::remove_dir_all(root).expect("remove incomplete-output test directory");
    }

    #[test]
    fn changed_inputs_reject_commit_and_remove_staging() {
        let root = unique_temp_directory("changed-inputs");
        let input = root.join("input");
        fs::write(&input, b"before").expect("write original input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "transform", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));
        let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
        fs::write(transaction.output_path("output").unwrap(), b"output")
            .expect("write staged output");
        let staging = transaction.staging_directory().to_owned();
        fs::write(&input, b"after").expect("change input during transaction");

        let error = transaction
            .commit()
            .expect_err("input race must reject commit");

        assert!(matches!(
            error,
            ArtifactCacheError::InputsChangedDuringBuild { .. }
        ));
        assert!(!staging.exists());
        fs::remove_dir_all(root).expect("remove changed-inputs test directory");
    }

    #[test]
    fn dropped_and_panicked_transactions_remove_staging() {
        let root = unique_temp_directory("dropped-transactions");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "drop", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));

        let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
        let dropped_staging = transaction.staging_directory().to_owned();
        drop(transaction);
        assert!(!dropped_staging.exists());

        let panicked_staging = Arc::new(std::sync::Mutex::new(None));
        let captured_staging = Arc::clone(&panicked_staging);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
            *captured_staging.lock().unwrap() = Some(transaction.staging_directory().to_owned());
            panic!("synthetic caller panic");
        }));
        assert!(result.is_err());
        assert!(!panicked_staging.lock().unwrap().as_ref().unwrap().exists());
        fs::remove_dir_all(root).expect("remove dropped-transactions test directory");
    }

    #[test]
    fn every_declared_identity_dimension_changes_the_content_key() {
        let root = unique_temp_directory("identity-dimensions");
        let input = root.join("input");
        let tool = root.join("tool");
        fs::write(&input, b"input-v1").expect("write input");
        fs::write(&tool, b"tool-v1").expect("write tool");
        let base = ArtifactCacheSpec::new(&root.join("cache"), "identity", "recipe/v1")
            .with_input("input", &input)
            .with_tool("optimizer", &tool)
            .with_arguments(&["-O2"])
            .with_environment(&[("MODE", "release")])
            .with_identity_bytes("pipeline", b"one")
            .with_output("output", &root.join("output"));
        let original = resolve_key(&base).unwrap().key;
        let changed_argument = resolve_key(&base.clone().with_arguments(&["-O3"]))
            .unwrap()
            .key;
        let changed_environment =
            resolve_key(&base.clone().with_environment(&[("MODE", "size-optimized")]))
                .unwrap()
                .key;
        let changed_recipe = resolve_key(&ArtifactCacheSpec {
            recipe_id: "recipe/v2".to_owned(),
            ..base.clone()
        })
        .unwrap()
        .key;
        fs::write(&tool, b"tool-v2").expect("change tool bytes");
        let changed_tool = resolve_key(&base).unwrap().key;
        fs::write(&tool, b"tool-v1").expect("restore tool bytes");
        fs::write(&input, b"input-v2").expect("change input bytes");
        let changed_input = resolve_key(&base).unwrap().key;

        for changed in [
            changed_argument,
            changed_environment,
            changed_recipe,
            changed_tool,
            changed_input,
        ] {
            assert_ne!(original, changed);
        }
        fs::remove_dir_all(root).expect("remove identity-dimensions test directory");
    }

    #[test]
    fn tampered_cache_entry_is_rebuilt_instead_of_reused() {
        let root = unique_temp_directory("tampered-entry");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "tamper", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));
        let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
        fs::write(transaction.output_path("output").unwrap(), b"valid")
            .expect("write staged output");
        let outcome = transaction.commit().expect("commit valid entry");
        let entry = entry_directory(&namespace_directory(&spec), outcome.record().key());
        fs::write(entry.join("outputs/0000.artifact"), b"tampered").expect("tamper cached output");

        let rebuilt = prepare_artifact_cache(&spec).expect("prepare after corruption");

        assert!(matches!(rebuilt, ArtifactCachePreparation::Build(_)));
        fs::remove_dir_all(root).expect("remove tampered-entry test directory");
    }

    #[test]
    fn pruning_protects_active_entry_and_removes_older_key() {
        let root = unique_temp_directory("transaction-pruning");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let base = ArtifactCacheSpec::new(&root.join("cache"), "prune", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));
        build_output(&base.clone().with_arguments(&["old"]), b"old");
        let active = base
            .clone()
            .with_arguments(&["active"])
            .with_prune_policy(ArtifactCachePrunePolicy::new().with_max_size_bytes(0));

        let outcome = build_output(&active, b"active");
        let report = outcome
            .record()
            .maintenance()
            .and_then(ArtifactCacheMaintenance::prune_report)
            .expect("successful configured pruning");

        assert_eq!(report.entries_scanned(), 2);
        assert_eq!(report.entries_removed(), 1);
        assert_eq!(report.entries_retained(), 1);
        assert!(matches!(
            prepare_artifact_cache(&base.with_arguments(&["old"])).unwrap(),
            ArtifactCachePreparation::Build(_)
        ));
        assert!(matches!(
            prepare_artifact_cache(&active).unwrap(),
            ArtifactCachePreparation::Reused(_)
        ));
        let strict = prune_artifact_cache(
            active.cache_root(),
            active.namespace(),
            ArtifactCachePrunePolicy::new().with_max_size_bytes(0),
        )
        .expect("strict namespace pruning");
        assert_eq!(strict.entries_removed(), 1);
        assert!(matches!(
            prepare_artifact_cache(&active).unwrap(),
            ArtifactCachePreparation::Build(_)
        ));
        fs::remove_dir_all(root).expect("remove transaction-pruning test directory");
    }

    #[test]
    fn overlapping_exact_acquisitions_build_once() {
        let root = unique_temp_directory("overlapping-acquisitions");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let spec = Arc::new(
            ArtifactCacheSpec::new(&root.join("cache"), "concurrent", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("output")),
        );
        let start = Arc::new(Barrier::new(3));
        let builds = Arc::new(AtomicUsize::new(0));
        let workers = std::array::from_fn::<_, 2, _>(|_| {
            let spec = Arc::clone(&spec);
            let start = Arc::clone(&start);
            let builds = Arc::clone(&builds);
            thread::spawn(move || {
                start.wait();
                match prepare_artifact_cache(&spec).expect("prepare overlapping acquisition") {
                    ArtifactCachePreparation::Reused(record) => record.key(),
                    ArtifactCachePreparation::Build(transaction) => {
                        builds.fetch_add(1, Ordering::SeqCst);
                        fs::write(transaction.output_path("output").unwrap(), b"built")
                            .expect("write concurrent staged output");
                        transaction
                            .commit()
                            .expect("commit concurrent output")
                            .record()
                            .key()
                    }
                }
            })
        });
        start.wait();
        let keys = workers.map(|worker| worker.join().expect("worker must not panic"));

        assert_eq!(keys[0], keys[1]);
        assert_eq!(builds.load(Ordering::SeqCst), 1);
        fs::remove_dir_all(root).expect("remove overlapping-acquisitions test directory");
    }

    #[test]
    fn different_keys_sharing_a_coordination_scope_do_not_overlap() {
        let root = unique_temp_directory("shared-coordination");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let base = ArtifactCacheSpec::new(&root.join("cache"), "coordinated", "recipe/v1")
            .with_coordination_scope("shared-external-tree")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));
        let start = Arc::new(Barrier::new(3));
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let workers = ["first", "second"].map(|argument| {
            let spec = base.clone().with_arguments(&[argument]);
            let start = Arc::clone(&start);
            let active = Arc::clone(&active);
            let maximum = Arc::clone(&maximum);
            thread::spawn(move || {
                start.wait();
                let transaction = expect_build(
                    prepare_artifact_cache(&spec).expect("prepare coordinated transaction"),
                );
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(current, Ordering::SeqCst);
                thread::sleep(std::time::Duration::from_millis(20));
                fs::write(transaction.output_path("output").unwrap(), argument)
                    .expect("write coordinated output");
                active.fetch_sub(1, Ordering::SeqCst);
                transaction.commit().expect("commit coordinated output");
            })
        });
        start.wait();
        for worker in workers {
            worker.join().expect("coordinated worker must not panic");
        }

        assert_eq!(maximum.load(Ordering::SeqCst), 1);
        fs::remove_dir_all(root).expect("remove shared-coordination test directory");
    }

    #[test]
    fn content_identity_is_independent_of_checkout_and_destination_paths() {
        let first = unique_temp_directory("checkout-first");
        let second = unique_temp_directory("checkout-second");
        for root in [&first, &second] {
            fs::create_dir_all(root.join("source")).expect("create source directory");
            fs::write(root.join("source/input"), b"same input").expect("write input");
            fs::write(root.join("tool"), b"same tool").expect("write tool");
        }
        let spec = |root: &Path| {
            ArtifactCacheSpec::new(&root.join("cache"), "portable", "recipe/v1")
                .with_input("source", &root.join("source"))
                .with_tool("optimizer", &root.join("tool"))
                .with_arguments(&["--exact"])
                .with_output("output", &root.join("different/public/output"))
        };

        let first_key = resolve_key(&spec(&first)).unwrap();
        let second_key = resolve_key(&spec(&second)).unwrap();

        assert_eq!(first_key.key, second_key.key);
        assert_eq!(first_key.input_digest, second_key.input_digest);
        fs::remove_dir_all(first).expect("remove first checkout");
        fs::remove_dir_all(second).expect("remove second checkout");
    }

    #[test]
    fn import_helper_and_debug_output_do_not_expose_identity_values() {
        let root = unique_temp_directory("import-output");
        let input = root.join("input");
        let external = root.join("fixed-build-location/output");
        fs::create_dir_all(external.parent().unwrap()).expect("create fixed output directory");
        fs::write(&input, b"input").expect("write input");
        fs::write(&external, b"external-output").expect("write external output");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "import", "recipe/v1")
            .with_input("input", &input)
            .with_environment(&[("SECRET_TOKEN", "do-not-render")])
            .with_identity_bytes("private-ish", b"also-do-not-render")
            .with_output("output", &root.join("public/output"));
        let debug = format!("{spec:?}");
        assert!(debug.contains("SECRET_TOKEN"));
        assert!(!debug.contains("do-not-render"));
        assert!(!debug.contains("also-do-not-render"));
        let transaction = expect_build(prepare_artifact_cache(&spec).unwrap());

        transaction
            .import_output("output", &external)
            .expect("import external output");
        let outcome = transaction.commit().expect("commit imported output");

        assert_eq!(
            fs::read(outcome.record().artifacts()[0].path()).unwrap(),
            b"external-output"
        );
        fs::remove_dir_all(root).expect("remove import-output test directory");
    }

    #[test]
    fn undeclared_staging_output_rejects_the_transaction() {
        let root = unique_temp_directory("undeclared-output");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "extra", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));
        let transaction = expect_build(prepare_artifact_cache(&spec).unwrap());
        fs::write(transaction.output_path("output").unwrap(), b"declared")
            .expect("write declared output");
        fs::write(
            transaction.staging_directory().join("outputs/extra"),
            b"undeclared",
        )
        .expect("write undeclared output");

        assert!(matches!(
            transaction.commit(),
            Err(ArtifactCacheError::InvalidOutputs { .. })
        ));
        fs::remove_dir_all(root).expect("remove undeclared-output test directory");
    }

    #[test]
    fn empty_output_requires_explicit_regular_file_validation() {
        let root = unique_temp_directory("empty-output");
        let input = root.join("input");
        fs::write(&input, b"input").expect("write input");
        let default_spec = ArtifactCacheSpec::new(&root.join("cache"), "empty", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &root.join("output"));
        let transaction = expect_build(prepare_artifact_cache(&default_spec).unwrap());
        fs::write(transaction.output_path("output").unwrap(), b"").expect("write empty output");
        assert!(matches!(
            transaction.commit(),
            Err(ArtifactCacheError::InvalidOutputs { .. })
        ));

        let regular_spec = ArtifactCacheSpec::new(&root.join("cache"), "empty", "recipe/v1")
            .with_input("input", &input)
            .with_output_validation(
                "output",
                &root.join("output"),
                ArtifactOutputValidation::RegularFile,
            );
        let transaction = expect_build(prepare_artifact_cache(&regular_spec).unwrap());
        fs::write(transaction.output_path("output").unwrap(), b"").expect("write empty output");
        transaction
            .commit()
            .expect("commit explicitly valid empty file");
        fs::remove_dir_all(root).expect("remove empty-output test directory");
    }

    fn expect_build(preparation: ArtifactCachePreparation) -> super::ArtifactBuildTransaction {
        match preparation {
            ArtifactCachePreparation::Build(transaction) => transaction,
            ArtifactCachePreparation::Reused(_) => panic!("expected a cache miss transaction"),
        }
    }

    fn build_output(spec: &ArtifactCacheSpec, contents: &[u8]) -> ArtifactCacheOutcome {
        let transaction = expect_build(prepare_artifact_cache(spec).expect("prepare build"));
        fs::write(transaction.output_path("output").unwrap(), contents)
            .expect("write staged output");
        transaction.commit().expect("commit output")
    }

    fn unique_temp_directory(label: &str) -> PathBuf {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ic-testkit-artifact-cache-{label}-{}-{sequence}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale test directory");
        }
        fs::create_dir_all(&path).expect("create test directory");
        path
    }
}
