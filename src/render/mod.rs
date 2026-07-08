//! Template rendering: turns a [`crate::config::SkillforgeConfig`] into the
//! text of an AI-agent instruction file.

mod engine;
mod error;

pub use engine::Renderer;
pub use error::RenderError;
