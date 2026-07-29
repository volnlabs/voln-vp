use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn version_prints() {
    Command::cargo_bin("voln-vp")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("voln-vp 0.1.0"));
}

#[test]
fn help_prints() {
    Command::cargo_bin("voln-vp")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("doctor"))
        .stdout(contains("run"))
        .stdout(contains("test"));
}
