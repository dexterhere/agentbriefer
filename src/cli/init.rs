//! `skillforge init` — interactive wizard that writes a project's
//! `skillforge.yaml`.

use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use strum::IntoEnumIterator;

use super::prompt::select_enum;
use crate::config::{
    self, DeveloperProfile, OutputFormat, ProjectProfile, SkillforgeConfig, Stack,
};

/// Runs `skillforge init` in the current directory.
pub fn run() -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    if config_path.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!(
                "{} already exists — overwrite it?",
                config::CONFIG_FILE_NAME
            ))
            .default(false)
            .interact()?;

        if !overwrite {
            println!("Left the existing configuration untouched.");
            return Ok(());
        }
    }

    println!("Let's configure SkillForge for this project.\n");

    let (developer, extends) = pick_developer_profile()?;

    let language: String = Input::new()
        .with_prompt("Primary language")
        .interact_text()?;
    let framework = optional_input("Framework (leave blank for none)")?;
    let database = optional_input("Database (leave blank for none)")?;
    let package_manager = optional_input("Package manager (leave blank for none)")?;
    let testing_tools =
        comma_separated_input("Testing tools (comma-separated, leave blank for none)")?;

    let project = ProjectProfile {
        project_type: select_enum("Project type")?,
        stack: Stack {
            language,
            framework,
            database,
            testing_tools,
            package_manager,
        },
        security_level: select_enum("Security level")?,
        testing_level: select_enum("Testing level")?,
        dependency_policy: select_enum("Dependency policy")?,
        architecture_style: select_enum("Architecture style")?,
    };

    let stop_rules = collect_stop_rules()?;
    let outputs = select_outputs()?;

    let config = SkillforgeConfig {
        extends,
        developer,
        project,
        stop_rules,
        outputs,
    };

    config::save(&config, &config_path)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    println!(
        "\nWrote {}. Run `skillforge generate` to produce instruction files.",
        config::CONFIG_FILE_NAME
    );

    Ok(())
}

/// Asks whether to pre-fill the developer profile from a saved one (see
/// `skillforge profile create`), or answer the two questions now. Returns
/// the resulting profile and the saved profile's name, if one was used.
fn pick_developer_profile() -> Result<(DeveloperProfile, Option<String>)> {
    let dir = super::profile::profiles_dir()?;
    let names = super::profile::list_profile_names(&dir)?;

    if names.is_empty() {
        return Ok((ask_developer_profile()?, None));
    }

    let mut options = vec!["Set up manually".to_string()];
    options.extend(names.iter().cloned());

    let index = Select::new()
        .with_prompt("Use a saved developer profile?")
        .items(&options)
        .default(0)
        .interact()?;

    if index == 0 {
        Ok((ask_developer_profile()?, None))
    } else {
        let name = &names[index - 1];
        Ok((
            super::profile::load_profile(&dir, name)?,
            Some(name.clone()),
        ))
    }
}

/// Asks the two developer-profile questions directly.
fn ask_developer_profile() -> Result<DeveloperProfile> {
    Ok(DeveloperProfile {
        style: select_enum("Developer style")?,
        explanation_style: select_enum("Explanation style")?,
    })
}

/// Prompts for a value that may be left blank, returning `None` in that case.
fn optional_input(prompt: &str) -> Result<Option<String>> {
    let value: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()?;
    let trimmed = value.trim();

    Ok(if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    })
}

/// Prompts for a comma-separated list, returning an empty `Vec` if left blank.
fn comma_separated_input(prompt: &str) -> Result<Vec<String>> {
    let value: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()?;

    Ok(value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Repeatedly prompts for stop rules until the user declines to add another.
fn collect_stop_rules() -> Result<Vec<String>> {
    let mut rules = Vec::new();

    loop {
        let prompt = if rules.is_empty() {
            "Add a stop rule?"
        } else {
            "Add another stop rule?"
        };

        if !Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()?
        {
            break;
        }

        let rule: String = Input::new().with_prompt("Stop rule").interact_text()?;
        rules.push(rule);
    }

    Ok(rules)
}

/// Prompts for which output formats to generate, with every format checked
/// by default.
fn select_outputs() -> Result<Vec<OutputFormat>> {
    let variants: Vec<OutputFormat> = OutputFormat::iter().collect();
    let labels: Vec<String> = variants.iter().map(ToString::to_string).collect();
    let defaults = vec![true; variants.len()];

    let selected = MultiSelect::new()
        .with_prompt("Which instruction files should `skillforge generate` produce?")
        .items(&labels)
        .defaults(&defaults)
        .interact()?;

    Ok(selected.into_iter().map(|i| variants[i]).collect())
}
