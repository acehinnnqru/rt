mod commands;
mod config;
mod git;
mod integrations;
mod path;
#[cfg(test)]
mod test_support;

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
        /// Clone as a bare repository with the default branch linked as a worktree
        /// inside it (for example `A/main`); accepts an optional branch name
        #[arg(
            long,
            short = 'w',
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = ""
        )]
        worktree: Option<String>,
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
    #[command(visible_alias = "wt")]
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },
}

#[derive(Subcommand)]
enum WorktreeCommands {
    /// Create a worktree for a branch
    #[command(alias = "a")]
    Add {
        /// Branch to check out; created from the remote when it does not exist locally
        branch: String,
        /// Start point for a new branch (defaults to the remote branch, else the remote HEAD)
        #[arg(long, value_name = "REF")]
        from: Option<String>,
        /// Do not fetch origin before resolving the branch
        #[arg(long)]
        no_fetch: bool,
    },

    /// List the worktrees of the current repository
    #[command(alias = "l")]
    List {
        /// Print only the paths, one per line
        #[arg(long)]
        paths: bool,
    },

    /// Remove a worktree, asking which one when no branch is given
    #[command(alias = "rm", alias = "delete", alias = "d")]
    Remove {
        /// Branch whose worktree to remove, or its path
        branch: Option<String>,
        /// Remove even with modified or untracked files
        #[arg(long, short)]
        force: bool,
    },

    /// Remove registrations of worktrees whose directory is gone
    Prune,

    /// Print the path of a branch's worktree
    Path {
        /// Branch name
        branch: String,
    },

    /// Lock a worktree, so that it is neither pruned nor removed
    Lock {
        /// Branch name
        branch: String,
        /// Reason recorded with the lock
        #[arg(long, value_name = "TEXT")]
        reason: Option<String>,
    },

    /// Unlock a worktree
    Unlock {
        /// Branch name
        branch: String,
    },

    /// Move a worktree to another directory
    Move {
        /// Branch name
        branch: String,
        /// Destination directory
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load()?;

    match cli.command {
        Commands::Clone { url, worktree } => {
            commands::clone::run(&config, &url, worktree.as_deref()).await
        }
        Commands::Delete { repo } => commands::delete::run(&config, repo.as_deref()).await,
        Commands::Tree => commands::tree::run(&config).await,
        Commands::Worktree { command } => match command {
            WorktreeCommands::Add {
                branch,
                from,
                no_fetch,
            } => commands::worktree::add::run(&config, &branch, from.as_deref(), no_fetch).await,
            WorktreeCommands::List { paths } => commands::worktree::list::run(paths).await,
            WorktreeCommands::Remove { branch, force } => {
                commands::worktree::remove::run(branch.as_deref(), force).await
            }
            WorktreeCommands::Prune => commands::worktree::prune::run().await,
            WorktreeCommands::Path { branch } => commands::worktree::path::run(&branch).await,
            WorktreeCommands::Lock { branch, reason } => {
                commands::worktree::lock::run_lock(&branch, reason.as_deref()).await
            }
            WorktreeCommands::Unlock { branch } => {
                commands::worktree::lock::run_unlock(&branch).await
            }
            WorktreeCommands::Move { branch, path } => {
                commands::worktree::r#move::run(&branch, &path).await
            }
        },
    }
}
