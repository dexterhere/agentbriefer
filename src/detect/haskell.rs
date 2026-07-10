//! Detects a Haskell project from `package.yaml` (the hpack convention),
//! falling back to a `*.cabal` file's `build-depends:` field if there's no
//! `package.yaml`. The `.cabal` fallback only looks at the top-level
//! `build-depends:` list — best-effort, since a `.cabal` file can define
//! per-component dependencies that a simple scan won't separate out.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize, find_file_with_extension};

const FRAMEWORKS: &[&str] = &["yesod", "scotty", "servant", "servant-server"];
const TESTING: &[&str] = &["hspec", "tasty", "QuickCheck"];
const DATABASES: &[&str] = &["persistent", "esqueleto", "postgresql-simple"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let package_yaml_path = root.join("package.yaml");

    let (dependency_names, source) = if let Ok(contents) = fs::read_to_string(&package_yaml_path) {
        let manifest: serde_yaml::Value = serde_yaml::from_str(&contents).ok()?;
        let names = manifest
            .get("dependencies")
            .and_then(serde_yaml::Value::as_sequence)
            .map(|deps| {
                deps.iter()
                    .filter_map(serde_yaml::Value::as_str)
                    .map(package_name_from_spec)
                    .collect()
            })
            .unwrap_or_default();
        (names, "package.yaml".to_string())
    } else {
        let cabal_path = find_file_with_extension(root, "cabal")?;
        let contents = fs::read_to_string(&cabal_path).ok()?;
        let name = cabal_path.file_name()?.to_string_lossy().into_owned();
        (dependencies_from_cabal(&contents), name)
    };

    let package_manager = if root.join("stack.yaml").exists() {
        "stack"
    } else {
        "cabal"
    };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("haskell".to_string()),
        package_manager: Some(package_manager.to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some(source),
    })
}

/// Extracts package names from a `.cabal` file's `build-depends:` field
/// and its indented continuation lines, e.g.:
/// ```text
/// build-depends:
///     base >= 4.7 && < 5,
///     text,
///     aeson
/// ```
fn dependencies_from_cabal(contents: &str) -> Vec<String> {
    let mut collecting = false;
    let mut buffer = String::new();

    for line in contents.lines() {
        let trimmed = line.trim_start();

        // Cabal field names are case-insensitive; "build-depends:" is 14
        // ASCII bytes regardless of case, so slicing `trimmed` (not the
        // lowercased copy) at that offset is safe.
        if trimmed.to_ascii_lowercase().starts_with("build-depends:") {
            collecting = true;
            buffer.push_str(&trimmed[14..]);
            buffer.push(',');
            continue;
        }

        if collecting {
            if line.starts_with(char::is_whitespace) && !trimmed.is_empty() {
                buffer.push_str(trimmed);
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
        .map(package_name_from_spec)
        .collect()
}

fn package_name_from_spec(spec: &str) -> String {
    spec.split_whitespace()
        .next()
        .unwrap_or(spec)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_via_package_yaml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.yaml"),
            r#"
name: example
dependencies:
- base >= 4.7 && < 5
- yesod
- persistent
- hspec
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("haskell"));
        assert_eq!(detected.package_manager.as_deref(), Some("cabal"));
        assert_eq!(detected.framework.as_deref(), Some("yesod"));
        assert_eq!(detected.database.as_deref(), Some("persistent"));
        assert_eq!(detected.testing_tools, vec!["hspec".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("package.yaml"));
    }

    #[test]
    fn detects_stack_as_package_manager_when_stack_yaml_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.yaml"),
            "name: example\ndependencies:\n- base\n",
        )
        .unwrap();
        fs::write(dir.path().join("stack.yaml"), "resolver: lts-21.0\n").unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.package_manager.as_deref(), Some("stack"));
    }

    #[test]
    fn falls_back_to_cabal_build_depends() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("example.cabal"),
            "name: example\n\nlibrary\n    build-depends:\n        base >= 4.7 && < 5,\n        scotty,\n        tasty\n    default-language: Haskell2010\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.framework.as_deref(), Some("scotty"));
        assert_eq!(detected.testing_tools, vec!["tasty".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("example.cabal"));
    }

    #[test]
    fn returns_none_when_neither_file_exists() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
