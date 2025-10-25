#[cfg(test)]
mod tests {
    use super::super::parse_worktrees;
    use crate::git::{Branch, GitRepo, Upstream, UpstreamStatus};
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn branch_needs_action_when_no_upstream() {
        let branch = Branch {
            refname: "feature".to_string(),
            upstream: None,
            worktree_path: None,
        };
        assert!(branch.needs_action());
    }

    #[test]
    fn branch_needs_action_when_upstream_status_not_identical() {
        let branch = Branch {
            refname: "feature".to_string(),
            upstream: Some(Upstream {
                name: "origin/feature".to_string(),
                status: UpstreamStatus::MergeNeeded,
            }),
            worktree_path: None,
        };
        assert!(branch.needs_action());
    }

    #[test]
    fn branch_no_action_when_identical() {
        let branch = Branch {
            refname: "feature".to_string(),
            upstream: Some(Upstream {
                name: "origin/feature".to_string(),
                status: UpstreamStatus::Identical,
            }),
            worktree_path: None,
        };
        assert!(!branch.needs_action());
    }

    #[test]
    fn git_repo_dir_returns_path() {
        let dir = PathBuf::from("/tmp/example");
        let repo = GitRepo::new(dir.clone());
        assert_eq!(repo.dir(), dir.as_path());
    }

    #[test]
    fn parse_worktrees_extracts_branches() -> Result<()> {
        let output = "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\nworktree /repo/feature\nHEAD def\nbranch refs/heads/feature\n";
        let worktrees = parse_worktrees(output)?;
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].path, PathBuf::from("/repo"));
        assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
        assert_eq!(worktrees[1].path, PathBuf::from("/repo/feature"));
        assert_eq!(worktrees[1].branch.as_deref(), Some("feature"));
        Ok(())
    }
}
