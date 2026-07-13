//! Reading and writing `agentbriefer.yaml` config files on disk.

use std::fs;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::error::ConfigError;

/// Conventional file name for a project's Agentbriefer configuration.
pub const CONFIG_FILE_NAME: &str = "agentbriefer.yaml";

/// Loads any YAML-serializable value (a [`super::AgentbrieferConfig`] or a
/// [`super::DeveloperProfile`]) from the given file.
pub fn load<T: DeserializeOwned>(path: &Path) -> Result<T, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// Serializes any YAML-serializable value as YAML and writes it to the
/// given path, creating or overwriting the file.
pub fn save<T: Serialize>(value: &T, path: &Path) -> Result<(), ConfigError> {
    let yaml = serde_yaml::to_string(value).map_err(ConfigError::Serialize)?;

    fs::write(path, yaml).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{
        AgentbrieferConfig, ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle,
        ExplanationStyle, OutputFormat, ProjectProfile, ProjectType, SecurityLevel, Stack,
        TestingLevel,
    };

    fn sample_config() -> AgentbrieferConfig {
        AgentbrieferConfig {
            extends: None,
            developer: DeveloperProfile {
                style: DeveloperStyle::Minimal,
                explanation_style: ExplanationStyle::Short,
            },
            project: ProjectProfile {
                project_type: ProjectType::Library,
                stack: Stack {
                    language: "rust".to_string(),
                    framework: None,
                    database: None,
                    testing_tools: vec![],
                    package_manager: Some("cargo".to_string()),
                    key_dependencies: vec![],
                },
                security_level: SecurityLevel::Basic,
                testing_level: TestingLevel::Light,
                dependency_policy: DependencyPolicy::Allow,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec![],
            custom_instructions: None,
            outputs: OutputFormat::all(),
            skills: vec![],
        }
    }

    #[test]
    fn saves_then_loads_the_same_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE_NAME);
        let config = sample_config();

        save(&config, &path).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(config, loaded);
    }

    #[test]
    fn load_reports_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.yaml");

        let err = load::<AgentbrieferConfig>(&path).unwrap_err();

        assert!(matches!(err, ConfigError::Read { .. }));
    }

    #[test]
    fn load_reports_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE_NAME);
        fs::write(&path, "not: [valid, agentbriefer, config").unwrap();

        let err = load::<AgentbrieferConfig>(&path).unwrap_err();

        assert!(matches!(err, ConfigError::Parse { .. }));
    }
}
