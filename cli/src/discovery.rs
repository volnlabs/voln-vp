use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::{Error, Result};
use crate::manifest::{BackendManifest, BoardManifest};

pub fn discover_backends(root: &Path) -> Result<Vec<BackendManifest>> {
    let paths = manifest_paths(&root.join("backends"), "manifest.toml")?;
    paths
        .into_iter()
        .map(|path| {
            let raw = fs::read_to_string(&path)?;
            let manifest: BackendManifest =
                toml::from_str(&raw).map_err(|error| Error::ManifestInvalid {
                    path: path.clone(),
                    reason: format!("TOML parse error: {error}"),
                })?;
            manifest
                .validate()
                .map_err(|error| attach_manifest_path(error, path))?;
            Ok(manifest)
        })
        .collect()
}

pub fn discover_boards(root: &Path) -> Result<Vec<BoardManifest>> {
    let paths = manifest_paths(&root.join("boards"), "board.toml")?;
    paths
        .into_iter()
        .map(|path| {
            let raw = fs::read_to_string(&path)?;
            let manifest: BoardManifest =
                toml::from_str(&raw).map_err(|error| Error::ManifestInvalid {
                    path: path.clone(),
                    reason: format!("TOML parse error: {error}"),
                })?;
            manifest
                .validate()
                .map_err(|error| attach_manifest_path(error, path))?;
            Ok(manifest)
        })
        .collect()
}

fn manifest_paths(directory: &Path, file_name: &str) -> Result<Vec<PathBuf>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let path = entry.path().join(file_name);
            if path.is_file() {
                paths.push(path);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

fn attach_manifest_path(error: Error, path: PathBuf) -> Error {
    match error {
        Error::ManifestInvalid { reason, .. } => Error::ManifestInvalid { path, reason },
        other => other,
    }
}
