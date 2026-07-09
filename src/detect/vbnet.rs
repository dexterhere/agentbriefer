//! Detects a Visual Basic .NET project from a `*.vbproj` file. Same XML
//! shape as C#'s `*.csproj` and the same NuGet ecosystem, so it reuses
//! `csharp`'s framework/testing/database lookup lists rather than
//! duplicating them.

use std::fs;
use std::path::Path;

use super::csharp::{DATABASES, FRAMEWORKS, TESTING};
use super::formats::xml;
use super::{DetectedStack, categorize, find_file_with_extension};

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = find_file_with_extension(root, "vbproj")?;
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let dependency_names = xml::element_attribute(&contents, "PackageReference", "Include");

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("vbnet".to_string()),
        package_manager: Some("nuget".to_string()),
        framework,
        database,
        testing_tools,
        key_dependencies,
        source: manifest_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_categorized_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Example.vbproj"),
            r#"
<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Microsoft.AspNetCore.Mvc" Version="2.2.0" />
    <PackageReference Include="xunit" Version="2.4.2" />
  </ItemGroup>
</Project>
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("vbnet"));
        assert_eq!(detected.package_manager.as_deref(), Some("nuget"));
        assert_eq!(
            detected.framework.as_deref(),
            Some("Microsoft.AspNetCore.Mvc")
        );
        assert_eq!(detected.testing_tools, vec!["xunit".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("Example.vbproj"));
    }

    #[test]
    fn returns_none_when_no_vbproj() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
