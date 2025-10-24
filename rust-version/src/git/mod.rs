use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Branch {
    pub refname: String,
    pub upstream: Option<Upstream>,
}

impl Branch {
    pub fn needs_action(&self) -> bool {
        self.upstream
            .as_ref()
            .map(|upstream| upstream.status != UpstreamStatus::Identical)
            .unwrap_or(true)
    }
}

#[derive(Debug, Clone)]
pub struct Upstream {
    pub name: String,
    pub status: UpstreamStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamStatus {
    Identical,
    UpstreamIsAheadOfLocal,
    LocalIsAheadOfUpstream,
    MergeNeeded,
    UpstreamIsGone,
}

pub struct GitRepo {
    dir: PathBuf,
}

impl GitRepo {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn get_branches(&self) -> Result<Vec<Branch>> {
        let output = self.run_and_capture(
            "git",
            &["branch", "--format=%(refname:short):%(upstream:short)"],
        )?;

        let mut branches = Vec::new();
        for line in output.lines().filter(|line| !line.trim().is_empty()) {
            let mut parts = line.splitn(2, ':');
            let local = parts
                .next()
                .ok_or_else(|| anyhow!("unexpected output from git branch: {line}"))?
                .trim()
                .to_string();
            let upstream_part = parts.next().unwrap_or("").trim();
            let upstream = if upstream_part.is_empty() {
                None
            } else {
                let status = self.get_upstream_status(&local, upstream_part)?;
                Some(Upstream {
                    name: upstream_part.to_string(),
                    status,
                })
            };
            branches.push(Branch {
                refname: local,
                upstream,
            });
        }

        Ok(branches)
    }

    pub fn push(&self, refname: &str) -> Result<()> {
        self.run_interactive_printing("git", &["push", "origin", refname])
    }

    pub fn push_creating_origin(&self, refname: &str) -> Result<()> {
        self.run_interactive_printing("git", &["push", "--set-upstream", "origin", refname])
    }

    pub fn rebase(&self, refname: &str, upstream: &str) -> Result<()> {
        self.run_interactive_printing("git", &["rebase", upstream, refname])
    }

    pub fn delete_branch_forcefully(&self, branch: &str) -> Result<()> {
        self.run_interactive_printing("git", &["branch", "-D", branch])
    }

    pub fn create_pull_request(&self, refname: &str) -> Result<()> {
        let default_branch = self.default_branch()?;
        self.run_interactive_printing(
            "gh",
            &["pr", "create", "--head", refname, "--base", &default_branch],
        )
    }

    pub fn show_log(&self, branch: &str) -> Result<()> {
        self.run_interactive("tig", &[branch])
    }

    pub fn checkout_branch(&self, branch: &str) -> Result<()> {
        self.run_and_capture("git", &["checkout", branch])?;
        Ok(())
    }

    pub fn checkout_first_available_branch(&self, branches: &[&str]) -> Result<()> {
        for branch in branches {
            if self.run_interactive_status("git", &["checkout", branch])? {
                return Ok(());
            }
        }
        Err(anyhow!("no available branch could be checked out"))
    }

    fn get_upstream_status(&self, local: &str, upstream: &str) -> Result<UpstreamStatus> {
        let local_is_ancestor = self.is_ancestor(local, upstream)?;
        let upstream_is_ancestor = self.is_ancestor(upstream, local)?;
        Ok(match (local_is_ancestor, upstream_is_ancestor) {
            (true, true) => UpstreamStatus::Identical,
            (true, false) => UpstreamStatus::UpstreamIsAheadOfLocal,
            (false, true) => UpstreamStatus::LocalIsAheadOfUpstream,
            (false, false) => {
                if self.branch_exists(upstream)? {
                    UpstreamStatus::MergeNeeded
                } else {
                    UpstreamStatus::UpstreamIsGone
                }
            }
        })
    }

    fn is_ancestor(&self, base: &str, commit: &str) -> Result<bool> {
        let status = self
            .command("git")
            .arg("merge-base")
            .arg("--is-ancestor")
            .arg(base)
            .arg(commit)
            .status()
            .with_context(|| {
                format!("failed to run git merge-base --is-ancestor {base} {commit}")
            })?;

        match status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            Some(code) => Err(anyhow!(
                "git merge-base returned unexpected exit code {code}"
            )),
            None => Err(anyhow!("git merge-base terminated by signal")),
        }
    }

    fn branch_exists(&self, branch: &str) -> Result<bool> {
        let status = self
            .command("git")
            .arg("rev-parse")
            .arg("--quiet")
            .arg("--verify")
            .arg(branch)
            .status()
            .with_context(|| format!("failed to run git rev-parse --verify {branch}"))?;

        match status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            Some(code) => Err(anyhow!(
                "git rev-parse returned unexpected exit code {code}"
            )),
            None => Err(anyhow!("git rev-parse terminated by signal")),
        }
    }

    fn default_branch(&self) -> Result<String> {
        let output = self.run_and_capture("gh", &["repo", "view", "--json", "defaultBranchRef"])?;
        let response: DefaultBranchResponse =
            serde_json::from_str(&output).context("failed to parse gh repo view output")?;
        Ok(response.default_branch_ref.name.trim().to_string())
    }

    fn run_and_capture(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = self
            .command(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("failed to run {}", format_command(program, args)))?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout).context("command output was not valid UTF-8")?)
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!(
                "{} failed (stdout: {}, stderr: {})",
                format_command(program, args),
                stdout.trim(),
                stderr.trim()
            ))
        }
    }

    fn run_interactive(&self, program: &str, args: &[&str]) -> Result<()> {
        let status = self
            .command(program)
            .args(args)
            .status()
            .with_context(|| format!("failed to run {}", format_command(program, args)))?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "{} exited with status {}",
                format_command(program, args),
                status
            ))
        }
    }

    fn run_interactive_printing(&self, program: &str, args: &[&str]) -> Result<()> {
        println!("{}", format_command(program, args));
        self.run_interactive(program, args)
    }

    fn run_interactive_status(&self, program: &str, args: &[&str]) -> Result<bool> {
        let status = self
            .command(program)
            .args(args)
            .status()
            .with_context(|| format!("failed to run {}", format_command(program, args)))?;
        Ok(status.success())
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command.current_dir(&self.dir);
        command
    }
}

#[derive(Deserialize)]
struct DefaultBranchResponse {
    #[serde(rename = "defaultBranchRef")]
    default_branch_ref: DefaultBranchRef,
}

#[derive(Deserialize)]
struct DefaultBranchRef {
    name: String,
}

fn format_command(program: &str, args: &[&str]) -> String {
    let parts: Vec<&str> = std::iter::once(program)
        .chain(args.iter().copied())
        .collect();
    parts.join(" ")
}
