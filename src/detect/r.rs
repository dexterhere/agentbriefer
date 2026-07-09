//! Detects an R project from its `DESCRIPTION` file (the DCF —
//! Debian Control File — format: `Key: value` lines, where a field's value
//! can continue onto following lines as long as they're indented).

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["shiny", "plumber"];
const TESTING: &[&str] = &["testthat"];
const DATABASES: &[&str] = &["DBI", "RSQLite", "RPostgres", "RMySQL"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("DESCRIPTION");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let mut dependency_names = dcf_field_entries(&contents, "Imports");
    dependency_names.extend(dcf_field_entries(&contents, "Suggests"));

    let package_manager = if root.join("renv.lock").exists() {
        "renv"
    } else {
        "CRAN"
    };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("r".to_string()),
        package_manager: Some(package_manager.to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("DESCRIPTION".to_string()),
    })
}

/// Reads a comma-separated DCF field (e.g. `Imports:`/`Suggests:`),
/// folding continuation lines (indented, following the field's own line)
/// into one buffer before splitting on commas. Each entry's version
/// constraint, e.g. `testthat (>= 3.0.0)`, is dropped, keeping just the
/// package name.
fn dcf_field_entries(contents: &str, field: &str) -> Vec<String> {
    let prefix = format!("{field}:");
    let mut collecting = false;
    let mut buffer = String::new();

    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix(&prefix) {
            collecting = true;
            buffer.push_str(rest);
            buffer.push(',');
            continue;
        }

        if collecting {
            if line.starts_with(char::is_whitespace) && !line.trim().is_empty() {
                buffer.push_str(line.trim());
                buffer.push(',');
            } else {
                collecting = false;
            }
        }
    }

    buffer
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.split_whitespace().next().unwrap_or(entry).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_folds_continuation_lines() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("DESCRIPTION"),
            "Package: example\nVersion: 1.0.0\nImports:\n    shiny,\n    DBI\nSuggests:\n    testthat (>= 3.0.0),\n    knitr\nLicense: MIT\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("r"));
        assert_eq!(detected.package_manager.as_deref(), Some("CRAN"));
        assert_eq!(detected.framework.as_deref(), Some("shiny"));
        assert_eq!(detected.database.as_deref(), Some("DBI"));
        assert_eq!(detected.testing_tools, vec!["testthat".to_string()]);
        assert!(detected.key_dependencies.contains(&"knitr".to_string()));
    }

    #[test]
    fn detects_renv_as_package_manager_when_lockfile_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("DESCRIPTION"),
            "Package: example\nImports: shiny\n",
        )
        .unwrap();
        fs::write(dir.path().join("renv.lock"), "{}").unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.package_manager.as_deref(), Some("renv"));
    }

    #[test]
    fn returns_none_when_no_description_file() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }

    #[test]
    fn dcf_field_entries_stops_at_the_next_field() {
        let contents = "Imports:\n    dplyr,\n    ggplot2\nLicense: MIT\n";

        assert_eq!(
            dcf_field_entries(contents, "Imports"),
            vec!["dplyr".to_string(), "ggplot2".to_string()]
        );
    }
}
