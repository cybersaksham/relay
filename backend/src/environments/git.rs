use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn validate_remote_access(git_ssh_url: &str, branch: &str) -> Result<()> {
    let mut command = Command::new("git");
    command
        .arg("ls-remote")
        .arg("--exit-code")
        .arg("--heads")
        .arg(git_ssh_url)
        .arg(branch)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let output = timeout(Duration::from_secs(30), command.output())
        .await
        .map_err(|_| anyhow!("git ls-remote timed out after 30 seconds"))?
        .context("failed to execute git ls-remote")?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git ls-remote failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
