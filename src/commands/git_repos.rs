use std::env;

use anyhow::Result;

use crate::repository::Repository;
use crate::services::git_repos_service::GitReposService;
use crate::task_result::TaskResult;

pub fn run() -> Result<i32> {
    let path = env::current_dir()?.canonicalize()?;
    let service = GitReposService::new();
    let result = service.handle_all_git_repos(&path)?;

    let exit_code = match &result {
        TaskResult::Proceed => 0,
        TaskResult::ShellActionRequired(_) => 10,
    };

    if let TaskResult::ShellActionRequired(directory) = &result {
        Repository::new().set_suggested_directory(directory)?;
    }

    Ok(exit_code)
}
