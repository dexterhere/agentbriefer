//! Subcommand implementations for the `agentbriefer` binary.

mod doctor;
mod generate;
mod init;
mod profile;
mod prompt;
mod skill;
mod sync;
mod ui;

pub use doctor::run as run_doctor;
pub use generate::run as run_generate;
pub use init::run as run_init;
pub use profile::run_create as run_profile_create;
pub use profile::run_list as run_profile_list;
pub use profile::run_switch as run_profile_switch;
pub use skill::run_add as run_skill_add;
pub use skill::run_info as run_skill_info;
pub use skill::run_list as run_skill_list;
pub use skill::run_profile_apply as run_skill_profile_apply;
pub use skill::run_profile_create as run_skill_profile_create;
pub use skill::run_profile_list as run_skill_profile_list;
pub use skill::run_remove as run_skill_remove;
pub use skill::run_update as run_skill_update;
pub use sync::run as run_sync;
