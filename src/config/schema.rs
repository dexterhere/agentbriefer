//! Data model for `skillforge.yaml`.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Root configuration for a project, as read from `skillforge.yaml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillforgeConfig {
    /// Name of a reusable developer profile (under `~/.config/skillforge/profiles/`)
    /// that this config extends. `None` means the config is fully self-contained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    pub developer: DeveloperProfile,
    pub project: ProjectProfile,

    /// Free-form rules describing when the agent must stop, ask, or avoid continuing.
    #[serde(default)]
    pub stop_rules: Vec<String>,

    /// Which instruction file formats `skillforge generate` should produce.
    #[serde(default = "OutputFormat::all")]
    pub outputs: Vec<OutputFormat>,
}

/// Defines the developer's working style, learning level, and preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperProfile {
    pub style: DeveloperStyle,
    pub explanation_style: ExplanationStyle,
}

/// Defines the stack, project type, and behavioral policies for the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectProfile {
    pub project_type: ProjectType,
    pub stack: Stack,
    pub security_level: SecurityLevel,
    pub testing_level: TestingLevel,
    pub dependency_policy: DependencyPolicy,
    pub architecture_style: ArchitectureStyle,
}

/// Language/framework/tooling details for a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    pub language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(default)]
    pub testing_tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DeveloperStyle {
    Minimal,
    Practical,
    LearningFocused,
    Enterprise,
    SecurityFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ExplanationStyle {
    Short,
    BeginnerFriendly,
    Detailed,
    TradeoffBased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ProjectType {
    BackendApi,
    FrontendApp,
    CliTool,
    FullStackApp,
    Library,
    DocumentationHeavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SecurityLevel {
    Basic,
    Standard,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum TestingLevel {
    Light,
    Practical,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DependencyPolicy {
    Allow,
    ExplainFirst,
    AskFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ArchitectureStyle {
    Simple,
    SimpleLayered,
    FeatureBased,
    Advanced,
}

/// An AI-agent instruction file format SkillForge can generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum OutputFormat {
    ClaudeMd,
    AgentsMd,
    CursorRules,
    CopilotInstructions,
}

impl OutputFormat {
    /// Default output set: generate every supported format.
    pub fn all() -> Vec<OutputFormat> {
        vec![
            OutputFormat::ClaudeMd,
            OutputFormat::AgentsMd,
            OutputFormat::CursorRules,
            OutputFormat::CopilotInstructions,
        ]
    }

    /// Name of the Tera template (under `templates/`) that renders this format.
    pub fn template_name(self) -> &'static str {
        match self {
            OutputFormat::ClaudeMd => "claude.md.tera",
            OutputFormat::AgentsMd => "agents.md.tera",
            OutputFormat::CursorRules => "cursor_rules.tera",
            OutputFormat::CopilotInstructions => "copilot_instructions.tera",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> SkillforgeConfig {
        SkillforgeConfig {
            extends: None,
            developer: DeveloperProfile {
                style: DeveloperStyle::Practical,
                explanation_style: ExplanationStyle::TradeoffBased,
            },
            project: ProjectProfile {
                project_type: ProjectType::CliTool,
                stack: Stack {
                    language: "rust".to_string(),
                    framework: None,
                    database: None,
                    testing_tools: vec!["cargo-test".to_string()],
                    package_manager: Some("cargo".to_string()),
                },
                security_level: SecurityLevel::Standard,
                testing_level: TestingLevel::Practical,
                dependency_policy: DependencyPolicy::ExplainFirst,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec!["Stop before modifying CI/CD configuration".to_string()],
            outputs: OutputFormat::all(),
        }
    }

    #[test]
    fn serializes_enum_variants_as_kebab_case() {
        let yaml = serde_yaml::to_string(&DeveloperStyle::LearningFocused).unwrap();
        assert_eq!(yaml.trim(), "learning-focused");
    }

    #[test]
    fn round_trips_through_yaml() {
        let config = sample_config();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: SkillforgeConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn defaults_outputs_to_all_formats_when_omitted() {
        let yaml = r#"
developer:
  style: minimal
  explanation_style: short
project:
  project_type: library
  stack:
    language: rust
  security_level: basic
  testing_level: light
  dependency_policy: allow
  architecture_style: simple
"#;
        let parsed: SkillforgeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.outputs, OutputFormat::all());
        assert!(parsed.stop_rules.is_empty());
        assert!(parsed.extends.is_none());
    }
}
