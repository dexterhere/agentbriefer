//! Thin `roxmltree` helpers for the manifest formats that are XML: Maven's
//! `pom.xml` (dependency names as nested element text) and .NET's
//! `.csproj`/`.fsproj` (dependency names as an attribute value).

/// Collects the text of `child_tag` for every `parent_tag` element, e.g.
/// Maven's `<dependency><artifactId>serde</artifactId></dependency>` —
/// scoping to only `artifactId`s that are direct children of a
/// `<dependency>` avoids also matching the project's *own* `artifactId`
/// (a sibling of `<dependencies>`, not nested inside one) or a `<parent>`
/// section's. Returns an empty list if `xml` doesn't parse — malformed
/// input just means "nothing detected," not a hard error.
pub(in crate::detect) fn nested_element_text(
    xml: &str,
    parent_tag: &str,
    child_tag: &str,
) -> Vec<String> {
    let Ok(doc) = roxmltree::Document::parse(xml) else {
        return Vec::new();
    };

    doc.descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == parent_tag)
        .filter_map(|parent| {
            parent
                .children()
                .find(|child| child.is_element() && child.tag_name().name() == child_tag)
        })
        .filter_map(|child| child.text())
        .map(str::to_string)
        .collect()
}

/// Collects the value of `attribute` on every element named `tag`, e.g.
/// .csproj's `<PackageReference Include="Newtonsoft.Json" .../>`.
pub(in crate::detect) fn element_attribute(xml: &str, tag: &str, attribute: &str) -> Vec<String> {
    let Ok(doc) = roxmltree::Document::parse(xml) else {
        return Vec::new();
    };

    doc.descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == tag)
        .filter_map(|node| node.attribute(attribute))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_element_text_collects_maven_style_dependency_names_only() {
        let xml = r#"
<project>
  <artifactId>my-own-app</artifactId>
  <parent>
    <artifactId>my-parent-pom</artifactId>
  </parent>
  <dependencies>
    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter-web</artifactId>
    </dependency>
    <dependency>
      <groupId>junit</groupId>
      <artifactId>junit</artifactId>
    </dependency>
  </dependencies>
</project>
"#;

        let names = nested_element_text(xml, "dependency", "artifactId");

        assert_eq!(
            names,
            vec!["spring-boot-starter-web".to_string(), "junit".to_string()]
        );
    }

    #[test]
    fn element_attribute_collects_csproj_style_dependency_names() {
        let xml = r#"
<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageReference Include="xunit" Version="2.4.2" />
  </ItemGroup>
</Project>
"#;

        let names = element_attribute(xml, "PackageReference", "Include");

        assert_eq!(
            names,
            vec!["Newtonsoft.Json".to_string(), "xunit".to_string()]
        );
    }

    #[test]
    fn returns_empty_on_malformed_xml() {
        assert!(nested_element_text("not xml at all <<<", "dependency", "artifactId").is_empty());
    }
}
