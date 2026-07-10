//! Detects a Lua project from a `*.rockspec` file's `dependencies` table.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize, find_file_with_extension};

const FRAMEWORKS: &[&str] = &["openresty", "lapis"];
const TESTING: &[&str] = &["busted"];
const DATABASES: &[&str] = &["luasql-sqlite3", "lsqlite3"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = find_file_with_extension(root, "rockspec")?;
    let contents = fs::read_to_string(&manifest_path).ok()?;

    // Entries look like `"luasocket >= 1.0"` or `"lpeg"` inside a
    // `dependencies = {...}` table; keep just the module name.
    let pattern =
        Regex::new(r#""([A-Za-z0-9_.\-]+)(?:\s*[><=~][^"]*)?""#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|name| !name.eq_ignore_ascii_case("lua"))
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("lua".to_string()),
        package_manager: Some("luarocks".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: manifest_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("example-1.0-1.rockspec"),
            r#"
package = "example"
version = "1.0-1"
dependencies = {
   "lua >= 5.1",
   "lapis",
   "busted",
   "luasql-sqlite3"
}
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("lua"));
        assert_eq!(detected.package_manager.as_deref(), Some("luarocks"));
        assert_eq!(detected.framework.as_deref(), Some("lapis"));
        assert_eq!(detected.database.as_deref(), Some("luasql-sqlite3"));
        assert_eq!(detected.testing_tools, vec!["busted".to_string()]);
        assert!(!detected.key_dependencies.iter().any(|d| d == "lua"));
    }

    #[test]
    fn returns_none_when_no_rockspec() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
