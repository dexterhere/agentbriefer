//! Detects an Erlang project from `rebar.config`'s `deps` list, e.g.
//! `{deps, [{cowboy, "2.9.0"}, {jsx, "3.1.0"}]}.`.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize};

const FRAMEWORKS: &[&str] = &["cowboy", "nitrogen_core"];
const TESTING: &[&str] = &["ct_helper", "meck"];
const DATABASES: &[&str] = &["epgsql", "mysql", "mongodb"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = root.join("rebar.config");
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let pattern =
        Regex::new(r#"\{([a-z][a-zA-Z0-9_]*)\s*,\s*"[^"]*"\}"#).expect("static regex is valid");
    let dependency_names: Vec<String> = pattern
        .captures_iter(&contents)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("erlang".to_string()),
        package_manager: Some("rebar3".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: Some("rebar.config".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("rebar.config"),
            r#"{deps, [
    {cowboy, "2.9.0"},
    {epgsql, "4.7.0"},
    {jsx, "3.1.0"}
]}.
{erl_opts, [debug_info]}."#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("erlang"));
        assert_eq!(detected.package_manager.as_deref(), Some("rebar3"));
        assert_eq!(detected.framework.as_deref(), Some("cowboy"));
        assert_eq!(detected.database.as_deref(), Some("epgsql"));
        assert!(detected.key_dependencies.contains(&"jsx".to_string()));
    }

    #[test]
    fn returns_none_when_no_rebar_config() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
