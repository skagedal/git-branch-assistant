use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod cleaner;
mod commands;
mod env;
mod fs_utils;
mod git;
mod repository;
mod services;
mod task_result;
mod ui;

#[derive(Parser)]
#[command(name = "git-branch-assistant")]
#[command(about = "Helper commands for managing git branches", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect the current repository and suggest actions for local branches.
    Clean {
        /// Path to the git repository (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Inspect child directories and highlight git repositories needing attention.
    Repos {
        /// Path to the directory to search (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Clean { path } => commands::git_clean::run(path)?,
        Command::Repos { path } => {
            let exit_code = commands::git_repos::run(path)?;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}
