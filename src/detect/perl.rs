//! Detects a Perl project from `cpanfile`'s `requires 'Module::Name';` lines.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["Dancer2", "Mojolicious", "Catalyst"];
const TESTING: &[&str] = &["Test::Simple", "Test::Class"];
// More specific drivers listed before the generic "DBI" wrapper, so a
// project's actual database backend wins over the generic interface it's
// almost always paired with.
const DATABASES: &[&str] = &["DBD::Pg", "DBD::mysql", "DBI"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("cpanfile");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern = Regex::new(r#"requires\s+['"]([^'"]+)['"]"#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("perl".to_string()),
        package_manager: Some("cpanm".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("cpanfile".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("cpanfile"),
            "requires 'Mojolicious', '9.0';\nrequires 'DBI';\nrequires \"DBD::Pg\";\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("perl"));
        assert_eq!(detected.package_manager.as_deref(), Some("cpanm"));
        assert_eq!(detected.framework.as_deref(), Some("Mojolicious"));
        assert_eq!(detected.database.as_deref(), Some("DBD::Pg"));
        assert!(detected.key_dependencies.contains(&"DBI".to_string()));
    }

    #[test]
    fn returns_none_when_no_cpanfile() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
