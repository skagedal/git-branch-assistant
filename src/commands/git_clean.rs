use std::env;

use anyhow::Result;

use crate::cleaner::GitCleaner;
use crate::git::GitRepo;
use crate::repository::Repository;
use crate::task_result::TaskResult;
use crate::ui::DialoguerPrompt;

pub fn run() -> Result<()> {
    let repo_path = env::current_dir()?;
    let repo = GitRepo::new(repo_path);
    let branches = repo.get_branches()?;
    let cleaner = GitCleaner::new(DialoguerPrompt::default());
    match cleaner.handle(&repo, branches)? {
        TaskResult::Proceed => Ok(()),
        TaskResult::ShellActionRequired(path) => {
            Repository::new().set_suggested_directory(&path)?;
            std::process::exit(10);
        }
    }
}
