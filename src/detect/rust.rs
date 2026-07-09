//! Detects a Rust project from `Cargo.toml`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &[
    "axum",
    "actix-web",
    "rocket",
    "warp",
    "tauri",
    "leptos",
    "yew",
    "bevy",
];
const TESTING: &[&str] = &["insta", "proptest", "mockall", "rstest", "criterion"];
const DATABASES: &[&str] = &["sqlx", "diesel", "sea-orm", "mongodb", "redis", "rusqlite"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("Cargo.toml");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: toml::Value = toml::from_str(&contents).ok()?;

    let mut dependency_names = Vec::new();
    for table_name in ["dependencies", "dev-dependencies"] {
        if let Some(table) = manifest.get(table_name).and_then(toml::Value::as_table) {
            dependency_names.extend(table.keys().cloned());
        }
    }

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("rust".to_string()),
        package_manager: Some("cargo".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("Cargo.toml".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_package_manager_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[package]
name = "example"
version = "0.1.0"

[dependencies]
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
sqlx = "0.7"

[dev-dependencies]
insta = "1.48"
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("rust"));
        assert_eq!(detected.package_manager.as_deref(), Some("cargo"));
        assert_eq!(detected.framework.as_deref(), Some("axum"));
        assert_eq!(detected.database.as_deref(), Some("sqlx"));
        assert_eq!(detected.testing_tools, vec!["insta".to_string()]);
        assert_eq!(detected.key_dependencies, vec!["serde".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("Cargo.toml"));
    }

    #[test]
    fn returns_none_when_no_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
