use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::{Error, Result};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Verb {
    Run,
    Test,
    Bench,
    Doctor,
}

impl Verb {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Test => "test",
            Self::Bench => "bench",
            Self::Doctor => "doctor",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BackendManifest {
    pub name: String,
    pub verbs: Vec<Verb>,
    pub boards: Vec<String>,
}

impl BackendManifest {
    pub fn supports(&self, verb: Verb) -> bool {
        self.verbs.contains(&verb)
    }

    pub fn validate(&self) -> Result<()> {
        if self.verbs.contains(&Verb::Bench) {
            return Err(Error::ManifestInvalid {
                path: PathBuf::from("<backend>"),
                reason: "verb `bench` is reserved; no implementation exists".into(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BoardManifest {
    pub name: String,
    pub memory: String,
    pub default_backend: String,
    pub backends: Vec<String>,
}

impl BoardManifest {
    pub fn supports_backend(&self, name: &str) -> bool {
        self.backends.iter().any(|backend| backend == name)
    }

    pub fn validate(&self) -> Result<()> {
        if !self.supports_backend(&self.default_backend) {
            return Err(Error::ManifestInvalid {
                path: PathBuf::from("<board>"),
                reason: format!(
                    "default_backend `{}` not in backends={:?}",
                    self.default_backend, self.backends
                ),
            });
        }

        Ok(())
    }
}
