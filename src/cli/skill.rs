//! `agentbriefer skill` — manage this project's installed skills, and
//! reusable named skill profiles under
//! `~/.config/agentbriefer/skill-profiles/`.
//!
//! The embedded catalog itself (`skills::SkillRegistry`) is pure and lives
//! outside `cli`; this module owns everything that touches the real
//! filesystem — a project's `.agentbriefer/skills/` materialization and the
//! user's real `~/.config` directory — mirroring how `cli::profile` owns
//! real I/O for developer profiles while `config`/`render`/`skills` stay
//! pure.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use super::generate::{OutputReport, refuse_if_symlink};
use super::sync;
use super::ui;
use crate::config::{self, AgentbrieferConfig};
use crate::detect;
use crate::render::Renderer;
use crate::skills::{Skill, SkillRegistry};

/// A named, reusable set of skill ids under
/// `~/.config/agentbriefer/skill-profiles/<name>.yaml` — a snapshot of a
/// project's `config.skills` at the time it was saved, not a live
/// reference (same "record, don't resolve" model as `cli::profile`'s
/// developer profiles).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SkillProfile {
    skills: Vec<String>,
}

/// Resolves `~/.config/agentbriefer/skill-profiles` (or the platform
/// equivalent) — deliberately a separate directory from
/// `cli::profile::profiles_dir()`, which stores unrelated `DeveloperProfile`
/// data.
fn skill_profiles_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "agentbriefer")
        .context("could not determine the user's config directory")?;

    Ok(dirs.config_dir().join("skill-profiles"))
}

/// Validates a skill-profile name and returns the YAML file it maps to
/// under `dir` — same validation as `cli::profile::profile_path`.
fn skill_profile_path(dir: &Path, name: &str) -> Result<PathBuf> {
    if name.trim().is_empty() {
        bail!("skill profile name cannot be empty");
    }
    if name.contains('/') || name.contains('\\') {
        bail!("skill profile name cannot contain '/' or '\\' — use a simple name");
    }

    Ok(dir.join(format!("{name}.yaml")))
}

/// Lists the names of saved skill profiles (file stems) under `dir`,
/// sorted. Returns an empty list if `dir` doesn't exist yet.
fn list_skill_profile_names(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("yaml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            names.push(stem.to_string());
        }
    }
    names.sort();

    Ok(names)
}

fn load_skill_profile(dir: &Path, name: &str) -> Result<SkillProfile> {
    config::load(&skill_profile_path(dir, name)?)
        .with_context(|| format!("failed to load skill profile '{name}'"))
}

/// Loads the current project's config if one exists, purely to annotate
/// which listed skills are already installed. Returns `None` rather than
/// erroring when there isn't one yet — browsing the catalog shouldn't
/// require a project to already be configured.
fn try_load_config(root: &Path) -> Option<AgentbrieferConfig> {
    config::load(&root.join(config::CONFIG_FILE_NAME)).ok()
}

/// Prints a `generate`/`sync`-style per-format success/failure report.
fn report_sync(report: &OutputReport) {
    for (format, path) in &report.succeeded {
        ui::success(&format!("synced {format} -> {}", path.display()));
    }
    for (format, reason) in &report.failed {
        ui::warn(&format!("skipped {format}: {reason}"));
    }
}

/// Regenerates `.agentbriefer/skills/<id>/SKILL.md` for every id in
/// `config.skills`, against `registry`. Removes any stale `<id>/` directory
/// under `.agentbriefer/skills/` whose id is no longer in `config.skills` —
/// a full resync (not additive), consistent with how the four generated
/// docs are already fully machine-owned outside their managed block. An id
/// with no match in `registry` is silently skipped — `doctor` is the layer
/// responsible for surfacing that as a finding. Hand-edits inside a
/// materialized `SKILL.md` do not survive this and are not a supported
/// customization point, hence the header comment written into the file.
fn materialize_skills(
    root: &Path,
    config: &AgentbrieferConfig,
    registry: &SkillRegistry,
) -> Result<()> {
    let skills_dir = root.join(".agentbriefer").join("skills");
    let wanted: HashSet<&str> = config.skills.iter().map(String::as_str).collect();

    if skills_dir.exists() {
        for entry in fs::read_dir(&skills_dir)
            .with_context(|| format!("failed to read {}", skills_dir.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if !wanted.contains(name.as_str()) {
                fs::remove_dir_all(entry.path())
                    .with_context(|| format!("failed to remove {}", entry.path().display()))?;
            }
        }
    }

    for id in &config.skills {
        let Some(skill) = registry.get(id) else {
            continue;
        };

        let dir = skills_dir.join(id);
        fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

        let path = dir.join("SKILL.md");
        refuse_if_symlink(&path)?;

        let frontmatter =
            serde_yaml::to_string(skill).context("failed to serialize skill frontmatter")?;
        let content = format!(
            "<!-- do not edit — regenerated by `agentbriefer skill update` -->\n---\n{frontmatter}---\n{}\n",
            skill.body
        );

        fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    }

    Ok(())
}

/// Returns an error if `id` isn't a real, bundled skill. Kept separate from
/// [`run_add`] so it's unit-testable without touching the real filesystem
/// or the process's current directory.
fn validate_known_skill(id: &str, registry: &SkillRegistry) -> Result<()> {
    if !registry.contains(id) {
        bail!(
            "'{id}' is not a known skill — run `agentbriefer skill list` to see what's available"
        );
    }

    Ok(())
}

/// Runs `agentbriefer skill list`. `role`/`recommended` are personal,
/// session-only filters — this only ever *reads* `agentbriefer.yaml` (to
/// annotate which listed skills are already installed); it never writes
/// it. Two teammates browsing under different `--role` values still see
/// the identical "installed" annotations, since installation state is
/// shared — only the view is personal.
pub fn run_list(role: Option<String>, recommended: bool) -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;
    let installed = try_load_config(&root).map(|c| c.skills).unwrap_or_default();

    let detected = recommended.then(|| detect::detect(&root));
    let recommended_ids: Option<HashSet<&str>> = detected.as_ref().map(|detected| {
        registry
            .recommended_for(detected)
            .map(|s| s.id.as_str())
            .collect()
    });

    let candidates: Vec<&Skill> = registry
        .all()
        .iter()
        .filter(|s| {
            role.as_deref()
                .is_none_or(|role| s.roles.iter().any(|r| r.eq_ignore_ascii_case(role)))
        })
        .filter(|s| {
            recommended_ids
                .as_ref()
                .is_none_or(|ids| ids.contains(s.id.as_str()))
        })
        .collect();

    if candidates.is_empty() {
        ui::hint("No skills match this filter.");
        return Ok(());
    }

    for skill in candidates {
        let suffix = if installed.iter().any(|id| id == &skill.id) {
            " [installed]"
        } else {
            ""
        };
        println!("{} ({}){suffix}", skill.id, skill.category);
        println!("    {}", skill.description);
    }

    Ok(())
}

/// Runs `agentbriefer skill add <id>`. Installing an unknown id is always a
/// mistake (a typo), so unlike [`run_remove`] this is a hard error.
pub fn run_add(id: &str) -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let mut config: AgentbrieferConfig = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `agentbriefer init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;
    validate_known_skill(id, &registry)?;

    if config.skills.iter().any(|existing| existing == id) {
        ui::hint(&format!("'{id}' is already installed."));
        return Ok(());
    }

    config.skills.push(id.to_string());
    config::save(&config, &config_path)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    materialize_skills(&root, &config, &registry)?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;
    let report = sync::sync(&root, &config, &renderer)?;
    report_sync(&report);

    ui::success(&format!("Added skill '{id}'."));

    Ok(())
}

/// Runs `agentbriefer skill remove <id>`. Removing an id that's already
/// absent is a no-op the user probably intended (e.g. re-running the
/// command), so this warns rather than erroring.
pub fn run_remove(id: &str) -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let mut config: AgentbrieferConfig = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `agentbriefer init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let original_len = config.skills.len();
    config.skills.retain(|existing| existing != id);

    if config.skills.len() == original_len {
        ui::warn(&format!("'{id}' was not installed — nothing to remove."));
        return Ok(());
    }

    config::save(&config, &config_path)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;
    materialize_skills(&root, &config, &registry)?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;
    let report = sync::sync(&root, &config, &renderer)?;
    report_sync(&report);

    ui::success(&format!("Removed skill '{id}'."));

    Ok(())
}

/// Runs `agentbriefer skill update` — re-renders the currently-configured
/// skills against whatever's bundled in the installed CLI. Never touches
/// `config.skills` itself: "updating" a skill's content means upgrading
/// the CLI, not fetching anything.
pub fn run_update() -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let config: AgentbrieferConfig = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `agentbriefer init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;
    materialize_skills(&root, &config, &registry)?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;
    let report = sync::sync(&root, &config, &renderer)?;
    report_sync(&report);

    ui::success("Updated installed skills against the current catalog.");

    Ok(())
}

/// Runs `agentbriefer skill info <id>`.
pub fn run_info(id: &str) -> Result<()> {
    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;

    let skill = registry.get(id).with_context(|| {
        format!(
            "'{id}' is not a known skill — run `agentbriefer skill list` to see what's available"
        )
    })?;

    println!("{} ({})", skill.name, skill.id);
    println!("{}", skill.description);
    println!();
    println!("Category: {}", skill.category);
    if !skill.roles.is_empty() {
        println!("Roles: {}", skill.roles.join(", "));
    }
    if !skill.compatible_stacks.is_empty() {
        println!("Compatible stacks: {}", skill.compatible_stacks.join(", "));
    }
    println!();
    println!("{}", skill.body);

    Ok(())
}

/// Runs `agentbriefer skill profile list`.
pub fn run_profile_list() -> Result<()> {
    let names = list_skill_profile_names(&skill_profiles_dir()?)?;

    if names.is_empty() {
        ui::hint(
            "No saved skill profiles yet — run `agentbriefer skill profile create <name>` to add one.",
        );
    } else {
        for name in names {
            println!("{name}");
        }
    }

    Ok(())
}

/// Runs `agentbriefer skill profile create <name>` — snapshots the current
/// project's installed skill set under a reusable name. Non-interactive by
/// design: "a reusable set of skills" is naturally "save what's installed
/// here", not a wizard.
pub fn run_profile_create(name: &str) -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let config: AgentbrieferConfig = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `agentbriefer init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let dir = skill_profiles_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = skill_profile_path(&dir, name)?;

    let profile = SkillProfile {
        skills: config.skills.clone(),
    };
    config::save(&profile, &path).with_context(|| format!("failed to write {}", path.display()))?;

    ui::success(&format!(
        "Saved skill profile '{name}' with {} skill(s).",
        profile.skills.len()
    ));

    Ok(())
}

/// Runs `agentbriefer skill profile apply <name>` — replaces the current
/// project's installed skill set with a saved profile's (full replace,
/// mirroring `cli::profile::run_switch`'s full-replace pattern).
pub fn run_profile_apply(name: &str) -> Result<()> {
    let dir = skill_profiles_dir()?;
    let profile = load_skill_profile(&dir, name)?;

    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let mut config: AgentbrieferConfig = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `agentbriefer init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    config.skills = profile.skills;
    config::save(&config, &config_path)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    let registry = SkillRegistry::load().context("failed to load the bundled skill catalog")?;
    materialize_skills(&root, &config, &registry)?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;
    let report = sync::sync(&root, &config, &renderer)?;
    report_sync(&report);

    ui::success(&format!("Applied skill profile '{name}'."));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle, ExplanationStyle,
        OutputFormat, ProjectProfile, ProjectType, SecurityLevel, Stack, TestingLevel,
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
    fn skill_profile_path_rejects_empty_and_path_like_names() {
        let dir = tempfile::tempdir().unwrap();

        assert!(skill_profile_path(dir.path(), "").is_err());
        assert!(skill_profile_path(dir.path(), "  ").is_err());
        assert!(skill_profile_path(dir.path(), "team/rust").is_err());
        assert!(skill_profile_path(dir.path(), "team\\rust").is_err());
    }

    #[test]
    fn skill_profile_path_accepts_a_simple_name() {
        let dir = tempfile::tempdir().unwrap();

        let path = skill_profile_path(dir.path(), "frontend-basics").unwrap();

        assert_eq!(path, dir.path().join("frontend-basics.yaml"));
    }

    #[test]
    fn saved_skill_profile_round_trips_through_list_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let profile = SkillProfile {
            skills: vec!["server-components-by-default".to_string()],
        };

        config::save(
            &profile,
            &skill_profile_path(dir.path(), "frontend-basics").unwrap(),
        )
        .unwrap();

        assert_eq!(
            list_skill_profile_names(dir.path()).unwrap(),
            vec!["frontend-basics".to_string()]
        );
        assert_eq!(
            load_skill_profile(dir.path(), "frontend-basics").unwrap(),
            profile
        );
    }

    #[test]
    fn list_skill_profile_names_is_empty_when_the_directory_does_not_exist_yet() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");

        assert!(list_skill_profile_names(&missing).unwrap().is_empty());
    }

    #[test]
    fn validate_known_skill_rejects_an_unknown_id() {
        let registry = SkillRegistry::load().unwrap();

        assert!(validate_known_skill("not-a-real-skill", &registry).is_err());
        assert!(validate_known_skill("server-components-by-default", &registry).is_ok());
    }

    #[test]
    fn materialize_skills_writes_one_file_per_installed_id() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.skills = vec![
            "server-components-by-default".to_string(),
            "no-secrets-in-repo".to_string(),
        ];

        materialize_skills(dir.path(), &config, &registry).unwrap();

        for id in &config.skills {
            let path = dir
                .path()
                .join(".agentbriefer")
                .join("skills")
                .join(id)
                .join("SKILL.md");
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("{path:?} should have been written"));
            assert!(content.contains("do not edit"));
            assert!(content.contains(id));
        }
    }

    #[test]
    fn materialize_skills_removes_stale_directories_for_uninstalled_ids() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.skills = vec!["server-components-by-default".to_string()];
        materialize_skills(dir.path(), &config, &registry).unwrap();

        config.skills = vec!["no-secrets-in-repo".to_string()];
        materialize_skills(dir.path(), &config, &registry).unwrap();

        let stale = dir
            .path()
            .join(".agentbriefer")
            .join("skills")
            .join("server-components-by-default");
        assert!(!stale.exists());

        let kept = dir
            .path()
            .join(".agentbriefer")
            .join("skills")
            .join("no-secrets-in-repo");
        assert!(kept.exists());
    }

    #[test]
    fn materialize_skills_silently_skips_an_unknown_id() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::load().unwrap();
        let mut config = sample_config();
        config.skills = vec!["not-a-real-skill".to_string()];

        materialize_skills(dir.path(), &config, &registry).unwrap();

        let path = dir
            .path()
            .join(".agentbriefer")
            .join("skills")
            .join("not-a-real-skill");
        assert!(!path.exists());
    }
}
