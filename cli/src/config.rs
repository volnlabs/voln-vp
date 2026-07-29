use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    if let Some(path) = std::env::var_os("VOLN_VP_ROOT") {
        return PathBuf::from(path);
    }

    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if current.join("Cargo.toml").is_file() && current.join("backends").is_dir() {
            return current;
        }
        if !current.pop() {
            return PathBuf::from(".");
        }
    }
}
