use super::{
    IncompleteBuildDirectory, MetadataPackage, ProgressReporter,
    SharedIncrementalTargetMaintenanceConfig, SharedIncrementalTargetMaintenanceFailureMode,
    SharedIncrementalTargetMaintenanceOutcome, SharedIncrementalTargetPrunePolicy,
    WasmBuildBatchInputResolver, WasmBuildError, WasmBuildOutcome, WasmBuildOutputStream,
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildProgressPhase, WasmBuildSpec,
    append_cargo_configuration_inputs, build_wasm_canisters_cached_with_progress,
    ensure_cache_directory_tag, finish_fingerprint_build, inspect_shared_incremental_target,
    integrated_shared_maintenance_result, lock_wasm_build_cache,
    lock_wasm_build_cache_with_progress, locked_package_identities,
    maintain_shared_incremental_target, maintain_shared_incremental_target_at_most_every,
    metadata_arguments, perform_configured_shared_incremental_target_maintenance,
    prune_wasm_build_cache, prune_wasm_build_cache_locked, resolve_cargo_build_inputs,
    run_cargo_build, semantic_package_identity, validate_spec,
};
use crate::artifacts::cache_fs::{
    ArtifactCachePrunePolicy, CACHE_DIRECTORY_TAG_SIGNATURE, directory_logical_size,
    write_last_used,
};
use crate::artifacts::test_support::unique_temp_directory;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use crate::artifacts::test_support::write_executable_script;

fn canonical_fixture(path: &Path) -> PathBuf {
    path.canonicalize().expect("canonicalize test fixture path")
}

fn write_projection_package(root: &Path, package: &str, manifest_suffix: &str) {
    let directory = root.join(package);
    fs::create_dir_all(directory.join("src")).expect("create projection fixture package");
    fs::write(
        directory.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{manifest_suffix}"
        ),
    )
    .expect("write projection fixture package manifest");
    fs::write(
        directory.join("src/lib.rs"),
        format!("pub fn {package}() {{}}\n"),
    )
    .expect("write projection fixture source");
}

#[test]
#[cfg(unix)]
fn package_selection_agrees_across_identity_artifacts_and_cargo_invocation() {
    let root = unique_temp_directory("canonical-wasm-packages");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"alpha\", \"zeta\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    write_projection_package(&root, "alpha", "");
    write_projection_package(&root, "zeta", "");
    let target = root.join("exact");
    let ordered = WasmBuildSpec::new(&root, &target, &["alpha", "zeta"], "debug");
    let redundant =
        WasmBuildSpec::new(&root, &target, &["zeta", "alpha", "zeta", "alpha"], "debug");
    assert_eq!(redundant.packages(), ordered.packages());
    assert_eq!(redundant, ordered);

    let ordered_inputs = resolve_cargo_build_inputs(&ordered).unwrap();
    let redundant_inputs = resolve_cargo_build_inputs(&redundant).unwrap();
    assert_eq!(redundant_inputs.fingerprint(), ordered_inputs.fingerprint());
    assert_eq!(redundant_inputs.inputs(), ordered_inputs.inputs());
    assert_eq!(
        super::expected_artifacts(&redundant, &target),
        ["alpha.wasm", "zeta.wasm"]
            .map(|name| target.join("wasm32-unknown-unknown/debug").join(name)),
    );

    // Read a script through the shell, avoiding execution of a newly written
    // fixture. Capture the actual arguments in both command-output modes.
    fs::write(
        root.join("build"),
        "printf '%s\\n' \"$@\" > \"$IC_TESTKIT_PACKAGE_ARGUMENTS\"\n",
    )
    .unwrap();
    let arguments = root.join("arguments");
    let spec = redundant
        .with_cargo_program("/bin/sh")
        .with_extra_env([("IC_TESTKIT_PACKAGE_ARGUMENTS", arguments.as_os_str())]);
    for observed in [false, true] {
        let mut observer = |_| {};
        let mut progress = if observed {
            ProgressReporter::observed(
                WasmBuildProgressConfig::new().without_heartbeats(),
                &mut observer,
            )
        } else {
            ProgressReporter::silent()
        };
        run_cargo_build(&spec, &root.join("cargo-target"), &mut progress).unwrap();
        assert_eq!(
            fs::read_to_string(&arguments).unwrap(),
            "--lib\n--target\nwasm32-unknown-unknown\n-p\nalpha\n-p\nzeta\n",
        );
        fs::remove_file(&arguments).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

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
fn wasm_stamps_require_exact_identity_and_unmodified_contents() {
    use super::{artifact_stamp_path, publish_artifact_stamps, validated_artifact_set};
    use crate::artifacts::digest::digest_bytes;

    let root = unique_temp_directory("wasm-stamp-validation");
    let artifact = root.join("fixture.wasm");
    let artifacts = [artifact.clone()];
    let fingerprint = digest_bytes("stamp-fixture-v1", b"first");
    let other_fingerprint = digest_bytes("stamp-fixture-v1", b"second");
    let original = b"\0asm\x01\0\0\0";
    fs::write(&artifact, original).expect("write artifact");
    assert!(validated_artifact_set(&artifacts, fingerprint).is_none());
    publish_artifact_stamps(&artifacts, fingerprint).expect("publish stamp");
    let stamp_path = artifact_stamp_path(&artifact);
    let stamp = fs::read_to_string(&stamp_path).expect("read stamp");
    let artifact_digest = digest_bytes("wasm-artifact-v1", original);
    assert_eq!(
        stamp,
        format!(
            "ic-testkit-wasm-build-v1\nbuild-sha256:{fingerprint}\nartifact-sha256:{artifact_digest}\n"
        ),
    );
    assert!(validated_artifact_set(&artifacts, fingerprint).is_some());
    assert!(validated_artifact_set(&artifacts, other_fingerprint).is_none());

    // A correct build header still requires checking the artifact's bytes.
    fs::write(&artifact, b"\0asm\x02\0\0\0").expect("modify same-size artifact");
    assert!(validated_artifact_set(&artifacts, fingerprint).is_none());
    fs::write(&artifact, original).expect("restore artifact bytes");
    assert!(validated_artifact_set(&artifacts, fingerprint).is_some());

    for invalid in [
        stamp.replace("ic-testkit-wasm-build-v1", "invalid-format"),
        stamp.replace(&fingerprint.to_hex(), &other_fingerprint.to_hex()),
        stamp.replace("artifact-sha256:", "unknown-digest:"),
        format!("{stamp}extra\n"),
        stamp.trim_end().to_owned(),
    ] {
        fs::write(&stamp_path, invalid).expect("write invalid stamp");
        assert!(validated_artifact_set(&artifacts, fingerprint).is_none());
    }
    fs::write(&stamp_path, [0xff]).expect("write invalid UTF-8 stamp");
    assert!(validated_artifact_set(&artifacts, fingerprint).is_none());

    // A matching header must not turn an oversized sidecar into an unbounded read.
    fs::write(&stamp_path, &stamp).expect("write matching stamp prefix");
    fs::OpenOptions::new()
        .write(true)
        .open(&stamp_path)
        .expect("open oversized stamp")
        .set_len(1024 * 1024 * 1024)
        .expect("extend oversized stamp");
    assert!(validated_artifact_set(&artifacts, fingerprint).is_none());
    publish_artifact_stamps(&artifacts, fingerprint).expect("replace oversized stamp");
    assert!(validated_artifact_set(&artifacts, fingerprint).is_some());
    fs::remove_dir_all(root).expect("remove stamp fixture");
}

#[test]
fn compact_feature_arguments_watch_enabled_optional_dependencies() {
    use crate::artifacts::WasmBuildInputSnapshot;

    let root = unique_temp_directory("compact-feature-inputs");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\", \"optional_dep\"]\nresolver = \"2\"\n",
    )
    .expect("write optional dependency workspace");
    write_projection_package(
        &root,
        "fixture",
        "[features]\nextra = [\"dep:optional_dep\"]\nmap = [\"dep:optional_dep\"]\n\
         [dependencies]\noptional_dep = { path = \"../optional_dep\", optional = true }\n",
    );
    write_projection_package(&root, "optional_dep", "");
    let dependency = root.join("optional_dep");
    let dependency_source = dependency.join("src/lib.rs");
    let base = WasmBuildSpec::new(&root, &root.join("target"), &["fixture"], "debug");

    for arguments in [
        vec!["-Fextra"],
        vec!["-F=extra"],
        vec!["-qFextra"],
        vec!["-qFmap"],
        vec!["-rF=extra"],
        vec!["-vF", "extra"],
    ] {
        fs::write(&dependency_source, "pub fn optional_dep() -> u8 { 1 }\n")
            .expect("reset optional dependency source");
        let mut spec = base.clone().with_cargo_profile_args(&arguments);
        if arguments == ["-rF=extra"] {
            spec.profile_target_dir = "release".to_owned();
        }
        validate_spec(&spec).expect("accept feature selection without input overrides");
        let snapshot = resolve_cargo_build_inputs(&spec).expect("resolve enabled dependency");
        assert!(
            snapshot
                .inputs()
                .iter()
                .any(|input| input.path() == dependency),
            "{arguments:?} must include the enabled optional dependency",
        );
        assert!(
            snapshot
                .is_content_current()
                .expect("check unchanged inputs")
        );
        {
            let source_exclusion = std::sync::Mutex::new(());
            let guard = source_exclusion.lock().expect("lock source fixture");
            let prepared = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
                &guard,
                &[base.clone(), spec],
            )
            .expect("prepare distinct feature graphs");
            assert_eq!(prepared.metrics().input_resolution_runs(), 2);
            assert_eq!(prepared.metrics().input_resolution_reuses(), 0);
        }
        fs::write(&dependency_source, "pub fn optional_dep() -> u8 { 2 }\n")
            .expect("change optional dependency source");
        assert!(
            !snapshot
                .is_content_current()
                .expect("detect dependency change"),
            "{arguments:?} must watch the enabled optional dependency",
        );
    }
    fs::remove_dir_all(root).expect("remove compact feature fixture");
}

#[test]
fn builders_preserve_os_native_values() {
    let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &["fixture"], "debug")
        .with_cargo_profile_args([OsString::from("--locked")])
        .with_extra_env([(OsString::from("MODE"), OsString::from("exact"))])
        .with_inherited_env([OsString::from("RUSTFLAGS")])
        .with_additional_inputs([PathBuf::from("schema")]);

    assert_eq!(spec.cargo_profile_args, [OsString::from("--locked")]);
    assert_eq!(
        spec.extra_env.get(&OsString::from("MODE")),
        Some(&OsString::from("exact"))
    );
    assert!(spec.inherited_env.contains(&OsString::from("RUSTFLAGS")));
    assert_eq!(spec.additional_inputs, [PathBuf::from("schema")]);
}

#[test]
fn cargo_target_overrides_are_rejected_before_acquisition() {
    use super::build_wasm_canisters_cached;
    let root = unique_temp_directory("cargo-target-overrides");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug");
    for override_spec in [
        spec.clone().with_extra_env([("CARGO_TARGET_DIR", "other")]),
        spec.clone()
            .with_cargo_profile_args(["--target-dir", "other"]),
        spec.with_cargo_profile_args(["--target-dir=other"]),
    ] {
        assert!(matches!(
            build_wasm_canisters_cached(&override_spec),
            Err(WasmBuildError::InvalidSpec { .. })
        ));
    }
    assert!(!root.join("exact").exists());
    fs::remove_dir_all(root).expect("remove Cargo target override fixture");
}

#[test]
#[cfg(unix)]
fn cargo_non_building_commands_cannot_publish_existing_shared_target_outputs() {
    use super::{build_wasm_canisters_cached, expected_artifacts};
    use crate::artifacts::test_support::fake_wasm_build_spec;

    // These modes must be rejected before invoking tools or certifying an old output.
    for arguments in [vec!["--help"], vec!["--unit-graph", "-Zunstable-options"]] {
        let (root, spec) = fake_wasm_build_spec("cargo-non-build-shared-output");
        let shared = root.join("incremental");
        let spec = spec
            .with_cargo_program(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .with_cargo_profile_args(&arguments)
            .with_shared_incremental_target(&shared);
        let existing = expected_artifacts(&spec, &shared);
        fs::create_dir_all(existing[0].parent().unwrap()).unwrap();
        fs::write(&existing[0], b"\0asm\x01\0\0\0").unwrap();

        let result = build_wasm_canisters_cached(&spec);
        assert!(
            matches!(result, Err(WasmBuildError::InvalidSpec { .. })),
            "{arguments:?} must not publish existing output: {result:?}",
        );
        assert!(!spec.target_dir.exists());
        assert_eq!(fs::read(&existing[0]).unwrap(), b"\0asm\x01\0\0\0");
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn cargo_input_overrides_are_rejected_before_resolution_or_acquisition() {
    use super::build_wasm_canisters_cached;

    let root = unique_temp_directory("cargo-input-overrides");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_cargo_program(root.join("missing-cargo"));
    for arguments in [
        vec!["--manifest-path", "other/Cargo.toml"],
        vec!["--manifest-path=other/Cargo.toml"],
        vec!["-m", "other/Cargo.toml"],
        vec!["-mother/Cargo.toml"],
        vec!["-qmother/Cargo.toml"],
        vec!["-C", "other"],
        vec!["-vCother"],
        vec!["--target", "other-target"],
        vec!["--target=other-target"],
        vec!["--package", "other"],
        vec!["--package=other"],
        vec!["-p", "other"],
        vec!["-pother"],
        vec!["-qpother"],
        vec!["--workspace"],
        vec!["--all"],
        vec!["--exclude", "fixture"],
        vec!["--exclude=fixture"],
        vec!["--config", "other.toml"],
        vec!["--config=build.rustflags=['--cfg=other']"],
        vec!["--help"],
        vec!["-h"],
        vec!["-qh"],
        vec!["-rh"],
        vec!["-vh"],
        vec!["-vrhFextra"],
        vec!["--unit-graph"],
        vec!["--unit-graph", "-Zunstable-options"],
        vec!["-Zunstable-options", "--unit-graph"],
    ] {
        let override_spec = spec.clone().with_cargo_profile_args(&arguments);
        for result in [
            resolve_cargo_build_inputs(&override_spec).map(|_| ()),
            build_wasm_canisters_cached(&override_spec).map(|_| ()),
        ] {
            assert!(
                matches!(result, Err(WasmBuildError::InvalidSpec { .. })),
                "{arguments:?} must fail before launching Cargo: {result:?}",
            );
        }
    }
    // Compact feature values containing 'h' are values, not help switches.
    for arguments in [
        vec!["-Fh"],
        vec!["-qFhighlights"],
        vec!["-rF=highlights"],
        vec!["-F", "highlights"],
        vec!["--features=highlights"],
        vec!["--features", "highlights"],
    ] {
        let mut feature_spec = spec.clone().with_cargo_profile_args(&arguments);
        if arguments == ["-rF=highlights"] {
            feature_spec.profile_target_dir = "release".to_owned();
        }
        super::validate_spec(&feature_spec).expect("accept feature values containing 'h'");
    }
    assert!(!root.join("exact").exists());
    fs::remove_dir_all(root).expect("remove Cargo input override fixture");
}

#[cfg(unix)]
#[test]
fn shared_target_boundaries_preserve_retained_exact_entries() {
    use super::build_wasm_canisters_cached;
    use crate::artifacts::test_support::fake_wasm_build_spec;

    let (root, base) = fake_wasm_build_spec("shared-exact-boundary");
    let unsafe_shared = root.join("shared");
    let mut nested = base.clone();
    nested.target_dir = unsafe_shared.join("exact");
    // A retained isolated build already exists before shared maintenance is enabled.
    let retained = build_wasm_canisters_cached(&nested).expect("build retained exact artifact");
    let artifact = retained.record().artifacts()[0].clone();
    nested = nested.with_shared_incremental_target(&unsafe_shared);
    let policy = SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(0);
    assert!(matches!(
        build_wasm_canisters_cached(&nested),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(matches!(
        maintain_shared_incremental_target(&nested, policy),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(matches!(
        maintain_shared_incremental_target_at_most_every(&nested, policy, Duration::ZERO),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(artifact.is_file());

    let inside_entries = base.clone().with_shared_incremental_target(
        base.target_dir.join(".ic-testkit/wasm-targets/incremental"),
    );
    assert!(matches!(
        resolve_cargo_build_inputs(&inside_entries),
        Err(WasmBuildError::InvalidSpec { .. })
    ));

    // Sharing the exact target container remains safe: its metadata is preserved.
    let same_root = base
        .clone()
        .with_shared_incremental_target(&base.target_dir);
    let safe = build_wasm_canisters_cached(&same_root).expect("build with common target container");
    maintain_shared_incremental_target(&same_root, policy)
        .expect("maintain common target container");
    assert!(safe.record().artifacts()[0].is_file());
    let sibling = base.with_shared_incremental_target(root.join("incremental"));
    resolve_cargo_build_inputs(&sibling).expect("separate target roots remain valid");
    drop(safe);
    drop(retained);
    fs::remove_dir_all(root).expect("remove shared boundary fixture");
}

#[cfg(unix)]
#[test]
fn relative_exact_target_is_shared_by_cargo_and_cache_operations() {
    use super::build_wasm_canisters_cached;
    use crate::artifacts::test_support::fake_wasm_build_spec;

    let (root, mut spec) = fake_wasm_build_spec("relative-exact-target");
    let caller_dir = std::env::current_dir().expect("read caller working directory");
    spec.workspace_root = root.join("nested/workspace/one/two/three/four");
    fs::create_dir_all(&spec.workspace_root).expect("create separate Cargo working directory");
    fs::rename(
        root.join("Cargo.toml"),
        spec.workspace_root.join("Cargo.toml"),
    )
    .expect("move workspace manifest");
    fs::rename(root.join("fixture"), spec.workspace_root.join("fixture"))
        .expect("move workspace package");
    // Reach an owned /tmp target using a relative path without changing the
    // process-wide working directory or creating test output in the checkout.
    spec.target_dir = caller_dir
        .components()
        .filter(|part| matches!(part, std::path::Component::Normal(_)))
        .fold(PathBuf::new(), |path, _| path.join(".."))
        .join(root.strip_prefix("/").unwrap())
        .join("exact");
    let cold = build_wasm_canisters_cached(&spec).expect("build relative exact target");
    assert!(cold.record().artifacts()[0].is_file());
    assert!(root.join("exact/.ic-testkit/wasm-targets").is_dir());
    let warm = build_wasm_canisters_cached(&spec).expect("reuse relative exact target");
    assert!(warm.is_reused());
    drop(warm);
    drop(cold);
    fs::remove_dir_all(root).expect("remove relative exact target fixture");
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

    let unsafe_path = package.join("src/generated-target");
    let unsafe_target = spec.clone().with_shared_incremental_target(&unsafe_path);
    assert!(matches!(
        resolve_cargo_build_inputs(&unsafe_target),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    fs::create_dir_all(&unsafe_path).expect("create unsafe maintenance target");
    let sentinel = unsafe_path.join("source-sentinel");
    fs::write(&sentinel, b"preserve").expect("write unsafe maintenance sentinel");
    let maintenance = maintain_shared_incremental_target(
        &unsafe_target,
        SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(0),
    );
    assert!(
        matches!(maintenance, Err(WasmBuildError::InvalidSpec { .. })),
        "unsafe maintenance returned {maintenance:?}",
    );
    assert!(sentinel.is_file());
    let scheduled = maintain_shared_incremental_target_at_most_every(
        &unsafe_target,
        SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(0),
        Duration::from_secs(60),
    );
    assert!(
        matches!(scheduled, Err(WasmBuildError::InvalidSpec { .. })),
        "unsafe scheduled maintenance returned {scheduled:?}",
    );
    assert!(sentinel.is_file());

    let broad_target = spec.with_shared_incremental_target(&root);
    assert!(matches!(
        resolve_cargo_build_inputs(&broad_target),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    fs::remove_dir_all(root).expect("remove Cargo input fixture");
}

#[test]
fn semantic_workspace_projection_ignores_unrelated_host_dependency_changes() {
    let root = unique_temp_directory("semantic-workspace-projection-host");
    write_projection_package(&root, "canister", "");
    write_projection_package(
        &root,
        "host",
        "\n[dependencies]\nhost_dep = { workspace = true }\n",
    );
    write_projection_package(&root, "host_dep_a", "");
    write_projection_package(&root, "host_dep_b", "");
    let workspace_manifest = root.join("Cargo.toml");
    let write_workspace = |dependency: &str| {
        fs::write(
            &workspace_manifest,
            format!(
                "[workspace]\nmembers = [\"canister\", \"host\"]\nexclude = [\"host_dep_a\", \"host_dep_b\"]\nresolver = \"2\"\n\n[workspace.dependencies]\nhost_dep = {{ package = \"{dependency}\", path = \"{dependency}\" }}\n"
            ),
        )
        .expect("write host projection workspace manifest");
    };
    write_workspace("host_dep_a");
    let spec = WasmBuildSpec::new(&root, &root.join("target"), &["canister"], "debug");

    let before = resolve_cargo_build_inputs(&spec).expect("resolve initial semantic projection");
    write_workspace("host_dep_b");
    let after = resolve_cargo_build_inputs(&spec).expect("resolve changed host projection");

    assert_eq!(before.input_digest(), after.input_digest());
    assert_eq!(before.fingerprint(), after.fingerprint());
    assert_ne!(before.validation_digest(), after.validation_digest());
    assert!(before.is_current(&spec).expect("recheck semantic identity"));
    assert!(
        !before
            .is_content_current()
            .expect("validate conservative snapshot")
    );
    fs::remove_dir_all(root).expect("remove host projection fixture");
}

#[test]
fn semantic_workspace_projection_tracks_selected_dependencies_and_profiles() {
    let root = unique_temp_directory("semantic-workspace-projection-selected");
    write_projection_package(
        &root,
        "canister",
        "\n[dependencies]\nselected_dep = { workspace = true }\n",
    );
    write_projection_package(&root, "selected_dep_a", "");
    write_projection_package(&root, "selected_dep_b", "");
    let workspace_manifest = root.join("Cargo.toml");
    let write_workspace = |dependency: &str, optimize: &str| {
        fs::write(
            &workspace_manifest,
            format!(
                "[workspace]\nmembers = [\"canister\"]\nexclude = [\"selected_dep_a\", \"selected_dep_b\"]\nresolver = \"2\"\n\n[workspace.dependencies]\nselected_dep = {{ package = \"{dependency}\", path = \"{dependency}\" }}\n\n[profile.release]\nopt-level = \"{optimize}\"\n"
            ),
        )
        .expect("write selected projection workspace manifest");
    };
    write_workspace("selected_dep_a", "s");
    let spec = WasmBuildSpec::new(&root, &root.join("target"), &["canister"], "release")
        .with_cargo_profile_args(["--release"]);

    let initial = resolve_cargo_build_inputs(&spec).expect("resolve selected projection");
    write_workspace("selected_dep_b", "s");
    let dependency_changed =
        resolve_cargo_build_inputs(&spec).expect("resolve changed selected dependency");
    assert_ne!(initial.input_digest(), dependency_changed.input_digest());
    assert_ne!(initial.fingerprint(), dependency_changed.fingerprint());

    write_workspace("selected_dep_b", "z");
    let profile_changed = resolve_cargo_build_inputs(&spec).expect("resolve changed profile");
    assert_ne!(
        dependency_changed.input_digest(),
        profile_changed.input_digest()
    );
    assert_ne!(
        dependency_changed.fingerprint(),
        profile_changed.fingerprint()
    );
    fs::remove_dir_all(root).expect("remove selected projection fixture");
}

#[test]
fn semantic_package_identity_uses_selected_registry_lock_checksum() {
    let root = unique_temp_directory("semantic-registry-lock-identity");
    let source = "registry+https://example.invalid/index";
    let package_id = format!("{source}#dependency@1.2.3");
    let package = MetadataPackage {
        id: &package_id,
        name: "dependency",
        version: "1.2.3",
        manifest_path: root.join("registry/dependency/Cargo.toml"),
        is_local: false,
        source: Some(source),
        semantic_fields: Vec::new(),
    };
    let write_lock = |checksum: Option<&str>| {
        let checksum = checksum.map_or_else(String::new, |checksum| {
            format!("checksum = \"{checksum}\"\n")
        });
        fs::write(
            root.join("Cargo.lock"),
            format!(
                "version = 4\n\n[[package]]\nname = \"dependency\"\nversion = \"1.2.3\"\nsource = \"{source}\"\n{checksum}"
            ),
        )
        .expect("write registry identity lockfile");
    };

    write_lock(Some("aaa"));
    let first_lock = locked_package_identities(&root).expect("project first lockfile");
    let first = semantic_package_identity(&package, &root, &first_lock)
        .expect("project checksummed registry package");
    write_lock(Some("bbb"));
    let second_lock = locked_package_identities(&root).expect("project second lockfile");
    let second = semantic_package_identity(&package, &root, &second_lock)
        .expect("project changed registry checksum");
    assert_ne!(first, second);

    write_lock(None);
    let missing_checksum = locked_package_identities(&root).expect("project incomplete lockfile");
    assert!(semantic_package_identity(&package, &root, &missing_checksum).is_none());
    fs::remove_dir_all(root).expect("remove registry identity fixture");
}

#[test]
fn semantic_workspace_projection_falls_back_for_workspace_root_packages() {
    let root = unique_temp_directory("semantic-workspace-projection-fallback");
    fs::create_dir_all(root.join("src")).expect("create root package source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"root_canister\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write root package manifest");
    fs::write(root.join("src/lib.rs"), "pub fn root_canister() {}\n")
        .expect("write root package source");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("target/exact"),
        &["root_canister"],
        "debug",
    );

    let resolved = resolve_cargo_build_inputs(&spec).expect("resolve fallback projection");
    assert_eq!(resolved.input_digest(), resolved.validation_digest());
    fs::remove_dir_all(root).expect("remove projection fallback fixture");
}

#[test]
#[cfg(unix)]
fn batch_input_snapshot_reuses_compatible_toolchain_and_metadata_resolution() {
    let root = unique_temp_directory("batch-input-snapshot");
    for package in ["fixture_a", "fixture_b"] {
        fs::create_dir_all(root.join(package).join("src")).expect("create batch fixture package");
        fs::write(
            root.join(package).join("Cargo.toml"),
            format!("[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
        )
        .expect("write batch fixture manifest");
        fs::write(
            root.join(package).join("src/lib.rs"),
            "pub fn fixture() {}\n",
        )
        .expect("write batch fixture source");
    }
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture_a\", \"fixture_b\"]\nresolver = \"2\"\n",
    )
    .expect("write batch fixture workspace");
    let invocation_log = root.join("cargo-invocations");
    let cargo_wrapper = root.join("cargo-wrapper.sh");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    write_executable_script(
        &cargo_wrapper,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\nexec '{}' \"$@\"\n",
            invocation_log.display(),
            PathBuf::from(cargo).display(),
        ),
    );
    let target = root.join("exact");
    let specs = [
        WasmBuildSpec::new(&root, &target, &["fixture_a"], "debug")
            .with_cargo_program(&cargo_wrapper),
        WasmBuildSpec::new(&root, &target, &["fixture_b"], "debug")
            .with_cargo_program(&cargo_wrapper),
    ];
    let mut resolver = WasmBuildBatchInputResolver::new(&specs, None);
    let mut progress = ProgressReporter::silent();

    let first = resolver
        .resolve(0, &mut progress)
        .expect("resolve first batched input");
    let second = resolver
        .resolve(1, &mut progress)
        .expect("reuse batched input snapshot");
    assert_eq!(resolver.metrics().runs, 1);
    assert_eq!(resolver.metrics().reuses, 1);
    assert!(first.timings().total() > Duration::ZERO);
    assert_eq!(second.timings().total(), Duration::ZERO);

    let invocations = fs::read_to_string(&invocation_log).expect("read Cargo invocation log");
    assert_eq!(
        invocations
            .lines()
            .filter(|line| *line == "--version")
            .count(),
        1
    );
    assert_eq!(
        invocations
            .lines()
            .filter(|line| *line == "metadata")
            .count(),
        1
    );
    assert_eq!(
        first.fingerprint(),
        resolve_cargo_build_inputs(&specs[0])
            .expect("resolve equivalent standalone inputs")
            .fingerprint(),
        "batch membership must not change an individual exact fingerprint",
    );
    assert_ne!(first.fingerprint(), second.fingerprint());
    fs::remove_dir_all(root).expect("remove batch input snapshot fixture");
}

#[test]
fn profile_output_directory_is_confined_before_resolution_or_acquisition() {
    use super::build_wasm_canisters_cached;

    let root = unique_temp_directory("wasm-profile-boundary");
    let target = root.join("target");
    let absolute = root.join("outside");
    for profile in [
        "",
        ".",
        "..",
        "../outside",
        "release/../../outside",
        "release/nested",
        absolute.to_str().unwrap(),
    ] {
        let spec = WasmBuildSpec::new(&root, &target, &["fixture"], profile)
            .with_cargo_program(root.join("missing-cargo"));
        assert!(
            matches!(
                resolve_cargo_build_inputs(&spec),
                Err(WasmBuildError::InvalidSpec { .. })
            ),
            "{profile:?} must fail before tool resolution",
        );
        assert!(
            matches!(
                build_wasm_canisters_cached(&spec),
                Err(WasmBuildError::InvalidSpec { .. })
            ),
            "{profile:?} must fail before acquisition",
        );
        assert!(!target.exists());
        assert!(!absolute.exists());
    }
    for profile in ["debug", "release", "fast", "release-with-debug"] {
        let spec = WasmBuildSpec::new(&root, &target, &["fixture"], profile)
            .with_cargo_profile_args(["--profile", profile]);
        validate_spec(&spec).expect("accept a normal profile output directory");
        assert!(super::expected_artifacts(&spec, &target)[0].starts_with(&target));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cargo_profile_arguments_match_output_directories() {
    for (directory, arguments) in [
        ("debug", vec![]),
        ("debug", vec!["--lib"]),
        ("debug", vec!["--offline"]),
        ("debug", vec!["--profile", "dev"]),
        ("debug", vec!["--profile=test"]),
        ("debug", vec!["--profile=debug"]),
        ("release", vec!["--release"]),
        ("release", vec!["-r"]),
        ("release", vec!["-qr"]),
        ("release", vec!["-vrFextra"]),
        ("release", vec!["-rF=extra"]),
        ("release", vec!["-rF", "extra"]),
        ("release", vec!["--profile=release"]),
        ("release", vec!["--profile", "bench"]),
        ("fast", vec!["--profile", "fast"]),
        ("fast", vec!["--profile=fast"]),
        ("debug", vec!["-Fextra"]),
        ("debug", vec!["-qF=extra"]),
        ("debug", vec!["-F", "extra"]),
        ("debug", vec!["--features=release"]),
        ("debug", vec!["--features", "release"]),
        ("debug", vec!["-j4"]),
    ] {
        let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &["fixture"], directory)
            .with_cargo_profile_args(&arguments);
        validate_spec(&spec)
            .unwrap_or_else(|error| panic!("{arguments:?} should write to {directory}: {error}"));
    }
}

#[test]
fn cargo_target_selectors_fail_before_resolution_or_acquisition() {
    let root = unique_temp_directory("wasm-target-selectors");
    let target = root.join("exact");
    let shared = root.join("shared");
    for arguments in [
        vec!["--bin", "helper"],
        vec!["--bin=helper"],
        vec!["--bins"],
        vec!["--example", "helper"],
        vec!["--example=helper"],
        vec!["--examples"],
        vec!["--test", "helper"],
        vec!["--test=helper"],
        vec!["--tests"],
        vec!["--bench", "helper"],
        vec!["--bench=helper"],
        vec!["--benches"],
        vec!["--all-targets"],
        vec!["--lib", "--bin=helper"],
    ] {
        let spec = WasmBuildSpec::new(&root, &target, &["fixture"], "debug")
            .with_cargo_profile_args(&arguments)
            .with_cargo_program(root.join("missing-cargo"));
        for spec in [spec.clone(), spec.with_shared_incremental_target(&shared)] {
            for result in [
                resolve_cargo_build_inputs(&spec).map(|_| ()),
                super::build_wasm_canisters_cached(&spec).map(|_| ()),
            ] {
                assert!(
                    matches!(result, Err(WasmBuildError::InvalidSpec { .. })),
                    "{arguments:?} must fail before invoking tools: {result:?}",
                );
            }
            assert!(!target.exists());
            assert!(!shared.exists());
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mismatched_cargo_profiles_fail_before_resolution_or_acquisition() {
    let root = unique_temp_directory("wasm-profile-mismatch");
    let target = root.join("exact");
    let shared = root.join("shared");
    for (directory, arguments) in [
        ("release", vec![]),
        ("release", vec!["--offline"]),
        ("release", vec!["--profile", "dev"]),
        ("release", vec!["--profile=test"]),
        ("debug", vec!["--release"]),
        ("debug", vec!["-qrFextra"]),
        ("debug", vec!["--profile=bench"]),
        ("debug", vec!["--profile", "fast"]),
        ("fast", vec!["--release"]),
        ("debug", vec!["--profile"]),
        ("debug", vec!["--profile="]),
        ("release", vec!["--release", "--profile=release"]),
        ("debug", vec!["--profile=dev", "--profile=test"]),
    ] {
        let spec = WasmBuildSpec::new(&root, &target, &["fixture"], directory)
            .with_cargo_profile_args(&arguments)
            .with_cargo_program(root.join("missing-cargo"));
        for spec in [spec.clone(), spec.with_shared_incremental_target(&shared)] {
            assert!(
                matches!(
                    resolve_cargo_build_inputs(&spec),
                    Err(WasmBuildError::InvalidSpec { .. })
                ),
                "{directory}: {arguments:?} must fail before resolution",
            );
            assert!(
                matches!(
                    super::build_wasm_canisters_cached(&spec),
                    Err(WasmBuildError::InvalidSpec { .. })
                ),
                "{directory}: {arguments:?} must fail before acquisition",
            );
            assert!(!target.exists());
            assert!(!shared.exists());
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn build_spec_requires_at_least_one_package() {
    let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &[], "debug");
    assert!(matches!(
        validate_spec(&spec),
        Err(WasmBuildError::InvalidSpec { .. })
    ));

    let isolated_maintenance =
        WasmBuildSpec::new(Path::new("."), Path::new("target"), &["fixture"], "debug")
            .with_shared_incremental_target_maintenance_at_most_every(
                SharedIncrementalTargetPrunePolicy::new(),
                Duration::from_secs(60),
            );
    assert!(matches!(
        validate_spec(&isolated_maintenance),
        Err(WasmBuildError::InvalidSpec { message }) if message.contains("requires a shared")
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
    let scheduled = maintain_shared_incremental_target_at_most_every(
        &shared,
        SharedIncrementalTargetPrunePolicy::new(),
        Duration::from_secs(60),
    )
    .expect("schedule missing shared target maintenance");
    assert!(matches!(
        &scheduled,
        SharedIncrementalTargetMaintenanceOutcome::Missing { .. }
    ));
    assert_eq!(scheduled.target_dir(), target);
    assert!(!scheduled.was_performed());
    assert_eq!(scheduled.lock_wait(), None);
    assert_eq!(scheduled.schedule_check(), None);
    assert!(!target.exists());
    fs::remove_dir_all(root).expect("remove shared-target inspection fixture");
}

#[test]
fn shared_target_maintenance_clears_cargo_state_but_preserves_coordination() {
    let root = unique_temp_directory("shared-target-maintenance");
    let package = root.join("fixture");
    fs::create_dir_all(package.join("src")).expect("create maintenance fixture package");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write maintenance fixture workspace");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write maintenance fixture manifest");
    fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n")
        .expect("write maintenance fixture source");
    let target = root.join("shared-target");
    fs::create_dir_all(target.join("debug/deps")).expect("create shared Cargo state");
    fs::write(target.join("debug/deps/object.o"), vec![7_u8; 128])
        .expect("write shared Cargo state");
    fs::write(target.join(".rustc_info.json"), b"rustc state")
        .expect("write shared Cargo metadata");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_shared_incremental_target(&target);

    let report = maintain_shared_incremental_target(
        &spec,
        SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(0),
    )
    .expect("maintain shared target")
    .expect("existing shared target report");

    assert!(report.was_cleared());
    assert!(report.logical_size_bytes_before() > report.logical_size_bytes_after());
    assert!(!target.join("debug").exists());
    assert!(!target.join(".rustc_info.json").exists());
    assert!(target.join("CACHEDIR.TAG").is_file());
    assert!(target.join(".ic-testkit/wasm-incremental.lock").is_file());

    let retained =
        maintain_shared_incremental_target(&spec, SharedIncrementalTargetPrunePolicy::new())
            .expect("reinspect shared target")
            .expect("existing shared target report");
    assert!(!retained.was_cleared());
    assert_eq!(
        retained.logical_size_bytes_before(),
        retained.logical_size_bytes_after()
    );
    fs::remove_dir_all(root).expect("remove shared-target maintenance fixture");
}

#[test]
fn scheduled_shared_target_maintenance_skips_expensive_work_inside_interval() {
    let root = unique_temp_directory("scheduled-shared-target-maintenance");
    let package = root.join("fixture");
    fs::create_dir_all(package.join("src")).expect("create scheduled fixture package");
    let workspace_manifest = root.join("Cargo.toml");
    fs::write(
        &workspace_manifest,
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write scheduled fixture workspace");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write scheduled fixture manifest");
    fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n")
        .expect("write scheduled fixture source");
    let target = root.join("shared-target");
    fs::create_dir_all(target.join("debug/deps")).expect("create scheduled Cargo state");
    fs::write(target.join("debug/deps/object.o"), vec![7_u8; 128])
        .expect("write scheduled Cargo state");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_shared_incremental_target(&target);
    let policy = SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(0);
    let interval = Duration::from_secs(60 * 60);

    let first = maintain_shared_incremental_target_at_most_every(&spec, policy, interval)
        .expect("perform scheduled shared-target maintenance");
    let report = first
        .maintenance()
        .expect("first scheduled maintenance must be performed");
    assert!(first.was_performed());
    assert!(report.was_cleared());
    assert!(first.lock_wait().is_some());
    assert!(first.schedule_check().is_some());

    let sentinel = target.join("debug/deps/preserve-on-skip");
    fs::create_dir_all(sentinel.parent().expect("scheduled sentinel parent"))
        .expect("recreate shared Cargo state");
    fs::write(&sentinel, b"preserve").expect("write scheduled skip sentinel");
    fs::remove_file(&workspace_manifest).expect("hide Cargo metadata from skipped call");

    let skipped = maintain_shared_incremental_target_at_most_every(&spec, policy, interval)
        .expect("skip recently completed shared-target maintenance");
    assert!(matches!(
        &skipped,
        SharedIncrementalTargetMaintenanceOutcome::Skipped { .. }
    ));
    assert!(!skipped.was_performed());
    assert!(skipped.lock_wait().is_some());
    assert!(skipped.schedule_check().is_some());
    assert!(sentinel.is_file());

    fs::write(
        &workspace_manifest,
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("restore scheduled fixture workspace");
    let changed_policy = SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(u64::MAX);
    let changed = maintain_shared_incremental_target_at_most_every(&spec, changed_policy, interval)
        .expect("changed policy must make scheduled maintenance due");
    assert!(changed.was_performed());
    assert!(sentinel.is_file());

    let zero_interval =
        maintain_shared_incremental_target_at_most_every(&spec, changed_policy, Duration::ZERO)
            .expect("zero interval must perform shared-target maintenance");
    assert!(zero_interval.was_performed());

    fs::remove_dir_all(root).expect("remove scheduled shared-target fixture");
}

#[test]
fn zero_progress_heartbeat_is_rejected_for_a_valid_build() {
    let root = unique_temp_directory("zero-progress-heartbeat");
    fs::create_dir_all(&root).expect("create heartbeat fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write heartbeat workspace");
    write_projection_package(&root, "fixture", "[lib]\ncrate-type = [\"cdylib\"]\n");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug");
    validate_spec(&spec).expect("valid heartbeat build spec");
    let result = build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::ZERO),
        |_| {},
    );
    assert!(matches!(result, Err(WasmBuildError::InvalidSpec { .. })));
    assert!(!root.join("exact").exists());
    let outcome = build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_secs(1)),
        |_| {},
    )
    .expect("the same build succeeds with a positive heartbeat interval");
    assert!(outcome.record().artifacts()[0].is_file());
    drop(outcome);
    fs::remove_dir_all(root).expect("remove heartbeat fixture");
}

#[test]
fn maintenance_configuration_is_readable_and_strict_by_default() {
    let shared_policy = SharedIncrementalTargetPrunePolicy::new().with_max_size_bytes(1024 * 1024);
    let shared_interval = Duration::from_secs(60 * 60);
    let shared_config =
        SharedIncrementalTargetMaintenanceConfig::new(shared_policy, shared_interval);
    let exact_policy = ArtifactCachePrunePolicy::new().with_max_size_bytes(2 * 1024 * 1024);
    let exact_interval = Duration::from_secs(30 * 60);
    let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &["fixture"], "debug")
        .with_shared_incremental_target("target/shared")
        .with_shared_incremental_target_maintenance(shared_config)
        .with_prune_policy_at_most_every(exact_policy, exact_interval);

    assert_eq!(shared_config.policy(), shared_policy);
    assert_eq!(shared_config.minimum_interval(), shared_interval);
    assert_eq!(
        shared_config.failure_mode(),
        SharedIncrementalTargetMaintenanceFailureMode::Strict
    );
    assert_eq!(
        spec.shared_incremental_target_maintenance(),
        Some(shared_config)
    );
    assert_eq!(spec.prune_policy(), Some(exact_policy));
    assert_eq!(spec.prune_interval(), Some(exact_interval));
}

#[test]
fn best_effort_integrated_maintenance_retains_a_failure_message() {
    let root = unique_temp_directory("best-effort-shared-maintenance-failure");
    let target = root.join("shared");
    let schedule_root = target.join(".ic-testkit");
    fs::create_dir_all(&schedule_root).expect("create corrupt schedule fixture");
    fs::write(schedule_root.join(".ic-testkit-last-maintenance"), [0xff])
        .expect("write invalid UTF-8 schedule marker");
    let config = SharedIncrementalTargetMaintenanceConfig::new(
        SharedIncrementalTargetPrunePolicy::new(),
        Duration::from_secs(60),
    )
    .with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_shared_incremental_target(&target)
        .with_shared_incremental_target_maintenance(config);
    let lock_wait = Duration::from_millis(7);
    let outcome = perform_configured_shared_incremental_target_maintenance(
        &spec,
        &target,
        lock_wait,
        &mut ProgressReporter::silent(),
    )
    .expect("best-effort maintenance must preserve acquisition");

    assert_eq!(outcome.target_dir(), target);
    assert_eq!(outcome.lock_wait(), Some(lock_wait));
    assert!(!outcome.was_performed());
    assert!(outcome.schedule_check().is_none());
    assert!(matches!(
        &outcome,
        SharedIncrementalTargetMaintenanceOutcome::Failed { .. }
    ));
    assert!(
        outcome
            .failure_message()
            .is_some_and(|message| !message.is_empty())
    );
    assert_eq!(
        fs::read(schedule_root.join(".ic-testkit-last-maintenance")).unwrap(),
        [0xff]
    );
    fs::remove_dir_all(root).expect("remove best-effort maintenance fixture");
}

#[test]
fn strict_integrated_maintenance_propagates_failure() {
    let config = SharedIncrementalTargetMaintenanceConfig::new(
        SharedIncrementalTargetPrunePolicy::new(),
        Duration::from_secs(60),
    );
    let result = integrated_shared_maintenance_result(
        config,
        Path::new("target/shared"),
        Duration::ZERO,
        Err(WasmBuildError::InvalidSpec {
            message: "synthetic strict failure".to_owned(),
        }),
    );

    assert!(matches!(
        result,
        Err(WasmBuildError::InvalidSpec { message }) if message == "synthetic strict failure"
    ));
}

#[test]
fn observed_phase_emits_phase_aware_heartbeats() {
    let (release, released) = mpsc::channel();
    let mut events = Vec::new();
    let value = {
        let mut observer = |event| {
            if matches!(
                event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildProgressPhase::ContentHashing,
                    ..
                }
            ) {
                let _ = release.send(());
            }
            events.push(event);
        };
        let mut progress = ProgressReporter::observed(
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(5)),
            &mut observer,
        );
        progress.run_phase(WasmBuildProgressPhase::ContentHashing, move || {
            released
                .recv_timeout(Duration::from_secs(10))
                .expect("observer must release the operation on its heartbeat");
            42
        })
    };

    assert_eq!(value, 42);
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::Heartbeat {
            phase: WasmBuildProgressPhase::ContentHashing,
            ..
        }
    )));
}

#[test]
fn observer_panic_joins_active_phase_worker() {
    let (release, released) = mpsc::channel::<()>();
    let mut release = Some(release);
    let completed = Arc::new(AtomicBool::new(false));
    let worker_completed = Arc::clone(&completed);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut observer = |event| {
            if matches!(
                event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildProgressPhase::InputDiscovery,
                    ..
                }
            ) {
                // Unwinding this callback releases the worker before scope cleanup
                // joins it; it never needs a callback after the observer panics.
                let _release_on_unwind = release.take().expect("one observer panic");
                panic!("synthetic observer panic");
            }
        };
        let mut progress = ProgressReporter::observed(
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(5)),
            &mut observer,
        );
        progress.run_phase(WasmBuildProgressPhase::InputDiscovery, move || {
            assert_eq!(
                released.recv_timeout(Duration::from_secs(10)),
                Err(mpsc::RecvTimeoutError::Disconnected),
                "observer unwinding must release the active worker"
            );
            worker_completed.store(true, Ordering::SeqCst);
        });
    }));

    let panic = result.expect_err("observer must panic");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"synthetic observer panic")
    );
    assert!(completed.load(Ordering::SeqCst));
}

#[test]
fn exact_cache_lock_wait_emits_phase_aware_heartbeats() {
    let target_dir = unique_temp_directory("observed-exact-lock-wait");
    let (held_lock, _) = lock_wasm_build_cache(&target_dir).expect("acquire fixture cache lock");
    let (heartbeat_sender, heartbeat_receiver) = mpsc::channel();
    let release = thread::spawn(move || {
        heartbeat_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("wait for exact-cache lock heartbeat");
        drop(held_lock);
    });
    let mut events = Vec::new();
    {
        let mut observer = |event| {
            if matches!(
                &event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildProgressPhase::ExactCacheLock,
                    ..
                }
            ) {
                let _ = heartbeat_sender.send(());
            }
            events.push(event);
        };
        let mut progress = ProgressReporter::observed(
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(5)),
            &mut observer,
        );
        let (_lock, wait) = lock_wasm_build_cache_with_progress(&target_dir, &mut progress)
            .expect("acquire observed fixture cache lock");
        assert!(wait >= Duration::from_millis(5));
    }
    release.join().expect("join fixture lock holder");

    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::Heartbeat {
            phase: WasmBuildProgressPhase::ExactCacheLock,
            ..
        }
    )));
    fs::remove_dir_all(target_dir).expect("remove observed lock fixture");
}

#[test]
#[cfg(unix)]
fn observed_cargo_build_forwards_raw_output_and_quiet_heartbeats() {
    let root = unique_temp_directory("observed-cargo-progress");
    // Cargo's first argument is `build`; let an existing shell read that fixture
    // instead of executing a freshly written file, which can fail with ETXTBSY.
    let script = root.join("build");
    let release = root.join("release-cargo");
    // Keep the producer alive until the observer actually sees a quiet interval.
    // A fixed sleep can finish before output readers are scheduled under load.
    fs::write(
        &script,
        r#"set -eu
printf 'observed-stdout'
printf 'observed-stderr' >&2
attempts=0
while [ ! -f "$IC_TESTKIT_OBSERVED_CARGO_RELEASE" ]; do
    attempts=$((attempts + 1))
    if [ "$attempts" -ge 1000 ]; then
        printf 'heartbeat observer did not release Cargo fixture\n' >&2
        exit 1
    fi
    sleep 0.01
done
"#,
    )
    .expect("write observed Cargo fixture");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_cargo_program("/bin/sh")
        .with_extra_env([("IC_TESTKIT_OBSERVED_CARGO_RELEASE", release.as_os_str())]);
    let mut events = Vec::new();
    {
        let mut observer = |event| {
            if matches!(
                &event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildProgressPhase::CargoBuild,
                    ..
                }
            ) {
                fs::write(&release, b"released").expect("release Cargo on its observed heartbeat");
            }
            events.push(event);
        };
        let mut progress = ProgressReporter::observed(
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(10)),
            &mut observer,
        );

        run_cargo_build(&spec, &root.join("cargo-target"), &mut progress)
            .expect("run observed Cargo fixture");
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    for event in &events {
        if let WasmBuildProgressEvent::CargoOutput { stream, bytes } = event {
            match stream {
                WasmBuildOutputStream::Stdout => stdout.extend_from_slice(bytes),
                WasmBuildOutputStream::Stderr => stderr.extend_from_slice(bytes),
            }
        }
    }
    assert_eq!(stdout, b"observed-stdout");
    assert_eq!(stderr, b"observed-stderr");
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::Heartbeat {
            phase: WasmBuildProgressPhase::CargoBuild,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::CargoFinished { success: true, .. }
    )));
    fs::remove_dir_all(root).expect("remove observed Cargo fixture");
}

#[test]
fn observed_output_reader_retries_interrupted_reads_without_losing_bytes() {
    use std::io;

    struct InterruptedReader {
        contents: &'static [u8],
        attempts: usize,
    }

    impl io::Read for InterruptedReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.attempts += 1;
            if matches!(self.attempts, 1 | 3) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            let limit = buffer.len().min(3);
            self.contents.read(&mut buffer[..limit])
        }
    }

    let reader = InterruptedReader {
        contents: b"failure-stdout",
        attempts: 0,
    };
    let (sender, chunks) = mpsc::sync_channel(8);
    super::read_process_output(reader, WasmBuildOutputStream::Stdout, sender)
        .expect("interrupted output reads must retry");
    let captured = super::capture_observed_cargo_output(
        chunks,
        &mut ProgressReporter::silent(),
        std::time::Instant::now(),
    );
    assert_eq!(captured.stdout, b"failure-stdout");
    assert_eq!(captured.stderr, [] as [u8; 0]);
}

#[test]
fn observed_output_drains_both_streams_through_a_bounded_queue() {
    let stdout = (0_u8..=255).cycle().take(128 * 1024).collect::<Vec<_>>();
    let stderr = b"diagnostic\0\xff\n".repeat(16 * 1024);
    for emit in [true, false] {
        let mut forwarded_stdout = Vec::new();
        let mut forwarded_stderr = Vec::new();
        let captured = thread::scope(|scope| {
            let (sender, chunks) = mpsc::sync_channel(1);
            let stdout_sender = sender.clone();
            let stdout_reader = scope.spawn(|| {
                super::read_process_output(
                    stdout.as_slice(),
                    WasmBuildOutputStream::Stdout,
                    stdout_sender,
                )
            });
            let stderr_reader = scope.spawn(|| {
                super::read_process_output(stderr.as_slice(), WasmBuildOutputStream::Stderr, sender)
            });
            let mut observer = |event| {
                if let WasmBuildProgressEvent::CargoOutput { stream, bytes } = event {
                    match stream {
                        WasmBuildOutputStream::Stdout => forwarded_stdout.extend(bytes),
                        WasmBuildOutputStream::Stderr => forwarded_stderr.extend(bytes),
                    }
                }
            };
            let mut progress = ProgressReporter::observed(
                WasmBuildProgressConfig::new()
                    .without_heartbeats()
                    .with_cargo_output(emit),
                &mut observer,
            );
            let captured = super::capture_observed_cargo_output(
                chunks,
                &mut progress,
                std::time::Instant::now(),
            );
            stdout_reader.join().unwrap().unwrap();
            stderr_reader.join().unwrap().unwrap();
            captured
        });
        assert_eq!(captured.stdout, stdout);
        assert_eq!(captured.stderr, stderr);
        assert_eq!(
            forwarded_stdout.as_slice(),
            if emit { stdout.as_slice() } else { &[] }
        );
        assert_eq!(
            forwarded_stderr.as_slice(),
            if emit { stderr.as_slice() } else { &[] }
        );
    }
}

#[test]
fn observed_output_reader_stops_when_the_consumer_disconnects() {
    let (sender, chunks) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        let mut reader = std::io::Cursor::new(vec![0; 128 * 1024]);
        super::read_process_output(&mut reader, WasmBuildOutputStream::Stdout, sender)
            .expect("disconnected consumers release output readers");
        reader
    });
    chunks.recv().expect("receive output before disconnecting");
    drop(chunks);
    let reader = worker.join().expect("join disconnected output reader");
    assert!(reader.position() < reader.get_ref().len() as u64);
}

#[test]
fn observed_output_reader_propagates_permanent_errors() {
    use std::io;

    struct FailedReader;

    impl io::Read for FailedReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }

    let (sender, _chunks) = mpsc::sync_channel(8);
    let error = super::read_process_output(FailedReader, WasmBuildOutputStream::Stderr, sender)
        .expect_err("permanent output errors must propagate");
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
}

#[test]
#[cfg(unix)]
fn observed_cargo_failure_retains_captured_diagnostics_and_exit_event() {
    let root = unique_temp_directory("observed-cargo-failure");
    // Each stream exceeds both the pipe buffer and the forwarding queue.
    let expected_stdout = "failure-stdout".repeat(16 * 1024);
    let expected_stderr = "failure-stderr".repeat(16 * 1024);
    fs::write(root.join("stdout"), &expected_stdout).unwrap();
    fs::write(root.join("stderr"), &expected_stderr).unwrap();
    // `sh build` reads the fixture without executing the freshly written file.
    fs::write(
        root.join("build"),
        "set -eu\ncat stdout\ncat stderr >&2\nexit 23\n",
    )
    .expect("write failing observed Cargo fixture");
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_cargo_program("/bin/sh");
    for emit in [true, false] {
        let mut events = Vec::new();
        let error = {
            let mut observer = |event| events.push(event);
            let mut progress = ProgressReporter::observed(
                WasmBuildProgressConfig::new()
                    .without_heartbeats()
                    .with_cargo_output(emit),
                &mut observer,
            );

            run_cargo_build(&spec, &root.join("cargo-target"), &mut progress)
                .expect_err("Cargo fixture must fail")
        };

        assert!(
            matches!(
                &error,
                WasmBuildError::CommandFailed {
                    status,
                    stdout,
                    stderr,
                    ..
                } if status.code() == Some(23)
                    && stdout == &expected_stdout
                    && stderr == &expected_stderr
            ),
            "unexpected Cargo fixture failure: {error:?}"
        );
        assert!(events.iter().any(|event| matches!(
            event,
            WasmBuildProgressEvent::CargoFinished {
                success: false,
                code: Some(23),
                ..
            }
        )));
        assert_eq!(
            events
                .iter()
                .any(|event| matches!(event, WasmBuildProgressEvent::CargoOutput { .. })),
            emit,
        );
    }
    fs::remove_dir_all(root).expect("remove failing observed Cargo fixture");
}

#[test]
fn cache_directory_tag_is_created_at_target_root() {
    let target_dir = unique_temp_directory("cache-directory-tag");
    fs::write(target_dir.join("CACHEDIR.TAG"), "not a cache tag").expect("write invalid cache tag");

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
    fs::write(fingerprint_dir.join("partial-output"), b"partial").expect("write incomplete output");
    let failure: Result<WasmBuildOutcome, WasmBuildError> = Err(WasmBuildError::InvalidSpec {
        message: "synthetic build failure".to_owned(),
    });

    let mut progress = ProgressReporter::silent();
    let result = finish_fingerprint_build(
        failure,
        IncompleteBuildDirectory::new(fingerprint_dir.clone()),
        &mut progress,
    );

    assert!(matches!(result, Err(WasmBuildError::InvalidSpec { .. })));
    assert!(progress.failure_timings.cleanup().is_some());
    assert!(!fingerprint_dir.exists());
    fs::remove_dir_all(target_dir).expect("remove cleanup test directory");
}

#[test]
fn reconstructed_wasm_stamps_describe_the_bytes_actually_copied() {
    use super::{
        expected_artifacts, publish_artifact_stamps, reconstruct_exact_cache_entry,
        validated_artifact_set,
    };
    use crate::artifacts::digest::digest_bytes;

    let root = unique_temp_directory("reconstructed-wasm-bytes");
    let source = root.join("public.wasm");
    fs::write(&source, b"original").unwrap();
    let sources = [source.clone()];
    let fingerprint = digest_bytes("wasm-reconstruction-test-v1", b"fixture");
    publish_artifact_stamps(&sources, fingerprint).unwrap();
    assert!(validated_artifact_set(&sources, fingerprint).is_some());
    // The public source is mutable across the verification/copy boundary.
    fs::write(&source, b"modified").unwrap();
    let spec = WasmBuildSpec::new(&root, &root.join("target"), &["fixture"], "debug");
    let entry = root.join("entry");
    let copied_info = reconstruct_exact_cache_entry(
        &spec,
        &sources,
        &entry,
        fingerprint,
        &mut ProgressReporter::silent(),
    )
    .unwrap();
    let cached = expected_artifacts(&spec, &entry);
    assert_eq!(fs::read(&cached[0]).unwrap(), b"modified");
    assert_eq!(
        validated_artifact_set(&cached, fingerprint).unwrap(),
        copied_info
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reconstruction_rejects_empty_copies_and_removes_the_incomplete_entry() {
    use super::{expected_artifacts, publish_artifact_stamps, reconstruct_exact_cache_entry};
    use crate::artifacts::digest::digest_bytes;

    let root = unique_temp_directory("empty-reconstructed-wasm");
    let sources = [root.join("public.wasm")];
    fs::write(&sources[0], b"original").unwrap();
    let fingerprint = digest_bytes("wasm-reconstruction-test-v1", b"fixture");
    publish_artifact_stamps(&sources, fingerprint).unwrap();
    fs::write(&sources[0], b"").unwrap();
    let spec = WasmBuildSpec::new(&root, &root.join("target"), &["fixture"], "debug");
    let entry = root.join("entry");
    let cached = expected_artifacts(&spec, &entry);
    let mut progress = ProgressReporter::silent();
    let error = reconstruct_exact_cache_entry(&spec, &sources, &entry, fingerprint, &mut progress)
        .expect_err("a copied Wasm must remain nonempty");
    assert!(matches!(error, WasmBuildError::MissingArtifacts { paths } if paths == cached));
    assert!(progress.failure_timings.cleanup().is_some());
    assert!(!entry.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_exact_entry_reconstruction_removes_partial_outputs() {
    use super::reconstruct_exact_cache_entry;
    use crate::artifacts::digest::digest_bytes;

    let root = unique_temp_directory("failed-exact-entry-reconstruction");
    let entry = root.join("cache-entry");
    let sources = [root.join("first.wasm"), root.join("missing.wasm")];
    fs::write(&sources[0], b"\0asm\x01\0\0\0").expect("write first source artifact");
    let spec = WasmBuildSpec::new(&root, &root.join("target"), &["first", "missing"], "debug");
    let mut progress = ProgressReporter::silent();
    let error = reconstruct_exact_cache_entry(
        &spec,
        &sources,
        &entry,
        digest_bytes("wasm-artifact-v1", b"reconstruction-fixture"),
        &mut progress,
    )
    .expect_err("a missing second source must fail after the first copy");

    assert!(matches!(
        error,
        WasmBuildError::Io { path, source, .. }
            if path.ends_with("missing.wasm") && source.kind() == std::io::ErrorKind::NotFound
    ));
    assert!(progress.failure_timings.cleanup().is_some());
    assert!(
        !entry.exists(),
        "failed reconstruction must remove its partial entry"
    );
    fs::remove_dir_all(root).expect("remove failed reconstruction fixture");
}

#[test]
#[cfg(unix)]
fn oversized_wasm_cache_stamp_recovers_from_verified_public_outputs() {
    use super::{artifact_stamp_path, build_wasm_canisters_cached, validated_artifact_set};
    use crate::artifacts::test_support::fake_wasm_build_spec;

    let (root, spec) = fake_wasm_build_spec("oversized-wasm-cache-stamp");
    let cold = build_wasm_canisters_cached(&spec).expect("build cold artifact");
    let fingerprint = cold.record().fingerprint();
    let cached = cold.record().artifacts()[0].clone();
    let original = fs::read(&cached).expect("read original artifact");
    let stamp_path = artifact_stamp_path(&cached);
    let stamp = fs::read(&stamp_path).expect("read original stamp");
    // Release the retained entry before simulating external cache damage.
    drop(cold);
    fs::remove_file(&stamp_path).expect("remove read-only stamp");
    fs::write(&stamp_path, stamp).expect("write matching stamp prefix");
    fs::OpenOptions::new()
        .write(true)
        .open(&stamp_path)
        .expect("open oversized stamp")
        .set_len(1024 * 1024 * 1024)
        .expect("extend oversized stamp");

    let recovered = build_wasm_canisters_cached(&spec).expect("recover from public artifacts");
    assert!(recovered.is_reused());
    assert!(recovered.record().timings().cargo_build().is_none());
    assert_eq!(
        fs::read(&recovered.record().artifacts()[0]).unwrap(),
        original
    );
    assert!(validated_artifact_set(recovered.record().artifacts(), fingerprint).is_some());
    drop(recovered);
    fs::remove_dir_all(root).expect("remove recovery fixture");
}

#[test]
#[cfg(unix)]
fn warm_wasm_outputs_follow_retained_entry_and_preserve_matching_files() {
    use super::{
        artifact_stamp_path, build_wasm_canisters_cached, expected_artifacts,
        publish_artifact_stamps,
    };
    use crate::artifacts::test_support::fake_wasm_build_spec;
    use std::{fs::FileTimes, os::unix::fs::MetadataExt as _};

    for mode in ["isolated", "shared", "scheduled"] {
        let (root, mut spec) = fake_wasm_build_spec("warm-wasm-publication");
        if mode != "isolated" {
            spec = spec.with_shared_incremental_target(root.join("incremental"));
        }
        if mode == "scheduled" {
            spec = spec.with_shared_incremental_target_maintenance_at_most_every(
                SharedIncrementalTargetPrunePolicy::new(),
                Duration::from_secs(60),
            );
        }
        let public = expected_artifacts(&spec, &spec.target_dir);
        fs::create_dir_all(public[0].parent().unwrap()).unwrap();
        let original = b"\0asm\x01\0\0\0";
        fs::write(&public[0], original).unwrap();
        let before_cold = fs::metadata(&public[0]).unwrap().ino();
        let cold = build_wasm_canisters_cached(&spec).unwrap();
        assert!(!cold.is_reused());
        assert_ne!(before_cold, fs::metadata(&public[0]).unwrap().ino());

        // Both stamps are valid for this fingerprint, but their bytes differ.
        fs::write(&public[0], b"stamped!").unwrap();
        publish_artifact_stamps(&public, cold.record().fingerprint()).unwrap();
        let reused = build_wasm_canisters_cached(&spec).unwrap();
        assert!(reused.is_reused());
        assert!(reused.record().timings().cargo_build().is_none());
        assert_eq!(fs::read(&public[0]).unwrap(), original, "{mode}");
        assert_eq!(fs::read(&reused.record().artifacts()[0]).unwrap(), original);

        let stamp = artifact_stamp_path(&public[0]);
        let expected_stamp = fs::read(&stamp).unwrap();
        let file_identity = fs::metadata(&public[0]).unwrap().ino();
        for damaged_stamp in [None, Some(b"invalid".as_slice())] {
            if let Some(bytes) = damaged_stamp {
                fs::write(&stamp, bytes).unwrap();
            } else {
                fs::remove_file(&stamp).unwrap();
            }
            assert!(build_wasm_canisters_cached(&spec).unwrap().is_reused());
            assert_eq!(fs::metadata(&public[0]).unwrap().ino(), file_identity);
            assert_eq!(fs::read(&stamp).unwrap(), expected_stamp);
        }

        let time = UNIX_EPOCH + Duration::from_secs(3600);
        for path in [&public[0], &stamp] {
            fs::File::open(path)
                .unwrap()
                .set_times(FileTimes::new().set_modified(time))
                .unwrap();
        }
        let stamp_identity = fs::metadata(&stamp).unwrap().ino();
        assert!(build_wasm_canisters_cached(&spec).unwrap().is_reused());
        assert_eq!(fs::metadata(&public[0]).unwrap().ino(), file_identity);
        assert_eq!(fs::metadata(&public[0]).unwrap().modified().unwrap(), time);
        assert_eq!(fs::metadata(&stamp).unwrap().ino(), stamp_identity);
        assert_eq!(fs::metadata(&stamp).unwrap().modified().unwrap(), time);

        fs::remove_file(&public[0]).unwrap();
        assert!(build_wasm_canisters_cached(&spec).unwrap().is_reused());
        assert_eq!(fs::read(&public[0]).unwrap(), original);
        assert_eq!(fs::metadata(&stamp).unwrap().ino(), stamp_identity);
        drop(reused);
        drop(cold);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
#[cfg(unix)]
fn warm_wasm_publication_detaches_linked_or_restricted_outputs_and_stamps() {
    use super::{artifact_stamp_path, build_wasm_canisters_cached, expected_artifacts};
    use crate::artifacts::test_support::fake_wasm_build_spec;
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _, symlink};

    let (root, spec) = fake_wasm_build_spec("warm-wasm-ownership");
    let cold = build_wasm_canisters_cached(&spec).unwrap();
    let public = expected_artifacts(&spec, &spec.target_dir);
    let cached = &cold.record().artifacts()[0];
    let public_stamp = artifact_stamp_path(&public[0]);
    let cached_stamp = artifact_stamp_path(cached);
    let expected = fs::read(cached).unwrap();
    let expected_stamp = fs::read(&cached_stamp).unwrap();
    for case in ["symlink", "hardlink", "readonly", "executable"] {
        for (source, destination) in [(cached, &public[0]), (&cached_stamp, &public_stamp)] {
            fs::remove_file(destination).unwrap();
            match case {
                "symlink" => symlink(source, destination).unwrap(),
                "hardlink" => fs::hard_link(source, destination).unwrap(),
                _ => {
                    fs::copy(source, destination).unwrap();
                    let mode = if case == "readonly" { 0o444 } else { 0o755 };
                    fs::set_permissions(destination, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
        assert!(build_wasm_canisters_cached(&spec).unwrap().is_reused());
        for destination in [&public[0], &public_stamp] {
            let metadata = fs::symlink_metadata(destination).unwrap();
            assert!(metadata.is_file(), "{case}");
            assert_eq!(metadata.nlink(), 1, "{case}");
            assert_eq!(metadata.mode() & 0o600, 0o600, "{case}");
            assert_eq!(metadata.mode() & 0o7111, 0, "{case}");
        }
        assert_eq!(fs::read(&public[0]).unwrap(), expected);
        assert_eq!(fs::read(&public_stamp).unwrap(), expected_stamp);
        fs::write(&public[0], b"changed").unwrap();
        fs::write(&public_stamp, b"changed").unwrap();
        assert_eq!(fs::read(cached).unwrap(), expected);
        assert_eq!(fs::read(&cached_stamp).unwrap(), expected_stamp);
    }
    drop(cold);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn warm_wasm_materialization_repairs_only_changed_members() {
    use super::{
        artifact_stamp_path, materialize_artifacts, publish_artifact_stamps, validated_artifact_set,
    };
    use crate::artifacts::digest::digest_bytes;
    use std::{fs::FileTimes, os::unix::fs::MetadataExt as _};

    let root = unique_temp_directory("selective-wasm-materialization");
    let cached = [root.join("first.wasm"), root.join("second.wasm")];
    fs::write(&cached[0], b"first").unwrap();
    fs::write(&cached[1], b"second").unwrap();
    let fingerprint = digest_bytes("wasm-materialization-test-v1", b"fixture");
    let info = publish_artifact_stamps(&cached, fingerprint).unwrap();
    let public = [
        root.join("public-first.wasm"),
        root.join("public-second.wasm"),
    ];
    fs::write(&public[0], b"first").unwrap();
    fs::write(&public[1], b"wrong!").unwrap();
    publish_artifact_stamps(&public, fingerprint).unwrap();
    let stamp = artifact_stamp_path(&public[0]);
    let time = UNIX_EPOCH + Duration::from_secs(3600);
    let identities = [&public[0], &stamp].map(|path| {
        fs::File::open(path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(time))
            .unwrap();
        fs::metadata(path).unwrap().ino()
    });

    materialize_artifacts(&cached, &public, fingerprint, &info, true).unwrap();

    assert_eq!(fs::read(&public[0]).unwrap(), b"first");
    assert_eq!(fs::read(&public[1]).unwrap(), b"second");
    assert_eq!(validated_artifact_set(&public, fingerprint).unwrap(), info);
    for (path, identity) in [&public[0], &stamp].into_iter().zip(identities) {
        assert_eq!(fs::metadata(path).unwrap().ino(), identity);
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), time);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_wasm_materialization_does_not_stamp_a_partial_public_set() {
    use super::{artifact_stamp_path, materialize_artifacts, publish_artifact_stamps};
    use crate::artifacts::digest::digest_bytes;

    let root = unique_temp_directory("partial-wasm-materialization");
    let cached = [root.join("first.wasm"), root.join("second.wasm")];
    for path in &cached {
        fs::write(path, b"wasm").unwrap();
    }
    let fingerprint = digest_bytes("wasm-materialization-test-v1", b"fixture");
    let info = publish_artifact_stamps(&cached, fingerprint).unwrap();
    fs::write(root.join("blocked"), b"file").unwrap();
    let public = [
        root.join("public/first.wasm"),
        root.join("blocked/second.wasm"),
    ];
    for preserve_matching in [false, true] {
        let error = materialize_artifacts(&cached, &public, fingerprint, &info, preserve_matching)
            .expect_err("second destination must fail before any stamp is published");
        assert!(matches!(
            error,
            WasmBuildError::Io {
                operation: "publish Wasm artifact",
                ..
            }
        ));
        assert_eq!(fs::read(&public[0]).unwrap(), b"wasm");
        assert!(!artifact_stamp_path(&public[0]).exists());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn warm_caller_artifacts_reconstruct_a_missing_exact_entry() {
    use super::build_wasm_canisters_cached;
    use crate::artifacts::test_support::fake_wasm_build_spec;

    let (root, spec) = fake_wasm_build_spec("reconstruct-exact-entry");
    let built = build_wasm_canisters_cached(&spec).expect("build initial exact entry");
    let entry = built.record().exact_cache_path().to_owned();
    let expected = fs::read(&built.record().artifacts()[0]).expect("read initial Wasm");
    drop(built);
    fs::remove_dir_all(&entry).expect("remove exact entry while keeping caller artifacts");

    let reused = build_wasm_canisters_cached(&spec).expect("reconstruct from caller artifacts");
    assert!(reused.is_reused());
    assert!(reused.record().timings().cargo_build().is_none());
    assert_eq!(reused.record().exact_cache_path(), entry);
    assert_eq!(fs::read(&reused.record().artifacts()[0]).unwrap(), expected);
    fs::write(&reused.record().artifacts()[0], b"corrupt").unwrap();
    let error = build_wasm_canisters_cached(&spec)
        .expect_err("an invalid retained entry must not be replaced from public artifacts");
    assert!(matches!(
        error,
        WasmBuildError::Io {
            operation: "replace retained cache entry",
            ..
        }
    ));
    assert_eq!(
        fs::read(&super::expected_artifacts(&spec, &spec.target_dir)[0]).unwrap(),
        expected
    );
    drop(reused);
    let recovered = build_wasm_canisters_cached(&spec)
        .expect("recover the invalid entry once its consumer releases it");
    assert!(recovered.is_reused());
    assert_eq!(
        fs::read(&recovered.record().artifacts()[0]).unwrap(),
        expected
    );
    drop(recovered);
    assert!(
        entry.is_dir(),
        "successful reconstruction must survive record drop"
    );
    fs::remove_dir_all(root).expect("remove reconstruction fixture");
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
        ArtifactCachePrunePolicy::new()
            .with_max_age(Duration::from_secs(60))
            .with_max_size_bytes(u64::MAX),
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
    let all_bytes = [&oldest, &middle, &newest]
        .into_iter()
        .map(|entry| directory_logical_size(entry).unwrap())
        .sum::<u64>();
    for limit in [all_bytes, all_bytes + 1] {
        let report = prune_wasm_build_cache(
            &target_dir,
            ArtifactCachePrunePolicy::new().with_max_size_bytes(limit),
        )
        .expect("retain entries within the size budget");
        assert_eq!(report.entries_scanned(), 3);
        assert_eq!(report.entries_removed(), 0);
        assert_eq!(report.bytes_before(), all_bytes);
        assert_eq!(report.bytes_retained(), all_bytes);
        assert!(oldest.exists() && middle.exists() && newest.exists());
    }

    let report = prune_wasm_build_cache(
        &target_dir,
        ArtifactCachePrunePolicy::new().with_max_size_bytes(newest_bytes),
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
        ArtifactCachePrunePolicy::new()
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
        .with_extra_env([("CARGO_HOME", cargo_home_text)]);
    let mut inputs = Vec::new();
    append_cargo_configuration_inputs(&mut inputs, &spec, &workspace)
        .expect("discover effective Cargo configuration");
    let paths = inputs
        .into_iter()
        .map(|(_, path)| path)
        .collect::<BTreeSet<_>>();

    assert!(paths.contains(&canonical_fixture(&workspace_cargo.join("config"))));
    assert!(paths.contains(&canonical_fixture(&workspace_cargo.join("included.toml"))));
    assert!(paths.contains(&canonical_fixture(&workspace_cargo.join("nested.toml"))));
    assert!(paths.contains(&canonical_fixture(&ancestor_cargo.join("config.toml"))));
    assert!(paths.contains(&canonical_fixture(&cargo_home.join("config"))));
    assert!(!paths.contains(&canonical_fixture(&workspace_cargo.join("config.toml"))));
    assert_eq!(paths.len(), 5);
    fs::remove_dir_all(root).expect("remove Cargo-configuration test directory");
}

#[test]
#[cfg(unix)]
fn cargo_configuration_symlinks_preserve_include_locations_and_mutation_guards() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_directory("cargo-configuration-symlinks");
    let workspace = root.join("workspace");
    let workspace_cargo = workspace.join(".cargo");
    let ancestor_cargo = root.join(".cargo");
    let external = root.join("external");
    for directory in [&workspace_cargo, &ancestor_cargo, &external] {
        fs::create_dir_all(directory).unwrap();
    }
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    write_projection_package(&workspace, "fixture", "");
    let shared_config = external.join("config.toml");
    fs::write(&shared_config, "include = [\"fragment.toml\"]\n").unwrap();
    let workspace_config = workspace_cargo.join("config.toml");
    let ancestor_config = ancestor_cargo.join("config.toml");
    symlink(&shared_config, &workspace_config).unwrap();
    symlink(&shared_config, &ancestor_config).unwrap();
    let workspace_fragment = workspace_cargo.join("fragment.toml");
    let ancestor_fragment = ancestor_cargo.join("fragment.toml");
    let fragments = [&workspace_fragment, &ancestor_fragment];
    for fragment in fragments {
        fs::write(fragment, "[build]\nrustflags = []\n").unwrap();
    }
    // This decoy exists so resolving beside the referent can silently watch
    // the wrong input instead of failing on a missing include.
    let decoy = external.join("fragment.toml");
    fs::write(&decoy, "[build]\nrustflags = []\n").unwrap();
    let spec = WasmBuildSpec::new(&workspace, &root.join("exact"), &["fixture"], "debug")
        .with_extra_env([("CARGO_HOME", &ancestor_cargo)]);
    let snapshot = resolve_cargo_build_inputs(&spec).expect("resolve symlinked Cargo configs");
    assert_eq!(
        snapshot
            .inputs()
            .iter()
            .map(super::CargoBuildInput::path)
            .collect::<BTreeSet<_>>()
            .len(),
        snapshot.inputs().len(),
        "Cargo home and ancestor discovery must not duplicate the same lookup path",
    );
    assert!(snapshot.is_content_current().unwrap());
    for fragment in fragments {
        fs::write(fragment, "[build]\nrustflags = ['--cfg=changed']\n").unwrap();
        assert!(
            !snapshot.is_content_current().unwrap(),
            "Cargo's include at {} must be watched",
            fragment.display(),
        );
        fs::write(fragment, "[build]\nrustflags = []\n").unwrap();
        assert!(snapshot.is_content_current().unwrap());
    }
    fs::write(&decoy, "[build]\nrustflags = ['--cfg=unused']\n").unwrap();
    assert!(snapshot.is_content_current().unwrap());

    // Retargeting the configured entry must be observed, even while the
    // previous referent remains unchanged.
    let replacement = external.join("replacement.toml");
    fs::write(&replacement, "[build]\nrustflags = ['--cfg=replaced']\n").unwrap();
    fs::remove_file(&workspace_config).unwrap();
    symlink(&replacement, &workspace_config).unwrap();
    assert!(!snapshot.is_content_current().unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn cargo_configuration_guards_follow_each_directory_alias() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_directory("cargo-configuration-directory-aliases");
    let workspace = root.join("workspace");
    let cargo_dir = workspace.join(".cargo");
    fs::create_dir_all(&cargo_dir).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    write_projection_package(&workspace, "fixture", "");
    fs::write(
        cargo_dir.join("config.toml"),
        "include = [\"first/config.toml\", \"second/config.toml\"]\n",
    )
    .unwrap();
    let original = root.join("original");
    let replacement = root.join("replacement");
    for directory in [&original, &replacement] {
        fs::create_dir_all(directory).unwrap();
        fs::write(
            directory.join("config.toml"),
            "include = [\"fragment.toml\"]\n",
        )
        .unwrap();
        fs::write(directory.join("fragment.toml"), "[build]\nrustflags = []\n").unwrap();
    }
    symlink(&original, cargo_dir.join("first")).unwrap();
    let second = cargo_dir.join("second");
    symlink(&original, &second).unwrap();
    let spec = WasmBuildSpec::new(&workspace, &root.join("exact"), &["fixture"], "debug")
        .with_extra_env([("CARGO_HOME", root.join("cargo-home"))]);
    let snapshot = resolve_cargo_build_inputs(&spec).unwrap();
    assert!(snapshot.is_content_current().unwrap());
    fs::write(
        replacement.join("fragment.toml"),
        "[build]\nrustflags = ['--cfg=replaced']\n",
    )
    .unwrap();
    assert!(snapshot.is_content_current().unwrap());
    fs::remove_file(&second).unwrap();
    symlink(&replacement, &second).unwrap();
    assert!(
        !snapshot.is_content_current().unwrap(),
        "the second alias's nested include must be guarded even when config bytes match",
    );

    // Directory-alias cycles must terminate discovery even for callers using
    // a custom Cargo program that does not reject them in metadata.
    fs::write(
        cargo_dir.join("config.toml"),
        "include = [\"loop/config.toml\"]\n",
    )
    .unwrap();
    symlink(&cargo_dir, cargo_dir.join("loop")).unwrap();
    append_cargo_configuration_inputs(&mut Vec::new(), &spec, &workspace).unwrap();
    fs::remove_dir_all(root).unwrap();
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
        .with_extra_env([("CARGO_HOME", isolated_home_text)]);

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

#[cfg(unix)]
#[test]
fn warm_hits_reject_source_mutation_for_active_and_materialized_artifacts() {
    use super::build_wasm_canisters_cached;
    use crate::artifacts::test_support::fake_wasm_build_spec;

    for mode in ["isolated", "shared", "scheduled"] {
        for active_artifact in [true, false] {
            let (root, mut spec) = fake_wasm_build_spec("warm-hit-source-race");
            if mode != "isolated" {
                spec = spec.with_shared_incremental_target(root.join("incremental"));
            }
            if mode == "scheduled" {
                spec = spec.with_shared_incremental_target_maintenance_at_most_every(
                    SharedIncrementalTargetPrunePolicy::new(),
                    Duration::from_secs(60),
                );
            }
            let cold = build_wasm_canisters_cached(&spec).expect("build initial artifacts");
            assert!(
                build_wasm_canisters_cached(&spec)
                    .expect("unchanged hit")
                    .is_reused()
            );
            if !active_artifact {
                fs::remove_file(&super::expected_artifacts(&spec, &spec.target_dir)[0])
                    .expect("remove caller artifact");
            }
            let mut mutated = false;
            let result = build_wasm_canisters_cached_with_progress(
                &spec,
                WasmBuildProgressConfig::new(),
                |event| {
                    if !mutated && matches!(event, WasmBuildProgressEvent::InputsResolved { .. }) {
                        fs::write(
                            root.join("fixture/src/lib.rs"),
                            "pub fn value() -> u8 { 2 }\n",
                        )
                        .expect("mutate source during acquisition");
                        mutated = true;
                    }
                },
            );
            assert!(mutated);
            assert!(
                matches!(
                    result,
                    Err(WasmBuildError::InputsChangedDuringAcquisition { .. })
                ),
                "{mode}, active={active_artifact}: {result:?}"
            );
            drop(cold);
            fs::remove_dir_all(root).expect("remove warm-hit race fixture");
        }
    }
}
