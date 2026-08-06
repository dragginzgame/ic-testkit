use std::{fs, os::unix::fs::PermissionsExt as _, path::Path};

pub fn write_executable_script(path: &Path, contents: &[u8]) {
    fs::write(path, contents).expect("write integration-test executable script");
    let mut permissions = fs::metadata(path)
        .expect("read integration-test script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make integration-test script executable");
}
