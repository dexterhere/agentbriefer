//! Project configuration: schema, (de)serialization, and file I/O for `agentbriefer.yaml`.

mod error;
mod loader;
mod schema;

pub use error::ConfigError;
pub use loader::{CONFIG_FILE_NAME, load, save};
pub use schema::{
    AgentbrieferConfig, ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle,
    ExplanationStyle, OutputFormat, ProjectProfile, ProjectType, SecurityLevel, Stack,
    TestingLevel,
};
