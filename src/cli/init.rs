//! `skillforge init` — interactive wizard that writes a project's
//! `skillforge.yaml`.

use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use strum::IntoEnumIterator;

use super::prompt::select_enum;
use super::ui;
use crate::config::{
    self, DeveloperProfile, OutputFormat, ProjectProfile, SkillforgeConfig, Stack,
};
use crate::detect::{self, DetectedStack};

/// Runs `skillforge init` in the current directory.
pub fn run() -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    if config_path.exists() {
        let overwrite = Confirm::with_theme(&ui::theme())
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

    ui::print_banner();
    println!("Let's configure SkillForge for this project.\n");

    let detected = detect::detect(&root);
    if let Some(source) = &detected.source {
        ui::success(&format!(
            "Detected a {} project via {source} — press Enter to accept a pre-filled answer, or type to override.",
            detected.language.as_deref().unwrap_or("unknown")
        ));
        println!();
    }

    let (developer, extends) = pick_developer_profile()?;

    let project = ask_project_profile(&detected)?;

    ui::hint("Specific situations where the agent must stop and ask before continuing.");
    let stop_rules = collect_stop_rules()?;

    ui::hint("Which instruction files `skillforge generate`/`sync` should produce.");
    let outputs = select_outputs()?;

    let mut config = SkillforgeConfig {
        extends,
        developer,
        project,
        stop_rules,
        outputs,
    };

    review_and_edit(&mut config)?;

    config::save(&config, &config_path)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    ui::success(&format!(
        "\nWrote {}. Run `skillforge generate` to produce instruction files.",
        config::CONFIG_FILE_NAME
    ));

    Ok(())
}

/// Asks every stack/project question, pre-filling from `detected` wherever
/// a value was found (an empty/absent detected value is a harmless no-op
/// pre-fill).
fn ask_project_profile(detected: &DetectedStack) -> Result<ProjectProfile> {
    ui::hint("The language your project is written in.");
    let language: String = Input::with_theme(&ui::theme())
        .with_prompt("Primary language")
        .with_initial_text(detected.language.clone().unwrap_or_default())
        .interact_text()?;

    ui::hint("Your framework, if any — leave blank if there isn't one.");
    let framework = optional_input("Framework", detected.framework.as_deref())?;

    ui::hint("Your database, if any — leave blank if there isn't one.");
    let database = optional_input("Database", detected.database.as_deref())?;

    ui::hint("How you manage dependencies (npm, cargo, pip, ...) — leave blank if unsure.");
    let package_manager = optional_input("Package manager", detected.package_manager.as_deref())?;

    ui::hint("Testing tools you use, comma-separated — leave blank if none.");
    let testing_tools =
        comma_separated_input("Testing tools", Some(&detected.testing_tools.join(", ")))?;

    ui::hint(
        "Key dependencies worth the agent knowing about, comma-separated — leave blank if none.",
    );
    let key_dependencies = comma_separated_input(
        "Key dependencies",
        Some(&detected.key_dependencies.join(", ")),
    )?;

    ui::hint("What kind of project this is — shapes which architecture advice applies.");
    let project_type = select_enum("Project type", None)?;

    ui::hint(
        "Controls how cautious the agent is about auth, secrets, validation, and permission changes.",
    );
    let security_level = select_enum("Security level", None)?;

    ui::hint("How much test coverage the agent should add or expect for changes.");
    let testing_level = select_enum("Testing level", None)?;

    ui::hint(
        "Whether the agent can add a new dependency freely, should explain first, or must ask first.",
    );
    let dependency_policy = select_enum("Dependency policy", None)?;

    ui::hint("How much structure/layering the agent should introduce or preserve.");
    let architecture_style = select_enum("Architecture style", None)?;

    Ok(ProjectProfile {
        project_type,
        stack: Stack {
            language,
            framework,
            database,
            testing_tools,
            package_manager,
            key_dependencies,
        },
        security_level,
        testing_level,
        dependency_policy,
        architecture_style,
    })
}

/// Shows a menu of every field's current value plus a "Save and finish"
/// entry; picking a field re-runs that field's own prompt — pre-filled
/// with its *current* value, not blank — and loops back here, so any
/// earlier answer can be changed before saving.
fn review_and_edit(config: &mut SkillforgeConfig) -> Result<()> {
    loop {
        let items = menu_items(config);

        let index = Select::with_theme(&ui::theme())
            .with_prompt("Review your configuration — pick a field to change it, or save")
            .items(&items)
            .default(0)
            .interact()?;

        match index {
            0 => break,
            1 => {
                config.developer.style =
                    select_enum("Developer style", Some(config.developer.style))?
            }
            2 => {
                config.developer.explanation_style = select_enum(
                    "Explanation style",
                    Some(config.developer.explanation_style),
                )?
            }
            3 => {
                config.project.project_type =
                    select_enum("Project type", Some(config.project.project_type))?
            }
            4 => {
                config.project.stack.language = Input::with_theme(&ui::theme())
                    .with_prompt("Primary language")
                    .with_initial_text(&config.project.stack.language)
                    .interact_text()?
            }
            5 => {
                config.project.stack.framework =
                    optional_input("Framework", config.project.stack.framework.as_deref())?
            }
            6 => {
                config.project.stack.database =
                    optional_input("Database", config.project.stack.database.as_deref())?
            }
            7 => {
                config.project.stack.package_manager = optional_input(
                    "Package manager",
                    config.project.stack.package_manager.as_deref(),
                )?
            }
            8 => {
                config.project.stack.testing_tools = comma_separated_input(
                    "Testing tools",
                    Some(&config.project.stack.testing_tools.join(", ")),
                )?
            }
            9 => {
                config.project.stack.key_dependencies = comma_separated_input(
                    "Key dependencies",
                    Some(&config.project.stack.key_dependencies.join(", ")),
                )?
            }
            10 => {
                config.project.security_level =
                    select_enum("Security level", Some(config.project.security_level))?
            }
            11 => {
                config.project.testing_level =
                    select_enum("Testing level", Some(config.project.testing_level))?
            }
            12 => {
                config.project.dependency_policy =
                    select_enum("Dependency policy", Some(config.project.dependency_policy))?
            }
            13 => {
                config.project.architecture_style = select_enum(
                    "Architecture style",
                    Some(config.project.architecture_style),
                )?
            }
            14 => config.stop_rules = collect_stop_rules()?,
            15 => config.outputs = select_outputs()?,
            _ => unreachable!("menu_items and this match must stay in sync"),
        }
    }

    Ok(())
}

/// Builds the review menu's labels from `config`'s current values. Pure
/// (no I/O) so it's unit-testable without a terminal.
fn menu_items(config: &SkillforgeConfig) -> Vec<String> {
    let stack = &config.project.stack;

    vec![
        "Save and finish".to_string(),
        format!("Developer style: {}", config.developer.style),
        format!("Explanation style: {}", config.developer.explanation_style),
        format!("Project type: {}", config.project.project_type),
        format!("Language: {}", stack.language),
        format!(
            "Framework: {}",
            stack.framework.as_deref().unwrap_or("(none)")
        ),
        format!(
            "Database: {}",
            stack.database.as_deref().unwrap_or("(none)")
        ),
        format!(
            "Package manager: {}",
            stack.package_manager.as_deref().unwrap_or("(none)")
        ),
        format!(
            "Testing tools: {}",
            if stack.testing_tools.is_empty() {
                "(none)".to_string()
            } else {
                stack.testing_tools.join(", ")
            }
        ),
        format!(
            "Key dependencies: {}",
            if stack.key_dependencies.is_empty() {
                "(none)".to_string()
            } else {
                stack.key_dependencies.join(", ")
            }
        ),
        format!("Security level: {}", config.project.security_level),
        format!("Testing level: {}", config.project.testing_level),
        format!("Dependency policy: {}", config.project.dependency_policy),
        format!("Architecture style: {}", config.project.architecture_style),
        format!("Stop rules: {} configured", config.stop_rules.len()),
        format!("Output formats: {} selected", config.outputs.len()),
    ]
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

    let index = Select::with_theme(&ui::theme())
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
    ui::hint(
        "Defines your working style and preferences — makes the agent's behavior personal, not generic.",
    );
    let style = select_enum("Developer style", None)?;

    ui::hint("How much detail/reasoning the agent should include when explaining what it did.");
    let explanation_style = select_enum("Explanation style", None)?;

    Ok(DeveloperProfile {
        style,
        explanation_style,
    })
}

/// Prompts for a value that may be left blank, returning `None` in that case.
/// `initial`, if given, pre-fills the editable input buffer.
fn optional_input(prompt: &str, initial: Option<&str>) -> Result<Option<String>> {
    let value: String = Input::with_theme(&ui::theme())
        .with_prompt(prompt)
        .with_initial_text(initial.unwrap_or(""))
        .allow_empty(true)
        .interact_text()?;
    let trimmed = value.trim();

    Ok(if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    })
}

/// Prompts for a comma-separated list, returning an empty `Vec` if left
/// blank. `initial`, if given, pre-fills the editable input buffer.
fn comma_separated_input(prompt: &str, initial: Option<&str>) -> Result<Vec<String>> {
    let value: String = Input::with_theme(&ui::theme())
        .with_prompt(prompt)
        .with_initial_text(initial.unwrap_or(""))
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

        if !Confirm::with_theme(&ui::theme())
            .with_prompt(prompt)
            .default(false)
            .interact()?
        {
            break;
        }

        let rule: String = Input::with_theme(&ui::theme())
            .with_prompt("Stop rule")
            .interact_text()?;
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

    let selected = MultiSelect::with_theme(&ui::theme())
        .with_prompt("Which instruction files should `skillforge generate` produce?")
        .items(&labels)
        .defaults(&defaults)
        .interact()?;

    Ok(selected.into_iter().map(|i| variants[i]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ArchitectureStyle, DependencyPolicy, DeveloperStyle, ExplanationStyle, ProjectType,
        SecurityLevel, TestingLevel,
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
                    package_manager: None,
                    key_dependencies: vec![],
                },
                security_level: SecurityLevel::Standard,
                testing_level: TestingLevel::Practical,
                dependency_policy: DependencyPolicy::ExplainFirst,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec![],
            outputs: vec![OutputFormat::ClaudeMd],
        }
    }

    #[test]
    fn menu_items_has_one_entry_per_field_plus_save() {
        let items = menu_items(&sample_config());

        assert_eq!(items.len(), 16);
        assert_eq!(items[0], "Save and finish");
    }

    #[test]
    fn menu_items_shows_placeholder_text_for_unset_optional_fields() {
        let items = menu_items(&sample_config());

        assert!(items.iter().any(|item| item == "Framework: (none)"));
        assert!(items.iter().any(|item| item == "Database: (none)"));
        assert!(items.iter().any(|item| item == "Package manager: (none)"));
        assert!(items.iter().any(|item| item == "Testing tools: (none)"));
        assert!(items.iter().any(|item| item == "Key dependencies: (none)"));
    }

    #[test]
    fn menu_items_shows_actual_values_when_set() {
        let mut config = sample_config();
        config.project.stack.framework = Some("axum".to_string());
        config.project.stack.testing_tools = vec!["cargo-test".to_string(), "insta".to_string()];
        config.project.stack.key_dependencies = vec!["serde".to_string(), "tokio".to_string()];
        config.stop_rules = vec!["Stop before touching CI".to_string()];

        let items = menu_items(&config);

        assert!(items.iter().any(|item| item == "Framework: axum"));
        assert!(
            items
                .iter()
                .any(|item| item == "Testing tools: cargo-test, insta")
        );
        assert!(
            items
                .iter()
                .any(|item| item == "Key dependencies: serde, tokio")
        );
        assert!(items.iter().any(|item| item == "Stop rules: 1 configured"));
        assert!(
            items
                .iter()
                .any(|item| item == "Output formats: 1 selected")
        );
    }
}
