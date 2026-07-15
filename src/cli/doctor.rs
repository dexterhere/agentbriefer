//! `agentbriefer doctor` — a small linter over the current project's
//! configuration and generated files, with an optional `--fix` that
//! resyncs drifted outputs and a `--json` mode for machine consumption.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use super::generate::output_path;
use super::sync::{self, find_managed_block, managed_block};
use super::ui;
use crate::config::{self, AgentbrieferConfig, DependencyPolicy, SecurityLevel, TestingLevel};
use crate::render::Renderer;
use crate::skills::SkillRegistry;
use crate::textutil::split_frontmatter;

/// How serious a [`Finding`] is. `Error`-severity findings are what drive
/// `doctor`'s exit code (unless `--no-fail` is passed); `Warning`-severity
/// findings are surfaced but never fail the command on their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Warning,
    Error,
}

/// A single diagnostic emitted by a `check_*` function.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub fixable: bool,
}

/// Findings from a doctor pass.
#[derive(Debug, Default)]
pub struct DoctorReport {
    pub findings: Vec<Finding>,
}

impl DoctorReport {
    fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    fn warning_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    fn has_fixable(&self) -> bool {
        self.findings.iter().any(|f| f.fixable)
    }
}

/// JSON-serializable summary counts for a [`DoctorReport`].
#[derive(Debug, Serialize)]
pub struct Summary {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub fixed: usize,
}

/// The `--json` wrapper around a [`DoctorReport`]. Kept separate from
/// `DoctorReport` (which stays the internal currency every check function
/// writes into) so the on-disk JSON shape is independent of internal
/// refactors.
#[derive(Debug, Serialize)]
pub struct DoctorReportJson {
    pub schema_version: u32,
    pub findings: Vec<Finding>,
    pub summary: Summary,
    pub clean: bool,
}

impl DoctorReportJson {
    fn from_report(report: DoctorReport, fixed: usize) -> Self {
        let errors = report.error_count();
        let warnings = report.warning_count();
        let total = report.findings.len();

        DoctorReportJson {
            schema_version: 1,
            findings: report.findings,
            summary: Summary {
                total,
                errors,
                warnings,
                fixed,
            },
            clean: total == 0,
        }
    }
}

/// Runs `agentbriefer doctor` in the current directory.
///
/// `fix` resyncs any drifted/missing output before reporting. `json` emits
/// a [`DoctorReportJson`] document on stdout instead of the human-readable
/// listing. `no_fail` always exits 0, regardless of findings.
pub fn run(fix: bool, json: bool, no_fail: bool) -> Result<()> {
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

    let initial_report = doctor(&root, &config, &renderer, &registry)?;

    let mut fixed = 0;
    let report = if fix && initial_report.has_fixable() {
        let initial_errors = initial_report.error_count();

        let sync_report = fix_drift(&root, &config, &renderer)?;
        for (format, path) in &sync_report.succeeded {
            ui::success(&format!("synced {format} -> {}", path.display()));
        }
        for (format, reason) in &sync_report.failed {
            ui::warn(&format!("skipped {format}: {reason}"));
        }

        let final_report = doctor(&root, &config, &renderer, &registry)?;
        fixed = initial_errors.saturating_sub(final_report.error_count());
        final_report
    } else {
        initial_report
    };

    if json {
        let has_errors = report.error_count() > 0;
        let report_json = DoctorReportJson::from_report(report, fixed);
        println!("{}", serde_json::to_string_pretty(&report_json)?);

        // `--no-fail`/exit-code handling happens via `process::exit` here
        // rather than `anyhow::bail!`, because stdout must remain a single,
        // valid JSON document for CI consumers to parse — an error printed
        // to stderr by the normal `Result::Err` path doesn't affect stdout,
        // but returning `Err` from `run` would still be fine functionally;
        // we exit directly anyway to keep this branch's control flow
        // self-contained and make the "stdout is pure JSON" guarantee
        // obvious at the call site.
        if !no_fail && has_errors {
            std::process::exit(1);
        }

        return Ok(());
    }

    if report.findings.is_empty() {
        ui::success("No issues found.");
    } else {
        for finding in &report.findings {
            ui::warn(&format!("- {}", finding.message));
        }
    }

    if !no_fail {
        let errors = report.error_count();
        if errors > 0 {
            bail!("agentbriefer doctor found {errors} issue(s)");
        }
    }

    Ok(())
}

/// Resyncs every configured output. A thin call into [`sync::sync`] — the
/// only write path `doctor --fix` gains; it never touches `config.skills`
/// or anything under `.agentbriefer/skills/`.
fn fix_drift(
    root: &Path,
    config: &AgentbrieferConfig,
    renderer: &Renderer,
) -> Result<super::generate::OutputReport> {
    sync::sync(root, config, renderer)
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
    findings: &mut Vec<Finding>,
) {
    for id in &config.skills {
        if !registry.contains(id) {
            findings.push(Finding {
                code: "unknown-skill",
                severity: Severity::Warning,
                message: format!(
                    "skill '{id}' is configured but not found in the bundled catalog — update \
                     agentbriefer to a version that includes it, or remove it with \
                     `agentbriefer skill remove {id}`."
                ),
                fixable: false,
            });
        }
    }
}

fn check_conflicting_settings(config: &AgentbrieferConfig, findings: &mut Vec<Finding>) {
    let project = &config.project;

    if project.dependency_policy == DependencyPolicy::Allow
        && project.security_level == SecurityLevel::Strict
    {
        findings.push(Finding {
            code: "conflicting-settings",
            severity: Severity::Warning,
            message: "dependency policy is 'allow' but security level is 'strict' — consider 'explain-first' or \
             'ask-first'."
                .to_string(),
            fixable: false,
        });
    }

    if project.testing_level == TestingLevel::Light
        && project.security_level == SecurityLevel::Strict
    {
        findings.push(Finding {
            code: "conflicting-settings",
            severity: Severity::Warning,
            message:
                "testing level is 'light' but security level is 'strict' — consider a stricter testing level."
                    .to_string(),
            fixable: false,
        });
    }
}

fn check_missing_fields(config: &AgentbrieferConfig, findings: &mut Vec<Finding>) {
    if config.project.stack.language.trim().is_empty() {
        findings.push(Finding {
            code: "missing-field",
            severity: Severity::Warning,
            message:
                "project.stack.language is empty — edit agentbriefer.yaml or run `agentbriefer init` again."
                    .to_string(),
            fixable: false,
        });
    }
}

fn check_generated_files(
    root: &Path,
    config: &AgentbrieferConfig,
    renderer: &Renderer,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    for &format in &config.outputs {
        let path = root.join(output_path(format));

        if !path.exists() {
            findings.push(Finding {
                code: "output-missing",
                severity: Severity::Error,
                message: format!(
                    "{format} is configured but hasn't been generated yet — run `agentbriefer generate` or \
                     `agentbriefer sync`."
                ),
                fixable: true,
            });
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
            findings.push(Finding {
                code: "output-stale",
                severity: Severity::Error,
                message: format!(
                    "{format} is out of sync with the current configuration — run `agentbriefer sync` to update it."
                ),
                fixable: true,
            });
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
                .any(|f| f.message.contains("dependency policy is 'allow'"))
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
                .any(|f| f.message.contains("testing level is 'light'"))
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
                .any(|f| f.message.contains("project.stack.language is empty"))
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
                .any(|f| f.message.contains("hasn't been generated yet"))
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

        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("out of sync"))
        );
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
                .any(|f| f.message.contains("not-a-real-skill")
                    && f.message.contains("skill remove"))
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

    #[test]
    fn fix_resyncs_a_stale_output_and_a_second_doctor_pass_is_clean() {
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

        let before = doctor(dir.path(), &config, &renderer, &registry).unwrap();
        assert!(
            before.findings.iter().any(|f| f.code == "output-stale"),
            "setup should have produced a stale finding: {:?}",
            before.findings
        );

        fix_drift(dir.path(), &config, &renderer).unwrap();

        let after = doctor(dir.path(), &config, &renderer, &registry).unwrap();
        assert!(
            after.findings.is_empty(),
            "expected a clean report after fix: {:?}",
            after.findings
        );
    }

    #[test]
    fn fix_does_not_touch_skills_or_delete_anything_under_agentbriefer_skills() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.outputs = vec![OutputFormat::ClaudeMd];
        // An id that doesn't resolve in the bundled catalog — this is the
        // regression surface: `fix` must never react to an unknown-skill
        // finding by touching `config.skills` or by deleting anything under
        // `.agentbriefer/skills/`, since it only ever calls `sync::sync`.
        config.skills = vec!["not-a-real-skill".to_string()];

        // Simulate a materialized skills directory with content that would
        // be considered "stale" by `materialize_skills`'s cleanup pass, to
        // prove `fix` never runs anything resembling that pass.
        let sentinel_dir = dir
            .path()
            .join(".agentbriefer")
            .join("skills")
            .join("some-other-id");
        fs::create_dir_all(&sentinel_dir).unwrap();
        fs::write(sentinel_dir.join("SKILL.md"), "sentinel content").unwrap();

        let before_report = doctor(dir.path(), &config, &renderer, &registry).unwrap();
        assert!(
            before_report
                .findings
                .iter()
                .any(|f| f.code == "unknown-skill"),
            "setup should have produced an unknown-skill finding: {:?}",
            before_report.findings
        );

        fix_drift(dir.path(), &config, &renderer).unwrap();

        assert_eq!(
            config.skills,
            vec!["not-a-real-skill".to_string()],
            "fix must never mutate config.skills"
        );
        assert!(
            sentinel_dir.join("SKILL.md").exists(),
            "fix must never delete anything under .agentbriefer/skills/"
        );
    }

    #[test]
    fn doctor_report_json_serializes_with_expected_shape() {
        let report = DoctorReport {
            findings: vec![Finding {
                code: "unknown-skill",
                severity: Severity::Warning,
                message: "skill 'foo' is configured but not found".to_string(),
                fixable: false,
            }],
        };

        let json = DoctorReportJson::from_report(report, 0);

        assert_eq!(json.schema_version, 1);
        assert_eq!(json.summary.total, 1);
        assert_eq!(json.summary.errors, 0);
        assert_eq!(json.summary.warnings, 1);
        assert_eq!(json.summary.fixed, 0);
        assert!(!json.clean);

        let value = serde_json::to_value(&json).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["summary"]["total"], 1);
        assert_eq!(value["clean"], false);
        assert_eq!(value["findings"][0]["code"], "unknown-skill");
        assert_eq!(value["findings"][0]["severity"], "warning");
    }

    #[test]
    fn severity_and_fixable_are_assigned_correctly_per_code() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.skills = vec!["not-a-real-skill".to_string()];
        config.project.security_level = SecurityLevel::Strict;
        config.project.dependency_policy = DependencyPolicy::Allow;
        config.project.testing_level = TestingLevel::Light;
        config.project.stack.language = "  ".to_string();
        config.outputs = vec![OutputFormat::ClaudeMd];

        let report = doctor(dir.path(), &config, &renderer, &registry).unwrap();

        let error_fixable_codes = ["output-missing", "output-stale"];
        let warning_non_fixable_codes = ["conflicting-settings", "missing-field", "unknown-skill"];

        for finding in &report.findings {
            if error_fixable_codes.contains(&finding.code) {
                assert_eq!(
                    finding.severity,
                    Severity::Error,
                    "{} should be Severity::Error",
                    finding.code
                );
                assert!(finding.fixable, "{} should be fixable", finding.code);
            } else if warning_non_fixable_codes.contains(&finding.code) {
                assert_eq!(
                    finding.severity,
                    Severity::Warning,
                    "{} should be Severity::Warning",
                    finding.code
                );
                assert!(!finding.fixable, "{} should not be fixable", finding.code);
            } else {
                panic!("unexpected finding code: {}", finding.code);
            }
        }

        // Sanity: make sure at least the warning codes we set up actually fired.
        for code in warning_non_fixable_codes {
            assert!(
                report.findings.iter().any(|f| f.code == code),
                "expected a `{code}` finding to have fired in this scenario"
            );
        }
    }
}
