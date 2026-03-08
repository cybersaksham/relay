use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use tokio::process::Command;

pub async fn validate_remote_access(git_ssh_url: &str) -> Result<()> {
    let mut command = Command::new("git");
    command
        .arg("ls-remote")
        .arg(git_ssh_url)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let output = command.output().await.context("failed to execute git ls-remote")?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git ls-remote failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
