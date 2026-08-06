use super::{
    IncompleteBuildDirectory, ProgressReporter, SharedIncrementalTargetMaintenanceOutcome,
    SharedIncrementalTargetPrunePolicy, WasmBuildCachePrunePolicy, WasmBuildError,
    WasmBuildOutcome, WasmBuildOutputStream, WasmBuildProgressConfig, WasmBuildProgressEvent,
    WasmBuildSpec, append_cargo_configuration_inputs, build_wasm_canisters_cached_with_progress,
    ensure_cache_directory_tag, finish_fingerprint_build, inspect_shared_incremental_target,
    maintain_shared_incremental_target, maintain_shared_incremental_target_at_most_every,
    metadata_arguments, prune_wasm_build_cache, prune_wasm_build_cache_locked,
    resolve_cargo_build_inputs, run_cargo_build, validate_spec,
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

#[cfg(unix)]
use crate::artifacts::test_support::write_executable_script;

fn canonical_fixture(path: &Path) -> PathBuf {
    path.canonicalize().expect("canonicalize test fixture path")
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
    assert!(first.to_string().contains("action=cleared"));

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
    assert!(skipped.to_string().contains("action=skipped"));
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
fn zero_progress_heartbeat_is_rejected_before_build_validation() {
    let spec = WasmBuildSpec::new(Path::new("."), Path::new("target"), &[], "debug");
    let result = build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::ZERO),
        |_| {},
    );
    assert!(
        matches!(result, Err(WasmBuildError::InvalidSpec { message }) if message.contains("heartbeat"))
    );
}

#[test]
#[cfg(unix)]
fn observed_cargo_build_forwards_raw_output_and_quiet_heartbeats() {
    let root = unique_temp_directory("observed-cargo-progress");
    let cargo = root.join("observed-cargo.sh");
    write_executable_script(
        &cargo,
        "#!/bin/sh\nprintf 'observed-stdout'\nprintf 'observed-stderr' >&2\nsleep 0.05\n",
    );
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_cargo_program(&cargo);
    let mut events = Vec::new();
    {
        let mut observer = |event| events.push(event);
        let mut progress = ProgressReporter {
            config: WasmBuildProgressConfig::new()
                .with_heartbeat_interval(Duration::from_millis(10)),
            observer: Some(&mut observer),
        };

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
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WasmBuildProgressEvent::CargoHeartbeat { .. }))
    );
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::CargoFinished { success: true, .. }
    )));
    fs::remove_dir_all(root).expect("remove observed Cargo fixture");
}

#[test]
#[cfg(unix)]
fn observed_cargo_failure_retains_captured_diagnostics_and_exit_event() {
    let root = unique_temp_directory("observed-cargo-failure");
    let cargo = root.join("failing-cargo.sh");
    write_executable_script(
        &cargo,
        "#!/bin/sh\nprintf 'failure-stdout'\nprintf 'failure-stderr' >&2\nexit 23\n",
    );
    let spec = WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
        .with_cargo_program(&cargo);
    let mut events = Vec::new();
    let error = {
        let mut observer = |event| events.push(event);
        let mut progress = ProgressReporter {
            config: WasmBuildProgressConfig::new().without_heartbeats(),
            observer: Some(&mut observer),
        };

        run_cargo_build(&spec, &root.join("cargo-target"), &mut progress)
            .expect_err("Cargo fixture must fail")
    };

    assert!(matches!(
        error,
        WasmBuildError::CommandFailed {
            status,
            stdout,
            stderr,
            ..
        } if status.code() == Some(23)
            && stdout == "failure-stdout"
            && stderr == "failure-stderr"
    ));
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::CargoFinished {
            success: false,
            code: Some(23),
            ..
        }
    )));
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
