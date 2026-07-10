//! Detects a Python project from `pyproject.toml` (PEP 621's
//! `[project.dependencies]` array or Poetry's `[tool.poetry.dependencies]`
//! table), falling back to `requirements.txt` if there's no `pyproject.toml`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["django", "flask", "fastapi", "pyramid", "tornado"];
const TESTING: &[&str] = &["pytest", "nose", "tox", "unittest2"];
const DATABASES: &[&str] = &["sqlalchemy", "psycopg2", "pymongo", "redis", "pymysql"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let pyproject_path = root.join("pyproject.toml");

    let dependency_names = if let Ok(contents) = fs::read_to_string(&pyproject_path) {
        let manifest: toml::Value = toml::from_str(&contents).ok()?;
        dependencies_from_pyproject(&manifest)
    } else {
        let requirements = fs::read_to_string(root.join("requirements.txt")).ok()?;
        dependencies_from_requirements_txt(&requirements)
    };

    let package_manager = if root.join("poetry.lock").exists() {
        "poetry"
    } else if root.join("uv.lock").exists() {
        "uv"
    } else if root.join("pdm.lock").exists() {
        "pdm"
    } else if root.join("Pipfile.lock").exists() {
        "pipenv"
    } else {
        "pip"
    };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("python".to_string()),
        package_manager: Some(package_manager.to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some(
            if pyproject_path.exists() {
                "pyproject.toml"
            } else {
                "requirements.txt"
            }
            .to_string(),
        ),
    })
}

/// Reads dependency names from either PEP 621's `[project.dependencies]`
/// (an array of version-spec strings) or Poetry's
/// `[tool.poetry.dependencies]`/`[tool.poetry.dev-dependencies]` (tables
/// keyed by package name), whichever is present.
fn dependencies_from_pyproject(manifest: &toml::Value) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(deps) = manifest
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(toml::Value::as_array)
    {
        names.extend(
            deps.iter()
                .filter_map(toml::Value::as_str)
                .map(package_name_from_spec),
        );
    }

    // PEP 621 `[project.optional-dependencies]` is a table of named extras
    // (commonly `dev`/`test`), each an array of version-spec strings --
    // exactly where a real project declares `pytest`. Flattened across
    // every extra group since `categorize()` doesn't need to know which
    // named extra a dependency came from.
    if let Some(optional) = manifest
        .get("project")
        .and_then(|p| p.get("optional-dependencies"))
        .and_then(toml::Value::as_table)
    {
        for group in optional.values() {
            if let Some(deps) = group.as_array() {
                names.extend(
                    deps.iter()
                        .filter_map(toml::Value::as_str)
                        .map(package_name_from_spec),
                );
            }
        }
    }

    for table_path in [
        ["tool", "poetry", "dependencies"],
        ["tool", "poetry", "dev-dependencies"],
    ] {
        if let Some(table) = table_path
            .iter()
            .try_fold(manifest, |value, key| value.get(key))
            .and_then(toml::Value::as_table)
        {
            names.extend(table.keys().filter(|key| key.as_str() != "python").cloned());
        }
    }

    names
}

fn dependencies_from_requirements_txt(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('-'))
        .map(package_name_from_spec)
        .collect()
}

/// Strips a version specifier / environment marker / extras suffix from a
/// dependency spec, e.g. `"requests[socks]>=2.28; python_version >= '3.8'"`
/// becomes `"requests"`.
fn package_name_from_spec(spec: &str) -> String {
    spec.split(['=', '>', '<', '~', '!', '[', ' ', ';'])
        .next()
        .unwrap_or(spec)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_poetry_style_pyproject() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            r#"
[tool.poetry]
name = "example"

[tool.poetry.dependencies]
python = "^3.11"
django = "^4.2"
psycopg2 = "^2.9"

[tool.poetry.dev-dependencies]
pytest = "^7.0"
"#,
        )
        .unwrap();
        fs::write(dir.path().join("poetry.lock"), "").unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("python"));
        assert_eq!(detected.package_manager.as_deref(), Some("poetry"));
        assert_eq!(detected.framework.as_deref(), Some("django"));
        assert_eq!(detected.database.as_deref(), Some("psycopg2"));
        assert_eq!(detected.testing_tools, vec!["pytest".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("pyproject.toml"));
    }

    #[test]
    fn detects_pep_621_style_pyproject() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            r#"
[project]
name = "example"
dependencies = ["fastapi>=0.100", "sqlalchemy~=2.0"]
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.framework.as_deref(), Some("fastapi"));
        assert_eq!(detected.database.as_deref(), Some("sqlalchemy"));
        assert_eq!(detected.package_manager.as_deref(), Some("pip"));
    }

    #[test]
    fn detects_testing_tools_in_pep_621_optional_dependencies() {
        // Real PEP 621 projects declare dev/test-only deps (pytest, httpx,
        // ...) under `[project.optional-dependencies]`, not the main
        // `dependencies` array -- this must not be invisible to detection.
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            r#"
[project]
name = "example"
dependencies = ["fastapi>=0.100", "sqlalchemy~=2.0"]

[project.optional-dependencies]
dev = ["pytest>=7.0", "httpx>=0.24"]
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.testing_tools, vec!["pytest".to_string()]);
        assert!(detected.key_dependencies.contains(&"httpx".to_string()));
    }

    #[test]
    fn falls_back_to_requirements_txt() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("requirements.txt"),
            "flask==2.3.0\n# a comment\n-r other.txt\npytest\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.framework.as_deref(), Some("flask"));
        assert_eq!(detected.testing_tools, vec!["pytest".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("requirements.txt"));
    }

    #[test]
    fn returns_none_when_neither_file_exists() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
