use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{oneshot, Mutex};

use crate::config::SharedConfig;

use super::terminal_stream::persist_chunk;
use super::{Runner, RunnerInput, RunnerOutput};

#[derive(Clone)]
pub struct CodexCliRunner {
    config: SharedConfig,
    pool: SqlitePool,
    active_cancellations: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
}

impl CodexCliRunner {
    pub fn new(config: SharedConfig, pool: SqlitePool) -> Self {
        Self {
            config,
            pool,
            active_cancellations: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Runner for CodexCliRunner {
    async fn run(&self, input: RunnerInput) -> Result<RunnerOutput> {
        let output_last_message_path = format!(
            "{}/.relay-codex-last-message-{}.txt",
            input.workspace_path, input.task_run_id
        );
        let mut command = Command::new(&self.config.codex.bin);
        command
            .arg("exec")
            .arg("--skip-git-repo-check")
            .arg("--color")
            .arg("never")
            .arg("-o")
            .arg(&output_last_message_path)
            .args(&self.config.codex.default_args)
            .arg(&input.prompt)
            .current_dir(&input.workspace_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().context("failed to spawn codex")?;
        let stdout = child.stdout.take().context("missing stdout pipe")?;
        let stderr = child.stderr.take().context("missing stderr pipe")?;
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.active_cancellations
            .lock()
            .await
            .insert(input.task_run_id.clone(), cancel_tx);

        let pool_stdout = self.pool.clone();
        let task_run_stdout = input.task_run_id.clone();
        let stdout_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut sequence = 0_i64;
            let mut combined = String::new();
            while let Some(line) = reader.next_line().await? {
                persist_chunk(
                    &pool_stdout,
                    &task_run_stdout,
                    "stdout",
                    &format!("{line}\n"),
                    sequence,
                )
                .await?;
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
                persist_chunk(
                    &pool_stderr,
                    &task_run_stderr,
                    "stderr",
                    &format!("{line}\n"),
                    sequence,
                )
                .await?;
                combined.push_str(&line);
                combined.push('\n');
                sequence += 1;
            }
            Ok::<String, anyhow::Error>(combined)
        });

        let wait_outcome = tokio::select! {
            wait_result = child.wait() => {
                let wait_status = wait_result.context("failed waiting for codex")?;
                WaitOutcome::Exited(wait_status.success(), wait_status.code().map(i64::from))
            }
            _ = tokio::time::sleep(Duration::from_secs(input.timeout_seconds.unwrap_or(self.config.codex.timeout_seconds))) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                WaitOutcome::TimedOut
            }
            _ = &mut cancel_rx => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                WaitOutcome::Cancelled
            }
        };

        self.active_cancellations
            .lock()
            .await
            .remove(&input.task_run_id);

        let combined_stdout = stdout_handle.await??;
        let stderr = stderr_handle.await??;
        let stdout = read_last_message(&output_last_message_path, &combined_stdout).await;

        let output = match wait_outcome {
            WaitOutcome::Exited(success, exit_code) => RunnerOutput {
                status: if success {
                    "succeeded".to_string()
                } else {
                    "failed".to_string()
                },
                exit_code,
                stdout,
                stderr,
            },
            WaitOutcome::TimedOut => RunnerOutput {
                status: "timed_out".to_string(),
                exit_code: None,
                stdout,
                stderr,
            },
            WaitOutcome::Cancelled => RunnerOutput {
                status: "cancelled".to_string(),
                exit_code: None,
                stdout,
                stderr,
            },
        };

        Ok(output)
    }

    async fn cancel(&self, task_run_id: &str) -> Result<bool> {
        let sender = self.active_cancellations.lock().await.remove(task_run_id);
        if let Some(sender) = sender {
            let _ = sender.send(());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn kind(&self) -> &'static str {
        "codex_cli"
    }
}

enum WaitOutcome {
    Exited(bool, Option<i64>),
    TimedOut,
    Cancelled,
}

async fn read_last_message(path: &str, fallback: &str) -> String {
    match fs::read_to_string(path).await {
        Ok(content) if !content.trim().is_empty() => {
            let _ = fs::remove_file(path).await;
            content
        }
        _ => fallback.to_string(),
    }
}
