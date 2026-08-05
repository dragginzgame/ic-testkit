use fs2::FileExt as _;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
    time::{Duration, Instant},
};

use super::{
    digest::{
        InputDigest, InputHasher, digest_bytes, digest_labeled_paths, os_bytes, write_atomic,
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
    create_dir_all(&spec.target_dir, "create Cargo target directory")?;

    let lock_path = spec.target_dir.join(".ic-testkit/wasm-build.lock");
    let lock_file = open_lock_file(&lock_path)?;
    let lock_started = Instant::now();
    lock_file
        .lock_exclusive()
        .map_err(|source| WasmBuildError::Io {
            operation: "lock Wasm build cache",
            path: lock_path,
            source,
        })?;
    let lock_wait = lock_started.elapsed();

    let input_started = Instant::now();
    let resolved = build_fingerprint(spec)?;
    let mut input_resolution = input_started.elapsed();
    let fingerprint = resolved.fingerprint;
    let artifacts = expected_artifacts(spec, &spec.target_dir);

    if artifact_set_matches(&artifacts, fingerprint) {
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

    let build_target_dir = spec
        .target_dir
        .join(".ic-testkit/wasm-targets")
        .join(fingerprint.to_hex());
    let cached_artifacts = expected_artifacts(spec, &build_target_dir);
    if artifact_set_matches(&cached_artifacts, fingerprint) {
        materialize_artifacts(&cached_artifacts, &artifacts, fingerprint)?;
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
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(WasmBuildError::Io {
            operation: "remove incomplete content-addressed Cargo target directory",
            path: path.to_owned(),
            source,
        }),
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
        }
    }
}

impl std::error::Error for WasmBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::CommandSpawn { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{WasmBuildError, WasmBuildSpec, metadata_arguments, validate_spec};
    use std::{ffi::OsString, path::Path};

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
}
