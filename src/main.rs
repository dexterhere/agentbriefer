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
    /// Manage this project's installed skills and reusable skill profiles
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
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

#[derive(Subcommand)]
enum SkillAction {
    /// List available skills (--role/--recommended are personal, session-only filters)
    List {
        /// Only show skills tagged with this role (e.g. frontend, backend, security)
        #[arg(long)]
        role: Option<String>,
        /// Only show skills matching this project's detected stack
        #[arg(long)]
        recommended: bool,
    },
    /// Install a skill into this project
    Add {
        /// The skill id, as shown by `agentbriefer skill list`
        id: String,
    },
    /// Remove a skill from this project
    Remove {
        /// The skill id, as shown by `agentbriefer skill list`
        id: String,
    },
    /// Re-render installed skills against the currently-bundled catalog
    Update,
    /// Show full details for one skill
    Info {
        /// The skill id, as shown by `agentbriefer skill list`
        id: String,
    },
    /// Manage reusable named skill sets
    Profile {
        #[command(subcommand)]
        action: SkillProfileAction,
    },
}

#[derive(Subcommand)]
enum SkillProfileAction {
    /// List saved skill profiles
    List,
    /// Save this project's currently installed skills under a reusable name
    Create {
        /// Name for the new skill profile
        name: String,
    },
    /// Replace this project's installed skills with a saved profile's
    Apply {
        /// Name of the skill profile to apply
        name: String,
    },
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
        Commands::Skill { action } => match action {
            SkillAction::List { role, recommended } => {
                agentbriefer::cli::run_skill_list(role, recommended)
            }
            SkillAction::Add { id } => agentbriefer::cli::run_skill_add(&id),
            SkillAction::Remove { id } => agentbriefer::cli::run_skill_remove(&id),
            SkillAction::Update => agentbriefer::cli::run_skill_update(),
            SkillAction::Info { id } => agentbriefer::cli::run_skill_info(&id),
            SkillAction::Profile { action } => match action {
                SkillProfileAction::List => agentbriefer::cli::run_skill_profile_list(),
                SkillProfileAction::Create { name } => {
                    agentbriefer::cli::run_skill_profile_create(&name)
                }
                SkillProfileAction::Apply { name } => {
                    agentbriefer::cli::run_skill_profile_apply(&name)
                }
            },
        },
    }
}
