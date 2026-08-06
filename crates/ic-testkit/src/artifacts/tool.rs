use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

#[cfg(windows)]
use std::ffi::OsString;

/// Resolve one executable exactly as an artifact-cache tool input.
///
/// Paths containing more than one component are resolved directly. Bare
/// program names are searched through the current `PATH`. The returned path is
/// canonical, points to a regular file, and can be passed to
/// [`super::ArtifactCacheSpec::with_tool`].
pub fn resolve_executable(program: impl AsRef<OsStr>) -> io::Result<PathBuf> {
    let program = program.as_ref();
    if program.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "executable name must not be empty",
        ));
    }
    let current_dir = std::env::current_dir()?;
    let path = Path::new(program);
    if path.is_absolute() || path.components().count() > 1 {
        return canonical_executable(&current_dir.join(path));
    }

    let search_path = std::env::var_os("PATH").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "cannot resolve executable `{}` because PATH is unset",
                program.to_string_lossy()
            ),
        )
    })?;
    resolve_executable_in(program, &search_path, &current_dir)
}

fn resolve_executable_in(
    program: &OsStr,
    search_path: &OsStr,
    current_dir: &Path,
) -> io::Result<PathBuf> {
    for directory in std::env::split_paths(search_path) {
        let directory = if directory.as_os_str().is_empty() {
            current_dir.to_owned()
        } else if directory.is_absolute() {
            directory
        } else {
            current_dir.join(directory)
        };
        for candidate in executable_candidates(&directory, program) {
            match canonical_executable(&candidate) {
                Ok(path) => return Ok(path),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
                    ) => {}
                Err(error) => return Err(error),
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "executable `{}` was not found in PATH",
            program.to_string_lossy()
        ),
    ))
}

fn canonical_executable(path: &Path) -> io::Result<PathBuf> {
    let canonical = path.canonicalize()?;
    let metadata = fs::metadata(&canonical)?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("executable path is not a regular file: {}", path.display()),
        ));
    }
    if !is_executable(&metadata) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("file is not executable: {}", path.display()),
        ));
    }
    Ok(canonical)
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    true
}

#[cfg(windows)]
fn executable_candidates(directory: &Path, program: &OsStr) -> Vec<PathBuf> {
    let program_path = Path::new(program);
    if program_path.extension().is_some() {
        return vec![directory.join(program_path)];
    }
    let extensions =
        std::env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".COM;.EXE;.BAT;.CMD"));
    extensions
        .to_string_lossy()
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(|extension| {
            let mut name = program.to_os_string();
            name.push(extension);
            directory.join(name)
        })
        .collect()
}

#[cfg(not(windows))]
fn executable_candidates(directory: &Path, program: &OsStr) -> Vec<PathBuf> {
    vec![directory.join(program)]
}

#[cfg(test)]
mod tests {
    use super::resolve_executable_in;
    use crate::artifacts::test_support::unique_temp_directory;
    use std::{ffi::OsStr, fs};

    #[test]
    #[cfg(unix)]
    fn path_resolution_returns_one_canonical_executable_file() {
        use std::os::unix::fs::PermissionsExt as _;

        let root = unique_temp_directory("resolve-executable");
        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("create executable search directory");
        let tool = bin.join("optimizer");
        fs::write(&tool, b"#!/bin/sh\nexit 0\n").expect("write executable fixture");
        let mut permissions = fs::metadata(&tool).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&tool, permissions).expect("make fixture executable");

        let resolved = resolve_executable_in(OsStr::new("optimizer"), bin.as_os_str(), &root)
            .expect("resolve executable from supplied PATH");

        assert_eq!(resolved, tool.canonicalize().unwrap());
        fs::remove_dir_all(root).expect("remove executable-resolution fixture");
    }
}
