use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

fn make_repo(exit_code: i32) -> TempDir {
    let temporary = TempDir::new().unwrap();
    let backend = temporary.path().join("backends/fake");
    fs::create_dir_all(backend.join("adapters")).unwrap();
    fs::write(
        backend.join("manifest.toml"),
        r#"
name = "fake"
verbs = ["doctor"]
boards = []
"#,
    )
    .unwrap();
    write_executable(&backend.join("adapters/doctor.sh"), exit_code);
    temporary
}

#[cfg(unix)]
fn write_executable(path: &Path, exit_code: i32) {
    use std::os::unix::fs::PermissionsExt;

    fs::write(path, format!("#!/bin/sh\nexit {exit_code}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn write_executable(path: &Path, _exit_code: i32) {
    fs::write(path, "").unwrap();
}

#[cfg(unix)]
#[test]
fn doctor_succeeds_when_backend_doctor_succeeds() {
    let temporary = make_repo(0);

    Command::cargo_bin("voln-vp")
        .unwrap()
        .env("VOLN_VP_ROOT", temporary.path())
        .arg("doctor")
        .assert()
        .success();
}

#[cfg(unix)]
#[test]
fn doctor_propagates_backend_failure_code() {
    let temporary = make_repo(9);

    Command::cargo_bin("voln-vp")
        .unwrap()
        .env("VOLN_VP_ROOT", temporary.path())
        .arg("doctor")
        .assert()
        .code(9);
}
