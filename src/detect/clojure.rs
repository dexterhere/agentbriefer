//! Detects a Clojure project from `deps.edn` (the `tools.deps` convention),
//! falling back to Leiningen's `project.clj` if there's no `deps.edn`.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["ring", "compojure", "luminus", "reitit"];
const TESTING: &[&str] = &["midje"];
const DATABASES: &[&str] = &["next.jdbc", "korma", "datomic"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let (dependency_names, package_manager, source) =
        if let Ok(contents) = fs::read_to_string(root.join("deps.edn")) {
            (dependencies_from_deps_edn(&contents), "clj", "deps.edn")
        } else {
            let contents = fs::read_to_string(root.join("project.clj")).ok()?;
            (
                dependencies_from_project_clj(&contents),
                "leiningen",
                "project.clj",
            )
        };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("clojure".to_string()),
        package_manager: Some(package_manager.to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some(source.to_string()),
    })
}

/// `deps.edn` coordinates look like `ring/ring {:mvn/version "1.9.0"}` —
/// captures the coordinate (group/artifact or bare artifact) immediately
/// preceding a `{:mvn/version` map.
fn dependencies_from_deps_edn(contents: &str) -> Vec<String> {
    let pattern = Regex::new(r"([A-Za-z0-9][A-Za-z0-9_.\-/]*)\s*\{:mvn/version")
        .expect("static regex is valid");

    pattern
        .captures_iter(contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| artifact_from_coordinate(m.as_str()))
        .collect()
}

/// Leiningen's `:dependencies [[group/artifact "version"] ...]` vectors.
fn dependencies_from_project_clj(contents: &str) -> Vec<String> {
    let pattern = Regex::new(r#"\[([A-Za-z0-9][A-Za-z0-9_.\-/]*)\s+"[^"]*"\]"#)
        .expect("static regex is valid");

    pattern
        .captures_iter(contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| artifact_from_coordinate(m.as_str()))
        .collect()
}

/// Clojure coordinates are often `group/artifact` (e.g. `ring/ring`) —
/// keep just the artifact (last segment) to match lookup-list entries.
fn artifact_from_coordinate(coordinate: &str) -> String {
    coordinate
        .rsplit('/')
        .next()
        .unwrap_or(coordinate)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_via_deps_edn() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("deps.edn"),
            r#"{:deps {org.clojure/clojure {:mvn/version "1.11.1"}
                ring/ring {:mvn/version "1.9.0"}
                compojure/compojure {:mvn/version "1.7.0"}
                midje/midje {:mvn/version "1.10.9"}}}"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("clojure"));
        assert_eq!(detected.package_manager.as_deref(), Some("clj"));
        assert_eq!(detected.framework.as_deref(), Some("ring"));
        assert_eq!(detected.testing_tools, vec!["midje".to_string()]);
    }

    #[test]
    fn falls_back_to_project_clj() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("project.clj"),
            r#"(defproject example "0.1.0"
  :dependencies [[org.clojure/clojure "1.11.1"]
                 [compojure "1.7.0"]])"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.package_manager.as_deref(), Some("leiningen"));
        assert_eq!(detected.framework.as_deref(), Some("compojure"));
    }

    #[test]
    fn returns_none_when_neither_file_exists() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
