use std::process::Stdio;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::config::SharedConfig;

use super::terminal_stream::persist_chunk;
use super::{Runner, RunnerInput, RunnerOutput};

#[derive(Clone)]
pub struct CodexCliRunner {
    config: SharedConfig,
    pool: SqlitePool,
}

impl CodexCliRunner {
    pub fn new(config: SharedConfig, pool: SqlitePool) -> Self {
        Self { config, pool }
    }
}

#[async_trait]
impl Runner for CodexCliRunner {
    async fn run(&self, input: RunnerInput) -> Result<RunnerOutput> {
        let mut command = Command::new(&self.config.codex.bin);
        command
            .args(&self.config.codex.default_args)
            .arg(&input.prompt)
            .current_dir(&input.workspace_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().context("failed to spawn codex")?;
        let stdout = child.stdout.take().context("missing stdout pipe")?;
        let stderr = child.stderr.take().context("missing stderr pipe")?;

        let pool_stdout = self.pool.clone();
        let task_run_stdout = input.task_run_id.clone();
        let stdout_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut sequence = 0_i64;
            let mut combined = String::new();
            while let Some(line) = reader.next_line().await? {
                persist_chunk(&pool_stdout, &task_run_stdout, "stdout", &format!("{line}\n"), sequence).await?;
                combined.push_str(&line);
                combined.push('\n');
                sequence += 1;
            }
            Ok::<String, anyhow::Error>(combined)
        });

        let pool_stderr = self.pool.clone();
        let task_run_stderr = input.task_run_id.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            let mut sequence = 0_i64;
            let mut combined = String::new();
            while let Some(line) = reader.next_line().await? {
                persist_chunk(&pool_stderr, &task_run_stderr, "stderr", &format!("{line}\n"), sequence).await?;
                combined.push_str(&line);
                combined.push('\n');
                sequence += 1;
            }
            Ok::<String, anyhow::Error>(combined)
        });

        let status = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.codex.timeout_seconds),
            child.wait(),
        )
        .await;

        match status {
            Ok(wait_result) => {
                let wait_status = wait_result?;
                let stdout = stdout_handle.await??;
                let stderr = stderr_handle.await??;
                let success = wait_status.success();
                Ok(RunnerOutput {
                    status: if success { "succeeded" } else { "failed" }.to_string(),
                    exit_code: wait_status.code().map(i64::from),
                    stdout,
                    stderr,
                })
            }
            Err(_) => {
                let _ = child.kill().await;
                let stdout = stdout_handle.await??;
                let stderr = stderr_handle.await??;
                Ok(RunnerOutput {
                    status: "timed_out".to_string(),
                    exit_code: None,
                    stdout,
                    stderr,
                })
            }
        }
    }

    fn kind(&self) -> &'static str {
        "codex_cli"
    }
}
