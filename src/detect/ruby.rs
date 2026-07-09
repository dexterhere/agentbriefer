//! Detects a Ruby project from `Gemfile.lock`'s indented `specs:` list
//! (close to declarative — one gem name per line, no code execution
//! needed), falling back to regex-scanning `Gemfile` itself if there's no
//! lock file yet.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["rails", "sinatra", "hanami", "roda"];
const TESTING: &[&str] = &["rspec-core", "minitest", "cucumber"];
const DATABASES: &[&str] = &["pg", "mysql2", "sqlite3", "mongoid"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let (dependency_names, source) =
        if let Ok(contents) = fs::read_to_string(root.join("Gemfile.lock")) {
            (dependencies_from_lockfile(&contents), "Gemfile.lock")
        } else {
            let contents = fs::read_to_string(root.join("Gemfile")).ok()?;
            (dependencies_from_gemfile(&contents), "Gemfile")
        };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("ruby".to_string()),
        package_manager: Some("bundler".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some(source.to_string()),
    })
}

/// Reads gem names from `Gemfile.lock`'s `specs:` block, e.g.:
/// ```text
/// GEM
///   specs:
///     rails (7.0.4)
///     pg (1.4.5)
/// ```
fn dependencies_from_lockfile(contents: &str) -> Vec<String> {
    let mut in_specs = false;
    let mut names = Vec::new();

    for line in contents.lines() {
        if line.trim() == "specs:" {
            in_specs = true;
            continue;
        }
        if in_specs {
            // Spec lines are indented under "specs:"; a line indented less
            // (or not at all) ends the block. Top-level gems have no
            // further indentation beneath them (nested "depends on" lines
            // are indented deeper and are skipped by the space-count check).
            let indent = line.len() - line.trim_start().len();
            if line.trim().is_empty() {
                continue;
            }
            if indent != 4 {
                if indent < 4 {
                    in_specs = false;
                }
                continue;
            }
            if let Some(name) = line.split_whitespace().next() {
                names.push(name.to_string());
            }
        }
    }

    names
}

fn dependencies_from_gemfile(contents: &str) -> Vec<String> {
    let pattern = Regex::new(r#"gem\s+['"]([^'"]+)['"]"#).expect("static regex is valid");

    pattern
        .captures_iter(contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_via_gemfile_lock() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Gemfile.lock"),
            "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0.4)\n      activesupport (= 7.0.4)\n    pg (1.4.5)\n    rspec-core (3.12.0)\n\nPLATFORMS\n  ruby\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("ruby"));
        assert_eq!(detected.package_manager.as_deref(), Some("bundler"));
        assert_eq!(detected.framework.as_deref(), Some("rails"));
        assert_eq!(detected.database.as_deref(), Some("pg"));
        assert_eq!(detected.testing_tools, vec!["rspec-core".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("Gemfile.lock"));
        assert!(
            !detected
                .key_dependencies
                .contains(&"activesupport".to_string())
        );
    }

    #[test]
    fn falls_back_to_gemfile_when_no_lockfile() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Gemfile"),
            "source 'https://rubygems.org'\n\ngem 'sinatra'\ngem \"pg\"\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.framework.as_deref(), Some("sinatra"));
        assert_eq!(detected.source.as_deref(), Some("Gemfile"));
    }

    #[test]
    fn returns_none_when_neither_file_exists() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
