#[cfg(test)]
mod tests {
    use super::super::parse_worktrees;
    use crate::git::{Branch, GitRepo, Upstream, UpstreamStatus};
    use anyhow::Result;
    use std::path::PathBuf;
    use std::process::Command;

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

    fn test_repo(repo_name: &str) -> Result<GitRepo> {
        let temp_dir = tempfile::tempdir()?;
        let tarball_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("resources")
            .join(format!("{}.tar.gz", repo_name));

        let status = Command::new("tar")
            .arg("xzf")
            .arg(&tarball_path)
            .current_dir(temp_dir.path())
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("tar extraction failed"));
        }

        let repo_path = temp_dir.path().join(repo_name);
        let _ = temp_dir.keep();
        Ok(GitRepo::new(repo_path))
    }

    #[test]
    fn test_getting_branches() -> Result<()> {
        let repo = test_repo("repo-with-some-branches")?;
        let branches = repo.get_branches()?;
        let mut refnames: Vec<String> = branches.iter().map(|b| b.refname.clone()).collect();
        refnames.sort();

        assert_eq!(refnames, vec!["existing", "master"]);
        Ok(())
    }
}
