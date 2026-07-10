//! Detects an Elixir project from `mix.exs`'s `deps` list — Elixir code,
//! but the `{:package_name, "~> version"}` tuple convention is regular
//! enough to regex-scan reliably.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["phoenix", "plug"];
const TESTING: &[&str] = &["ex_machina", "wallaby"];
const DATABASES: &[&str] = &["ecto", "ecto_sql", "postgrex", "mongodb_driver"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("mix.exs");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern = Regex::new(r"\{:([a-z_][a-zA-Z0-9_]*)\s*,").expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("elixir".to_string()),
        package_manager: Some("mix".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("mix.exs".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mix.exs"),
            r#"
defmodule Example.MixProject do
  use Mix.Project

  defp deps do
    [
      {:phoenix, "~> 1.7"},
      {:ecto_sql, "~> 3.10"},
      {:postgrex, ">= 0.0.0"},
      {:wallaby, "~> 0.30", only: :test}
    ]
  end
end
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("elixir"));
        assert_eq!(detected.package_manager.as_deref(), Some("mix"));
        assert_eq!(detected.framework.as_deref(), Some("phoenix"));
        assert_eq!(detected.database.as_deref(), Some("ecto_sql"));
        assert_eq!(detected.testing_tools, vec!["wallaby".to_string()]);
        assert!(detected.key_dependencies.contains(&"postgrex".to_string()));
    }

    #[test]
    fn returns_none_when_no_mix_exs() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
