//! Error types for loading the bundled skill catalog.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("embedded skill '{name}' is not valid UTF-8")]
    InvalidUtf8 { name: String },

    #[error("embedded skill '{name}' has no YAML frontmatter")]
    MissingFrontmatter { name: String },

    #[error("embedded skill '{name}' has invalid frontmatter")]
    InvalidFrontmatter {
        name: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error(
        "embedded skill at '{path}' declares id '{declared}' but lives under directory '{expected}'"
    )]
    IdMismatch {
        path: String,
        declared: String,
        expected: String,
    },
}
