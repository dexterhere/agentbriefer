//! Detects a Node/JavaScript/TypeScript project from `package.json`.

use std::fs;
use std::path::Path;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &[
    "react",
    "next",
    "vue",
    "nuxt",
    "svelte",
    "express",
    "fastify",
    "@nestjs/core",
    "@angular/core",
];
const TESTING: &[&str] = &["jest", "vitest", "mocha", "cypress", "playwright"];
const DATABASES: &[&str] = &["prisma", "mongoose", "sequelize", "typeorm", "pg", "mysql2"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("package.json");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let mut dependency_names = Vec::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(object) = manifest.get(key).and_then(serde_json::Value::as_object) {
            dependency_names.extend(object.keys().cloned());
        }
    }

    let is_typescript = root.join("tsconfig.json").exists()
        || dependency_names.iter().any(|name| name == "typescript");
    let language = if is_typescript {
        "typescript"
    } else {
        "javascript"
    };

    let package_manager = if root.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if root.join("yarn.lock").exists() {
        "yarn"
    } else if root.join("bun.lockb").exists() {
        "bun"
    } else {
        "npm"
    };

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some(language.to_string()),
        package_manager: Some(package_manager.to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("package.json".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_package_json(dir: &std::path::Path, contents: &str) {
        fs::write(dir.join("package.json"), contents).unwrap();
    }

    #[test]
    fn detects_javascript_and_npm_by_default() {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(
            dir.path(),
            r#"{"dependencies": {"express": "^4.0.0", "pg": "^8.0.0"}, "devDependencies": {"jest": "^29.0.0"}}"#,
        );

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("javascript"));
        assert_eq!(detected.package_manager.as_deref(), Some("npm"));
        assert_eq!(detected.framework.as_deref(), Some("express"));
        assert_eq!(detected.database.as_deref(), Some("pg"));
        assert_eq!(detected.testing_tools, vec!["jest".to_string()]);
    }

    #[test]
    fn detects_typescript_via_tsconfig() {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"dependencies": {}}"#);
        fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("typescript"));
    }

    #[test]
    fn detects_pnpm_via_lockfile() {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"dependencies": {}}"#);
        fs::write(dir.path().join("pnpm-lock.yaml"), "lockfileVersion: '9.0'").unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.package_manager.as_deref(), Some("pnpm"));
    }

    #[test]
    fn returns_none_when_no_package_json() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
