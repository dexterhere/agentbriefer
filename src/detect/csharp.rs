//! Detects a C# / .NET project from a `*.csproj` file. Shares its
//! framework/testing/database lookup lists with `fsharp.rs`, since both
//! target the same NuGet package ecosystem and the same XML shape.

use std::fs;
use std::path::Path;

use super::formats::xml;
use super::{DetectedStack, categorize, find_file_with_extension};

pub(super) const FRAMEWORKS: &[&str] = &[
    "Microsoft.AspNetCore.App",
    "Microsoft.AspNetCore.Mvc",
    "Microsoft.NETCore.App",
];
pub(super) const TESTING: &[&str] = &["xunit", "NUnit", "MSTest.TestFramework"];
pub(super) const DATABASES: &[&str] =
    &["Microsoft.EntityFrameworkCore", "Dapper", "MongoDB.Driver"];

pub(super) fn detect(root: &Path) -> Option<DetectedStack> {
    let manifest_path = find_file_with_extension(root, "csproj")?;
    let contents = fs::read_to_string(&manifest_path).ok()?;

    let dependency_names = xml::element_attribute(&contents, "PackageReference", "Include");

    let (framework, database, testing_tools, key_dependencies) =
        categorize(dependency_names, FRAMEWORKS, TESTING, DATABASES);

    Some(DetectedStack {
        language: Some("csharp".to_string()),
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
            dir.path().join("Example.csproj"),
            r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
  <ItemGroup>
    <PackageReference Include="Microsoft.AspNetCore.Mvc" Version="2.2.0" />
    <PackageReference Include="Dapper" Version="2.1.0" />
    <PackageReference Include="xunit" Version="2.4.2" />
  </ItemGroup>
</Project>
"#,
        )
        .unwrap();

        let detected = detect(dir.path()).unwrap();

        assert_eq!(detected.language.as_deref(), Some("csharp"));
        assert_eq!(detected.package_manager.as_deref(), Some("nuget"));
        assert_eq!(
            detected.framework.as_deref(),
            Some("Microsoft.AspNetCore.Mvc")
        );
        assert_eq!(detected.database.as_deref(), Some("Dapper"));
        assert_eq!(detected.testing_tools, vec!["xunit".to_string()]);
        assert_eq!(detected.source.as_deref(), Some("Example.csproj"));
    }

    #[test]
    fn returns_none_when_no_csproj() {
        let dir = tempfile::tempdir().unwrap();

        assert!(detect(dir.path()).is_none());
    }
}
