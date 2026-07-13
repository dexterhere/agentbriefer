//! Loads templates (embedded in release builds, read live from disk in debug
//! builds) and renders them against a [`AgentbrieferConfig`].

use rust_embed::RustEmbed;
use serde::Serialize;
use tera::{Context, Tera};

use crate::config::{AgentbrieferConfig, OutputFormat};
use crate::skills::SkillRegistry;

use super::error::RenderError;

/// The `templates/` directory shipped alongside the crate. In debug builds
/// `RustEmbed` reads these files straight off disk on every call, so editing
/// template content needs no recompile; release builds embed the bytes into
/// the binary at compile time.
#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

/// Renders Agentbriefer output formats from a loaded configuration.
///
/// Construction (`new`) does the real work of registering every template
/// with the underlying Tera engine, so a `Renderer` should be built once and
/// reused across multiple `render` calls rather than rebuilt per call.
pub struct Renderer {
    tera: Tera,
    skills: SkillRegistry,
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

        let skills = SkillRegistry::load().map_err(RenderError::Skills)?;

        Ok(Self { tera, skills })
    }

    /// Renders the named template against `config`.
    pub fn render(
        &self,
        template_name: &str,
        config: &AgentbrieferConfig,
    ) -> Result<String, RenderError> {
        let mut context = Context::from_serialize(config).map_err(RenderError::Context)?;
        insert_descriptions(&mut context, config);
        insert_installed_skills(&mut context, config, &self.skills);

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
        config: &AgentbrieferConfig,
    ) -> Result<String, RenderError> {
        self.render(format.template_name(), config)
    }
}

/// Inserts a concrete, agent-facing description for every configured enum
/// value into `context`, keyed by name for templates to reference (e.g.
/// `{{ security_level_description }}`). These are computed from `config`'s
/// current values, not persisted — they never touch `AgentbrieferConfig`'s
/// `Serialize` impl or `agentbriefer.yaml` itself.
fn insert_descriptions(context: &mut Context, config: &AgentbrieferConfig) {
    context.insert(
        "developer_style_description",
        config.developer.style.description(),
    );
    context.insert(
        "explanation_style_description",
        config.developer.explanation_style.description(),
    );
    context.insert(
        "project_type_description",
        config.project.project_type.description(),
    );
    context.insert(
        "security_level_description",
        config.project.security_level.description(),
    );
    context.insert(
        "testing_level_description",
        config.project.testing_level.description(),
    );
    context.insert(
        "dependency_policy_description",
        config.project.dependency_policy.description(),
    );
    context.insert(
        "architecture_style_description",
        config.project.architecture_style.description(),
    );
}

/// A skill's rendering-relevant fields, exposed to `installed_skills.tera`
/// as `{{ skill.name }}` / `{{ skill.body }}`.
#[derive(Serialize)]
struct InstalledSkillView<'a> {
    name: &'a str,
    description: &'a str,
    body: &'a str,
}

/// Resolves `config.skills` ids against `registry` and inserts the matches
/// as `installed_skills` for `partials/installed_skills.tera` to loop over.
/// An id with no match in the registry is silently skipped here — a stale
/// or unknown skill id must never fail a render, since that would break a
/// whole team's `generate`/`sync` the moment one person's CLI binary is
/// older than a teammate's `agentbriefer.yaml`. `doctor` is the layer
/// responsible for surfacing that mismatch as a finding.
fn insert_installed_skills(context: &mut Context, config: &AgentbrieferConfig, registry: &SkillRegistry) {
    let views: Vec<InstalledSkillView> = config
        .skills
        .iter()
        .filter_map(|id| registry.get(id))
        .map(|skill| InstalledSkillView {
            name: &skill.name,
            description: &skill.description,
            body: &skill.body,
        })
        .collect();

    context.insert("installed_skills", &views);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle, ExplanationStyle,
        ProjectProfile, ProjectType, SecurityLevel, Stack, TestingLevel,
    };

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
            custom_instructions: None,
            outputs: OutputFormat::all(),
            skills: vec![],
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
                output.contains("Prefer stopping to ask over guessing"),
                "{format} missing strict security level description"
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
    fn description_context_is_driven_by_the_configured_value_not_static() {
        let renderer = Renderer::new().unwrap();
        let mut config = sample_config();
        config.developer.style = DeveloperStyle::Minimal;

        let minimal_output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();
        assert!(
            minimal_output.contains(DeveloperStyle::Minimal.description()),
            "expected minimal's own description to render"
        );
        assert!(
            !minimal_output.contains(DeveloperStyle::Enterprise.description()),
            "must not render a different style's description"
        );

        config.developer.style = DeveloperStyle::Enterprise;
        let enterprise_output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();
        assert!(
            enterprise_output.contains(DeveloperStyle::Enterprise.description()),
            "expected enterprise's own description to render"
        );
        assert!(
            !enterprise_output.contains(DeveloperStyle::Minimal.description()),
            "must not still render the previous style's description"
        );
    }

    #[test]
    fn renders_custom_instructions_only_when_set() {
        let renderer = Renderer::new().unwrap();
        let mut config = sample_config();

        let without = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();
        assert!(!without.contains("## Custom Instructions"));

        config.custom_instructions = Some("Never touch the legacy billing module.".to_string());
        let with = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();
        assert!(with.contains("## Custom Instructions"));
        assert!(with.contains("Never touch the legacy billing module."));
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
    fn renders_nothing_for_installed_skills_when_config_skills_is_empty() {
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();

        assert!(!output.contains("## Installed Skills"));
    }

    #[test]
    fn renders_an_installed_skills_body_when_configured() {
        let renderer = Renderer::new().unwrap();
        let mut config = sample_config();
        config.skills = vec!["server-components-by-default".to_string()];

        let output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();

        assert!(output.contains("## Installed Skills"));
        assert!(output.contains("Server Components by Default"));
        assert!(output.contains("Add `\"use client\"` only where"));
    }

    #[test]
    fn skips_an_unknown_configured_skill_id_without_failing_render() {
        let renderer = Renderer::new().unwrap();
        let mut config = sample_config();
        config.skills = vec!["not-a-real-skill".to_string()];

        let output = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();

        assert!(!output.contains("## Installed Skills"));
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
