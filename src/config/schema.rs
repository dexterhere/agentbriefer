//! Data model for `agentbriefer.yaml`.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Root configuration for a project, as read from `agentbriefer.yaml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentbrieferConfig {
    /// Name of a reusable developer profile (under `~/.config/agentbriefer/profiles/`)
    /// that this config extends. `None` means the config is fully self-contained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    pub developer: DeveloperProfile,
    pub project: ProjectProfile,

    /// Free-form rules describing when the agent must stop, ask, or avoid continuing.
    #[serde(default)]
    pub stop_rules: Vec<String>,

    /// Ids of skills installed into this project (materialized under
    /// `.agentbriefer/skills/<id>/` and inlined into every generated
    /// output). Resolved against the CLI's embedded skill catalog at
    /// generate/sync time — this field only ever holds ids, never skill
    /// content itself.
    #[serde(default)]
    pub skills: Vec<String>,

    /// Free-form, project-specific guidance beyond the structured settings
    /// above — the place for whatever doesn't fit a dropdown (a quirky
    /// build step, a library with surprising behavior, a convention unique
    /// to this codebase). `None` means nothing has been added yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,

    /// Which instruction file formats `agentbriefer generate` should produce.
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
    /// Key direct dependencies detected from the project's manifest (e.g.
    /// `Cargo.toml`), excluding whatever is already captured by
    /// `framework`/`database`/`testing_tools`. A snapshot from whenever
    /// `init` last ran — not kept in sync automatically.
    #[serde(default)]
    pub key_dependencies: Vec<String>,
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

impl DeveloperStyle {
    /// Concrete, agent-facing meaning of this value — rendered into every
    /// generated instruction file so "minimal" is an actual behavioral
    /// expectation, not just a label.
    pub fn description(self) -> &'static str {
        match self {
            DeveloperStyle::Minimal => {
                "Ship the smallest correct change. No proactive refactors, extra \
                 abstractions, or unsolicited suggestions beyond what was asked."
            }
            DeveloperStyle::Practical => {
                "Balance speed and quality: reasonable defaults, light structure, \
                 pragmatic tradeoffs over textbook-perfect solutions."
            }
            DeveloperStyle::LearningFocused => {
                "Prioritize clarity and teaching over speed: explain reasoning, name \
                 the concepts involved, prefer well-known patterns over clever ones."
            }
            DeveloperStyle::Enterprise => {
                "Favor consistency and long-term maintainability over speed: follow \
                 existing patterns strictly, flag deviations before introducing them."
            }
            DeveloperStyle::SecurityFirst => {
                "Treat every change as a potential attack surface: default to the \
                 most conservative option, flag security-relevant tradeoffs \
                 explicitly even for small changes."
            }
        }
    }
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

impl ExplanationStyle {
    /// Concrete, agent-facing meaning of this value.
    pub fn description(self) -> &'static str {
        match self {
            ExplanationStyle::Short => {
                "State what changed and why in 1-3 sentences. No step-by-step \
                 narration, no restating the request."
            }
            ExplanationStyle::BeginnerFriendly => {
                "Explain what changed and why in plain language, defining any \
                 non-obvious terms or concepts used."
            }
            ExplanationStyle::Detailed => {
                "Explain what changed, why, and what alternatives were considered \
                 — enough detail to review without re-reading the diff."
            }
            ExplanationStyle::TradeoffBased => {
                "Focus explanations on the tradeoffs made: what was gained, what \
                 was given up, and why this option won."
            }
        }
    }
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

impl ProjectType {
    /// Concrete, agent-facing meaning of this value: what to prioritize
    /// for this shape of project.
    pub fn description(self) -> &'static str {
        match self {
            ProjectType::BackendApi => {
                "A server-side API. Prioritize request validation, error handling, \
                 and stable contracts over UI concerns."
            }
            ProjectType::FrontendApp => {
                "A client-side application. Prioritize UI state management, \
                 accessibility, and user-facing error handling."
            }
            ProjectType::CliTool => {
                "A command-line tool. Prioritize clear help text, predictable exit \
                 codes, and scriptability over interactive polish."
            }
            ProjectType::FullStackApp => {
                "Spans both frontend and backend. Keep client/server contracts \
                 explicit and changes to one side's assumptions visible to the \
                 other."
            }
            ProjectType::Library => {
                "Consumed by other code, not run directly. Prioritize a stable \
                 public API, semantic versioning discipline, and avoiding breaking \
                 changes."
            }
            ProjectType::DocumentationHeavy => {
                "Documentation is a first-class deliverable. Treat outdated or \
                 missing docs as a defect, not an afterthought."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SecurityLevel {
    Basic,
    Standard,
    Strict,
}

impl SecurityLevel {
    /// Concrete, agent-facing meaning of this value. `Strict`'s text is
    /// also what `config_summary.tera` renders as its callout block — one
    /// source of truth, so the label and the callout can't drift apart.
    pub fn description(self) -> &'static str {
        match self {
            SecurityLevel::Basic => {
                "Baseline hygiene only. Follow ordinary secure-coding practice; \
                 don't add extra confirmation steps for routine changes."
            }
            SecurityLevel::Standard => {
                "Treat auth, secrets, and permission changes with normal caution \
                 — validate inputs, avoid leaking sensitive data, ask when \
                 genuinely unsure."
            }
            SecurityLevel::Strict => {
                "Treat every change touching auth, validation, secrets, \
                 permissions, or data access as high-risk by default. Prefer \
                 stopping to ask over guessing when a security implication is \
                 unclear."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum TestingLevel {
    Light,
    Practical,
    Strict,
}

impl TestingLevel {
    /// Concrete, agent-facing meaning of this value.
    pub fn description(self) -> &'static str {
        match self {
            TestingLevel::Light => {
                "Only add tests when explicitly requested or for genuinely risky \
                 logic. Don't block a change on missing test coverage."
            }
            TestingLevel::Practical => {
                "Add tests for new logic and bug fixes when practical. Don't \
                 chase 100% coverage or test trivial code."
            }
            TestingLevel::Strict => {
                "Every behavior change needs a corresponding test. Treat missing \
                 coverage on risky logic as incomplete work, not optional polish."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DependencyPolicy {
    Allow,
    ExplainFirst,
    AskFirst,
}

impl DependencyPolicy {
    /// Concrete, agent-facing meaning of this value.
    pub fn description(self) -> &'static str {
        match self {
            DependencyPolicy::Allow => {
                "Add a new dependency when it's the reasonable choice, without \
                 pausing to ask first."
            }
            DependencyPolicy::ExplainFirst => {
                "A new dependency is fine, but explain why it's needed and what \
                 it replaces before adding it."
            }
            DependencyPolicy::AskFirst => {
                "Never add a new dependency without asking first, even if it \
                 seems obviously justified."
            }
        }
    }
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

impl ArchitectureStyle {
    /// Concrete, agent-facing meaning of this value.
    pub fn description(self) -> &'static str {
        match self {
            ArchitectureStyle::Simple => {
                "Prefer the flattest structure that works. Don't introduce \
                 layers, interfaces, or abstractions the codebase doesn't \
                 already have."
            }
            ArchitectureStyle::SimpleLayered => {
                "Keep a light separation of concerns (e.g. handlers/logic/data), \
                 but don't over-generalize beyond what's already established."
            }
            ArchitectureStyle::FeatureBased => {
                "Organize by feature/domain rather than by technical layer. New \
                 code should follow the existing feature-folder convention."
            }
            ArchitectureStyle::Advanced => {
                "The codebase already uses deliberate architectural patterns \
                 (e.g. hexagonal, DDD, plugin systems) — follow them precisely \
                 rather than simplifying."
            }
        }
    }
}

/// An AI-agent instruction file format Agentbriefer can generate.
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
    use strum::IntoEnumIterator;

    fn sample_config() -> AgentbrieferConfig {
        AgentbrieferConfig {
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
                    key_dependencies: vec![],
                },
                security_level: SecurityLevel::Standard,
                testing_level: TestingLevel::Practical,
                dependency_policy: DependencyPolicy::ExplainFirst,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec!["Stop before modifying CI/CD configuration".to_string()],
            custom_instructions: None,
            outputs: OutputFormat::all(),
            skills: vec![],
        }
    }

    #[test]
    fn every_developer_style_has_a_non_empty_description() {
        for style in DeveloperStyle::iter() {
            assert!(!style.description().is_empty());
        }
    }

    #[test]
    fn every_explanation_style_has_a_non_empty_description() {
        for style in ExplanationStyle::iter() {
            assert!(!style.description().is_empty());
        }
    }

    #[test]
    fn every_project_type_has_a_non_empty_description() {
        for project_type in ProjectType::iter() {
            assert!(!project_type.description().is_empty());
        }
    }

    #[test]
    fn every_security_level_has_a_non_empty_description() {
        for level in SecurityLevel::iter() {
            assert!(!level.description().is_empty());
        }
    }

    #[test]
    fn every_testing_level_has_a_non_empty_description() {
        for level in TestingLevel::iter() {
            assert!(!level.description().is_empty());
        }
    }

    #[test]
    fn every_dependency_policy_has_a_non_empty_description() {
        for policy in DependencyPolicy::iter() {
            assert!(!policy.description().is_empty());
        }
    }

    #[test]
    fn every_architecture_style_has_a_non_empty_description() {
        for style in ArchitectureStyle::iter() {
            assert!(!style.description().is_empty());
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
        let parsed: AgentbrieferConfig = serde_yaml::from_str(&yaml).unwrap();
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
        let parsed: AgentbrieferConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.outputs, OutputFormat::all());
        assert!(parsed.stop_rules.is_empty());
        assert!(parsed.extends.is_none());
        assert!(parsed.skills.is_empty());
    }
}
