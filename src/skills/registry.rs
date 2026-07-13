//! Parses the CLI's embedded skill catalog (`skills/**/SKILL.md`, bundled
//! into the binary at compile time — debug builds read live from disk,
//! release builds embed the bytes) into a queryable, read-only registry.

use std::path::Path;

use rust_embed::RustEmbed;

use crate::detect::DetectedStack;
use crate::textutil::split_frontmatter;

use super::error::SkillError;
use super::schema::Skill;

/// The `skills/` directory shipped alongside the crate — identical
/// embedding pattern to `render::engine::TemplateAssets` for `templates/`.
#[derive(RustEmbed)]
#[folder = "skills/"]
struct SkillAssets;

/// The full set of skills bundled with this build of the CLI.
pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl SkillRegistry {
    /// Parses every embedded `SKILL.md`. Fails only on a malformed bundled
    /// skill — a maintainer/build-time bug, never on the absence of skills.
    pub fn load() -> Result<Self, SkillError> {
        let mut skills = Vec::new();

        for name in SkillAssets::iter() {
            if !name.ends_with("SKILL.md") {
                continue;
            }

            let file = SkillAssets::get(&name).expect("name came from iter(), get() must find it");
            let content = std::str::from_utf8(&file.data).map_err(|_| SkillError::InvalidUtf8 {
                name: name.to_string(),
            })?;

            skills.push(parse_skill(&name, content)?);
        }

        skills.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Self { skills })
    }

    /// Every bundled skill, sorted by id.
    pub fn all(&self) -> &[Skill] {
        &self.skills
    }

    /// Looks up a skill by id.
    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|skill| skill.id == id)
    }

    /// Whether `id` names a real, bundled skill.
    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }

    /// Skills tagged with `role` (case-insensitive, any-match against each
    /// skill's `roles`).
    pub fn by_role<'a>(&'a self, role: &'a str) -> impl Iterator<Item = &'a Skill> {
        self.skills
            .iter()
            .filter(move |skill| skill.roles.iter().any(|r| r.eq_ignore_ascii_case(role)))
    }

    /// Skills whose `compatible_stacks` intersects any tag present in
    /// `detected` (case-insensitive, since a maintainer's frontmatter
    /// casing shouldn't silently break a match). A skill with an empty
    /// `compatible_stacks` is stack-agnostic and always included, rather
    /// than matching nothing.
    pub fn recommended_for<'a>(
        &'a self,
        detected: &'a DetectedStack,
    ) -> impl Iterator<Item = &'a Skill> {
        let detected_tags: Vec<String> = [
            detected.language.as_deref(),
            detected.framework.as_deref(),
            detected.package_manager.as_deref(),
            detected.database.as_deref(),
        ]
        .into_iter()
        .flatten()
        .chain(detected.testing_tools.iter().map(String::as_str))
        .chain(detected.key_dependencies.iter().map(String::as_str))
        .map(str::to_lowercase)
        .collect();

        self.skills.iter().filter(move |skill| {
            skill.compatible_stacks.is_empty()
                || skill
                    .compatible_stacks
                    .iter()
                    .any(|tag| detected_tags.contains(&tag.to_lowercase()))
        })
    }
}

/// Parses one embedded `SKILL.md`'s frontmatter + body, and validates that
/// the frontmatter's declared `id` matches the directory it lives under
/// (`<category>/<id>/SKILL.md`) — a mismatch is a bundled-catalog bug, not
/// something a project's `agentbriefer.yaml` could ever trigger.
fn parse_skill(name: &str, content: &str) -> Result<Skill, SkillError> {
    let (frontmatter, body) = split_frontmatter(content);

    if frontmatter.is_empty() {
        return Err(SkillError::MissingFrontmatter {
            name: name.to_string(),
        });
    }

    let yaml = frontmatter
        .trim_start_matches("---\n")
        .trim_end_matches("\n---\n");

    let mut skill: Skill =
        serde_yaml::from_str(yaml).map_err(|source| SkillError::InvalidFrontmatter {
            name: name.to_string(),
            source,
        })?;
    skill.body = body.trim().to_string();

    let expected_id = Path::new(name)
        .parent()
        .and_then(Path::file_name)
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    if skill.id != expected_id {
        return Err(SkillError::IdMismatch {
            path: name.to_string(),
            declared: skill.id,
            expected: expected_id.to_string(),
        });
    }

    Ok(skill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_every_bundled_skill_without_error() {
        let registry = SkillRegistry::load().unwrap();

        assert!(
            !registry.all().is_empty(),
            "expected at least the seed skills to be bundled"
        );
    }

    #[test]
    fn ids_are_unique_across_the_catalog() {
        let registry = SkillRegistry::load().unwrap();
        let mut ids: Vec<&str> = registry.all().iter().map(|s| s.id.as_str()).collect();
        let original_len = ids.len();

        ids.sort();
        ids.dedup();

        assert_eq!(ids.len(), original_len, "duplicate skill id in the catalog");
    }

    #[test]
    fn get_returns_none_for_an_unknown_id() {
        let registry = SkillRegistry::load().unwrap();

        assert!(registry.get("not-a-real-skill").is_none());
        assert!(!registry.contains("not-a-real-skill"));
    }

    #[test]
    fn get_finds_a_known_seed_skill() {
        let registry = SkillRegistry::load().unwrap();

        let skill = registry
            .get("server-components-by-default")
            .expect("seed skill should be bundled");

        assert_eq!(skill.category, "frontend");
        assert!(!skill.body.is_empty());
    }

    #[test]
    fn by_role_filters_case_insensitively() {
        let registry = SkillRegistry::load().unwrap();

        let frontend: Vec<&str> = registry
            .by_role("Frontend")
            .map(|s| s.id.as_str())
            .collect();

        assert!(frontend.contains(&"server-components-by-default"));
        assert!(frontend.contains(&"css-first-motion"));
    }

    #[test]
    fn recommended_for_matches_stack_tags_and_always_includes_stack_agnostic_skills() {
        let registry = SkillRegistry::load().unwrap();
        let detected = DetectedStack {
            framework: Some("next".to_string()),
            ..Default::default()
        };

        let ids: Vec<&str> = registry
            .recommended_for(&detected)
            .map(|s| s.id.as_str())
            .collect();

        assert!(ids.contains(&"server-components-by-default"));
        assert!(ids.contains(&"css-first-motion"));
        assert!(
            ids.contains(&"no-secrets-in-repo"),
            "a stack-agnostic skill (empty compatible_stacks) should always be recommended"
        );
    }

    #[test]
    fn recommended_for_excludes_stack_specific_skills_that_do_not_match() {
        let registry = SkillRegistry::load().unwrap();
        let detected = DetectedStack {
            language: Some("rust".to_string()),
            ..Default::default()
        };

        let ids: Vec<&str> = registry
            .recommended_for(&detected)
            .map(|s| s.id.as_str())
            .collect();

        assert!(!ids.contains(&"server-components-by-default"));
        assert!(ids.contains(&"no-secrets-in-repo"));
    }
}
