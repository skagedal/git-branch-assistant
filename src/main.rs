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
    Clean,
    /// Inspect child directories and highlight git repositories needing attention.
    Repos,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Clean => commands::git_clean::run()?,
        Command::Repos => {
            let exit_code = commands::git_repos::run()?;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}
