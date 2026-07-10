//! Detects a Gradle-built JVM project (`build.gradle`/`build.gradle.kts`)
//! and picks Kotlin, Groovy, or Java as the language.
//!
//! Kept as one detector rather than three separate `kotlin.rs`/`groovy.rs`/
//! `java_gradle.rs` files: the three outcomes share one manifest and one
//! decision, so splitting them would mean either duplicating the file
//! lookup or relying on fragile cross-file ordering in `detect::detect`'s
//! `.or_else()` chain (whichever ran first would wrongly win). One
//! function making the one decision it needs to make is simpler and
//! actually more correct.
//!
//! `build.gradle` doesn't say by itself whether project *source* is Java,
//! Kotlin, or Groovy — all three can build with Gradle. Priority:
//! 1. `build.gradle.kts` exists → Kotlin (the `.kts` extension is a real signal).
//! 2. `build.gradle` contains a Groovy plugin marker (`id 'groovy'` /
//!    `apply plugin: 'groovy'`) → Groovy.
//! 3. `build.gradle` otherwise → Java, the historically most common case
//!    for a plain Gradle file (and Java doesn't already own Gradle the way
//!    it owns Maven's `pom.xml` from round 1).

use std::fs;
use std::path::Path;

use regex::Regex;

use super::formats::gradle;
use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &[
    "spring-boot-starter-web",
    "spring-boot-starter",
    "ktor-server-core",
    "micronaut-core",
];
const TESTING: &[&str] = &["junit", "junit-jupiter", "kotlintest", "mockk", "spek"];
const DATABASES: &[&str] = &["hibernate-core", "exposed-core", "r2dbc-core"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let (contents, language, source) =
        if let Ok(contents) = fs::read_to_string(root.join("build.gradle.kts")) {
            (contents, "kotlin", "build.gradle.kts")
        } else {
            let contents = fs::read_to_string(root.join("build.gradle")).ok()?;
            let language = if has_groovy_plugin(&contents) {
                "groovy"
            } else {
                "java"
            };
            (contents, language, "build.gradle")
        };

    let dependency_names = gradle::dependency_names(&contents);

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some(language.to_string()),
        package_manager: Some("gradle".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some(source.to_string()),
    })
}

fn has_groovy_plugin(contents: &str) -> bool {
    let pattern = Regex::new(r#"(?:id\s*\(?\s*['"]groovy['"]|apply\s+plugin:\s*['"]groovy['"])"#)
        .expect("static regex is valid");

    pattern.is_match(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_kotlin_via_kts_extension() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("build.gradle.kts"),
            r#"
dependencies {
    implementation("io.ktor:ktor-server-core:2.3.0")
    testImplementation("io.mockk:mockk:1.13.5")
}
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("kotlin"));
        assert_eq!(detected.package_manager.as_deref(), Some("gradle"));
        assert_eq!(detected.framework.as_deref(), Some("ktor-server-core"));
        assert_eq!(detected.testing_tools, vec!["mockk".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("build.gradle.kts"));
    }

    #[test]
    fn detects_groovy_via_plugin_marker() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("build.gradle"),
            "apply plugin: 'groovy'\n\ndependencies {\n    implementation 'org.springframework.boot:spring-boot-starter:2.7.0'\n}\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("groovy"));
    }

    #[test]
    fn defaults_to_java_for_plain_build_gradle() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("build.gradle"),
            "dependencies {\n    implementation 'org.springframework.boot:spring-boot-starter-web:2.7.0'\n}\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("java"));
        assert_eq!(
            detected.framework.as_deref(),
            Some("spring-boot-starter-web")
        );
    }

    #[test]
    fn returns_none_when_no_gradle_file() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
