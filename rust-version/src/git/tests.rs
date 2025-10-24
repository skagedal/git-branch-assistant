#[cfg(test)]
mod tests {
    use crate::git::{Branch, GitRepo, Upstream, UpstreamStatus};
    use std::path::PathBuf;

    #[test]
    fn branch_needs_action_when_no_upstream() {
        let branch = Branch {
            refname: "feature".to_string(),
            upstream: None,
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
        };
        assert!(!branch.needs_action());
    }

    #[test]
    fn git_repo_dir_returns_path() {
        let dir = PathBuf::from("/tmp/example");
        let repo = GitRepo::new(dir.clone());
        assert_eq!(repo.dir(), dir.as_path());
    }
}
