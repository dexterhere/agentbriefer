//! Detects a Go project from `go.mod`. Hand-rolled parsing rather than a
//! dependency: `go.mod`'s `require` syntax is small and stable enough that
//! pulling in a crate for it isn't worth it.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["gin", "echo", "fiber", "chi"];
const TESTING: &[&str] = &["testify", "ginkgo"];
const DATABASES: &[&str] = &["gorm", "pgx", "mongo-driver"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("go.mod");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let dependency_names: Vec<String> = parse_required_modules(&contents)
        .iter()
        .map(|path| short_name(path))
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("go".to_string()),
        package_manager: Some("go modules".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("go.mod".to_string()),
    })
}

/// Extracts every module path named in a `require (...)` block or a
/// single-line `require modpath version` statement. Trailing
/// `// indirect` comments are dropped naturally, since only the first
/// whitespace-separated token (the module path) is kept.
fn parse_required_modules(contents: &str) -> Vec<String> {
    let mut modules = Vec::new();
    let mut in_require_block = false;

    for line in contents.lines() {
        let line = line.trim();

        if line == "require (" {
            in_require_block = true;
            continue;
        }

        if in_require_block {
            if line == ")" {
                in_require_block = false;
            } else if let Some(module) = line.split_whitespace().next() {
                modules.push(module.to_string());
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("require ")
            && let Some(module) = rest.split_whitespace().next()
        {
            modules.push(module.to_string());
        }
    }

    modules
}

/// Reduces a full module path (e.g. `github.com/gin-gonic/gin` or
/// `github.com/jackc/pgx/v5`) to the short name used in our lookup lists,
/// by taking the last path segment that isn't a bare version marker
/// (`v2`, `v5`, ...).
fn short_name(module_path: &str) -> String {
    module_path
        .split('/')
        .rev()
        .find(|segment| !is_version_segment(segment))
        .unwrap_or(module_path)
        .to_string()
}

fn is_version_segment(segment: &str) -> bool {
    segment.len() > 1
        && segment.starts_with('v')
        && segment[1..].chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies_from_a_require_block() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            r#"
module example.com/myapp

go 1.21

require (
	github.com/gin-gonic/gin v1.9.1
	github.com/stretchr/testify v1.8.4
	github.com/jackc/pgx/v5 v5.5.0
	github.com/pkg/errors v0.9.1 // indirect
)
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("go"));
        assert_eq!(detected.package_manager.as_deref(), Some("go modules"));
        assert_eq!(detected.framework.as_deref(), Some("gin"));
        assert_eq!(detected.database.as_deref(), Some("pgx"));
        assert_eq!(detected.testing_tools, vec!["testify".to_string()]);
        assert_eq!(detected.key_dependencies, vec!["errors".to_string()]);
    }

    #[test]
    fn detects_single_line_require_statements() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/myapp\n\ngo 1.21\n\nrequire github.com/labstack/echo/v4 v4.11.0\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.framework.as_deref(), Some("echo"));
    }

    #[test]
    fn returns_none_when_no_go_mod() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }

    #[test]
    fn short_name_strips_trailing_version_segment() {
        assert_eq!(short_name("github.com/jackc/pgx/v5"), "pgx");
        assert_eq!(short_name("gorm.io/gorm"), "gorm");
        assert_eq!(short_name("go-chi/chi"), "chi");
    }
}
