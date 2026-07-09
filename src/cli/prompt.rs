//! Interactive prompt helpers shared across `init` and `profile create`.

use anyhow::Result;
use dialoguer::Select;
use strum::IntoEnumIterator;

use super::ui;

/// Prompts the user to pick one variant of an enum that lists all of its
/// values via `strum`'s `IntoEnumIterator`, displaying each with its `Display`
/// impl (the same kebab-case string that gets written to YAML). `current`,
/// if given, becomes the initially-highlighted item instead of the first one
/// — used when re-editing an already-answered question.
pub(super) fn select_enum<T>(prompt: &str, current: Option<T>) -> Result<T>
where
    T: IntoEnumIterator + std::fmt::Display + Copy + PartialEq,
{
    let variants: Vec<T> = T::iter().collect();
    let labels: Vec<String> = variants.iter().map(ToString::to_string).collect();
    let default_index = current
        .and_then(|value| variants.iter().position(|v| *v == value))
        .unwrap_or(0);

    let index = Select::with_theme(&ui::theme())
        .with_prompt(prompt)
        .items(&labels)
        .default(default_index)
        .interact()?;

    Ok(variants[index])
}
