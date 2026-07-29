use std::path::{Path, PathBuf};
use std::process::Command;

use crate::discovery::{discover_backends, discover_boards};
use crate::errors::{Error, Result};
use crate::manifest::Verb;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchSpec {
    pub backend_name: String,
    pub board_name: String,
    pub verb: Verb,
    pub verb_path: PathBuf,
}

pub fn resolve_target(
    repo_root: &Path,
    board_name: &str,
    backend_override: Option<&str>,
) -> Result<LaunchSpec> {
    resolve_target_for(repo_root, board_name, backend_override, Verb::Run)
}

pub fn resolve_target_for(
    repo_root: &Path,
    board_name: &str,
    backend_override: Option<&str>,
    verb: Verb,
) -> Result<LaunchSpec> {
    let board = discover_boards(repo_root)?
        .into_iter()
        .find(|board| board.name == board_name)
        .ok_or_else(|| Error::BoardNotFound(board_name.into()))?;

    let backend_name = backend_override.unwrap_or(&board.default_backend);
    if !board.supports_backend(backend_name) {
        return Err(Error::BackendUnsupportedForBoard {
            board: board.name,
            backend: backend_name.into(),
        });
    }

    let backend = discover_backends(repo_root)?
        .into_iter()
        .find(|backend| backend.name == backend_name)
        .ok_or_else(|| Error::BackendNotFound(backend_name.into()))?;

    if !backend.boards.iter().any(|name| name == board_name) {
        return Err(Error::BackendUnsupportedForBoard {
            board: board.name,
            backend: backend.name,
        });
    }
    if !backend.supports(verb) {
        return Err(Error::VerbUnsupported {
            backend: backend.name,
            verb: verb.as_str().into(),
        });
    }

    let verb_path = repo_root
        .join("backends")
        .join(&backend.name)
        .join("adapters")
        .join(format!("{}.sh", verb.as_str()));
    if !verb_path.is_file() {
        return Err(Error::VerbUnsupported {
            backend: backend.name,
            verb: verb.as_str().into(),
        });
    }

    Ok(LaunchSpec {
        backend_name: backend.name,
        board_name: board.name,
        verb,
        verb_path,
    })
}

pub fn execute(spec: &LaunchSpec, args: &[String], dry_run: bool) -> Result<()> {
    if dry_run {
        println!("backend: {}", spec.backend_name);
        println!("board:   {}", spec.board_name);
        println!("adapter: {}", spec.verb_path.display());
        println!("args:    {args:?}");
        return Ok(());
    }

    let status = Command::new(&spec.verb_path).args(args).status()?;
    if status.success() {
        return Ok(());
    }

    Err(Error::SimulatorFailed {
        backend: spec.backend_name.clone(),
        code: status.code().unwrap_or(1),
    })
}
