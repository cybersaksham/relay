use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use tokio::fs;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::config::SharedConfig;

#[derive(Debug, Clone)]
pub struct PreparedWorkspace {
    pub workspace_id: String,
    pub workspace_path: PathBuf,
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
        self.delete_source_clone(previous_slug).await?;
        if previous_slug != next_slug {
            self.delete_source_clone(next_slug).await?;
        }
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
        })
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
}
