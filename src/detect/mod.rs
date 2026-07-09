//! Detects a project's language, package manager, framework, testing
//! tools, and key dependencies from real manifest files (`Cargo.toml`,
//! `package.json`, `go.mod`), so `skillforge init` can pre-fill its stack
//! questions instead of starting from a blank field.
//!
//! Environment-agnostic in spirit but not in practice: unlike `config`,
//! this module does read real files — that's its entire job — but it
//! never touches stdin or prompts, and always returns a value rather than
//! erroring, so a malformed or unreadable manifest just means "nothing
//! detected" instead of failing `init` outright.

mod crystal;
mod csharp;
mod dart;
mod formats;
mod fsharp;
mod go;
mod haskell;
mod java;
mod julia;
mod node;
mod php;
mod python;
mod r;
mod rust;

use std::fs;
use std::path::{Path, PathBuf};

/// Maximum number of raw dependency names kept in `key_dependencies`,
/// so a project with hundreds of packages doesn't flood the generated
/// instruction files.
const MAX_KEY_DEPENDENCIES: usize = 20;

/// What was detected about a project's stack, and where it came from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetectedStack {
    pub language: Option<String>,
    pub package_manager: Option<String>,
    pub framework: Option<String>,
    pub database: Option<String>,
    pub testing_tools: Vec<String>,
    pub key_dependencies: Vec<String>,
    /// The manifest file detection was based on, e.g. `"Cargo.toml"` —
    /// purely for a transparency message shown to the user.
    pub source: Option<String>,
}

/// Detects a project's stack by trying each language's detector in turn,
/// returning the first non-empty result, or `DetectedStack::default()` if
/// none of them find anything. Order mostly doesn't matter — each detector
/// keys on a distinct manifest filename, so collisions only happen in
/// unusual polyglot repos, where first-match-wins is an accepted tradeoff
/// (every detected value is just an editable pre-fill anyway).
pub fn detect(root: &Path) -> DetectedStack {
    rust::detect(root)
        .or_else(|| node::detect(root))
        .or_else(|| go::detect(root))
        .or_else(|| python::detect(root))
        .or_else(|| php::detect(root))
        .or_else(|| java::detect(root))
        .or_else(|| csharp::detect(root))
        .or_else(|| fsharp::detect(root))
        .or_else(|| dart::detect(root))
        .or_else(|| julia::detect(root))
        .or_else(|| crystal::detect(root))
        .or_else(|| haskell::detect(root))
        .or_else(|| r::detect(root))
        .unwrap_or_default()
}

/// Finds the first file directly inside `root` whose extension matches
/// `extension`, e.g. locating a project's single `*.csproj` file (whose
/// name varies per project, unlike a fixed manifest filename). Not
/// recursive, and picks arbitrarily if more than one match exists.
pub(super) fn find_file_with_extension(root: &Path, extension: &str) -> Option<PathBuf> {
    fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some(extension))
}

/// Shared helper: given the full set of detected dependency names and
/// lookup lists for framework/testing/database crates, picks the
/// framework/database match, collects all testing matches, and returns the
/// remaining names (capped, alphabetically sorted) as `key_dependencies`.
///
/// Framework/database are picked by walking `frameworks`/`databases` in
/// *their own* order, not the alphabetized dependency list — so a lookup
/// list can express priority (e.g. Dart's list puts `flutter` first so it
/// wins over a state-management library like `bloc` that's also a listed
/// framework and would otherwise win purely by alphabetical luck).
pub(super) fn categorize(
    mut dependency_names: Vec<String>,
    frameworks: &[&str],
    testing: &[&str],
    databases: &[&str],
) -> (Option<String>, Option<String>, Vec<String>, Vec<String>) {
    dependency_names.sort();

    let framework = frameworks
        .iter()
        .find(|name| dependency_names.iter().any(|dep| dep == *name))
        .map(|name| name.to_string());
    let database = databases
        .iter()
        .find(|name| dependency_names.iter().any(|dep| dep == *name))
        .map(|name| name.to_string());
    let testing_tools: Vec<String> = dependency_names
        .iter()
        .filter(|name| testing.contains(&name.as_str()))
        .cloned()
        .collect();

    let key_dependencies = dependency_names
        .into_iter()
        .filter(|name| {
            Some(name) != framework.as_ref()
                && Some(name) != database.as_ref()
                && !testing_tools.contains(name)
        })
        .take(MAX_KEY_DEPENDENCIES)
        .collect();

    (framework, database, testing_tools, key_dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorize_excludes_framework_database_and_testing_from_key_dependencies() {
        let deps = vec![
            "axum".to_string(),
            "serde".to_string(),
            "insta".to_string(),
            "sqlx".to_string(),
        ];

        let (framework, database, testing_tools, key_dependencies) =
            categorize(deps, &["axum"], &["insta"], &["sqlx"]);

        assert_eq!(framework, Some("axum".to_string()));
        assert_eq!(database, Some("sqlx".to_string()));
        assert_eq!(testing_tools, vec!["insta".to_string()]);
        assert_eq!(key_dependencies, vec!["serde".to_string()]);
    }

    #[test]
    fn categorize_prefers_frameworks_list_order_over_alphabetical_dependency_order() {
        // "bloc" sorts before "flutter" alphabetically, but a lookup list
        // that lists "flutter" first should still win — this is the exact
        // shape of a real Flutter + bloc project.
        let deps = vec!["bloc".to_string(), "flutter".to_string()];

        let (framework, _, _, key_dependencies) = categorize(deps, &["flutter", "bloc"], &[], &[]);

        assert_eq!(framework, Some("flutter".to_string()));
        assert_eq!(key_dependencies, vec!["bloc".to_string()]);
    }

    #[test]
    fn categorize_caps_key_dependencies() {
        let deps: Vec<String> = (0..30).map(|i| format!("dep-{i:02}")).collect();

        let (_, _, _, key_dependencies) = categorize(deps, &[], &[], &[]);

        assert_eq!(key_dependencies.len(), MAX_KEY_DEPENDENCIES);
    }

    #[test]
    fn detect_returns_default_when_nothing_matches() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(detect(dir.path()), DetectedStack::default());
    }
}
