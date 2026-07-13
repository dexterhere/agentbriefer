//! Embedded skill catalog: parses `skills/**/SKILL.md` (bundled into the
//! binary at compile time) into a queryable, read-only registry.
//!
//! Pure — no real filesystem access beyond the embedded assets, no
//! `~/.config`, no `std::env`. Anything that touches a user's actual
//! project (`.agentbriefer/skills/`) or their real config directory
//! (`~/.config/agentbriefer/skill-profiles/`) belongs in `cli::skill`
//! instead, mirroring how `config`/`render` stay pure while `cli` owns all
//! real I/O elsewhere in this crate.

mod error;
mod registry;
mod schema;

pub use error::SkillError;
pub use registry::SkillRegistry;
pub use schema::Skill;
