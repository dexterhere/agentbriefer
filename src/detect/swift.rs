//! Detects a Swift Package Manager project from `Package.swift`. Doesn't
//! handle CocoaPods-based Swift projects (`Podfile`) — that manifest is
//! owned by `objective_c.rs`, since CocoaPods is overwhelmingly an
//! Objective-C convention and SwiftPM is the modern default for Swift.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["vapor", "perfect", "kitura"];
const TESTING: &[&str] = &["quick", "nimble"];
const DATABASES: &[&str] = &["grdb.swift", "sqlite.swift"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("Package.swift");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern = Regex::new(r#"\.package\(\s*url:\s*"([^"]+)""#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| package_name_from_url(m.as_str()))
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("swift".to_string()),
        package_manager: Some("swiftpm".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("Package.swift".to_string()),
    })
}

/// Reduces a package URL (e.g. `https://github.com/vapor/vapor.git`) to
/// its repository name, lowercased to match lookup-list entries.
fn package_name_from_url(url: &str) -> String {
    url.trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Package.swift"),
            r#"
// swift-tools-version:5.7
import PackageDescription

let package = Package(
    name: "Example",
    dependencies: [
        .package(url: "https://github.com/vapor/vapor.git", from: "4.0.0"),
        .package(url: "https://github.com/Quick/Nimble.git", from: "12.0.0"),
    ]
)
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("swift"));
        assert_eq!(detected.package_manager.as_deref(), Some("swiftpm"));
        assert_eq!(detected.framework.as_deref(), Some("vapor"));
        assert_eq!(detected.testing_tools, vec!["nimble".to_string()]);
    }

    #[test]
    fn returns_none_when_no_package_swift() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
