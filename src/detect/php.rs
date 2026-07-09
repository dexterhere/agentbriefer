//! Detects a PHP project from `composer.json`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &[
    "laravel/framework",
    "symfony/framework-bundle",
    "slim/slim",
    "cakephp/cakephp",
    "laminas/laminas-mvc",
];
const TESTING: &[&str] = &["phpunit/phpunit", "pestphp/pest", "mockery/mockery"];
const DATABASES: &[&str] = &["doctrine/orm", "illuminate/database"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("composer.json");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let mut dependency_names = Vec::new();
    for key in ["require", "require-dev"] {
        if let Some(object) = manifest.get(key).and_then(serde_json::Value::as_object) {
            dependency_names.extend(
                object
                    .keys()
                    .filter(|name| name.as_str() != "php" && !name.starts_with("ext-"))
                    .cloned(),
            );
        }
    }

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("php".to_string()),
        package_manager: Some("composer".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("composer.json".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("composer.json"),
            r#"{
                "require": {"php": "^8.2", "laravel/framework": "^10.0", "doctrine/orm": "^2.16"},
                "require-dev": {"phpunit/phpunit": "^10.0"}
            }"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("php"));
        assert_eq!(detected.package_manager.as_deref(), Some("composer"));
        assert_eq!(detected.framework.as_deref(), Some("laravel/framework"));
        assert_eq!(detected.database.as_deref(), Some("doctrine/orm"));
        assert_eq!(detected.testing_tools, vec!["phpunit/phpunit".to_string()]);
    }

    #[test]
    fn returns_none_when_no_composer_json() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
