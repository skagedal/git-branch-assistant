use std::env;

use anyhow::Result;

use crate::cleaner::GitCleaner;
use crate::git::GitRepo;
use crate::ui::UserInterface;

pub fn run() -> Result<()> {
    let repo_path = env::current_dir()?;
    let repo = GitRepo::new(repo_path);
    let branches = repo.get_branches()?;
    let cleaner = GitCleaner::new(UserInterface::new());
    let _ = cleaner.handle(&repo, branches)?;
    Ok(())
}
