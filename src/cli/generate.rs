//! `skillforge generate` — renders every configured output format into the
//! current project.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::ui;
use crate::config::{self, OutputFormat, SkillforgeConfig};
use crate::render::Renderer;

/// Outcome of a `generate` or `sync` pass: which formats were written
/// successfully, and which didn't (with a human-readable reason).
#[derive(Debug, Default)]
pub struct OutputReport {
    pub succeeded: Vec<(OutputFormat, PathBuf)>,
    pub failed: Vec<(OutputFormat, String)>,
}

/// Runs `skillforge generate` in the current directory.
pub fn run() -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let config = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `skillforge init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;

    let report = generate(&root, &config, &renderer)?;

    for (format, path) in &report.succeeded {
        ui::success(&format!("generated {format} -> {}", path.display()));
    }
    for (format, reason) in &report.failed {
        ui::warn(&format!("skipped {format}: {reason}"));
    }

    Ok(())
}

/// Renders and writes every format in `config.outputs`, relative to `root`.
///
/// Kept separate from [`run`] so it can be exercised in tests against a
/// temporary directory instead of the real current working directory.
fn generate(root: &Path, config: &SkillforgeConfig, renderer: &Renderer) -> Result<OutputReport> {
    let mut report = OutputReport::default();

    for &format in &config.outputs {
        match renderer.render_output(format, config) {
            Ok(content) => {
                let path = root.join(output_path(format));
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create directory {}", parent.display())
                    })?;
                }
                fs::write(&path, content)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                report.succeeded.push((format, path));
            }
            Err(err) => {
                report.failed.push((format, err.to_string()));
            }
        }
    }

    Ok(report)
}

/// Where each output format is written, relative to the project root.
/// These are the actual paths Cursor and GitHub Copilot read from.
pub(super) fn output_path(format: OutputFormat) -> PathBuf {
    match format {
        OutputFormat::ClaudeMd => PathBuf::from("CLAUDE.md"),
        OutputFormat::AgentsMd => PathBuf::from("AGENTS.md"),
        OutputFormat::CursorRules => PathBuf::from(".cursor/rules/skillforge.mdc"),
        OutputFormat::CopilotInstructions => PathBuf::from(".github/copilot-instructions.md"),
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
                explanation_style: ExplanationStyle::Short,
            },
            project: ProjectProfile {
                project_type: ProjectType::CliTool,
                stack: Stack {
                    language: "rust".to_string(),
                    framework: None,
                    database: None,
                    testing_tools: vec![],
                    package_manager: Some("cargo".to_string()),
                },
                security_level: SecurityLevel::Standard,
                testing_level: TestingLevel::Practical,
                dependency_policy: DependencyPolicy::ExplainFirst,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec![],
            outputs: OutputFormat::all(),
        }
    }

    #[test]
    fn output_path_matches_real_tool_locations() {
        assert_eq!(
            output_path(OutputFormat::ClaudeMd),
            PathBuf::from("CLAUDE.md")
        );
        assert_eq!(
            output_path(OutputFormat::AgentsMd),
            PathBuf::from("AGENTS.md")
        );
        assert_eq!(
            output_path(OutputFormat::CursorRules),
            PathBuf::from(".cursor/rules/skillforge.mdc")
        );
        assert_eq!(
            output_path(OutputFormat::CopilotInstructions),
            PathBuf::from(".github/copilot-instructions.md")
        );
    }

    #[test]
    fn writes_every_configured_output_format() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let report = generate(dir.path(), &config, &renderer).unwrap();

        assert!(
            report.failed.is_empty(),
            "unexpected failures: {:?}",
            report.failed
        );
        assert_eq!(report.succeeded.len(), 4);

        for (_, path) in &report.succeeded {
            assert!(path.exists());
            assert!(
                fs::read_to_string(path)
                    .unwrap()
                    .contains("## Decision Loop")
            );
        }
    }
}
