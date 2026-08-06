mod support;
#[path = "support/wait.rs"]
mod wait_support;

use ic_testkit::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache,
};
use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};
use support::unique_temp_directory;
use wait_support::wait_for_path;

const WORKER_ROOT_ENV: &str = "IC_TESTKIT_ARTIFACT_PROCESS_ROOT";
const WORKER_ID_ENV: &str = "IC_TESTKIT_ARTIFACT_PROCESS_WORKER";

#[test]
fn overlapping_processes_build_one_exact_artifact_set() {
    let root = unique_temp_directory("artifact-process-lock");
    fs::write(root.join("input"), b"input").expect("write process-test input");
    let executable = std::env::current_exe().expect("resolve current test executable");
    let mut first = spawn_worker(&executable, &root, "first");
    let mut second = spawn_worker(&executable, &root, "second");

    wait_for_path(&root.join("ready-first"));
    wait_for_path(&root.join("ready-second"));
    fs::write(root.join("go"), b"go").expect("release process-test workers");

    assert!(first.wait().expect("wait for first worker").success());
    assert!(second.wait().expect("wait for second worker").success());
    let builds = fs::read_to_string(root.join("builds"))
        .expect("read process build count")
        .lines()
        .count();
    assert_eq!(
        builds, 1,
        "exactly one process should receive a build transaction"
    );
    assert_eq!(
        fs::read(root.join("output")).expect("read process-test output"),
        b"built"
    );
    fs::remove_dir_all(root).expect("remove process-lock test directory");
}

#[test]
#[ignore = "subprocess worker selected explicitly by the parent locking test"]
fn artifact_cache_process_worker() {
    let root = PathBuf::from(
        std::env::var_os(WORKER_ROOT_ENV).expect("worker root environment must be set"),
    );
    let worker = std::env::var(WORKER_ID_ENV).expect("worker identity environment must be set");
    fs::write(root.join(format!("ready-{worker}")), b"ready").expect("mark worker ready");
    wait_for_path(&root.join("go"));

    let spec = ArtifactCacheSpec::new(&root.join("cache"), "process-lock", "recipe/v1")
        .with_coordination_scope(&format!("process-{worker}"))
        .with_input("input", &root.join("input"))
        .with_output("output", &root.join("output"));
    match prepare_artifact_cache(&spec).expect("prepare process cache acquisition") {
        ArtifactCachePreparation::Reused(record) => {
            let artifact = record
                .artifacts()
                .first()
                .expect("reused process cache must contain its output");
            assert_eq!(
                fs::read(artifact.path()).expect("read reused process output"),
                b"built"
            );
        }
        ArtifactCachePreparation::Build(transaction) => {
            let mut builds = OpenOptions::new()
                .create(true)
                .append(true)
                .open(root.join("builds"))
                .expect("open process build counter");
            writeln!(builds, "{}", std::process::id()).expect("record process build");
            builds.sync_all().expect("sync process build counter");
            thread::sleep(Duration::from_millis(250));
            fs::write(
                transaction
                    .output_path("output")
                    .expect("resolve process output path"),
                b"built",
            )
            .expect("write process output");
            assert!(matches!(
                transaction.commit().expect("commit process output"),
                ArtifactCacheOutcome::Built(_)
            ));
        }
    }
}

fn spawn_worker(executable: &Path, root: &Path, worker: &str) -> std::process::Child {
    Command::new(executable)
        .args(["--ignored", "--exact", "artifact_cache_process_worker"])
        .env(WORKER_ROOT_ENV, root)
        .env(WORKER_ID_ENV, worker)
        .spawn()
        .expect("spawn artifact-cache process worker")
}
