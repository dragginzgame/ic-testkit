// Unit and integration tests compile as separate crates. Keep this small
// unit-test fixture in the ordinary module tree without exposing a library API.
use std::{
    fs,
    io::Write as _,
    os::unix::fs::PermissionsExt as _,
    path::Path,
    process::{Command, Stdio},
};

pub fn write_executable_script(path: &Path, contents: impl AsRef<[u8]>) {
    // A parallel subprocess can inherit a writable file descriptor between
    // fork and exec, keeping a freshly written script busy after fs::write
    // returns. Write in a child instead: the test runner only holds a pipe,
    // and waiting for the writer closes every writable handle before use.
    let mut writer = Command::new("/bin/sh")
        .args(["-c", "cat > \"$1\"", "write-test-script"])
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn executable test script writer");
    let written = writer
        .stdin
        .take()
        .expect("script writer stdin")
        .write_all(contents.as_ref());
    let status = writer
        .wait()
        .expect("wait for executable test script writer");
    written.expect("write executable test script");
    assert!(status.success(), "script writer failed: {status}");
    let mut permissions = fs::metadata(path)
        .expect("read executable test script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make test script executable");
}
