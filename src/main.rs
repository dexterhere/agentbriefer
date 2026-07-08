use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skillforge",
    version,
    about = "Configure how AI coding agents think, code, test, and stop inside your project."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup; writes the project's skillforge.yaml
    Init,
    /// Generate AI instruction files from skillforge.yaml
    Generate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => skill_forge_cli::cli::run_init(),
        Commands::Generate => skill_forge_cli::cli::run_generate(),
    }
}
