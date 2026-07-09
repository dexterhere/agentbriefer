//! Detects a Julia project from `Project.toml`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["Genie", "Dash", "Oxygen"];
const TESTING: &[&str] = &["Test"];
const DATABASES: &[&str] = &["LibPQ", "MySQL", "SQLite"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("Project.toml");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: toml::Value = toml::from_str(&contents).ok()?;

    let dependency_names: Vec<String> = manifest
        .get("deps")
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("julia".to_string()),
        package_manager: Some("Pkg".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("Project.toml".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Project.toml"),
            r#"
name = "Example"
uuid = "00000000-0000-0000-0000-000000000000"

[deps]
Genie = "c43c736e-a2d1-11e8-161f-af95117fbd1e"
LibPQ = "194296ae-ab2e-5f79-8cd4-7183deed42ae"
Test = "8dfed614-e22c-5e08-85e1-65c5234f0b40"
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("julia"));
        assert_eq!(detected.package_manager.as_deref(), Some("Pkg"));
        assert_eq!(detected.framework.as_deref(), Some("Genie"));
        assert_eq!(detected.database.as_deref(), Some("LibPQ"));
        assert_eq!(detected.testing_tools, vec!["Test".to_string()]);
    }

    #[test]
    fn returns_none_when_no_project_toml() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
