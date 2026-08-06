use super::{
    ArtifactCacheError, ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec,
    ArtifactOutputValidation, entry_directory, namespace_directory, prepare_artifact_cache,
    prune_artifact_cache, resolve_key,
};
use crate::artifacts::{
    ArtifactCacheMaintenance, ArtifactCachePrunePolicy, test_support::unique_temp_directory,
};
use std::{
    ffi::OsString,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

#[test]
fn os_native_argument_and_environment_builders_affect_identity() {
    let root = unique_temp_directory("os-native-identity");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write OS-native identity input");
    let base = ArtifactCacheSpec::new(&root.join("cache"), "native", "recipe/v1")
        .with_input("input", &input)
        .with_output("output", &root.join("output"));
    let arguments = base.clone().with_arguments_os([OsString::from("--exact")]);
    let environment = base
        .clone()
        .with_environment_os([(OsString::from("MODE"), OsString::from("exact"))]);
    let unset = base
        .clone()
        .with_unset_environment_os([OsString::from("MODE")]);

    let base_key = resolve_key(&base).unwrap().key;
    assert_ne!(base_key, resolve_key(&arguments).unwrap().key);
    assert_ne!(base_key, resolve_key(&environment).unwrap().key);
    assert_ne!(base_key, resolve_key(&unset).unwrap().key);
    fs::remove_dir_all(root).expect("remove OS-native identity test directory");
}

#[test]
#[cfg(unix)]
fn non_utf8_argument_bytes_affect_artifact_identity_exactly() {
    use std::os::unix::ffi::OsStringExt as _;

    let root = unique_temp_directory("non-utf8-identity");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write non-UTF-8 identity input");
    let base = ArtifactCacheSpec::new(&root.join("cache"), "native", "recipe/v1")
        .with_input("input", &input)
        .with_output("output", &root.join("output"));
    let first = base
        .clone()
        .with_arguments_os([OsString::from_vec(vec![b'a', 0xff])]);
    let second = base.with_arguments_os([OsString::from_vec(vec![b'a', 0xfe])]);

    assert_ne!(
        resolve_key(&first).unwrap().key,
        resolve_key(&second).unwrap().key
    );
    fs::remove_dir_all(root).expect("remove non-UTF-8 identity test directory");
}

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
    let reordered = ArtifactCacheSpec::new(&root.join("cache"), "release-set", "recipe/v2")
        .with_input("source", &input)
        .with_output("root.wasm", &root.join("public/root.wasm"))
        .with_output("role-b.wasm", &root.join("public/role-b.wasm"))
        .with_output("metadata.json", &root.join("public/metadata.json"));
    assert_eq!(spec, reordered);
    assert_eq!(
        resolve_key(&spec).unwrap().key,
        resolve_key(&reordered).unwrap().key
    );
    let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
    for (name, contents) in [
        ("root.wasm", b"root".as_slice()),
        ("role-b.wasm", b"role-b".as_slice()),
        ("metadata.json", b"{}".as_slice()),
    ] {
        fs::write(transaction.output_path(name).unwrap(), contents).expect("write staged output");
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
    assert!(matches!(
        transaction.output_path("undeclared"),
        Err(ArtifactCacheError::UnknownOutput { .. })
    ));
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
    fs::write(transaction.output_path("output").unwrap(), b"output").expect("write staged output");
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
    let mut changed_namespace_spec = base.clone();
    changed_namespace_spec.namespace = "identity-other".to_owned();
    let changed_namespace = resolve_key(&changed_namespace_spec).unwrap().key;
    let changed_argument = resolve_key(&base.clone().with_arguments(&["-O3"]))
        .unwrap()
        .key;
    let changed_environment =
        resolve_key(&base.clone().with_environment(&[("MODE", "size-optimized")]))
            .unwrap()
            .key;
    let changed_unset_environment = resolve_key(&base.clone().with_unset_environment(&["MODE"]))
        .unwrap()
        .key;
    let changed_recipe = resolve_key(&ArtifactCacheSpec {
        recipe_id: "recipe/v2".to_owned(),
        ..base.clone()
    })
    .unwrap()
    .key;
    let mut changed_identity_spec = base.clone();
    changed_identity_spec.identities[0].value = b"two".to_vec();
    let changed_identity = resolve_key(&changed_identity_spec).unwrap().key;
    let mut changed_input_label_spec = base.clone();
    changed_input_label_spec.inputs[0].label = "renamed-input".to_owned();
    let changed_input_label = resolve_key(&changed_input_label_spec).unwrap().key;
    let mut changed_tool_label_spec = base.clone();
    changed_tool_label_spec.tools[0].label = "renamed-optimizer".to_owned();
    let changed_tool_label = resolve_key(&changed_tool_label_spec).unwrap().key;
    let mut changed_output_schema_spec = base.clone();
    changed_output_schema_spec.outputs[0].validation = ArtifactOutputValidation::RegularFile;
    let changed_output_schema = resolve_key(&changed_output_schema_spec).unwrap().key;
    let mut changed_output_name_spec = base.clone();
    changed_output_name_spec.outputs[0].name = "renamed-output".to_owned();
    let changed_output_name = resolve_key(&changed_output_name_spec).unwrap().key;
    fs::write(&tool, b"tool-v2").expect("change tool bytes");
    let changed_tool = resolve_key(&base).unwrap().key;
    fs::write(&tool, b"tool-v1").expect("restore tool bytes");
    fs::write(&input, b"input-v2").expect("change input bytes");
    let changed_input = resolve_key(&base).unwrap().key;

    for changed in [
        changed_namespace,
        changed_argument,
        changed_environment,
        changed_unset_environment,
        changed_recipe,
        changed_identity,
        changed_input_label,
        changed_tool_label,
        changed_output_schema,
        changed_output_name,
        changed_tool,
        changed_input,
    ] {
        assert_ne!(original, changed);
    }

    fs::write(&input, b"input-v1").expect("restore input bytes");
    let mut unkeyed_changes = base;
    unkeyed_changes.coordination_scope = "another-lock".to_owned();
    unkeyed_changes.outputs[0].destination = root.join("another-output");
    assert_eq!(original, resolve_key(&unkeyed_changes).unwrap().key);
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
    fs::write(transaction.output_path("output").unwrap(), b"valid").expect("write staged output");
    let outcome = transaction.commit().expect("commit valid entry");
    let entry = entry_directory(&namespace_directory(&spec), outcome.record().key());
    fs::write(entry.join("outputs/0000.artifact"), b"tampered").expect("tamper cached output");

    let rebuilt = prepare_artifact_cache(&spec).expect("prepare after corruption");

    assert!(matches!(rebuilt, ArtifactCachePreparation::Build(_)));
    fs::remove_dir_all(root).expect("remove tampered-entry test directory");
}

#[test]
fn malformed_manifests_and_nondirectory_entries_are_rebuilt() {
    let root = unique_temp_directory("malformed-entry");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write input");
    let spec = ArtifactCacheSpec::new(&root.join("cache"), "malformed", "recipe/v1")
        .with_input("input", &input)
        .with_output("output", &root.join("output"));
    let outcome = build_output(&spec, b"valid");
    let entry = entry_directory(&namespace_directory(&spec), outcome.record().key());
    fs::write(entry.join(super::MANIFEST_FILE), [0xff, 0xfe])
        .expect("write invalid UTF-8 manifest");

    let transaction =
        expect_build(prepare_artifact_cache(&spec).expect("prepare after malformed manifest"));
    assert!(!entry.exists());
    transaction.abort().expect("abort manifest recovery");

    fs::write(&entry, b"not a directory").expect("write nondirectory cache entry");
    let transaction =
        expect_build(prepare_artifact_cache(&spec).expect("prepare after nondirectory entry"));
    assert!(!entry.exists());
    transaction.abort().expect("abort nondirectory recovery");
    fs::remove_dir_all(root).expect("remove malformed-entry test directory");
}

#[test]
fn undeclared_entry_root_files_are_never_published_or_reused() {
    let root = unique_temp_directory("undeclared-entry-root");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write input");
    let spec = ArtifactCacheSpec::new(&root.join("cache"), "root-schema", "recipe/v1")
        .with_input("input", &input)
        .with_output("output", &root.join("output"));
    let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
    fs::write(transaction.output_path("output").unwrap(), b"output").expect("write staged output");
    fs::write(transaction.staging_directory().join("build.log"), b"log")
        .expect("write undeclared root file");
    assert!(matches!(
        transaction.commit(),
        Err(ArtifactCacheError::InvalidOutputs { .. })
    ));

    let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare next miss"));
    fs::remove_dir_all(transaction.staging_directory().join("outputs"))
        .expect("remove staging output directory");
    fs::write(
        transaction.staging_directory().join("outputs"),
        b"not a directory",
    )
    .expect("replace staging output directory");
    assert!(matches!(
        transaction.commit(),
        Err(ArtifactCacheError::InvalidOutputs { .. })
    ));

    let outcome = build_output(&spec, b"valid");
    let entry = entry_directory(&namespace_directory(&spec), outcome.record().key());
    fs::write(entry.join("unexpected"), b"extra").expect("write corrupt root file");
    let transaction =
        expect_build(prepare_artifact_cache(&spec).expect("prepare after root-schema corruption"));
    assert!(!entry.exists());
    transaction.abort().expect("abort root-schema recovery");
    fs::remove_dir_all(root).expect("remove undeclared-entry-root test directory");
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
fn pruning_removes_abandoned_staging_without_touching_active_transactions() {
    let root = unique_temp_directory("staging-pruning");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write input");
    let spec = ArtifactCacheSpec::new(&root.join("cache"), "staging-prune", "recipe/v1")
        .with_input("input", &input)
        .with_output("output", &root.join("output"));
    let transaction = expect_build(prepare_artifact_cache(&spec).expect("prepare miss"));
    let active_staging = transaction.staging_directory().to_owned();

    let active_report = prune_artifact_cache(
        spec.cache_root(),
        spec.namespace(),
        ArtifactCachePrunePolicy::new(),
    )
    .expect("prune around active transaction");
    assert_eq!(active_report.uncommitted_directories_removed(), 0);
    assert!(active_staging.exists());
    transaction.abort().expect("abort active transaction");

    let key = resolve_key(&spec).unwrap().key;
    let orphan = namespace_directory(&spec)
        .join("staging")
        .join(format!("{key}-terminated-0"));
    fs::create_dir_all(orphan.join("outputs")).expect("create orphan staging");
    fs::write(orphan.join("outputs/payload"), b"abandoned").expect("write orphan payload");

    let report = prune_artifact_cache(
        spec.cache_root(),
        spec.namespace(),
        ArtifactCachePrunePolicy::new(),
    )
    .expect("prune abandoned staging");
    assert_eq!(report.uncommitted_directories_removed(), 1);
    assert!(report.uncommitted_bytes_removed() >= 9);
    assert!(!orphan.exists());
    fs::remove_dir_all(root).expect("remove staging-pruning test directory");
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

#[test]
fn invalid_specifications_are_rejected_before_acquisition() {
    let root = unique_temp_directory("invalid-specifications");
    let input = root.join("input");
    let tool = root.join("tool");
    let output = root.join("output");
    fs::write(&input, b"input").expect("write input");
    fs::write(&tool, b"tool").expect("write tool");
    let cache = root.join("cache");
    let specs = [
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1").with_input("input", &input),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_input("bad label", &input)
            .with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_input("input", &input)
            .with_input("input", &input)
            .with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_tool("tool", &tool)
            .with_tool("tool", &tool)
            .with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_identity_bytes("identity", b"one")
            .with_identity_bytes("identity", b"two")
            .with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_environment(&[("", "value")])
            .with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_output("same", &output)
            .with_output("same", &root.join("other")),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_output("first", &output)
            .with_output("second", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1").with_output(".", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_output("output", Path::new("")),
        ArtifactCacheSpec::new(&cache, "", "recipe/v1").with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "").with_output("output", &output),
        ArtifactCacheSpec::new(&cache, "namespace", "recipe/v1")
            .with_coordination_scope("")
            .with_output("output", &output),
        ArtifactCacheSpec::new(Path::new(""), "namespace", "recipe/v1")
            .with_output("output", &output),
    ];

    for spec in specs {
        expect_invalid_spec(prepare_artifact_cache(&spec));
    }
    fs::remove_dir_all(root).expect("remove invalid-specification test directory");
}

#[test]
fn filesystem_boundaries_reject_cache_inputs_and_destination_aliases() {
    let root = unique_temp_directory("filesystem-boundaries");
    let cache = root.join("cache");
    fs::create_dir_all(&cache).expect("create cache root");
    let cache_input = cache.join("input");
    let cache_tool = cache.join("tool");
    let input = root.join("input");
    let input_directory = root.join("source");
    fs::write(&cache_input, b"cache input").expect("write cache input");
    fs::write(&cache_tool, b"cache tool").expect("write cache tool");
    fs::write(&input, b"input").expect("write input");
    fs::create_dir_all(&input_directory).expect("create input directory");
    fs::write(input_directory.join("source"), b"source").expect("write source input");

    let invalid = [
        ArtifactCacheSpec::new(&cache, "cache-input", "recipe/v1")
            .with_input("input", &cache_input)
            .with_output("output", &root.join("public/cache-input")),
        ArtifactCacheSpec::new(&cache, "cache-tool", "recipe/v1")
            .with_tool("tool", &cache_tool)
            .with_output("output", &root.join("public/cache-tool")),
        ArtifactCacheSpec::new(&cache, "cache-output", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &cache.join("public-output")),
        ArtifactCacheSpec::new(&cache, "input-output", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &input),
        ArtifactCacheSpec::new(&cache, "directory-output", "recipe/v1")
            .with_input("source", &input_directory)
            .with_output("output", &input_directory.join("generated")),
        ArtifactCacheSpec::new(&cache, "alias-output", "recipe/v1")
            .with_input("input", &input)
            .with_output("first", &root.join("public/result"))
            .with_output("second", &root.join("public/nested/../result")),
        ArtifactCacheSpec::new(&cache, "directory-destination", "recipe/v1")
            .with_input("input", &input)
            .with_output("output", &input_directory),
    ];
    for spec in invalid {
        expect_invalid_spec(prepare_artifact_cache(&spec));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let public = root.join("symlink-public");
        fs::create_dir_all(&public).expect("create symlink destination directory");
        let alias = root.join("symlink-alias");
        symlink(&public, &alias).expect("create destination-directory symlink");
        let spec = ArtifactCacheSpec::new(&cache, "symlink-output", "recipe/v1")
            .with_input("input", &input)
            .with_output("first", &public.join("result"))
            .with_output("second", &alias.join("result"));
        expect_invalid_spec(prepare_artifact_cache(&spec));
    }

    let source = root.join("ancestor-source");
    fs::create_dir_all(&source).expect("create ancestor source");
    fs::write(source.join("source"), b"source").expect("write ancestor source");
    let allowed =
        ArtifactCacheSpec::new(&source.join("nested-cache"), "ancestor-input", "recipe/v1")
            .with_input("source", &source)
            .with_output("output", &root.join("outside-source/output"));
    expect_build(prepare_artifact_cache(&allowed).expect("prepare ancestor input"))
        .abort()
        .expect("abort ancestor input transaction");

    fs::remove_dir_all(root).expect("remove filesystem-boundary test directory");
}

fn expect_build(preparation: ArtifactCachePreparation) -> super::ArtifactBuildTransaction {
    match preparation {
        ArtifactCachePreparation::Build(transaction) => transaction,
        ArtifactCachePreparation::Reused(_) => panic!("expected a cache miss transaction"),
    }
}

fn expect_invalid_spec(result: Result<ArtifactCachePreparation, ArtifactCacheError>) {
    assert!(matches!(
        result,
        Err(ArtifactCacheError::InvalidSpec { .. })
    ));
}

fn build_output(spec: &ArtifactCacheSpec, contents: &[u8]) -> ArtifactCacheOutcome {
    let transaction = expect_build(prepare_artifact_cache(spec).expect("prepare build"));
    fs::write(transaction.output_path("output").unwrap(), contents).expect("write staged output");
    transaction.commit().expect("commit output")
}
