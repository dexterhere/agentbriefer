//! `agentbriefer doctor` — a small, read-only linter over the current
//! project's configuration and generated files.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::generate::output_path;
use super::sync::{find_managed_block, managed_block};
use super::ui;
use crate::config::{self, AgentbrieferConfig, DependencyPolicy, SecurityLevel, TestingLevel};
use crate::render::Renderer;
use crate::skills::SkillRegistry;
use crate::textutil::split_frontmatter;

/// Findings from a doctor pass. Kept as a flat list of human-readable
/// strings — v1 doesn't need severity levels to be useful.
#[derive(Debug, Default)]
pub struct DoctorReport {
    pub findings: Vec<String>,
}

/// Runs `agentbriefer doctor` in the current directory.
pub fn run() -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let config = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `agentbriefer init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;
    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;

    let report = doctor(&root, &config, &renderer, &registry)?;

    if report.findings.is_empty() {
        ui::success("No issues found.");
    } else {
        for finding in &report.findings {
            ui::warn(&format!("- {finding}"));
        }
    }

    Ok(())
}

/// Runs every check against `config` and the files under `root`. Kept
/// separate from [`run`] so it can be exercised in tests against a
/// temporary directory.
fn doctor(
    root: &Path,
    config: &AgentbrieferConfig,
    renderer: &Renderer,
    registry: &SkillRegistry,
) -> Result<DoctorReport> {
    let mut findings = Vec::new();

    check_conflicting_settings(config, &mut findings);
    check_missing_fields(config, &mut findings);
    check_unknown_skill_ids(config, registry, &mut findings);
    check_generated_files(root, config, renderer, &mut findings)?;

    Ok(DoctorReport { findings })
}

/// Flags any id in `config.skills` that isn't in the bundled catalog — e.g.
/// a teammate's newer CLI added a skill and this machine's CLI predates it,
/// or the id was hand-typed into `agentbriefer.yaml` and misspelled. A soft
/// finding, not a hard error: `render`/`sync` already skip unknown ids
/// silently rather than failing outright, since a whole team's `generate`
/// breaking the moment one person's CLI binary is older than a teammate's
/// config would defeat the point of sharing `agentbriefer.yaml`.
fn check_unknown_skill_ids(
    config: &AgentbrieferConfig,
    registry: &SkillRegistry,
    findings: &mut Vec<String>,
) {
    for id in &config.skills {
        if !registry.contains(id) {
            findings.push(format!(
                "skill '{id}' is configured but not found in the bundled catalog — update \
                 agentbriefer to a version that includes it, or remove it with \
                 `agentbriefer skill remove {id}`."
            ));
        }
    }
}

fn check_conflicting_settings(config: &AgentbrieferConfig, findings: &mut Vec<String>) {
    let project = &config.project;

    if project.dependency_policy == DependencyPolicy::Allow
        && project.security_level == SecurityLevel::Strict
    {
        findings.push(
            "dependency policy is 'allow' but security level is 'strict' — consider 'explain-first' or \
             'ask-first'."
                .to_string(),
        );
    }

    if project.testing_level == TestingLevel::Light
        && project.security_level == SecurityLevel::Strict
    {
        findings.push(
            "testing level is 'light' but security level is 'strict' — consider a stricter testing level."
                .to_string(),
        );
    }
}

fn check_missing_fields(config: &AgentbrieferConfig, findings: &mut Vec<String>) {
    if config.project.stack.language.trim().is_empty() {
        findings.push(
            "project.stack.language is empty — edit agentbriefer.yaml or run `agentbriefer init` again.".to_string(),
        );
    }
}

fn check_generated_files(
    root: &Path,
    config: &AgentbrieferConfig,
    renderer: &Renderer,
    findings: &mut Vec<String>,
) -> Result<()> {
    for &format in &config.outputs {
        let path = root.join(output_path(format));

        if !path.exists() {
            findings.push(format!(
                "{format} is configured but hasn't been generated yet — run `agentbriefer generate` or \
                 `agentbriefer sync`."
            ));
            continue;
        }

        // A render failure here is `generate`/`sync`'s concern to surface;
        // doctor just skips the staleness check it can't perform.
        let Ok(rendered) = renderer.render_output(format, config) else {
            continue;
        };

        let existing = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        if is_stale(&existing, &rendered) {
            findings.push(format!(
                "{format} is out of sync with the current configuration — run `agentbriefer sync` to update it."
            ));
        }
    }

    Ok(())
}

/// Answers "would running `sync`/`generate` change this file right now?"
/// by comparing what's on disk against a fresh render, using the same
/// managed-block convention `sync` writes.
fn is_stale(existing: &str, rendered: &str) -> bool {
    let (_, fresh_body) = split_frontmatter(rendered);
    let fresh_block = managed_block(fresh_body);

    match find_managed_block(existing) {
        Some((start, end)) => existing[start..end].trim() != fresh_block.trim(),
        None => existing.trim() != rendered.trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ArchitectureStyle, DeveloperProfile, DeveloperStyle, ExplanationStyle, OutputFormat,
        ProjectProfile, ProjectType, Stack,
    };

    fn sample_config() -> AgentbrieferConfig {
        AgentbrieferConfig {
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
            custom_instructions: None,
            outputs: OutputFormat::all(),
            skills: vec![],
        }
    }

    #[test]
    fn flags_permissive_dependency_policy_under_strict_security() {
        let mut config = sample_config();
        config.project.security_level = SecurityLevel::Strict;
        config.project.dependency_policy = DependencyPolicy::Allow;
        let mut findings = Vec::new();

        check_conflicting_settings(&config, &mut findings);

        assert!(
            findings
                .iter()
                .any(|f| f.contains("dependency policy is 'allow'"))
        );
    }

    #[test]
    fn does_not_flag_consistent_settings() {
        let config = sample_config();
        let mut findings = Vec::new();

        check_conflicting_settings(&config, &mut findings);

        assert!(findings.is_empty());
    }

    #[test]
    fn flags_light_testing_under_strict_security() {
        let mut config = sample_config();
        config.project.security_level = SecurityLevel::Strict;
        config.project.testing_level = TestingLevel::Light;
        let mut findings = Vec::new();

        check_conflicting_settings(&config, &mut findings);

        assert!(
            findings
                .iter()
                .any(|f| f.contains("testing level is 'light'"))
        );
    }

    #[test]
    fn flags_empty_language() {
        let mut config = sample_config();
        config.project.stack.language = "  ".to_string();
        let mut findings = Vec::new();

        check_missing_fields(&config, &mut findings);

        assert!(
            findings
                .iter()
                .any(|f| f.contains("project.stack.language is empty"))
        );
    }

    #[test]
    fn flags_outputs_that_have_never_been_generated() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let config = sample_config();

        let report = doctor(dir.path(), &config, &renderer, &registry).unwrap();

        assert_eq!(
            report.findings.len(),
            4,
            "expected all 4 configured outputs to be flagged: {:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.contains("hasn't been generated yet"))
        );
    }

    #[test]
    fn does_not_flag_a_freshly_synced_file() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.outputs = vec![OutputFormat::ClaudeMd];

        let rendered = renderer
            .render_output(OutputFormat::ClaudeMd, &config)
            .unwrap();
        let (frontmatter, body) = split_frontmatter(&rendered);
        fs::write(
            dir.path().join("CLAUDE.md"),
            format!("{frontmatter}{}", managed_block(body)),
        )
        .unwrap();

        let report = doctor(dir.path(), &config, &renderer, &registry).unwrap();

        assert!(
            report.findings.is_empty(),
            "unexpected findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn flags_a_stale_managed_block() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.outputs = vec![OutputFormat::ClaudeMd];

        fs::write(
            dir.path().join("CLAUDE.md"),
            managed_block("stale content from a previous config"),
        )
        .unwrap();

        let report = doctor(dir.path(), &config, &renderer, &registry).unwrap();

        assert!(report.findings.iter().any(|f| f.contains("out of sync")));
    }

    #[test]
    fn flags_a_configured_skill_id_that_is_not_in_the_bundled_catalog() {
        let mut config = sample_config();
        config.skills = vec!["not-a-real-skill".to_string()];
        let registry = SkillRegistry::load().unwrap();
        let mut findings = Vec::new();

        check_unknown_skill_ids(&config, &registry, &mut findings);

        assert!(
            findings
                .iter()
                .any(|f| f.contains("not-a-real-skill") && f.contains("skill remove"))
        );
    }

    #[test]
    fn does_not_flag_a_skill_id_that_is_in_the_bundled_catalog() {
        let registry = SkillRegistry::load().unwrap();
        let real_id = registry
            .all()
            .first()
            .expect("the bundled catalog should ship with at least one seed skill")
            .id
            .clone();
        let mut config = sample_config();
        config.skills = vec![real_id];
        let mut findings = Vec::new();

        check_unknown_skill_ids(&config, &registry, &mut findings);

        assert!(findings.is_empty());
    }
}
