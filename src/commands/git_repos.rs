use std::env;
use std::path::PathBuf;

use anyhow::Result;

use crate::repository::Repository;
use crate::services::git_repos_service::GitReposService;
use crate::task_result::TaskResult;

pub fn run(path: Option<PathBuf>, dry: bool) -> Result<i32> {
    let path = path
        .map(Ok)
        .unwrap_or_else(env::current_dir)?
        .canonicalize()?;
    let service = GitReposService::new_with_dry_run(dry);
    let result = service.handle_all_git_repos(&path)?;

    let exit_code = match &result {
        TaskResult::Proceed => 0,
        TaskResult::ShellActionRequired(_) => if dry { 0 } else { 10 },
    };

    if !dry {
        if let TaskResult::ShellActionRequired(directory) = &result {
            Repository::new().set_suggested_directory(directory)?;
        }
    }

    Ok(exit_code)
}
