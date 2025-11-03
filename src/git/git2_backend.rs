use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use git2::{BranchType, ErrorCode, Repository, Status, StatusOptions};

use super::{Branch, GitRepo, Upstream, UpstreamStatus, Worktree};
use crate::services::git_repos_service::GitResult;

pub(super) fn get_branches(repo: &GitRepo) -> Result<Vec<Branch>> {
    let repository = open_repository(repo.dir())?;
    let worktree_map = branch_worktree_map(&repository)?;

    let mut branches = Vec::new();
    let mut iter = repository
        .branches(Some(BranchType::Local))
        .context("failed to enumerate local branches")?;

    while let Some(branch) = iter.next() {
        let (branch, _) = branch.context("failed to read branch")?;
        let name = branch_name(&branch)?;
        let worktree_path = worktree_map.get(&name).cloned();
        let upstream = upstream_for_branch(&repository, &branch, &name)?;

        branches.push(Branch {
            refname: name,
            upstream,
            worktree_path,
        });
    }

    Ok(branches)
}

pub(super) fn worktrees(repo: &GitRepo) -> Result<Vec<Worktree>> {
    let repository = open_repository(repo.dir())?;

    let mut worktrees = Vec::new();
    if let Some(path) = repository.workdir() {
        let branch = current_branch_name(&repository)?;
        worktrees.push(Worktree {
            path: path.to_path_buf(),
            branch,
        });
    }

    for name in repository
        .worktrees()
        .context("failed to enumerate linked worktrees")?
    {
        let worktree = repository
            .find_worktree(&name)
            .with_context(|| format!("failed to open worktree {name}"))?;
        let path = worktree
            .path()
            .ok_or_else(|| anyhow!("worktree {name} missing path"))?
            .to_path_buf();
        let worktree_repo = Repository::open(&path)
            .with_context(|| format!("failed to open worktree repository at {}", path.display()))?;
        let branch = current_branch_name(&worktree_repo)?;
        worktrees.push(Worktree { path, branch });
    }

    Ok(worktrees)
}

pub(super) fn find_dirty_worktree(repo: &GitRepo) -> Result<Option<Worktree>> {
    for worktree in worktrees(repo)? {
        if matches!(worktree_status(&worktree.path)?, GitResult::Dirty(_)) {
            return Ok(Some(worktree));
        }
    }
    Ok(None)
}

pub(super) fn worktree_status(path: &Path) -> Result<GitResult> {
    let repository = match Repository::open(path) {
        Ok(repo) => repo,
        Err(err) if err.code() == ErrorCode::NotFound || err.code() == ErrorCode::InvalidSpec => {
            return Ok(GitResult::NotGitRepository)
        }
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to open repository at {}", path.display()))
        }
    };

    let mut options = StatusOptions::new();
    options.include_untracked(true);
    options.recurse_untracked_dirs(true);

    let statuses = repository
        .statuses(Some(&mut options))
        .with_context(|| format!("failed to read status for {}", path.display()))?;

    let is_clean = statuses
        .iter()
        .all(|entry| entry.status() == Status::CURRENT);

    Ok(if is_clean {
        GitResult::Clean
    } else {
        GitResult::Dirty(path.to_path_buf())
    })
}

fn open_repository(path: &Path) -> Result<Repository> {
    Repository::open(path)
        .with_context(|| format!("failed to open repository at {}", path.display()))
}

fn branch_worktree_map(repo: &Repository) -> Result<HashMap<String, PathBuf>> {
    let mut map = HashMap::new();

    if let Some(path) = repo.workdir() {
        if let Some(branch) = current_branch_name(repo)? {
            map.insert(branch, path.to_path_buf());
        }
    }

    for name in repo
        .worktrees()
        .context("failed to enumerate linked worktrees")?
    {
        let worktree = repo
            .find_worktree(&name)
            .with_context(|| format!("failed to open worktree {name}"))?;
        if let Some(path) = worktree.path() {
            let repo = Repository::open(path).with_context(|| {
                format!(
                    "failed to open repository for worktree {} at {}",
                    name,
                    path.display()
                )
            })?;
            if let Some(branch) = current_branch_name(&repo)? {
                map.insert(branch, path.to_path_buf());
            }
        }
    }

    Ok(map)
}

fn current_branch_name(repo: &Repository) -> Result<Option<String>> {
    match repo.head() {
        Ok(head) => {
            if head.is_branch() {
                Ok(head
                    .shorthand()
                    .map(|name| name.to_string()))
            } else {
                Ok(None)
            }
        }
        Err(err) if err.code() == ErrorCode::NotFound || err.code() == ErrorCode::UnbornBranch => {
            Ok(None)
        }
        Err(err) => Err(err).context("failed to resolve HEAD"),
    }
}

fn branch_name(branch: &git2::Branch<'_>) -> Result<String> {
    match branch.name() {
        Ok(Some(name)) => Ok(name.to_string()),
        Ok(None) => Ok(String::from_utf8_lossy(
            branch
                .name_bytes()
                .context("branch name missing bytes")?,
        )
        .into_owned()),
        Err(err) => Err(err).context("failed to read branch name"),
    }
}

fn upstream_for_branch(
    repo: &Repository,
    branch: &git2::Branch<'_>,
    local_name: &str,
) -> Result<Option<Upstream>> {
    let upstream_name = match branch.upstream_name() {
        Ok(Some(name)) => String::from_utf8_lossy(&name).into_owned(),
        Ok(None) => return Ok(None),
        Err(err) if err.code() == ErrorCode::NotFound => return Ok(None),
        Err(err) => return Err(err).context("failed to resolve upstream configuration"),
    };

    match branch.upstream() {
        Ok(upstream_branch) => {
            let upstream_short = branch_name(&upstream_branch)?;
            let status = compute_upstream_status(repo, branch, &upstream_branch, local_name)?;
            Ok(Some(Upstream {
                name: upstream_short,
                status,
            }))
        }
        Err(err) if err.code() == ErrorCode::NotFound => Ok(Some(Upstream {
            name: short_upstream_name(&upstream_name),
            status: UpstreamStatus::UpstreamIsGone,
        })),
        Err(err) => Err(err).context("failed to load upstream branch"),
    }
}

fn compute_upstream_status(
    repo: &Repository,
    local: &git2::Branch<'_>,
    upstream: &git2::Branch<'_>,
    local_name: &str,
) -> Result<UpstreamStatus> {
    let local_oid = local
        .get()
        .target()
        .ok_or_else(|| anyhow!("local branch {local_name} has no commit"))?;
    let upstream_name = branch_name(upstream)?;
    let upstream_oid = upstream
        .get()
        .target()
        .ok_or_else(|| anyhow!("upstream branch {upstream_name} has no commit"))?;

    let upstream_is_ancestor = repo
        .graph_descendant_of(local_oid, upstream_oid)
        .with_context(|| format!("failed to compare commits for upstream {upstream_name}"))?;
    let local_is_ancestor = repo
        .graph_descendant_of(upstream_oid, local_oid)
        .with_context(|| format!("failed to compare commits for local {local_name}"))?;

    Ok(match (local_is_ancestor, upstream_is_ancestor) {
        (true, true) => UpstreamStatus::Identical,
        (true, false) => UpstreamStatus::UpstreamIsAheadOfLocal,
        (false, true) => UpstreamStatus::LocalIsAheadOfUpstream,
        (false, false) => UpstreamStatus::MergeNeeded,
    })
}

fn short_upstream_name(name: &str) -> String {
    name.strip_prefix("refs/heads/")
        .or_else(|| name.strip_prefix("refs/remotes/"))
        .map(|rest| rest.to_string())
        .unwrap_or_else(|| name.to_string())
}
