use std::path::Path;
use std::process::Command;

use crate::discovery::discover_backends;
use crate::errors::{Error, Result};
use crate::manifest::Verb;

pub fn run(repo_root: &Path) -> Result<()> {
    let backends = discover_backends(repo_root)?;
    if backends.is_empty() {
        eprintln!(
            "warning: no backends discovered under {}/backends",
            repo_root.display()
        );
        return Ok(());
    }

    for backend in backends {
        if !backend.supports(Verb::Doctor) {
            continue;
        }
        let script = repo_root
            .join("backends")
            .join(&backend.name)
            .join("adapters/doctor.sh");
        if !script.is_file() {
            return Err(Error::VerbUnsupported {
                backend: backend.name,
                verb: Verb::Doctor.as_str().into(),
            });
        }

        eprintln!("checking backend: {}", backend.name);
        let status = Command::new(script).status()?;
        if !status.success() {
            return Err(Error::SimulatorFailed {
                backend: backend.name,
                code: status.code().unwrap_or(1),
            });
        }
    }

    Ok(())
}
