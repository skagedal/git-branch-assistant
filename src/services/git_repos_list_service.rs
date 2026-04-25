use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::fs_utils::is_globally_ignored;
use crate::git::{Branch, GitRepo, UpstreamStatus};
use crate::task_result::TaskResult;
use crate::ui::Prompt;

#[derive(Debug, Clone)]
pub struct BranchListEntry {
    pub repo_name: String,
    pub repo_path: PathBuf,
    pub refname: String,
    pub status: BranchStatus,
    pub commit_timestamp: i64,
    pub commit_date: String,
    pub committer: String,
    pub worktree_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchStatus {
    Identical,
    UpstreamAhead,
    LocalAhead,
    Diverged,
    UpstreamGone,
    NoUpstream,
}

impl BranchStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Identical => "ok",
            Self::UpstreamAhead => "behind",
            Self::LocalAhead => "ahead",
            Self::Diverged => "diverged",
            Self::UpstreamGone => "gone",
            Self::NoUpstream => "no upstream",
        }
    }
}

pub struct GitReposListService<P: Prompt> {
    prompt: Option<P>,
}

impl<P: Prompt> GitReposListService<P> {
    pub fn new(prompt: Option<P>) -> Self {
        Self { prompt }
    }

    pub fn list_all_branches(&self, path: &Path) -> Result<TaskResult> {
        let mut entries = collect_branch_entries(path)?;
        entries.sort_by(|a, b| {
            a.commit_timestamp
                .cmp(&b.commit_timestamp)
                .then_with(|| a.repo_name.cmp(&b.repo_name))
                .then_with(|| a.refname.cmp(&b.refname))
        });

        print_entries(&entries);

        if let Some(prompt) = &self.prompt {
            if entries.is_empty() {
                println!("No branches found.");
                return Ok(TaskResult::Proceed);
            }
            let options: Vec<String> = entries.iter().map(format_entry_option).collect();
            let index = prompt.select("Select a branch to check out", &options)?;
            let entry = &entries[index];
            return select_entry(entry);
        }

        Ok(TaskResult::Proceed)
    }
}

fn collect_branch_entries(path: &Path) -> Result<Vec<BranchListEntry>> {
    let mut entries = Vec::new();
    for dir_entry in fs::read_dir(path)? {
        let dir_entry = dir_entry?;
        let entry_path = dir_entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        if is_globally_ignored(&entry_path) {
            continue;
        }
        let repo = GitRepo::new(entry_path.clone());
        let branches = match repo.get_branches() {
            Ok(branches) => branches,
            Err(_) => continue,
        };
        let commit_infos = match repo.branch_commit_infos() {
            Ok(infos) => infos,
            Err(_) => continue,
        };
        let repo_name = entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| entry_path.to_string_lossy().into_owned());

        for branch in branches {
            let Some(info) = commit_infos.get(&branch.refname) else {
                continue;
            };
            entries.push(BranchListEntry {
                repo_name: repo_name.clone(),
                repo_path: entry_path.clone(),
                refname: branch.refname.clone(),
                status: branch_status(&branch),
                commit_timestamp: info.commit_timestamp,
                commit_date: info.commit_date.clone(),
                committer: info.committer.clone(),
                worktree_path: branch.worktree_path.clone(),
            });
        }
    }
    Ok(entries)
}

fn branch_status(branch: &Branch) -> BranchStatus {
    match branch.upstream.as_ref() {
        None => BranchStatus::NoUpstream,
        Some(upstream) => match upstream.status {
            UpstreamStatus::Identical => BranchStatus::Identical,
            UpstreamStatus::UpstreamIsAheadOfLocal => BranchStatus::UpstreamAhead,
            UpstreamStatus::LocalIsAheadOfUpstream => BranchStatus::LocalAhead,
            UpstreamStatus::MergeNeeded => BranchStatus::Diverged,
            UpstreamStatus::UpstreamIsGone => BranchStatus::UpstreamGone,
        },
    }
}

fn print_entries(entries: &[BranchListEntry]) {
    if entries.is_empty() {
        return;
    }
    let status_width = entries
        .iter()
        .map(|entry| entry.status.label().len())
        .max()
        .unwrap_or(0);
    let committer_width = entries
        .iter()
        .map(|entry| entry.committer.chars().count())
        .max()
        .unwrap_or(0);
    let location_width = entries
        .iter()
        .map(|entry| entry.repo_name.chars().count() + 1 + entry.refname.chars().count())
        .max()
        .unwrap_or(0);
    for entry in entries {
        let location = format!("{}/{}", entry.repo_name, entry.refname);
        println!(
            "{date}  {status:<status_width$}  {committer:<committer_width$}  {location:<location_width$}",
            date = entry.commit_date,
            status = entry.status.label(),
            committer = entry.committer,
            location = location,
            status_width = status_width,
            committer_width = committer_width,
            location_width = location_width,
        );
    }
}

fn format_entry_option(entry: &BranchListEntry) -> String {
    format!(
        "{}  {:<11}  {:<20}  {}/{}",
        entry.commit_date,
        entry.status.label(),
        entry.committer,
        entry.repo_name,
        entry.refname,
    )
}

fn select_entry(entry: &BranchListEntry) -> Result<TaskResult> {
    if let Some(worktree_path) = &entry.worktree_path
        && !paths_equivalent(worktree_path, &entry.repo_path)
    {
        println!(
            "Branch '{}' is checked out in worktree {}",
            entry.refname,
            worktree_path.display()
        );
        return Ok(TaskResult::ShellActionRequired(worktree_path.clone()));
    }
    let repo = GitRepo::new(entry.repo_path.clone());
    repo.checkout_branch(&entry.refname)?;
    Ok(TaskResult::ShellActionRequired(entry.repo_path.clone()))
}

fn paths_equivalent(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a_can), Ok(b_can)) => a_can == b_can,
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, anyhow};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct TestPrompt {
        selections: Arc<Mutex<Vec<usize>>>,
    }

    impl TestPrompt {
        fn with_selections(selections: Vec<usize>) -> Self {
            Self {
                selections: Arc::new(Mutex::new(selections)),
            }
        }
    }

    impl Prompt for TestPrompt {
        fn select(&self, _message: &str, options: &[String]) -> Result<usize> {
            let mut selections = self.selections.lock().expect("lock poisoned");
            if selections.is_empty() {
                return Err(anyhow!("no selections remaining"));
            }
            let index = selections.remove(0);
            if index >= options.len() {
                return Err(anyhow!("selected index {index} out of bounds"));
            }
            Ok(index)
        }
    }

    fn entry(timestamp: i64, repo: &str, refname: &str) -> BranchListEntry {
        BranchListEntry {
            repo_name: repo.to_string(),
            repo_path: PathBuf::from("/tmp").join(repo),
            refname: refname.to_string(),
            status: BranchStatus::Identical,
            commit_timestamp: timestamp,
            commit_date: "2024-01-01".to_string(),
            committer: "alice".to_string(),
            worktree_path: None,
        }
    }

    #[test]
    fn entries_sort_oldest_first() {
        let mut entries = [
            entry(2000, "repo-a", "main"),
            entry(1000, "repo-b", "main"),
            entry(3000, "repo-a", "feature"),
        ];
        entries.sort_by(|a, b| {
            a.commit_timestamp
                .cmp(&b.commit_timestamp)
                .then_with(|| a.repo_name.cmp(&b.repo_name))
                .then_with(|| a.refname.cmp(&b.refname))
        });
        assert_eq!(entries[0].repo_name, "repo-b");
        assert_eq!(entries[1].repo_name, "repo-a");
        assert_eq!(entries[1].refname, "main");
        assert_eq!(entries[2].refname, "feature");
    }

    #[test]
    fn select_entry_redirects_to_worktree_path() -> Result<()> {
        let temp_repo = tempfile::tempdir()?;
        let temp_worktree = tempfile::tempdir()?;
        let mut entry = entry(1000, "repo", "feature");
        entry.repo_path = temp_repo.path().to_path_buf();
        entry.worktree_path = Some(temp_worktree.path().to_path_buf());

        let result = select_entry(&entry)?;
        match result {
            TaskResult::ShellActionRequired(path) => {
                assert_eq!(path, temp_worktree.path().to_path_buf());
            }
            TaskResult::Proceed => panic!("expected shell action"),
        }
        Ok(())
    }

    #[test]
    fn list_service_without_prompt_proceeds() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let service = GitReposListService::<TestPrompt>::new(None);
        let result = service.list_all_branches(temp.path())?;
        assert!(matches!(result, TaskResult::Proceed));
        Ok(())
    }

    #[test]
    fn empty_list_with_prompt_proceeds() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let service = GitReposListService::new(Some(TestPrompt::with_selections(vec![])));
        let result = service.list_all_branches(temp.path())?;
        assert!(matches!(result, TaskResult::Proceed));
        Ok(())
    }
}
