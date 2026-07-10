//! Detects a Crystal project from `shard.yml`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["kemal", "lucky", "amber"];
const TESTING: &[&str] = &["spec"];
const DATABASES: &[&str] = &["pg", "mysql", "sqlite3"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("shard.yml");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&contents).ok()?;

    let mut dependency_names = Vec::new();
    for key in ["dependencies", "development_dependencies"] {
        if let Some(mapping) = manifest.get(key).and_then(serde_yaml::Value::as_mapping) {
            dependency_names.extend(
                mapping
                    .keys()
                    .filter_map(|key| key.as_str())
                    .map(str::to_string),
            );
        }
    }

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("crystal".to_string()),
        package_manager: Some("shards".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("shard.yml".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("shard.yml"),
            r#"
name: example
version: 0.1.0

dependencies:
  kemal:
    github: kemalcr/kemal
  pg:
    github: will/crystal-pg
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("crystal"));
        assert_eq!(detected.package_manager.as_deref(), Some("shards"));
        assert_eq!(detected.framework.as_deref(), Some("kemal"));
        assert_eq!(detected.database.as_deref(), Some("pg"));
    }

    #[test]
    fn returns_none_when_no_shard_yml() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
