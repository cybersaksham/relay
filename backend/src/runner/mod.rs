pub mod codex_cli;
pub mod terminal_stream;

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RunnerInput {
    pub task_run_id: String,
    pub workspace_path: String,
    pub prompt: String,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RunnerOutput {
    pub status: String,
    pub exit_code: Option<i64>,
    pub stdout: String,
    pub stderr: String,
}

#[async_trait]
pub trait Runner: Send + Sync {
    async fn run(&self, input: RunnerInput) -> anyhow::Result<RunnerOutput>;
    async fn cancel(&self, task_run_id: &str) -> anyhow::Result<bool>;
    fn kind(&self) -> &'static str;
}
