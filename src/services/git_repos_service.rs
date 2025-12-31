use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(feature = "timings")]
use std::time::Instant;

use crate::cleaner::GitCleaner;
use crate::fs_utils::is_globally_ignored;
use crate::git::{Branch, GitRepo};
use crate::task_result::TaskResult;
use crate::ui::{DialoguerPrompt, DryRunPrompt};

pub struct GitReposService {
    dry_run: bool,
}

impl GitReposService {
    pub fn new_with_dry_run(dry_run: bool) -> Self {
        Self { dry_run }
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
        #[cfg(feature = "timings")]
        let start = Instant::now();

        let result = if !dir.is_dir() {
            if is_globally_ignored(dir) {
                GitResult::Clean
            } else {
                GitResult::NotDirectory
            }
        } else {
            let repo = GitRepo::new(dir.to_path_buf());
            let branches = repo.get_branches()?;
            let branches_needing_action: Vec<Branch> = branches
                .into_iter()
                .filter(|branch| branch.needs_action())
                .collect();

            if branches_needing_action.is_empty() {
                GitResult::Clean
            } else {
                GitResult::BranchesNeedingAction(branches_needing_action)
            }
        };

        #[cfg(feature = "timings")]
        {
            eprintln!(
                "[timing] repo_result {} => {} ({:?})",
                dir.display(),
                summarize_git_result(&result),
                start.elapsed()
            );
        }

        Ok(result)
    }

    fn handle_non_clean_repo_result(&self, result_with_path: ResultWithPath) -> Result<TaskResult> {
        match result_with_path.result {
            GitResult::NotGitRepository => {
                if self.dry_run {
                    println!(
                        "[DRY RUN] Not a git repository: {}",
                        result_with_path.path.display()
                    );
                    Ok(TaskResult::Proceed)
                } else {
                    eprintln!("Not a git repository: {}", result_with_path.path.display());
                    Ok(TaskResult::ShellActionRequired(result_with_path.path))
                }
            }
            GitResult::NotDirectory => {
                if self.dry_run {
                    println!(
                        "[DRY RUN] Not a directory: {}",
                        result_with_path.path.display()
                    );
                    Ok(TaskResult::Proceed)
                } else {
                    eprintln!("Not a directory: {}", result_with_path.path.display());
                    let parent = result_with_path
                        .path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| result_with_path.path.clone());
                    Ok(TaskResult::ShellActionRequired(parent))
                }
            }
            GitResult::Clean => Ok(TaskResult::Proceed),
            GitResult::BranchesNeedingAction(branches) => {
                if self.dry_run {
                    println!(
                        "[DRY RUN] Has branches needing action: {}",
                        result_with_path.path.display()
                    );
                } else {
                    eprintln!(
                        "Has branches needing action: {}",
                        result_with_path.path.display()
                    );
                }
                let repo = GitRepo::new(result_with_path.path);
                if self.dry_run {
                    let cleaner = GitCleaner::new_with_dry_run(DryRunPrompt::default(), true);
                    cleaner.handle(&repo, branches)
                } else {
                    let cleaner = GitCleaner::new(DialoguerPrompt::default());
                    cleaner.handle(&repo, branches)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum GitResult {
    Clean,
    NotGitRepository,
    NotDirectory,
    BranchesNeedingAction(Vec<Branch>),
}

#[cfg(feature = "timings")]
fn summarize_git_result(result: &GitResult) -> &'static str {
    match result {
        GitResult::Clean => "Clean",
        GitResult::NotGitRepository => "NotGitRepository",
        GitResult::NotDirectory => "NotDirectory",
        GitResult::BranchesNeedingAction(_) => "BranchesNeedingAction",
    }
}

struct ResultWithPath {
    path: PathBuf,
    result: GitResult,
}
