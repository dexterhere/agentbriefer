//! Interactive prompt helpers shared across `init` and `profile create`.

use anyhow::Result;
use dialoguer::Select;
use strum::IntoEnumIterator;

/// Prompts the user to pick one variant of an enum that lists all of its
/// values via `strum`'s `IntoEnumIterator`, displaying each with its `Display`
/// impl (the same kebab-case string that gets written to YAML).
pub(super) fn select_enum<T>(prompt: &str) -> Result<T>
where
    T: IntoEnumIterator + std::fmt::Display + Copy,
{
    let variants: Vec<T> = T::iter().collect();
    let labels: Vec<String> = variants.iter().map(ToString::to_string).collect();

    let index = Select::new()
        .with_prompt(prompt)
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(variants[index])
}
