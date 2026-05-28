use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::startup::PicStartError;

const POCKET_IC_BIN_ENV: &str = "POCKET_IC_BIN";
const ALLOW_DOWNLOAD_ENV: &str = "IC_TESTKIT_ALLOW_POCKET_IC_DOWNLOAD";

const SERVER_NAME: &str = "pocket-ic";

///
/// Runtime policy for resolving the PocketIC server binary used by [`PicBuilder`](super::PicBuilder).
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PicRuntimeConfig {
    pocket_ic_bin: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    allow_download: bool,
    server_sha256: Option<String>,
}

impl PicRuntimeConfig {
    /// Build a runtime config from supported environment variables.
    ///
    /// Supported variables:
    ///
    /// - `POCKET_IC_BIN`: explicit trusted PocketIC server binary path
    /// - `IC_TESTKIT_ALLOW_POCKET_IC_DOWNLOAD=1`: allow network download on cache miss
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            pocket_ic_bin: non_empty_env_path(POCKET_IC_BIN_ENV),
            cache_dir: None,
            allow_download: env::var(ALLOW_DOWNLOAD_ENV).is_ok_and(|value| env_truthy(&value)),
            server_sha256: None,
        }
    }

    /// Set an explicit PocketIC server binary path.
    #[must_use]
    pub fn pocket_ic_bin(mut self, path: impl Into<PathBuf>) -> Self {
        self.pocket_ic_bin = Some(path.into());
        self
    }

    /// Set the cache root used for resolved PocketIC server binaries.
    #[must_use]
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(path.into());
        self
    }

    /// Allow or deny downloading the pinned PocketIC server binary on cache miss.
    #[must_use]
    pub const fn allow_download(mut self, allow_download: bool) -> Self {
        self.allow_download = allow_download;
        self
    }

    /// Set an expected SHA-256 digest for the ungzipped PocketIC server binary.
    #[must_use]
    pub fn server_sha256(mut self, sha256: impl Into<String>) -> Self {
        self.server_sha256 = Some(sha256.into().trim().to_ascii_lowercase());
        self
    }

    /// Resolve, validate, and optionally download the PocketIC server binary.
    pub fn ensure_binary(&self) -> Result<PathBuf, PicStartError> {
        if let Some(path) = self.pocket_ic_bin.as_deref() {
            return self.validate_binary(path);
        }

        let path = self.cache_binary_path();
        if path.exists() {
            return self.validate_binary(&path);
        }

        if !self.allow_download {
            return Err(PicStartError::BinaryUnavailable {
                message: missing_binary_message(&path),
            });
        }

        self.download_binary(&path)?;
        self.validate_binary(&path)
    }

    fn cache_binary_path(&self) -> PathBuf {
        self.cache_root()
            .join(format!(
                "pocket-ic-server-{}",
                pocket_ic::LATEST_SERVER_VERSION
            ))
            .join(SERVER_NAME)
    }

    fn cache_root(&self) -> PathBuf {
        self.cache_dir.clone().unwrap_or_else(default_cache_root)
    }

    fn validate_binary(&self, path: &Path) -> Result<PathBuf, PicStartError> {
        if path.as_os_str().is_empty() {
            return Err(PicStartError::BinaryUnavailable {
                message: missing_binary_message(path),
            });
        }

        let metadata = fs::metadata(path).map_err(|err| PicStartError::BinaryUnavailable {
            message: format!(
                "PocketIC server binary is unavailable at {}: {err}. {}",
                path.display(),
                setup_guidance()
            ),
        })?;

        if !metadata.is_file() {
            return Err(PicStartError::BinaryInvalid {
                message: format!(
                    "PocketIC server binary path {} is not a file.",
                    path.display()
                ),
            });
        }

        #[cfg(unix)]
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(PicStartError::BinaryInvalid {
                message: format!(
                    "PocketIC server binary at {} is not executable.",
                    path.display()
                ),
            });
        }

        if let Some(expected) = self.server_sha256.as_deref() {
            validate_sha256(path, expected)?;
        }

        Ok(path.to_path_buf())
    }

    fn download_binary(&self, path: &Path) -> Result<(), PicStartError> {
        let url = pocket_ic_download_url()?;
        let parent = path.parent().ok_or_else(|| PicStartError::DownloadFailed {
            message: format!(
                "failed to resolve parent directory for PocketIC cache path {}",
                path.display()
            ),
        })?;
        fs::create_dir_all(parent).map_err(|err| PicStartError::DownloadFailed {
            message: format!(
                "failed to create PocketIC cache directory {}: {err}",
                parent.display()
            ),
        })?;

        if path.exists() {
            return Ok(());
        }

        let temp_path = download_temp_path(parent);
        let result = download_gzip_to_file(&url, &temp_path)
            .and_then(|()| make_executable(&temp_path))
            .and_then(|()| {
                if let Some(expected) = self.server_sha256.as_deref() {
                    validate_sha256(&temp_path, expected)?;
                }
                Ok(())
            })
            .and_then(|()| {
                if path.exists() {
                    fs::remove_file(&temp_path).map_err(download_failed)?;
                    Ok(())
                } else {
                    fs::rename(&temp_path, path).map_err(download_failed)
                }
            });

        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }

        result
    }
}

pub(super) fn ensure_pocket_ic_bin_from_env() -> Result<PathBuf, PicStartError> {
    PicRuntimeConfig::from_env().ensure_binary()
}

fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn env_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn default_cache_root() -> PathBuf {
    env::temp_dir()
}

const fn setup_guidance() -> &'static str {
    "Set POCKET_IC_BIN to an existing ungzipped executable PocketIC server binary, or set IC_TESTKIT_ALLOW_POCKET_IC_DOWNLOAD=1 to let ic-testkit download the pinned server binary into its cache."
}

fn missing_binary_message(path: &Path) -> String {
    format!(
        "PocketIC server binary is unavailable. Checked cache path {}. {}",
        path.display(),
        setup_guidance()
    )
}

fn pocket_ic_download_url() -> Result<String, PicStartError> {
    Ok(format!(
        "https://github.com/dfinity/pocketic/releases/download/{}/pocket-ic-{}-{}.gz",
        pocket_ic::LATEST_SERVER_VERSION,
        pocket_ic_arch()?,
        pocket_ic_os()?
    ))
}

fn pocket_ic_arch() -> Result<&'static str, PicStartError> {
    match env::consts::ARCH {
        "aarch64" => Ok("arm64"),
        "x86_64" => Ok("x86_64"),
        arch => Err(PicStartError::DownloadFailed {
            message: format!("PocketIC server download is unsupported on architecture {arch}."),
        }),
    }
}

fn pocket_ic_os() -> Result<&'static str, PicStartError> {
    match env::consts::OS {
        "linux" => Ok("linux"),
        "macos" => Ok("darwin"),
        os => Err(PicStartError::DownloadFailed {
            message: format!("PocketIC server download is unsupported on operating system {os}."),
        }),
    }
}

fn download_temp_path(parent: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!(".pocket-ic-download-{}-{nanos}.tmp", process::id()))
}

fn download_gzip_to_file(url: &str, path: &Path) -> Result<(), PicStartError> {
    let response = reqwest::blocking::get(url)
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|err| PicStartError::DownloadFailed {
            message: format!("failed to download PocketIC server from {url}: {err}"),
        })?;
    let bytes = response
        .bytes()
        .map_err(|err| PicStartError::DownloadFailed {
            message: format!("failed to read PocketIC server download from {url}: {err}"),
        })?;
    let mut gz = GzDecoder::new(&bytes[..]);
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(download_failed)?;
    io::copy(&mut gz, &mut out).map_err(download_failed)?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), PicStartError> {
    let mut permissions = fs::metadata(path).map_err(download_failed)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(download_failed)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), PicStartError> {
    Ok(())
}

fn validate_sha256(path: &Path, expected: &str) -> Result<(), PicStartError> {
    if !is_sha256_hex(expected) {
        return Err(PicStartError::BinaryInvalid {
            message:
                "PocketIC server SHA-256 must be a 64-character lowercase or uppercase hex digest"
                    .to_string(),
        });
    }

    let actual = sha256_file(path).map_err(|err| PicStartError::BinaryInvalid {
        message: format!(
            "failed to calculate SHA-256 for PocketIC server binary {}: {err}",
            path.display()
        ),
    })?;

    if actual != expected {
        return Err(PicStartError::BinaryInvalid {
            message: format!(
                "PocketIC server binary {} has SHA-256 {actual}, expected {expected}.",
                path.display()
            ),
        });
    }

    Ok(())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();

    loop {
        let bytes = file.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buffer[..bytes]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn download_failed(err: io::Error) -> PicStartError {
    PicStartError::DownloadFailed {
        message: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{PicRuntimeConfig, env_truthy, is_sha256_hex, missing_binary_message};

    #[test]
    fn truthy_env_accepts_common_opt_in_values() {
        assert!(env_truthy("1"));
        assert!(env_truthy("true"));
        assert!(env_truthy("YES"));
        assert!(!env_truthy("0"));
        assert!(!env_truthy(""));
    }

    #[test]
    fn sha256_validation_requires_hex_digest() {
        assert!(is_sha256_hex(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        assert!(!is_sha256_hex("not-a-sha"));
    }

    #[test]
    fn empty_explicit_binary_is_unavailable() {
        let error = PicRuntimeConfig::default()
            .pocket_ic_bin("")
            .ensure_binary()
            .unwrap_err();

        assert!(matches!(
            error,
            super::PicStartError::BinaryUnavailable { .. }
        ));
    }

    #[test]
    fn missing_binary_guidance_mentions_opt_in_download() {
        let message = missing_binary_message(std::path::Path::new("/tmp/missing-pocket-ic"));

        assert!(message.contains("POCKET_IC_BIN"));
        assert!(message.contains("IC_TESTKIT_ALLOW_POCKET_IC_DOWNLOAD=1"));
    }
}
