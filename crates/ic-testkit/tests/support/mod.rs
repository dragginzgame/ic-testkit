// Each integration target uses only the fixtures it needs.
#![allow(dead_code)]

#[cfg(unix)]
pub mod executable;
pub mod wait;

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn startup_config() -> ic_testkit::pic::PocketIcStartupConfig {
    ic_testkit::pic::PocketIcStartupConfig::from_env(std::time::Duration::from_secs(30))
        .expect("prepare POCKET_IC_BIN or configure IC_TESTKIT_POCKET_IC_URL for live tests")
}

pub fn pocket_ic() -> ic_testkit::pic::PocketIc {
    use ic_testkit::pic::PocketIcBuilderExt as _;
    ic_testkit::pic::PocketIcBuilder::new()
        .with_application_subnet()
        .try_build(startup_config())
        .expect("construct explicitly configured test instance")
}

pub fn unique_temp_directory(label: &str) -> PathBuf {
    let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ic-testkit-{label}-{}-{sequence}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("remove stale integration-test directory");
    }
    fs::create_dir_all(&path).expect("create integration-test directory");
    path
}
