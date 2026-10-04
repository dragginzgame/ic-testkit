use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(unix)]
pub(super) use crate::test_executable::write_executable_script;

static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn unique_temp_directory(label: &str) -> PathBuf {
    let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ic-testkit-{label}-{}-{sequence}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("remove stale test directory");
    }
    fs::create_dir_all(&path).expect("create test directory");
    path
}

/// Real Cargo metadata and identities, with a deterministic minimal Wasm producer.
#[cfg(unix)]
pub(super) fn fake_wasm_build_spec(label: &str) -> (PathBuf, super::wasm_cache::WasmBuildSpec) {
    let root = unique_temp_directory(label);
    fs::create_dir_all(root.join("fixture/src")).expect("create fixture sources");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");
    fs::write(root.join("fixture/Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\ncrate-type = [\"cdylib\"]\n")
        .expect("write fixture manifest");
    fs::write(
        root.join("fixture/src/lib.rs"),
        "pub fn value() -> u8 { 1 }\n",
    )
    .expect("write fixture source");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let quoted_cargo = format!("'{}'", cargo.to_string_lossy().replace('\'', "'\"'\"'"));
    let wrapper = root.join("cargo.sh");
    write_executable_script(
        &wrapper,
        format!(
            r#"#!/bin/sh
set -eu
if [ "$1" = build ]; then
    target="$CARGO_TARGET_DIR"
    mkdir -p "$target/wasm32-unknown-unknown/debug"
    printf '\000asm\001\000\000\000' > "$target/wasm32-unknown-unknown/debug/fixture.wasm"
    exit 0
fi
exec {quoted_cargo} "$@"
"#
        ),
    );
    let spec =
        super::wasm_cache::WasmBuildSpec::new(&root, &root.join("exact"), &["fixture"], "debug")
            .with_cargo_program(wrapper);
    (root, spec)
}
