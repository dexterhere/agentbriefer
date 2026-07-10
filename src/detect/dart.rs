//! Detects a Dart/Flutter project from `pubspec.yaml`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["flutter", "get", "provider", "bloc", "flutter_bloc"];
const TESTING: &[&str] = &["test", "flutter_test", "mockito"];
const DATABASES: &[&str] = &["sqflite", "hive", "drift"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("pubspec.yaml");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&contents).ok()?;

    let mut dependency_names = Vec::new();
    for key in ["dependencies", "dev_dependencies"] {
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
        language: Some("dart".to_string()),
        package_manager: Some("pub".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("pubspec.yaml".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pubspec.yaml"),
            r#"
name: example
dependencies:
  flutter:
    sdk: flutter
  provider: ^6.0.0
  sqflite: ^2.0.0
dev_dependencies:
  flutter_test:
    sdk: flutter
  mockito: ^5.0.0
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("dart"));
        assert_eq!(detected.package_manager.as_deref(), Some("pub"));
        assert_eq!(detected.framework.as_deref(), Some("flutter"));
        assert_eq!(detected.database.as_deref(), Some("sqflite"));
        assert_eq!(
            detected.testing_tools,
            vec!["flutter_test".to_string(), "mockito".to_string()]
        );
    }

    #[test]
    fn returns_none_when_no_pubspec() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
