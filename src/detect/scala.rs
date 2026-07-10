//! Detects a Scala project from `build.sbt`.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["play", "akka-http", "http4s-dsl"];
const TESTING: &[&str] = &["scalatest", "munit", "specs2-core"];
const DATABASES: &[&str] = &["slick", "doobie-core", "quill-jdbc"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("build.sbt");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    // Matches `"group" %% "artifact" %` or `"group" % "artifact" %`,
    // capturing the artifact (2nd quoted string).
    let pattern = Regex::new(r#""[^"]+"\s*%%?\s*"([^"]+)"\s*%"#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("scala".to_string()),
        package_manager: Some("sbt".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("build.sbt".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("build.sbt"),
            r#"
libraryDependencies ++= Seq(
  "com.typesafe.play" %% "play" % "2.8.18",
  "com.typesafe.slick" %% "slick" % "3.4.1",
  "org.scalatest" %% "scalatest" % "3.2.9" % Test
)
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("scala"));
        assert_eq!(detected.package_manager.as_deref(), Some("sbt"));
        assert_eq!(detected.framework.as_deref(), Some("play"));
        assert_eq!(detected.database.as_deref(), Some("slick"));
        assert_eq!(detected.testing_tools, vec!["scalatest".to_string()]);
    }

    #[test]
    fn returns_none_when_no_build_sbt() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
