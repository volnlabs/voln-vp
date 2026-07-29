use std::fs;
use std::path::Path;

use tempfile::TempDir;
use voln_vp::discovery::discover_backends;

fn write_backend(root: &Path, name: &str, body: &str) {
    let directory = root.join("backends").join(name);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("manifest.toml"), body).unwrap();
}

#[test]
fn discovers_two_backends_in_name_order() {
    let temporary = TempDir::new().unwrap();
    write_backend(
        temporary.path(),
        "renode",
        r#"
name = "renode"
verbs = ["run"]
boards = ["virt-pi5"]
"#,
    );
    write_backend(
        temporary.path(),
        "qemu",
        r#"
name = "qemu"
verbs = ["run", "test"]
boards = ["virt"]
"#,
    );

    let found = discover_backends(temporary.path()).unwrap();
    let names: Vec<_> = found.iter().map(|backend| backend.name.as_str()).collect();

    assert_eq!(names, ["qemu", "renode"]);
}

#[test]
fn invalid_manifest_returns_named_path_error() {
    let temporary = TempDir::new().unwrap();
    write_backend(temporary.path(), "broken", "this is not valid toml ===");

    let error = discover_backends(temporary.path()).unwrap_err();
    let message = error.to_string();

    assert!(message.contains("manifest invalid"), "got: {message}");
    assert!(message.contains("broken"), "got: {message}");
}

#[test]
fn missing_backends_directory_returns_empty_collection() {
    let temporary = TempDir::new().unwrap();

    let found = discover_backends(temporary.path()).unwrap();

    assert!(found.is_empty());
}
