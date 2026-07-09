//! Detects a Nim project from a `*.nimble` file's `requires` lines.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize, find_file_with_extension};

const FRAMEWORKS: &[&str] = &["jester", "prologue", "karax"];
const TESTING: &[&str] = &["unittest2", "balls"];
const DATABASES: &[&str] = &["norm", "ormin"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = find_file_with_extension(root, "nimble")?;
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern = Regex::new(r#"requires\s+"([^"]+)""#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| package_name_from_spec(m.as_str()))
        .filter(|name| name != "nim")
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("nim".to_string()),
        package_manager: Some("nimble".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: manifest_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
    })
}

/// A `requires` spec is usually `name >= x.y.z` (or just `name`), but can
/// also be a bare URL (`requires "https://github.com/user/pkg"`) — in that
/// case, use the last path segment as the name, same trick `swift.rs` uses
/// for package URLs.
fn package_name_from_spec(spec: &str) -> String {
    let first_token = spec.split_whitespace().next().unwrap_or(spec);

    if first_token.starts_with("http") {
        first_token
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .unwrap_or(first_token)
            .to_string()
    } else {
        first_token.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("example.nimble"),
            "version       = \"0.1.0\"\nauthor        = \"me\"\n\nrequires \"nim >= 1.6.0\"\nrequires \"jester >= 0.5.0\"\nrequires \"norm\"\nrequires \"balls\"\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("nim"));
        assert_eq!(detected.package_manager.as_deref(), Some("nimble"));
        assert_eq!(detected.framework.as_deref(), Some("jester"));
        assert_eq!(detected.database.as_deref(), Some("norm"));
        assert_eq!(detected.testing_tools, vec!["balls".to_string()]);
        assert!(!detected.key_dependencies.iter().any(|d| d == "nim"));
        assert_eq!(detected.source.as_deref(), Some("example.nimble"));
    }

    #[test]
    fn extracts_package_name_from_a_url_spec() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("example.nimble"),
            "requires \"https://github.com/user/pkg.git\"\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert!(detected.key_dependencies.contains(&"pkg".to_string()));
    }

    #[test]
    fn returns_none_when_no_nimble_file() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
