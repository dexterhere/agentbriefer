//! Detects a PowerShell module from a `*.psd1` manifest's `RequiredModules`
//! entry, which can list either bare module-name strings or hashtables
//! with a `ModuleName` key — both are valid PowerShell manifest syntax.

use std::fs;
use std::path::Path;

use regex::Regex;

use super::{DetectedStack, categorize, find_file_with_extension};

const FRAMEWORKS: &[&str] = &["Pode"];
const TESTING: &[&str] = &["Pester"];
const DATABASES: &[&str] = &["SqlServer", "PSSQLite"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = find_file_with_extension(root, "psd1")?;
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let dependency_names = required_modules(&contents);

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("powershell".to_string()),
        package_manager: Some("powershellget".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: manifest_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
    })
}

fn required_modules(contents: &str) -> Vec<String> {
    let block_pattern =
        Regex::new(r"RequiredModules\s*=\s*@\(([\s\S]*?)\)").expect("static regex is valid");
    let Some(block) = block_pattern
        .captures(contents)
        .and_then(|caps| caps.get(1))
    else {
        return Vec::new();
    };
    let block = block.as_str();

    // Hashtable-style entries (`@{ModuleName = 'Pode'; ModuleVersion = '2.0.0'}`)
    // first, matched specifically on the `ModuleName` key so a sibling
    // field like `ModuleVersion`'s quoted value is never picked up.
    let hashtable_name_pattern =
        Regex::new(r#"ModuleName\s*=\s*['"]([^'"]+)['"]"#).expect("static regex is valid");
    let mut names: Vec<String> = hashtable_name_pattern
        .captures_iter(block)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    // Then strip every `@{...}` hashtable out of the block entirely, so
    // only bare string entries (`'Pester'`) are left for the next pass —
    // otherwise a hashtable's other quoted fields would leak in too.
    let hashtable_pattern = Regex::new(r"@\{[^}]*\}").expect("static regex is valid");
    let without_hashtables = hashtable_pattern.replace_all(block, "");

    let bare_string_pattern = Regex::new(r#"['"]([^'"]+)['"]"#).expect("static regex is valid");
    names.extend(
        bare_string_pattern
            .captures_iter(&without_hashtables)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().to_string()),
    );

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bare_string_required_modules() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Example.psd1"),
            "@{\n    ModuleVersion = '1.0.0'\n    RequiredModules = @('Pester', 'SqlServer')\n}\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("powershell"));
        assert_eq!(detected.package_manager.as_deref(), Some("powershellget"));
        assert_eq!(detected.testing_tools, vec!["Pester".to_string()]);
        assert_eq!(detected.database.as_deref(), Some("SqlServer"));
        assert_eq!(detected.source.as_deref(), Some("Example.psd1"));
    }

    #[test]
    fn detects_hashtable_style_required_modules() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Example.psd1"),
            "@{\n    RequiredModules = @(\n        @{ModuleName = 'Pode'; ModuleVersion = '2.0.0'},\n        'Pester'\n    )\n}\n",
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.framework.as_deref(), Some("Pode"));
        assert_eq!(detected.testing_tools, vec!["Pester".to_string()]);
        // Regression: a hashtable entry's ModuleVersion value ("2.0.0")
        // must not leak in as if it were its own dependency name.
        assert!(!detected.key_dependencies.iter().any(|d| d == "2.0.0"));
        assert!(detected.key_dependencies.is_empty());
    }

    #[test]
    fn returns_none_when_no_psd1_file() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
