//! Detects a C/C++ project. There's no single canonical manifest the way
//! Rust has `Cargo.toml` — tries `vcpkg.json`, then `conanfile.txt`, then a
//! best-effort `find_package(...)` scan of `CMakeLists.txt`, in that order
//! of decreasing confidence.
//!
//! Doesn't distinguish C from C++: both build via CMake and there's no
//! reliable manifest-level signal to tell them apart without inspecting
//! source file extensions, which is out of scope (detection only reads
//! manifests). Always reports `language: cpp` — a real, bounded
//! limitation, not silently wrong forever, since it's just an editable
//! pre-fill in `init`.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["qt5", "qt6", "wxwidgets", "sfml"];
const TESTING: &[&str] = &["gtest", "catch2", "doctest", "boost-test"];
const DATABASES: &[&str] = &["libpqxx", "sqlite3", "mongo-cxx-driver"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let (dependency_names, package_manager, source) =
        if let Ok(contents) = fs::read_to_string(root.join("vcpkg.json")) {
            (dependencies_from_vcpkg(&contents)?, "vcpkg", "vcpkg.json")
        } else if let Ok(contents) = fs::read_to_string(root.join("conanfile.txt")) {
            (
                dependencies_from_conanfile(&contents),
                "conan",
                "conanfile.txt",
            )
        } else {
            let contents = fs::read_to_string(root.join("CMakeLists.txt")).ok()?;
            (
                dependencies_from_cmake(&contents),
                "cmake",
                "CMakeLists.txt",
            )
        };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("cpp".to_string()),
        package_manager: Some(package_manager.to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some(source.to_string()),
    })
}

fn dependencies_from_vcpkg(contents: &str) -> Option<Vec<String>> {
    let manifest: serde_json::Value = serde_json::from_str(contents).ok()?;
    let deps = manifest.get("dependencies")?.as_array()?;

    Some(
        deps.iter()
            .filter_map(|dep| {
                dep.as_str().map(str::to_string).or_else(|| {
                    dep.get("name")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
            })
            .collect(),
    )
}

/// Conan's `[requires]` section lists one `name/version` entry per line.
fn dependencies_from_conanfile(contents: &str) -> Vec<String> {
    let mut in_requires = false;
    let mut names = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "[requires]" {
            in_requires = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_requires = false;
            continue;
        }
        if in_requires
            && !trimmed.is_empty()
            && let Some(name) = trimmed.split('/').next()
        {
            names.push(name.to_string());
        }
    }

    names
}

fn dependencies_from_cmake(contents: &str) -> Vec<String> {
    let pattern = Regex::new(r"find_package\(\s*([A-Za-z0-9_]+)").expect("static regex is valid");

    pattern
        .captures_iter(contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_via_vcpkg_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("vcpkg.json"),
            r#"{"name": "example", "dependencies": ["fmt", "gtest"]}"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("cpp"));
        assert_eq!(detected.package_manager.as_deref(), Some("vcpkg"));
        assert_eq!(detected.testing_tools, vec!["gtest".to_string()]);
    }

    #[test]
    fn detects_via_conanfile_requires_section() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("conanfile.txt"),
            "[requires]\nfmt/9.1.0\ncatch2/3.3.2\n\n[generators]\ncmake\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.package_manager.as_deref(), Some("conan"));
        assert_eq!(detected.testing_tools, vec!["catch2".to_string()]);
    }

    #[test]
    fn detects_via_cmake_find_package() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("CMakeLists.txt"),
            "find_package(Qt6 REQUIRED)\nfind_package(GTest REQUIRED)\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.package_manager.as_deref(), Some("cmake"));
        assert_eq!(detected.framework.as_deref(), Some("qt6"));
        assert_eq!(detected.testing_tools, vec!["gtest".to_string()]);
    }

    #[test]
    fn returns_none_when_no_manifest_exists() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
