//! Subcommand implementations for the `skillforge` binary.

mod doctor;
mod generate;
mod init;
mod profile;
mod prompt;
mod sync;

pub use doctor::run as run_doctor;
pub use generate::run as run_generate;
pub use init::run as run_init;
pub use profile::run_create as run_profile_create;
pub use profile::run_list as run_profile_list;
pub use profile::run_switch as run_profile_switch;
pub use sync::run as run_sync;
