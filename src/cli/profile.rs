//! `skillforge profile` — manage reusable developer profiles under
//! `~/.config/skillforge/profiles/`.
//!
//! Profiles are a pre-fill source, not a live-resolved reference: a
//! project's `skillforge.yaml` always holds a concrete `developer:` section.
//! `extends: <name>` is written alongside it only as a record of which
//! saved profile it came from — nothing re-reads it at `generate`/`sync`
//! time.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use dialoguer::{Confirm, Input, Select};

use super::prompt::select_enum;
use super::ui;
use crate::config::{self, DeveloperProfile, SkillforgeConfig};

/// Resolves `~/.config/skillforge/profiles` (or the platform equivalent).
pub(super) fn profiles_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "skillforge")
        .context("could not determine the user's config directory")?;

    Ok(dirs.config_dir().join("profiles"))
}

/// Validates a profile name and returns the YAML file it maps to under `dir`.
fn profile_path(dir: &Path, name: &str) -> Result<PathBuf> {
    if name.trim().is_empty() {
        bail!("profile name cannot be empty");
    }
    if name.contains('/') || name.contains('\\') {
        bail!("profile name cannot contain '/' or '\\' — use a simple name");
    }

    Ok(dir.join(format!("{name}.yaml")))
}

/// Lists the names of saved profiles (file stems) under `dir`, sorted.
/// Returns an empty list if `dir` doesn't exist yet.
pub(super) fn list_profile_names(dir: &Path) -> Result<Vec<String>> {
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

/// Loads the saved profile named `name` from `dir`.
pub(super) fn load_profile(dir: &Path, name: &str) -> Result<DeveloperProfile> {
    config::load(&profile_path(dir, name)?)
        .with_context(|| format!("failed to load profile '{name}'"))
}

/// Runs `skillforge profile list`.
pub fn run_list() -> Result<()> {
    let names = list_profile_names(&profiles_dir()?)?;

    if names.is_empty() {
        ui::hint("No saved profiles yet — run `skillforge profile create` to add one.");
    } else {
        for name in names {
            println!("{name}");
        }
    }

    Ok(())
}

/// Runs `skillforge profile create`.
pub fn run_create() -> Result<()> {
    let dir = profiles_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    let name: String = Input::with_theme(&ui::theme())
        .with_prompt("Profile name")
        .interact_text()?;
    let path = profile_path(&dir, &name)?;

    if path.exists() {
        let overwrite = Confirm::with_theme(&ui::theme())
            .with_prompt(format!("Profile '{name}' already exists — overwrite it?"))
            .default(false)
            .interact()?;

        if !overwrite {
            println!("Left the existing profile untouched.");
            return Ok(());
        }
    }

    ui::hint(
        "Defines your working style and preferences — makes the agent's behavior personal, not generic.",
    );
    let style = select_enum("Developer style", None)?;

    ui::hint("How much detail/reasoning the agent should include when explaining what it did.");
    let explanation_style = select_enum("Explanation style", None)?;

    let profile = DeveloperProfile {
        style,
        explanation_style,
    };

    config::save(&profile, &path).with_context(|| format!("failed to write {}", path.display()))?;

    ui::success(&format!("Saved profile '{name}'."));

    Ok(())
}

/// Runs `skillforge profile switch` — applies a saved profile's developer
/// style to the current project's `skillforge.yaml`.
pub fn run_switch() -> Result<()> {
    let dir = profiles_dir()?;
    let names = list_profile_names(&dir)?;

    if names.is_empty() {
        bail!("no saved profiles yet — run `skillforge profile create` first");
    }

    let index = Select::with_theme(&ui::theme())
        .with_prompt("Switch to which profile?")
        .items(&names)
        .default(0)
        .interact()?;
    let name = &names[index];
    let developer = load_profile(&dir, name)?;

    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let mut project_config: SkillforgeConfig = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `skillforge init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    project_config.developer = developer;
    project_config.extends = Some(name.clone());

    config::save(&project_config, &config_path)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    ui::success(&format!("Switched developer profile to '{name}'."));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DeveloperStyle, ExplanationStyle};

    #[test]
    fn profile_path_rejects_empty_and_path_like_names() {
        let dir = tempfile::tempdir().unwrap();

        assert!(profile_path(dir.path(), "").is_err());
        assert!(profile_path(dir.path(), "  ").is_err());
        assert!(profile_path(dir.path(), "team/rust").is_err());
        assert!(profile_path(dir.path(), "team\\rust").is_err());
    }

    #[test]
    fn profile_path_accepts_a_simple_name() {
        let dir = tempfile::tempdir().unwrap();

        let path = profile_path(dir.path(), "security-first").unwrap();

        assert_eq!(path, dir.path().join("security-first.yaml"));
    }

    #[test]
    fn saved_profile_round_trips_through_list_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let profile = DeveloperProfile {
            style: DeveloperStyle::SecurityFirst,
            explanation_style: ExplanationStyle::Detailed,
        };

        config::save(
            &profile,
            &profile_path(dir.path(), "security-first").unwrap(),
        )
        .unwrap();

        assert_eq!(
            list_profile_names(dir.path()).unwrap(),
            vec!["security-first".to_string()]
        );
        assert_eq!(load_profile(dir.path(), "security-first").unwrap(), profile);
    }

    #[test]
    fn list_profile_names_is_empty_when_the_directory_does_not_exist_yet() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");

        assert!(list_profile_names(&missing).unwrap().is_empty());
    }
}
