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
/// Absolute paths do not depend on the current directory. Missing, non-directory,
/// and non-executable search candidates are skipped; explicit paths retain
/// their errors.
pub fn resolve_executable(program: impl AsRef<OsStr>) -> io::Result<PathBuf> {
    let program = program.as_ref();
    if program.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "executable name must not be empty",
        ));
    }
    let path = Path::new(program);
    if path.is_absolute() {
        return canonical_executable(path);
    }
    let current_dir = std::env::current_dir()?;
    if path.components().count() > 1 {
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
                        io::ErrorKind::NotFound
                            | io::ErrorKind::NotADirectory
                            | io::ErrorKind::PermissionDenied
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

    #[cfg(unix)]
    use crate::artifacts::test_support::write_executable_script;

    #[test]
    #[cfg(unix)]
    fn path_resolution_returns_one_canonical_executable_file() {
        let root = unique_temp_directory("resolve-executable")
            .canonicalize()
            .unwrap();
        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("create executable search directory");
        let tool = bin.join("optimizer");
        write_executable_script(&tool, b"#!/bin/sh\nexit 0\n");

        for (search, current) in [
            (bin.as_os_str(), &root),
            (OsStr::new("bin"), &root),
            (OsStr::new(""), &bin),
        ] {
            let resolved = resolve_executable_in(OsStr::new("optimizer"), search, current)
                .expect("resolve executable from supplied PATH");
            assert_eq!(resolved, tool.canonicalize().unwrap());
        }
        fs::remove_dir_all(root).expect("remove executable-resolution fixture");
    }

    #[test]
    #[cfg(unix)]
    fn path_resolution_skips_unusable_search_entries() {
        let root = unique_temp_directory("resolve-executable-search")
            .canonicalize()
            .unwrap();
        let not_directory = root.join("not-directory");
        fs::write(&not_directory, b"file").unwrap();
        let not_executable = root.join("not-executable");
        fs::create_dir(&not_executable).unwrap();
        fs::write(not_executable.join("optimizer"), b"not executable").unwrap();
        let directory_candidate = root.join("directory-candidate");
        fs::create_dir_all(directory_candidate.join("optimizer")).unwrap();
        let bin = root.join("bin");
        fs::create_dir(&bin).unwrap();
        let tool = bin.join("optimizer");
        write_executable_script(&tool, b"#!/bin/sh\nexit 0\n");
        let search = std::env::join_paths([
            root.join("missing"),
            not_directory.clone(),
            not_executable.clone(),
            directory_candidate,
            bin,
        ])
        .unwrap();
        assert_eq!(
            resolve_executable_in(OsStr::new("optimizer"), &search, &root).unwrap(),
            tool.canonicalize().unwrap()
        );
        assert_eq!(
            super::resolve_executable(not_directory.join("optimizer"))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::NotADirectory
        );
        assert_eq!(
            super::resolve_executable(not_executable.join("optimizer"))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            resolve_executable_in(OsStr::new("missing"), &search, &root)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::NotFound
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn absolute_executable_resolution_does_not_require_current_directory() {
        const CHILD_ENV: &str = "IC_TESTKIT_EXECUTABLE_REMOVED_CWD_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            // Removing the working directory must not affect other test threads.
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "artifacts::tool::tests::absolute_executable_resolution_does_not_require_current_directory",
                    "--test-threads=1",
                ])
                .env(CHILD_ENV, "1")
                .output()
                .unwrap();
            assert!(
                child.status.success(),
                "absolute resolution regression failed: {}{}",
                String::from_utf8_lossy(&child.stdout),
                String::from_utf8_lossy(&child.stderr)
            );
            return;
        }
        let root = unique_temp_directory("resolve-executable-removed-cwd")
            .canonicalize()
            .unwrap();
        let tool = root.join("optimizer");
        write_executable_script(&tool, b"#!/bin/sh\nexit 0\n");
        let expected = tool.canonicalize().unwrap();
        std::env::set_current_dir(&root).unwrap();
        assert_eq!(super::resolve_executable("./optimizer").unwrap(), expected);
        let removed = root.join("removed");
        fs::create_dir(&removed).unwrap();
        std::env::set_current_dir(&removed).unwrap();
        fs::remove_dir(&removed).unwrap();
        assert!(std::env::current_dir().is_err());
        assert_eq!(super::resolve_executable(&tool).unwrap(), expected);
        assert!(super::resolve_executable("optimizer").is_err());
        std::env::set_current_dir(&root).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
