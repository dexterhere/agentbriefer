//! Error types for template loading and rendering.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("failed to load templates")]
    Load(#[source] tera::Error),

    #[error("template '{name}' is not valid UTF-8")]
    InvalidUtf8 { name: String },

    #[error("failed to build template context")]
    Context(#[source] tera::Error),

    #[error("failed to render template '{name}'")]
    Render {
        name: String,
        #[source]
        source: tera::Error,
    },

    #[error("failed to load the bundled skill catalog")]
    Skills(#[source] crate::skills::SkillError),
}
