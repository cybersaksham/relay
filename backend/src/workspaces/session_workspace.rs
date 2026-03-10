use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use tokio::fs;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::config::SharedConfig;

#[derive(Debug, Clone)]
pub struct PreparedWorkspace {
    pub workspace_id: String,
    pub workspace_path: PathBuf,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct HookRunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i64>,
    pub timed_out: bool,
}

impl HookRunResult {
    pub fn succeeded(&self) -> bool {
        !self.timed_out && self.exit_code == Some(0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceGitDiff {
    pub available: bool,
    pub reason: Option<String>,
    pub files: Vec<WorkspaceGitDiffFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceGitDiffFile {
    pub path: String,
    pub status: String,
    pub staged: bool,
    pub can_stage: bool,
    pub diff: String,
}

#[derive(Clone)]
pub struct WorkspaceManager {
    config: SharedConfig,
}

impl WorkspaceManager {
    pub fn new(config: SharedConfig) -> Self {
        Self { config }
    }

    pub fn source_path(&self, env_slug: &str) -> PathBuf {
        self.config.paths.sources_dir.join(env_slug)
    }

    pub async fn delete_source_clone(&self, env_slug: &str) -> Result<()> {
        let path = self.source_path(env_slug);
        if fs::metadata(&path).await.is_ok() {
            fs::remove_dir_all(&path).await?;
        }
        Ok(())
    }

    pub async fn reset_source_clone(
        &self,
        previous_slug: &str,
        next_slug: &str,
        git_ssh_url: &str,
        default_branch: &str,
    ) -> Result<PathBuf> {
        if previous_slug == next_slug {
            return self
                .ensure_source_clone(next_slug, git_ssh_url, default_branch)
                .await;
        }

        self.delete_source_clone(previous_slug).await?;
        self.delete_source_clone(next_slug).await?;
        self.ensure_source_clone(next_slug, git_ssh_url, default_branch)
            .await
    }

    pub fn environment_workspace_path(&self, env_slug: &str, workspace_id: &str) -> PathBuf {
        self.config
            .paths
            .workspaces_dir
            .join(env_slug)
            .join(workspace_id)
    }

    pub fn general_workspace_path(&self, workspace_id: &str) -> PathBuf {
        self.config
            .paths
            .workspaces_dir
            .join("general")
            .join(workspace_id)
    }

    pub async fn ensure_source_clone(
        &self,
        env_slug: &str,
        git_ssh_url: &str,
        default_branch: &str,
    ) -> Result<PathBuf> {
        let path = self.source_path(env_slug);
        if path.exists() {
            if self
                .sync_existing_source_clone(&path, default_branch)
                .await
                .is_err()
            {
                self.delete_source_clone(env_slug).await?;
                return self
                    .fresh_clone_source(&path, git_ssh_url, default_branch)
                    .await;
            }
            return Ok(path);
        }

        self.fresh_clone_source(&path, git_ssh_url, default_branch)
            .await
    }

    async fn sync_existing_source_clone(&self, path: &Path, default_branch: &str) -> Result<()> {
        if fs::metadata(path.join(".git")).await.is_err() {
            return Err(anyhow!("existing source path is not a git repository"));
        }

        self.git(
            path,
            ["fetch", "--depth", "1", "--prune", "origin", default_branch],
        )
        .await?;
        self.git(path, ["reset", "--hard", "HEAD"]).await?;
        self.git(path, ["clean", "-fdx"]).await?;
        self.git(
            path,
            [
                "checkout",
                "-B",
                default_branch,
                &format!("origin/{default_branch}"),
            ],
        )
        .await?;
        self.git(
            path,
            ["reset", "--hard", &format!("origin/{default_branch}")],
        )
        .await?;
        self.git(path, ["clean", "-fdx"]).await?;
        Ok(())
    }

    async fn fresh_clone_source(
        &self,
        path: &Path,
        git_ssh_url: &str,
        default_branch: &str,
    ) -> Result<PathBuf> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut command = Command::new("git");
        command
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(default_branch)
            .arg("--single-branch")
            .arg(git_ssh_url)
            .arg(path);
        let output = timeout(Duration::from_secs(900), command.output())
            .await
            .map_err(|_| anyhow!("git clone timed out after 15 minutes"))?
            .context("failed to clone source repo")?;
        if !output.status.success() {
            return Err(anyhow!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(path.to_path_buf())
    }

    pub async fn prepare_repo_workspace(
        &self,
        env_slug: &str,
        source_path: &Path,
        workspace_id: Option<&str>,
    ) -> Result<PreparedWorkspace> {
        let workspace_id = workspace_id
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let workspace_path = self.environment_workspace_path(env_slug, &workspace_id);

        if workspace_path.exists() {
            return Ok(PreparedWorkspace {
                workspace_id,
                workspace_path,
                created: false,
            });
        }

        if let Some(parent) = workspace_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut command = Command::new("git");
        command.arg("clone").arg(source_path).arg(&workspace_path);
        let output = timeout(Duration::from_secs(300), command.output())
            .await
            .map_err(|_| anyhow!("git clone from source timed out after 5 minutes"))?
            .context("failed to clone local source workspace")?;
        if !output.status.success() {
            return Err(anyhow!(
                "git clone from source failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(PreparedWorkspace {
            workspace_id,
            workspace_path,
            created: true,
        })
    }

    pub async fn prepare_general_workspace(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<PreparedWorkspace> {
        let workspace_id = workspace_id
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let workspace_path = self.general_workspace_path(&workspace_id);
        fs::create_dir_all(&workspace_path).await?;
        Ok(PreparedWorkspace {
            workspace_id,
            workspace_path,
            created: true,
        })
    }

    pub async fn run_shell_hook(&self, cwd: &Path, script: &str) -> HookRunResult {
        let trimmed = script.trim();
        if trimmed.is_empty() {
            return HookRunResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: Some(0),
                timed_out: false,
            };
        }

        let mut command = Command::new("/bin/sh");
        command
            .arg("-lc")
            .arg(trimmed)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let timeout_seconds = self.config.hooks.timeout_seconds;
        match timeout(Duration::from_secs(timeout_seconds), command.output()).await {
            Ok(Ok(output)) => HookRunResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().map(i64::from),
                timed_out: false,
            },
            Ok(Err(error)) => HookRunResult {
                stdout: String::new(),
                stderr: format!("failed to execute script: {error}"),
                exit_code: None,
                timed_out: false,
            },
            Err(_) => HookRunResult {
                stdout: String::new(),
                stderr: format!("script timed out after {timeout_seconds} seconds"),
                exit_code: None,
                timed_out: true,
            },
        }
    }

    pub async fn inspect_git_diff(&self, workspace_path: &Path) -> Result<WorkspaceGitDiff> {
        if self
            .git_output(workspace_path, ["rev-parse", "--show-toplevel"])
            .await
            .is_err()
        {
            return Ok(WorkspaceGitDiff {
                available: false,
                reason: Some("Git is not configured for this workspace.".to_string()),
                files: Vec::new(),
            });
        }

        let status_output = self
            .git_output(
                workspace_path,
                ["status", "--porcelain=v1", "--untracked-files=all"],
            )
            .await?;
        let mut files = Vec::new();

        for line in status_output.lines() {
            if line.len() < 4 {
                continue;
            }

            let index_status = line.as_bytes()[0] as char;
            let worktree_status = line.as_bytes()[1] as char;
            if index_status == '!' && worktree_status == '!' {
                continue;
            }

            let raw_path = line[3..].trim();
            let path = parse_status_path(raw_path);
            let status = describe_git_status(index_status, worktree_status).to_string();
            let staged = index_status != ' ' && index_status != '?';
            let can_stage =
                worktree_status != ' ' || (index_status == '?' && worktree_status == '?');
            let diff = if index_status == '?' && worktree_status == '?' {
                self.untracked_diff(workspace_path, &path).await?
            } else {
                self.git_output(
                    workspace_path,
                    ["diff", "--no-ext-diff", "HEAD", "--", &path],
                )
                .await?
            };

            files.push(WorkspaceGitDiffFile {
                path,
                status,
                staged,
                can_stage,
                diff,
            });
        }

        Ok(WorkspaceGitDiff {
            available: true,
            reason: None,
            files,
        })
    }

    pub async fn stage_git_file(&self, workspace_path: &Path, file_path: &str) -> Result<()> {
        let relative_path = sanitize_workspace_relative_path(file_path)?;
        self.git(
            workspace_path,
            [
                OsStr::new("add"),
                OsStr::new("--"),
                relative_path.as_os_str(),
            ],
        )
        .await
    }

    pub async fn revert_git_file(&self, workspace_path: &Path, file_path: &str) -> Result<()> {
        let relative_path = sanitize_workspace_relative_path(file_path)?;
        if self
            .git_output(
                workspace_path,
                [
                    OsStr::new("ls-files"),
                    OsStr::new("--error-unmatch"),
                    OsStr::new("--"),
                    relative_path.as_os_str(),
                ],
            )
            .await
            .is_ok()
        {
            self.git(
                workspace_path,
                [
                    OsStr::new("restore"),
                    OsStr::new("--source=HEAD"),
                    OsStr::new("--staged"),
                    OsStr::new("--worktree"),
                    OsStr::new("--"),
                    relative_path.as_os_str(),
                ],
            )
            .await?;
            return Ok(());
        }

        let absolute_path = workspace_path.join(&relative_path);
        match fs::metadata(&absolute_path).await {
            Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(&absolute_path).await?,
            Ok(_) => fs::remove_file(&absolute_path).await?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        Ok(())
    }

    async fn git<I, S>(&self, cwd: &Path, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let output = timeout(Duration::from_secs(300), command.output())
            .await
            .map_err(|_| anyhow!("git command timed out after 5 minutes"))?
            .context("failed to run git command")?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    async fn git_output<I, S>(&self, cwd: &Path, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = self.run_git_command(cwd, args).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!(
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    async fn git_output_allow_failure<I, S>(
        &self,
        cwd: &Path,
        args: I,
        allowed_exit_codes: &[i32],
    ) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = self.run_git_command(cwd, args).await?;
        if output.status.success()
            || output
                .status
                .code()
                .is_some_and(|code| allowed_exit_codes.contains(&code))
        {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!(
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    async fn run_git_command<I, S>(&self, cwd: &Path, args: I) -> Result<std::process::Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        timeout(Duration::from_secs(300), command.output())
            .await
            .map_err(|_| anyhow!("git command timed out after 5 minutes"))?
            .context("failed to run git command")
    }

    async fn untracked_diff(&self, cwd: &Path, path: &str) -> Result<String> {
        self.git_output_allow_failure(
            cwd,
            [
                "diff",
                "--no-index",
                "--no-ext-diff",
                "--",
                "/dev/null",
                path,
            ],
            &[1],
        )
        .await
    }
}

fn sanitize_workspace_relative_path(file_path: &str) -> Result<PathBuf> {
    let path = Path::new(file_path);
    if path.is_absolute() {
        return Err(anyhow!("workspace file path must be relative"));
    }

    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => cleaned.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(anyhow!(
                    "workspace file path cannot traverse parent directories"
                ));
            }
            _ => return Err(anyhow!("workspace file path is invalid")),
        }
    }

    if cleaned.as_os_str().is_empty() {
        return Err(anyhow!("workspace file path cannot be empty"));
    }

    Ok(cleaned)
}

fn parse_status_path(raw_path: &str) -> String {
    let path = raw_path
        .rsplit_once(" -> ")
        .map(|(_, next)| next)
        .unwrap_or(raw_path)
        .trim();
    path.trim_matches('"').to_string()
}

fn describe_git_status(index_status: char, worktree_status: char) -> &'static str {
    match (index_status, worktree_status) {
        ('?', '?') => "untracked",
        ('A', _) | (_, 'A') => "added",
        ('D', _) | (_, 'D') => "deleted",
        ('R', _) | (_, 'R') => "renamed",
        ('C', _) | (_, 'C') => "copied",
        ('U', _) | (_, 'U') => "conflicted",
        ('T', _) | (_, 'T') => "type changed",
        ('M', _) | (_, 'M') => "modified",
        _ => "changed",
    }
}
