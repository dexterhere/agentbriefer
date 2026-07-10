//! Detects an Objective-C (CocoaPods) project from `Podfile`. Doesn't
//! handle Swift Package Manager — that's `swift.rs`'s `Package.swift`.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["afnetworking", "reactiveobjc"];
const TESTING: &[&str] = &["specta", "expecta", "ocmock"];
const DATABASES: &[&str] = &["fmdb", "realm"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("Podfile");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern = Regex::new(r#"pod\s+['"]([^'"]+)['"]"#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_ascii_lowercase())
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("objective-c".to_string()),
        package_manager: Some("cocoapods".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("Podfile".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Podfile"),
            "platform :ios, '13.0'\n\ntarget 'Example' do\n  pod 'AFNetworking'\n  pod \"Realm\"\n  pod 'OCMock'\nend\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("objective-c"));
        assert_eq!(detected.package_manager.as_deref(), Some("cocoapods"));
        assert_eq!(detected.framework.as_deref(), Some("afnetworking"));
        assert_eq!(detected.database.as_deref(), Some("realm"));
        assert_eq!(detected.testing_tools, vec!["ocmock".to_string()]);
    }

    #[test]
    fn returns_none_when_no_podfile() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
