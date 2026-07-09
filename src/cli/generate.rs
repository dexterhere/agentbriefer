//! `skillforge generate` — renders every configured output format into the
//! current project.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

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
                refuse_if_symlink(&path)?;
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

/// Refuses to write through an existing symlink at `path`. `fs::write`
/// follows symlinks transparently, so without this check a project
/// containing a symlink at one of the fixed output paths (e.g. a
/// repository someone clones and runs `generate`/`sync` inside) could
/// silently redirect a write to anywhere on disk the user has permission
/// to write, rather than staying inside the project. Shared with `sync`,
/// which writes through the same fixed set of output paths.
pub(super) fn refuse_if_symlink(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        bail!(
            "{} is a symlink — refusing to write through it",
            path.display()
        );
    }

    Ok(())
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
                    key_dependencies: vec![],
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

    #[test]
    #[cfg(unix)]
    fn refuses_to_write_through_a_symlinked_output_path() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let sentinel = outside.path().join("sensitive.txt");
        fs::write(&sentinel, "do not touch").unwrap();
        std::os::unix::fs::symlink(&sentinel, dir.path().join("CLAUDE.md")).unwrap();

        let renderer = Renderer::new().unwrap();
        let mut config = sample_config();
        config.outputs = vec![OutputFormat::ClaudeMd];

        let result = generate(dir.path(), &config, &renderer);

        assert!(result.is_err(), "generate should refuse the symlink");
        assert_eq!(fs::read_to_string(&sentinel).unwrap(), "do not touch");
    }
}
