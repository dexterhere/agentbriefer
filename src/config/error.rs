//! Error types for loading and saving Agentbriefer configuration files.

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file at {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config file at {path}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("failed to serialize config")]
    Serialize(#[source] serde_yaml::Error),

    #[error("failed to write config file at {path}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
