//! Reading and writing `skillforge.yaml` config files on disk.

use std::fs;
use std::path::Path;

use super::error::ConfigError;
use super::schema::SkillforgeConfig;

/// Conventional file name for a project's SkillForge configuration.
pub const CONFIG_FILE_NAME: &str = "skillforge.yaml";

/// Loads a [`SkillforgeConfig`] from the given YAML file.
pub fn load(path: &Path) -> Result<SkillforgeConfig, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// Serializes a [`SkillforgeConfig`] as YAML and writes it to the given path,
/// creating or overwriting the file.
pub fn save(config: &SkillforgeConfig, path: &Path) -> Result<(), ConfigError> {
    let yaml = serde_yaml::to_string(config).map_err(ConfigError::Serialize)?;

    fs::write(path, yaml).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{
        ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle, ExplanationStyle,
        OutputFormat, ProjectProfile, ProjectType, SecurityLevel, Stack, TestingLevel,
    };

    fn sample_config() -> SkillforgeConfig {
        SkillforgeConfig {
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
                },
                security_level: SecurityLevel::Basic,
                testing_level: TestingLevel::Light,
                dependency_policy: DependencyPolicy::Allow,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec![],
            outputs: OutputFormat::all(),
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

        let err = load(&path).unwrap_err();

        assert!(matches!(err, ConfigError::Read { .. }));
    }

    #[test]
    fn load_reports_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE_NAME);
        fs::write(&path, "not: [valid, skillforge, config").unwrap();

        let err = load(&path).unwrap_err();

        assert!(matches!(err, ConfigError::Parse { .. }));
    }
}
