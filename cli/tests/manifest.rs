use voln_vp::manifest::{BackendManifest, BoardManifest, Verb};

#[test]
fn parses_minimal_backend_manifest() {
    let raw = r#"
name = "qemu"
verbs = ["run", "test"]
boards = ["virt", "virt-pi5"]
"#;

    let manifest: BackendManifest = toml::from_str(raw).unwrap();

    assert_eq!(manifest.name, "qemu");
    assert_eq!(manifest.verbs, vec![Verb::Run, Verb::Test]);
    assert_eq!(manifest.boards, vec!["virt", "virt-pi5"]);
}

#[test]
fn parses_board_with_default_backend() {
    let raw = r#"
name = "virt-pi5"
memory = "8GB"
default_backend = "renode"
backends = ["renode", "qemu"]
"#;

    let manifest: BoardManifest = toml::from_str(raw).unwrap();

    assert_eq!(manifest.default_backend, "renode");
    assert!(manifest.supports_backend("renode"));
}

#[test]
fn board_default_backend_must_be_supported() {
    let raw = r#"
name = "virt-pi5"
memory = "8GB"
default_backend = "gem5"
backends = ["renode", "qemu"]
"#;

    let manifest: BoardManifest = toml::from_str(raw).unwrap();

    assert!(manifest.validate().is_err());
}

#[test]
fn unknown_verb_is_rejected_during_deserialization() {
    let raw = r#"
name = "qemu"
verbs = ["run", "trace"]
boards = ["virt"]
"#;

    assert!(toml::from_str::<BackendManifest>(raw).is_err());
}

#[test]
fn reserved_bench_verb_is_rejected_during_validation() {
    let raw = r#"
name = "gem5"
verbs = ["bench"]
boards = ["virt-pi5"]
"#;

    let manifest: BackendManifest = toml::from_str(raw).unwrap();

    assert!(manifest.validate().is_err());
}
