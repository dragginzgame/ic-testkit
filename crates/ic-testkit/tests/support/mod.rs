// Each integration target uses only the fixtures it needs.
#![allow(dead_code)]

pub mod wait;

// These fixtures exchange small scalar/tuple replies, never fixture Wasm or logs.
pub const REPLY_LIMITS: ic_testkit_candid::DecodeLimits = ic_testkit_candid::DecodeLimits {
    reply_bytes: 65_536,
    decoding_quota: 1_000_000,
    skipping_quota: 100_000,
    type_entries: 1024,
    header_bytes: 8192,
};

pub fn startup_config() -> ic_testkit::pic::PocketIcStartupConfig {
    ic_testkit::pic::PocketIcStartupConfig::from_env(std::time::Duration::from_secs(30))
        .expect("prepare POCKET_IC_BIN or configure IC_TESTKIT_POCKET_IC_URL for live tests")
}

pub fn pocket_ic() -> ic_testkit::pic::PocketIcInstance<ic_testkit::pic::PocketIc> {
    startup_config()
        .build_instance(ic_testkit::pic::PocketIcBuilder::new().with_application_subnet())
        .expect("construct explicitly configured test instance")
}

pub fn unique_temp_directory(label: &str) -> tempfile::TempDir {
    let mut directory = tempfile::Builder::new();
    directory.prefix(label);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        directory.permissions(std::fs::Permissions::from_mode(0o700));
    }
    directory
        .tempdir_in(
            std::env::temp_dir()
                .canonicalize()
                .expect("resolve test temporary root"),
        )
        .expect("create private test directory")
}
