//! Shared dependency extraction for Gradle build files (`build.gradle` and
//! `build.gradle.kts`), used by both `kotlin.rs` and `groovy.rs` since
//! both declare dependencies with the same `group:artifact:version`
//! coordinate syntax, whether in Groovy or Kotlin DSL:
//! `implementation 'org.springframework.boot:spring-boot-starter-web:2.7.0'`
//! or `implementation("org.springframework.boot:spring-boot-starter-web:2.7.0")`.

use regex::Regex;

/// Extracts the *artifact* segment (the middle of `group:artifact:version`)
/// from every dependency-configuration line (`implementation`, `api`,
/// `compile`, `testImplementation`, `kapt`, ...) in a Gradle build file.
pub(in crate::detect) fn dependency_names(contents: &str) -> Vec<String> {
    let pattern = Regex::new(
        r#"(?:implementation|api|compile|testImplementation|androidTestImplementation|kapt|runtimeOnly)\s*[\("']+([\w.\-]+):([\w.\-]+):"#,
    )
    .expect("static regex is valid");

    pattern
        .captures_iter(contents)
        .filter_map(|caps| caps.get(2))
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_artifact_names_from_groovy_and_kotlin_dsl_styles() {
        let contents = r#"
dependencies {
    implementation 'org.springframework.boot:spring-boot-starter-web:2.7.0'
    implementation("io.ktor:ktor-server-core:2.3.0")
    testImplementation("junit:junit:4.13.2")
}
"#;

        let names = dependency_names(contents);

        assert_eq!(
            names,
            vec![
                "spring-boot-starter-web".to_string(),
                "ktor-server-core".to_string(),
                "junit".to_string()
            ]
        );
    }

    #[test]
    fn ignores_lines_without_a_full_coordinate() {
        let contents = "implementation project(':core')\n";

        assert!(dependency_names(contents).is_empty());
    }
}
