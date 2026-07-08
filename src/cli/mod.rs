//! Subcommand implementations for the `skillforge` binary.

mod generate;
mod init;

pub use generate::run as run_generate;
pub use init::run as run_init;
