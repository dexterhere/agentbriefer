//! Loads templates (embedded in release builds, read live from disk in debug
//! builds) and renders them against a [`SkillforgeConfig`].

use rust_embed::RustEmbed;
use tera::{Context, Tera};

use crate::config::{OutputFormat, SkillforgeConfig};

use super::error::RenderError;

/// The `templates/` directory shipped alongside the crate. In debug builds
/// `RustEmbed` reads these files straight off disk on every call, so editing
/// template content needs no recompile; release builds embed the bytes into
/// the binary at compile time.
#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

/// Renders SkillForge output formats from a loaded configuration.
///
/// Construction (`new`) does the real work of registering every template
/// with the underlying Tera engine, so a `Renderer` should be built once and
/// reused across multiple `render` calls rather than rebuilt per call.
pub struct Renderer {
    tera: Tera,
}

impl Renderer {
    pub fn new() -> Result<Self, RenderError> {
        // Collect every template's (name, content) before registering any of
        // them: `add_raw_templates` validates `{% include %}`/`{% extends %}`
        // references across the whole batch it's given, so a template can
        // reference one added later in the same call. Registering files one
        // at a time via `add_raw_template` would fail on the first template
        // that includes a partial not yet seen.
        let mut templates = Vec::new();

        for name in TemplateAssets::iter() {
            let file =
                TemplateAssets::get(&name).expect("name came from iter(), get() must find it");

            let content = std::str::from_utf8(&file.data)
                .map_err(|_| RenderError::InvalidUtf8 {
                    name: name.to_string(),
                })?
                .to_string();

            templates.push((name.to_string(), content));
        }

        let mut tera = Tera::new();
        tera.add_raw_templates(templates)
            .map_err(RenderError::Load)?;

        Ok(Self { tera })
    }

    /// Renders the named template against `config`.
    pub fn render(
        &self,
        template_name: &str,
        config: &SkillforgeConfig,
    ) -> Result<String, RenderError> {
        let context = Context::from_serialize(config).map_err(RenderError::Context)?;

        self.tera
            .render(template_name, &context)
            .map_err(|source| RenderError::Render {
                name: template_name.to_string(),
                source,
            })
    }

    /// Renders the template registered for a specific [`OutputFormat`].
    pub fn render_output(
        &self,
        format: OutputFormat,
        config: &SkillforgeConfig,
    ) -> Result<String, RenderError> {
        self.render(format.template_name(), config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle, ExplanationStyle,
        ProjectProfile, ProjectType, SecurityLevel, Stack, TestingLevel,
    };

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
                    testing_tools: vec!["cargo-test".to_string(), "insta".to_string()],
                    package_manager: Some("cargo".to_string()),
                    key_dependencies: vec!["serde".to_string(), "tokio".to_string()],
                },
                security_level: SecurityLevel::Strict,
                testing_level: TestingLevel::Practical,
                dependency_policy: DependencyPolicy::ExplainFirst,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec!["Stop before modifying CI/CD configuration".to_string()],
            outputs: OutputFormat::all(),
        }
    }

    /// All four formats share the same partials, so they share the same
    /// content-presence guarantees: config values, the conditional strict
    /// security note, both loop partials, and the configured stop rule.
    #[test]
    fn every_format_renders_config_values_and_shared_partials() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        for format in OutputFormat::all() {
            let output = renderer.render_output(format, &config).unwrap();

            assert!(
                output.contains("`practical`"),
                "{format} missing developer style"
            );
            assert!(output.contains("`rust`"), "{format} missing language");
            assert!(
                output.contains("`cargo`"),
                "{format} missing package manager"
            );
            assert!(
                output.contains("cargo-test"),
                "{format} missing testing tools"
            );
            assert!(
                output.contains("`serde`"),
                "{format} missing key dependencies"
            );
            assert!(
                output.contains("Strict security mode"),
                "{format} missing strict security note"
            );
            assert!(
                output.contains("## Decision Loop"),
                "{format} missing decision loop partial"
            );
            assert!(
                output.contains("## Workflow Loops"),
                "{format} missing workflow loops partial"
            );
            assert!(
                output.contains("Stop before modifying CI/CD configuration"),
                "{format} missing configured stop rule"
            );
        }
    }

    #[test]
    fn each_format_has_its_own_header() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let claude = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();
        assert!(claude.starts_with("# CLAUDE.md"));

        let agents = renderer
            .render_output(OutputFormat::AgentsMd, &config)
            .unwrap();
        assert!(agents.starts_with("# AGENTS.md"));

        let cursor = renderer
            .render_output(OutputFormat::CursorRules, &config)
            .unwrap();
        assert!(cursor.starts_with("---\n"));
        assert!(cursor.contains("alwaysApply: true"));

        let copilot = renderer
            .render_output(OutputFormat::CopilotInstructions, &config)
            .unwrap();
        assert!(copilot.starts_with("# GitHub Copilot Instructions"));
    }

    #[test]
    fn falls_back_to_default_stop_rule_when_none_configured() {
        let renderer = Renderer::new().unwrap();
        let mut config = sample_config();
        config.stop_rules = vec![];

        let output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();

        assert!(output.contains("Stop once the requested change is complete"));
    }

    #[test]
    fn snapshot_of_rendered_claude_md() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();

        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_of_rendered_agents_md() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let output = renderer
            .render_output(OutputFormat::AgentsMd, &config)
            .unwrap();

        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_of_rendered_cursor_rules() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let output = renderer
            .render_output(OutputFormat::CursorRules, &config)
            .unwrap();

        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_of_rendered_copilot_instructions() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let output = renderer
            .render_output(OutputFormat::CopilotInstructions, &config)
            .unwrap();

        insta::assert_snapshot!(output);
    }
}
