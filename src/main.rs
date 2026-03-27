mod commands;
mod config;
mod integrations;
mod path;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rt", version, about = "Git repository manager with predictable directory tree")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone a repository into the computed tree path
    #[command(alias = "c")]
    Clone {
        /// Git remote URL
        url: String,
    },

    /// Delete a managed repository
    #[command(alias = "d")]
    Delete {
        /// Repository path or remote URL (defaults to current directory)
        repo: Option<String>,
    },

    /// Print the repository tree under root
    #[command(alias = "t")]
    Tree,

    /// Worktree management commands
    #[command(alias = "wt")]
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },
}

#[derive(Subcommand)]
enum WorktreeCommands {
    /// Create a new worktree
    #[command(alias = "a")]
    Add {
        /// Branch name for the new worktree
        branch: String,
    },

    /// List worktrees for the current repository
    #[command(alias = "l")]
    List,

    /// Interactively delete a worktree
    #[command(alias = "d")]
    Delete,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load()?;

    match cli.command {
        Commands::Clone { url } => commands::clone::run(&config, &url).await,
        Commands::Delete { repo } => commands::delete::run(&config, repo.as_deref()).await,
        Commands::Tree => commands::tree::run(&config).await,
        Commands::Worktree { command } => match command {
            WorktreeCommands::Add { branch } => {
                commands::worktree::add::run(&config, &branch).await
            }
            WorktreeCommands::List => commands::worktree::list::run(&config).await,
            WorktreeCommands::Delete => commands::worktree::delete::run(&config).await,
        },
    }
}
