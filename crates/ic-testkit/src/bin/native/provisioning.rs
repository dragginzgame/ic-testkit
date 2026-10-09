//! Explicit, release-bound provisioning. Checks and launches never download.

use ic_testkit::{
    ic_host_artifacts::artifact::{Sha256Digest, decode_gzip, hash_gzip, verify_reader},
    ic_host_fs::{
        durable,
        read::{hash_file_no_follow, read_file_no_follow},
    },
    ic_host_process::tool::{
        AdmittedTool, ExecutionContext, OutputLimits, ToolSpec, capture_group_command,
    },
    pic::POCKET_IC_SERVER_VERSION,
};
use std::{
    error::Error,
    ffi::OsString,
    fs::{self, File},
    io::{self, Write as _},
    os::unix::fs::DirBuilderExt as _,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ARCHIVE_LIMIT: usize = 80 * 1024 * 1024;
const EXECUTABLE_LIMIT: usize = 512 * 1024 * 1024;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
pub(super) const DEFAULT_DIRECTORY: &str = ".tools/ic-testkit-server";

struct Asset {
    host: &'static str,
    digest: Sha256Digest,
}

fn selected_asset() -> Result<Asset> {
    // Official GitHub release asset digests, reviewed 2026-10-09.
    let (host, digest) = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => (
            "x86_64-linux",
            "131219d90dcf9bf6f3ed8ee02d8f55504ea24a6db382f5680675b7bd6b7ed3bf",
        ),
        ("macos", "x86_64") => (
            "x86_64-darwin",
            "af9ad2d781530a43556ef2d1c8f93db99a2425c6cabdc78520f61922399ed530",
        ),
        ("macos", "aarch64") => (
            "arm64-darwin",
            "9ae843fbb7ae6c6eb30137671c8629a80ef53b3a3652eb85b344847ac39f9fcd",
        ),
        _ => return Err("no reviewed PocketIC server asset for this host".into()),
    };
    Ok(Asset {
        host,
        digest: digest.parse()?,
    })
}

fn root_path(directory: &Path) -> Result<PathBuf> {
    if directory.as_os_str().is_empty() {
        return Err("server directory must not be empty".into());
    }
    let absolute = if directory.is_absolute() {
        directory.to_owned()
    } else {
        std::env::current_dir()?.join(directory)
    };
    let mut path = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir | Component::Normal(_) => path.push(component),
            Component::CurDir => continue,
            _ => return Err("server directory must not contain parent traversal".into()),
        }
        match fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.is_dir() || metadata.is_symlink() => {
                return Err(format!(
                    "server directory component is not a physical directory: {}",
                    path.display()
                )
                .into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(path)
}

fn bundle_path(root: &Path, asset: &Asset) -> PathBuf {
    root.join(format!("{}-{}", POCKET_IC_SERVER_VERSION, asset.host))
}

fn authenticated_archive(archive: &Path, digest: Sha256Digest) -> Result<Vec<u8>> {
    let bytes = read_file_no_follow(archive, ARCHIVE_LIMIT)?;
    verify_reader(bytes.as_slice(), ARCHIVE_LIMIT as u64, digest)?;
    Ok(bytes)
}

fn payload(archive: &Path, digest: Sha256Digest) -> Result<Vec<u8>> {
    let bytes = authenticated_archive(archive, digest)?;
    Ok(decode_gzip(&bytes, ARCHIVE_LIMIT, EXECUTABLE_LIMIT)?)
}

fn admit(executable: &Path, digest: Sha256Digest) -> Result<()> {
    // Refuse final symlinks and special files before the tool's version execution.
    let identity = hash_file_no_follow(executable, EXECUTABLE_LIMIT as u64)?;
    if identity.sha256 != digest {
        return Err("installed PocketIC bytes differ from the authenticated archive".into());
    }
    let directory = executable.parent().ok_or("missing executable parent")?;
    AdmittedTool::admit(
        &ToolSpec {
            executable,
            executable_bytes: EXECUTABLE_LIMIT as u64,
            sha256: digest,
            version_arguments: &[OsString::from("--version")],
            version_identity: &format!("pocket-ic-server {POCKET_IC_SERVER_VERSION}"),
        },
        &ExecutionContext {
            current_dir: directory,
            environment: &[],
        },
        OutputLimits {
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            timeout: Duration::from_secs(10),
        },
    )?;
    Ok(())
}

fn check_bundle(bundle: &Path, asset: &Asset) -> Result<PathBuf> {
    if !fs::symlink_metadata(bundle)?.is_dir() || fs::symlink_metadata(bundle)?.is_symlink() {
        return Err("PocketIC bundle must be a physical directory".into());
    }
    let bytes = authenticated_archive(&bundle.join("archive.gz"), asset.digest)?;
    let digest = hash_gzip(&bytes, ARCHIVE_LIMIT, EXECUTABLE_LIMIT as u64)?.sha256;
    drop(bytes);
    let executable = bundle.join("pocket-ic");
    admit(&executable, digest)?;
    Ok(executable)
}

pub(super) fn check(directory: &Path) -> Result<PathBuf> {
    let asset = selected_asset()?;
    check_bundle(&bundle_path(&root_path(directory)?, &asset), &asset)
}

pub(super) fn setup(directory: &Path) -> Result<PathBuf> {
    let asset = selected_asset()?;
    setup_with(directory, &asset, |archive| {
        let url = format!(
            "https://github.com/dfinity/pocketic/releases/download/{POCKET_IC_SERVER_VERSION}/pocket-ic-{}.gz",
            asset.host
        );
        let outcome = capture_group_command(
            Command::new("curl")
                .args([
                    "--proto",
                    "=https",
                    "--proto-redir",
                    "=https",
                    "--tlsv1.2",
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--connect-timeout",
                    "15",
                    "--max-time",
                    "300",
                    "--max-filesize",
                    "83886080",
                    "--output",
                ])
                .arg(archive)
                .arg(url),
            OutputLimits {
                stdout_bytes: 4096,
                stderr_bytes: 64 * 1024,
                timeout: Duration::from_secs(310),
            },
        );
        if let Some(evidence) = outcome
            .as_ref()
            .ok()
            .or_else(|| outcome.as_ref().err().and_then(|error| error.evidence()))
        {
            durable::write_bytes(
                &archive
                    .parent()
                    .ok_or("missing archive parent")?
                    .join("download.stderr"),
                &evidence.stderr,
            )?;
        }
        outcome?;
        Ok(())
    })
}

fn setup_with(
    directory: &Path,
    asset: &Asset,
    download: impl FnOnce(&Path) -> Result<()>,
) -> Result<PathBuf> {
    let root = root_path(directory)?;
    // Namespace custody remains with the caller. All cooperating setups serialize
    // through Host's admitted regular lock; checks are read-only.
    let _lock = durable::lock_regular_file_with_parents(&root.join(".setup-v1.lock"))?;
    let bundle = bundle_path(&root, asset);
    match fs::symlink_metadata(&bundle) {
        Ok(_) => return check_bundle(&bundle, asset),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let attempt = root.join(format!(
        ".setup-v1-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::DirBuilder::new().mode(0o700).create(&attempt)?;
    eprintln!("PocketIC setup evidence: {}", attempt.display());
    let result = install(&attempt, asset, download);
    if let Err(error) = result {
        if let Err(record_error) =
            durable::write_bytes(&attempt.join("failure.txt"), error.to_string().as_bytes())
        {
            eprintln!("could not record setup failure: {record_error}");
        }
        return Err(error);
    }
    // A complete immutable bundle is visible only after all admission succeeds.
    // No previous version, failed attempt or installed selection is removed.
    fs::rename(&attempt, &bundle)?;
    eprintln!("PocketIC setup evidence retained: {}", bundle.display());
    File::open(&root)?.sync_all()?;
    Ok(bundle.join("pocket-ic"))
}

fn install(
    attempt: &Path,
    asset: &Asset,
    download: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let archive = attempt.join("archive.gz");
    download(&archive)?;
    let bytes = payload(&archive, asset.digest)?;
    File::open(&archive)?.sync_all()?;
    let digest = Sha256Digest::compute(&bytes);
    let executable = attempt.join("pocket-ic");
    durable::write_with(
        &executable,
        durable::WriteOptions {
            mode: durable::PublicationMode::CreateNew,
            permissions: 0o755,
        },
        |file| -> io::Result<()> { file.write_all(&bytes) },
    )?;
    drop(bytes);
    // PocketIC requires its canonical basename. The whole directory is private
    // and unselected until this admission succeeds; no existing bundle is replaced.
    if let Err(error) = admit(&executable, digest) {
        if let Some(evidence) = error
            .downcast_ref::<ic_testkit::ic_host_process::tool::ToolError>()
            .and_then(|error| error.evidence())
        {
            for (name, bytes) in [
                ("version.stdout", &evidence.stdout),
                ("version.stderr", &evidence.stderr),
            ] {
                if let Err(record_error) = durable::write_bytes(&attempt.join(name), bytes) {
                    eprintln!("could not record version evidence: {record_error}");
                }
            }
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_testkit::ic_host_artifacts::artifact::encode_gzip;
    use std::os::unix::fs::symlink;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "testkit-provision-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn archive(version: &str) -> (Asset, Vec<u8>) {
        let script = format!("#!/bin/sh\nprintf 'pocket-ic-server {version}\\n'\n");
        let mut bytes = Vec::new();
        encode_gzip(script.as_bytes(), &mut bytes, 1, 65536).unwrap();
        (
            Asset {
                host: "fixture",
                digest: Sha256Digest::compute(&bytes),
            },
            bytes,
        )
    }

    #[test]
    fn authenticated_setup_reuses_offline_and_refuses_changed_bytes_before_execution() {
        let root = Fixture::new();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        let executable = setup_with(&root.0, &asset, |path| {
            fs::write(path, &bytes)?;
            Ok(())
        })
        .unwrap();
        let original = fs::read(&executable).unwrap();
        assert_eq!(
            setup_with(&root.0, &asset, |_| panic!(
                "verified reuse must not download"
            ))
            .unwrap(),
            executable
        );
        let entries = fs::read_dir(&root.0).unwrap().count();
        assert_eq!(
            check_bundle(&bundle_path(&root.0, &asset), &asset).unwrap(),
            executable
        );
        assert_eq!(fs::read_dir(&root.0).unwrap().count(), entries);
        let marker = root.0.join("executed");
        fs::write(
            &executable,
            format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
        )
        .unwrap();
        assert!(check_bundle(&bundle_path(&root.0, &asset), &asset).is_err());
        assert!(!marker.exists());
        fs::write(&executable, original).unwrap();
        assert!(check_bundle(&bundle_path(&root.0, &asset), &asset).is_ok());
        let saved = root.0.join("saved");
        fs::rename(&executable, &saved).unwrap();
        symlink(&saved, &executable).unwrap();
        assert!(check_bundle(&bundle_path(&root.0, &asset), &asset).is_err());
    }

    #[test]
    fn offline_check_authenticates_before_decoding_and_rejects_invalid_gzip() {
        use ic_testkit::ic_host_artifacts::artifact::{ArtifactError, GzipError};

        let root = Fixture::new();
        let (mut asset, original) = archive(POCKET_IC_SERVER_VERSION);
        let executable = setup_with(&root.0, &asset, |path| {
            fs::write(path, &original)?;
            Ok(())
        })
        .unwrap();
        let installed = fs::read(&executable).unwrap();
        let bundle = bundle_path(&root.0, &asset);
        let path = bundle.join("archive.gz");

        // Both authentication and decoding would fail; authentication wins.
        fs::write(&path, b"not gzip").unwrap();
        let error = check_bundle(&bundle, &asset).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ArtifactError>(),
            Some(ArtifactError::DigestMismatch { .. })
        ));

        let mut bad_crc = original.clone();
        let crc = bad_crc.len() - 8;
        bad_crc[crc] ^= 1;
        let mut bad_length = original.clone();
        let length = bad_length.len() - 4;
        bad_length[length] ^= 1;
        for bytes in [
            b"not gzip".to_vec(),
            original[..original.len() - 1].to_vec(),
            bad_crc,
            bad_length,
            [original.as_slice(), b"trailing"].concat(),
            [original.as_slice(), original.as_slice()].concat(),
        ] {
            fs::write(&path, &bytes).unwrap();
            asset.digest = Sha256Digest::compute(&bytes);
            let error = check_bundle(&bundle, &asset).unwrap_err();
            // Gzip refusal occurs before executable admission/version execution.
            assert!(error.downcast_ref::<GzipError>().is_some());
            assert_eq!(fs::read(&executable).unwrap(), installed);
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        fs::write(&path, &original).unwrap();
        asset.digest = Sha256Digest::compute(&original);
        assert_eq!(check_bundle(&bundle, &asset).unwrap(), executable);
    }

    #[test]
    fn failed_setup_retains_evidence_and_previous_bundles_then_can_retry() {
        let root = Fixture::new();
        let previous = root.0.join("previous-version");
        fs::create_dir(&previous).unwrap();
        fs::write(previous.join("pocket-ic"), b"previous bytes").unwrap();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        assert!(
            setup_with(&root.0, &asset, |path| {
                fs::write(path, b"partial download")?;
                Err("interrupted download".into())
            })
            .is_err()
        );
        assert!(!bundle_path(&root.0, &asset).exists());
        let failed = fs::read_dir(&root.0)
            .unwrap()
            .find_map(|entry| {
                let path = entry.unwrap().path();
                path.join("failure.txt").is_file().then_some(path)
            })
            .unwrap();
        assert_eq!(
            fs::read(failed.join("archive.gz")).unwrap(),
            b"partial download"
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = setup_with(&root.0, &asset, |path| {
                    fs::write(path, b"interrupted candidate")?;
                    panic!("injected interruption before admission");
                });
            }))
            .is_err()
        );
        assert!(!bundle_path(&root.0, &asset).exists());
        assert!(fs::read_dir(&root.0).unwrap().any(|entry| {
            fs::read(entry.unwrap().path().join("archive.gz"))
                .ok()
                .as_deref()
                == Some(b"interrupted candidate")
        }));
        assert!(
            setup_with(&root.0, &asset, |path| {
                fs::write(path, &bytes)?;
                Ok(())
            })
            .is_ok()
        );
        assert!(failed.join("failure.txt").is_file());
        assert_eq!(
            fs::read(previous.join("pocket-ic")).unwrap(),
            b"previous bytes"
        );
    }

    #[test]
    fn digest_decode_and_version_failures_never_publish() {
        for mode in [0, 1, 2] {
            let root = Fixture::new();
            let (mut asset, mut bytes) = archive("15.0.0");
            if mode == 0 {
                asset.digest = Sha256Digest::compute(b"wrong");
            }
            if mode == 1 {
                bytes = b"not gzip".to_vec();
                asset.digest = Sha256Digest::compute(&bytes);
            }
            assert!(
                setup_with(&root.0, &asset, |path| {
                    fs::write(path, &bytes)?;
                    Ok(())
                })
                .is_err()
            );
            assert!(!bundle_path(&root.0, &asset).exists());
        }
    }

    #[test]
    fn redirected_paths_are_refused_without_touching_the_target() {
        let root = Fixture::new();
        let (asset, _) = archive(POCKET_IC_SERVER_VERSION);
        let target = root.0.join("target");
        fs::create_dir(&target).unwrap();
        let redirected = root.0.join("redirected");
        symlink(&target, &redirected).unwrap();
        assert!(
            setup_with(&redirected.join("child"), &asset, |_| panic!(
                "redirected ancestor reached downloader"
            ))
            .is_err()
        );
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
        symlink(&target, root.0.join(".setup-v1.lock")).unwrap();
        assert!(
            setup_with(&root.0, &asset, |_| panic!(
                "redirected lock reached downloader"
            ))
            .is_err()
        );
    }

    #[test]
    fn concurrent_setups_download_once_and_recheck_under_the_lock() {
        let root = Fixture::new();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        let calls = AtomicU64::new(0);
        std::thread::scope(|scope| {
            let handles = (0..2)
                .map(|_| {
                    scope.spawn(|| {
                        setup_with(&root.0, &asset, |path| {
                            calls.fetch_add(1, Ordering::Relaxed);
                            fs::write(path, &bytes)?;
                            Ok(())
                        })
                        .unwrap()
                    })
                })
                .collect::<Vec<_>>();
            for handle in handles {
                assert_eq!(
                    handle.join().unwrap(),
                    bundle_path(&root.0, &asset).join("pocket-ic")
                );
            }
        });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
