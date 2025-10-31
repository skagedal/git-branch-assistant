use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::cleaner::GitCleaner;
use crate::fs_utils::is_globally_ignored;
use crate::git::{Branch, GitRepo};
use crate::task_result::TaskResult;
use crate::ui::DialoguerPrompt;

pub struct GitReposService {
    prompt: DialoguerPrompt,
}

impl GitReposService {
    pub fn new() -> Self {
        Self {
            prompt: DialoguerPrompt::default(),
        }
    }

    pub fn handle_all_git_repos(&self, path: &Path) -> Result<TaskResult> {
        let results = self.fetch_all_results(path)?;
        let mut task_result = TaskResult::Proceed;

        for result in results
            .into_iter()
            .filter(|result| !matches!(result.result, GitResult::Clean))
        {
            if !matches!(task_result, TaskResult::Proceed) {
                break;
            }
            task_result = self.handle_non_clean_repo_result(result)?;
        }

        Ok(task_result)
    }

    fn fetch_all_results(&self, path: &Path) -> Result<Vec<ResultWithPath>> {
        let mut results = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let result = self.repo_result(&entry_path)?;
            results.push(ResultWithPath {
                path: entry_path,
                result,
            });
        }
        Ok(results)
    }

    fn repo_result(&self, dir: &Path) -> Result<GitResult> {
        if !dir.is_dir() {
            return Ok(if is_globally_ignored(dir) {
                GitResult::Clean
            } else {
                GitResult::NotDirectory
            });
        }

        let repo = GitRepo::new(dir.to_path_buf());

        if let Some(worktree) = repo.find_dirty_worktree()? {
            return Ok(GitResult::Dirty(worktree.path));
        }

        let branches = repo.get_branches()?;
        let branches_needing_action: Vec<Branch> = branches
            .into_iter()
            .filter(|branch| branch.needs_action())
            .collect();

        if branches_needing_action.is_empty() {
            Ok(GitResult::Clean)
        } else {
            Ok(GitResult::BranchesNeedingAction(branches_needing_action))
        }
    }

    fn handle_non_clean_repo_result(&self, result_with_path: ResultWithPath) -> Result<TaskResult> {
        match result_with_path.result {
            GitResult::Dirty(path) => {
                eprintln!("Dirty git worktree: {}", path.display());
                Ok(TaskResult::ShellActionRequired(path))
            }
            GitResult::NotGitRepository => {
                eprintln!("Not a git repository: {}", result_with_path.path.display());
                Ok(TaskResult::ShellActionRequired(result_with_path.path))
            }
            GitResult::NotDirectory => {
                eprintln!("Not a directory: {}", result_with_path.path.display());
                let parent = result_with_path
                    .path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| result_with_path.path.clone());
                Ok(TaskResult::ShellActionRequired(parent))
            }
            GitResult::Clean => Ok(TaskResult::Proceed),
            GitResult::BranchesNeedingAction(branches) => {
                eprintln!(
                    "Has branches needing action: {}",
                    result_with_path.path.display()
                );
                let repo = GitRepo::new(result_with_path.path);
                let cleaner = GitCleaner::new(self.prompt.clone());
                cleaner.handle(&repo, branches)
            }
        }
    }
}

#[derive(Debug)]
pub enum GitResult {
    Clean,
    Dirty(PathBuf),
    NotGitRepository,
    NotDirectory,
    BranchesNeedingAction(Vec<Branch>),
}

struct ResultWithPath {
    path: PathBuf,
    result: GitResult,
}
