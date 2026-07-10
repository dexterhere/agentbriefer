use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agentbriefer",
    version,
    about = "Configure how AI coding agents think, code, test, and stop inside your project."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup; writes the project's agentbriefer.yaml
    Init,
    /// Generate AI instruction files from agentbriefer.yaml
    Generate,
    /// Re-render AI instruction files, preserving manual edits outside the
    /// agentbriefer-managed block. Note: running `generate` afterwards does
    /// not respect those markers and will overwrite manual edits.
    Sync,
    /// Manage reusable developer profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Check the current project's configuration and generated files for
    /// conflicting settings, missing fields, and drift
    Doctor,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List saved developer profiles
    List,
    /// Save a new reusable developer profile
    Create,
    /// Apply a saved profile's developer style to this project
    Switch,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => agentbriefer::cli::run_init(),
        Commands::Generate => agentbriefer::cli::run_generate(),
        Commands::Sync => agentbriefer::cli::run_sync(),
        Commands::Profile { action } => match action {
            ProfileAction::List => agentbriefer::cli::run_profile_list(),
            ProfileAction::Create => agentbriefer::cli::run_profile_create(),
            ProfileAction::Switch => agentbriefer::cli::run_profile_switch(),
        },
        Commands::Doctor => agentbriefer::cli::run_doctor(),
    }
}
