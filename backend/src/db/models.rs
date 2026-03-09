use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub git_ssh_url: String,
    pub default_branch: String,
    pub aliases: String,
    pub enabled: bool,
    pub source_sync_status: String,
    pub source_sync_error: Option<String>,
    pub source_synced_at: Option<DateTime<Utc>>,
    pub source_setup_script: Option<String>,
    pub workspace_setup_script: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub team_id: String,
    pub channel_id: String,
    pub thread_ts: String,
    pub workspace_id: String,
    pub workspace_path: String,
    pub environment_id: Option<String>,
    pub current_workflow_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRun {
    pub id: String,
    pub session_id: String,
    pub trigger_message_ts: String,
    pub status: String,
    pub workflow_id: Option<String>,
    pub workflow_name: Option<String>,
    pub runner_kind: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i64>,
    pub error_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskMessage {
    pub id: String,
    pub session_id: String,
    pub task_run_id: Option<String>,
    pub direction: String,
    pub slack_user_id: Option<String>,
    pub raw_payload: String,
    pub resolved_payload: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TerminalEvent {
    pub id: i64,
    pub task_run_id: String,
    pub stream: String,
    pub chunk: String,
    pub sequence: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnvironmentSyncEvent {
    pub id: i64,
    pub environment_id: String,
    pub stream: String,
    pub chunk: String,
    pub sequence: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PolicyViolation {
    pub id: String,
    pub slack_user_id: String,
    pub team_id: String,
    pub channel_id: String,
    pub thread_ts: String,
    pub rule_type: String,
    pub rule_id: String,
    pub request_excerpt: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ban {
    pub id: String,
    pub slack_user_id: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
