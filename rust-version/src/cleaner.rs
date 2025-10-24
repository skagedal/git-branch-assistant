use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::git::{Branch, GitRepo, UpstreamStatus};
use crate::task_result::TaskResult;
use crate::ui::{Choice, UserInterface};

#[derive(Clone)]
pub struct GitCleaner {
    ui: UserInterface,
}

impl GitCleaner {
    pub fn new(ui: UserInterface) -> Self {
        Self { ui }
    }

    pub fn handle(&self, repo: &GitRepo, branches: Vec<Branch>) -> Result<TaskResult> {
        let mut result = TaskResult::Proceed;
        for branch in branches {
            if !matches!(result, TaskResult::Proceed) {
                break;
            }
            result = self.handle_branch(repo, &branch)?;
        }
        Ok(result)
    }

    pub fn handle_branch(&self, repo: &GitRepo, branch: &Branch) -> Result<TaskResult> {
        if let Some(upstream) = &branch.upstream {
            match upstream.status {
                UpstreamStatus::Identical => Ok(TaskResult::Proceed),
                UpstreamStatus::UpstreamIsAheadOfLocal => {
                    repo.rebase(&branch.refname, &upstream.name)?;
                    Ok(TaskResult::Proceed)
                }
                UpstreamStatus::LocalIsAheadOfUpstream => self.select_action(
                    repo,
                    branch,
                    "Branch is ahead of upstream",
                    &[
                        BranchAction::Push,
                        BranchAction::Log,
                        BranchAction::Shell,
                        BranchAction::Nothing,
                    ],
                ),
                UpstreamStatus::MergeNeeded => self.select_action(
                    repo,
                    branch,
                    "Different commits on local and upstream",
                    &[
                        BranchAction::Rebase,
                        BranchAction::Log,
                        BranchAction::Delete,
                        BranchAction::Shell,
                        BranchAction::Nothing,
                    ],
                ),
                UpstreamStatus::UpstreamIsGone => self.select_action(
                    repo,
                    branch,
                    "Upstream is set, but it is gone",
                    &[
                        BranchAction::Delete,
                        BranchAction::Log,
                        BranchAction::Shell,
                        BranchAction::Nothing,
                    ],
                ),
            }
        } else {
            self.select_action(
                repo,
                branch,
                "Branch has no upstream",
                &[
                    BranchAction::CreatePr,
                    BranchAction::PushCreatingOrigin,
                    BranchAction::Delete,
                    BranchAction::Log,
                    BranchAction::Shell,
                    BranchAction::Nothing,
                ],
            )
        }
    }

    fn select_action(
        &self,
        repo: &GitRepo,
        branch: &Branch,
        message: &str,
        actions: &[BranchAction],
    ) -> Result<TaskResult> {
        loop {
            let repo_display = repo
                .dir()
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
                .unwrap_or_else(|| repo.dir().to_string_lossy().into_owned());

            let prompt = format!("{}:{}: {}", repo_display, branch.refname, message);
            let choices: Vec<Choice<BranchAction>> = actions
                .iter()
                .copied()
                .map(|action| Choice::new(action, action.description()))
                .collect();

            let action = self.ui.pick_one(&prompt, &choices)?;
            match self.perform_action(repo, branch, action)? {
                ActionResult::Handled => return Ok(TaskResult::Proceed),
                ActionResult::NotHandled => continue,
                ActionResult::ExitToShell(path) => {
                    return Ok(TaskResult::ShellActionRequired(path));
                }
            }
        }
    }

    fn perform_action(
        &self,
        repo: &GitRepo,
        branch: &Branch,
        action: BranchAction,
    ) -> Result<ActionResult> {
        match action {
            BranchAction::CreatePr => {
                repo.push_creating_origin(&branch.refname)?;
                repo.create_pull_request(&branch.refname)?;
                Ok(ActionResult::Handled)
            }
            BranchAction::Push => {
                repo.push(&branch.refname)?;
                Ok(ActionResult::Handled)
            }
            BranchAction::PushCreatingOrigin => {
                repo.push_creating_origin(&branch.refname)?;
                Ok(ActionResult::Handled)
            }
            BranchAction::Rebase => {
                let upstream = branch
                    .upstream
                    .as_ref()
                    .ok_or_else(|| anyhow!("branch has no upstream to rebase onto"))?;
                repo.rebase(&branch.refname, &upstream.name)?;
                Ok(ActionResult::Handled)
            }
            BranchAction::Delete => {
                repo.checkout_first_available_branch(&["release", "master", "main"])?;
                repo.delete_branch_forcefully(&branch.refname)?;
                Ok(ActionResult::Handled)
            }
            BranchAction::Log => {
                repo.show_log(&branch.refname)?;
                Ok(ActionResult::NotHandled)
            }
            BranchAction::Shell => {
                repo.checkout_branch(&branch.refname)?;
                Ok(ActionResult::ExitToShell(repo.dir().to_path_buf()))
            }
            BranchAction::Nothing => Ok(ActionResult::Handled),
        }
    }
}

#[derive(Clone, Copy)]
enum BranchAction {
    Push,
    PushCreatingOrigin,
    CreatePr,
    Rebase,
    Delete,
    Log,
    Shell,
    Nothing,
}

impl BranchAction {
    fn description(&self) -> &'static str {
        match self {
            BranchAction::Push => "Push to origin",
            BranchAction::PushCreatingOrigin => "Push to create origin",
            BranchAction::CreatePr => "Push and create pull request",
            BranchAction::Rebase => "Rebase onto origin",
            BranchAction::Delete => "Delete it",
            BranchAction::Log => "Show git log",
            BranchAction::Shell => "Exit to shell with branch checked out",
            BranchAction::Nothing => "Do nothing",
        }
    }
}

enum ActionResult {
    Handled,
    NotHandled,
    ExitToShell(PathBuf),
}
