//! Detects a Java project from Maven's `pom.xml`.

use std::fs;
use std::path::Path;

use super::formats::xml;
use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &[
    "spring-boot-starter-web",
    "spring-boot-starter",
    "spring-core",
    "quarkus-core",
    "micronaut-core",
];
const TESTING: &[&str] = &["junit", "junit-jupiter", "testng", "mockito-core"];
const DATABASES: &[&str] = &["hibernate-core", "mybatis", "mongodb-driver-sync"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("pom.xml");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let dependency_names = xml::nested_element_text(&contents, "dependency", "artifactId");

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("java".to_string()),
        package_manager: Some("maven".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("pom.xml".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies_and_ignores_project_artifact_id() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pom.xml"),
            r#"
<project>
  <artifactId>my-own-app</artifactId>
  <dependencies>
    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter-web</artifactId>
    </dependency>
    <dependency>
      <groupId>org.junit.jupiter</groupId>
      <artifactId>junit-jupiter</artifactId>
      <scope>test</scope>
    </dependency>
  </dependencies>
</project>
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("java"));
        assert_eq!(detected.package_manager.as_deref(), Some("maven"));
        assert_eq!(
            detected.framework.as_deref(),
            Some("spring-boot-starter-web")
        );
        assert_eq!(detected.testing_tools, vec!["junit-jupiter".to_string()]);
        assert!(
            !detected
                .key_dependencies
                .contains(&"my-own-app".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_pom_xml() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
