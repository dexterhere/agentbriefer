//! Project configuration: schema, (de)serialization, and file I/O for `skillforge.yaml`.

mod error;
mod loader;
mod schema;

pub use error::ConfigError;
pub use loader::{CONFIG_FILE_NAME, load, save};
pub use schema::{
    ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle, ExplanationStyle,
    OutputFormat, ProjectProfile, ProjectType, SecurityLevel, SkillforgeConfig, Stack,
    TestingLevel,
};
