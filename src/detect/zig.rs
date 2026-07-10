//! Detects a Zig project from `build.zig.zon` — Zig's own struct-literal
//! object notation (`.field = value`), not JSON. Each dependency is a
//! named nested object under `.dependencies`, e.g.:
//! ```text
//! .dependencies = .{
//!     .zap = .{ .url = "...", .hash = "..." },
//! },
//! ```
//!
//! **Least battle-tested detector in this project, by nature of what it's
//! detecting, not a shortcut taken:** `build.zig.zon` is a newer, less
//! standardized format than everything else detected so far — Zig's
//! package ecosystem is still forming.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["zap", "http.zig"];
const TESTING: &[&str] = &[];
const DATABASES: &[&str] = &["pg.zig", "sqlite"];

/// Top-level `.zon` fields that are objects (`.field = .{ ... }`) but
/// aren't dependencies, so they'd otherwise false-match the dependency
/// pattern below.
const NON_DEPENDENCY_OBJECT_FIELDS: &[&str] = &["dependencies", "paths"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("build.zig.zon");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern =
        Regex::new(r"\.([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\.\{").expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|name| !NON_DEPENDENCY_OBJECT_FIELDS.contains(&name.as_str()))
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("zig".to_string()),
        package_manager: Some("zig".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("build.zig.zon".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("build.zig.zon"),
            r#"
.{
    .name = "example",
    .version = "0.1.0",
    .paths = .{""},
    .dependencies = .{
        .zap = .{
            .url = "https://github.com/zigzap/zap/archive/abc.tar.gz",
            .hash = "1220abc",
        },
        .sqlite = .{
            .path = "../sqlite",
        },
    },
}
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("zig"));
        assert_eq!(detected.package_manager.as_deref(), Some("zig"));
        assert_eq!(detected.framework.as_deref(), Some("zap"));
        assert_eq!(detected.database.as_deref(), Some("sqlite"));
        assert!(
            !detected
                .key_dependencies
                .iter()
                .any(|d| d == "dependencies" || d == "paths")
        );
    }

    #[test]
    fn returns_none_when_no_build_zig_zon() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
