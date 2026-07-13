//! The `Skill` data model — parsed from an embedded `SKILL.md`'s YAML
//! frontmatter plus its markdown body.

use serde::{Deserialize, Serialize};

/// A single bundled skill: identifying metadata plus the markdown
/// instruction body that gets inlined into generated output files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    /// Open-ended tags (e.g. "frontend", "security") a skill applies to.
    /// Deliberately a plain `Vec<String>`, not an enum: unlike the
    /// developer/project settings in `config::schema`, this set keeps
    /// growing as skills are added, without requiring a Rust enum change
    /// each time.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Stack tags (language/framework/etc.) this skill is relevant to. An
    /// empty list means "stack-agnostic" — always recommended, not "matches
    /// nothing".
    #[serde(default)]
    pub compatible_stacks: Vec<String>,
    /// The markdown instruction body — everything after the frontmatter.
    /// Never part of the YAML frontmatter itself, so it's populated by
    /// `SkillRegistry::load` after parsing, not deserialized from the file.
    #[serde(skip)]
    pub body: String,
}
