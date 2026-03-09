use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::{Ban, Environment, Session, TaskMessage, TaskRun, TerminalEvent};

pub async fn list_environments(pool: &SqlitePool) -> Result<Vec<Environment>> {
    Ok(
        sqlx::query_as::<_, Environment>("SELECT * FROM environments ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn count_environments(pool: &SqlitePool) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM environments")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn get_environment(pool: &SqlitePool, id: &str) -> Result<Option<Environment>> {
    Ok(
        sqlx::query_as::<_, Environment>("SELECT * FROM environments WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn get_environment_by_slug(pool: &SqlitePool, slug: &str) -> Result<Option<Environment>> {
    Ok(
        sqlx::query_as::<_, Environment>("SELECT * FROM environments WHERE slug = ?")
            .bind(slug)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn get_environment_by_slug_excluding_id(
    pool: &SqlitePool,
    slug: &str,
    exclude_id: &str,
) -> Result<Option<Environment>> {
    Ok(
        sqlx::query_as::<_, Environment>("SELECT * FROM environments WHERE slug = ? AND id != ?")
            .bind(slug)
            .bind(exclude_id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn insert_environment(
    pool: &SqlitePool,
    name: &str,
    slug: &str,
    git_ssh_url: &str,
    default_branch: &str,
    aliases: &str,
    enabled: bool,
) -> Result<Environment> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO environments (
            id, name, slug, git_ssh_url, default_branch, aliases, enabled, source_sync_status, source_sync_error, source_synced_at, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(slug)
    .bind(git_ssh_url)
    .bind(default_branch)
    .bind(aliases)
    .bind(enabled)
    .bind("pending")
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    get_environment(pool, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("environment insert missing row"))
}

pub async fn update_environment(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    slug: &str,
    git_ssh_url: &str,
    default_branch: &str,
    aliases: &str,
    enabled: bool,
) -> Result<Environment> {
    sqlx::query(
        "UPDATE environments
         SET name = ?, slug = ?, git_ssh_url = ?, default_branch = ?, aliases = ?, enabled = ?, source_sync_status = ?, source_sync_error = NULL, source_synced_at = NULL, updated_at = ?
         WHERE id = ?",
    )
    .bind(name)
    .bind(slug)
    .bind(git_ssh_url)
    .bind(default_branch)
    .bind(aliases)
    .bind(enabled)
    .bind("pending")
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;

    get_environment(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("environment update missing row"))
}

pub async fn update_environment_source_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    error: Option<&str>,
    synced_at: Option<chrono::DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(
        "UPDATE environments
         SET source_sync_status = ?, source_sync_error = ?, source_synced_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(status)
    .bind(error)
    .bind(synced_at)
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_environment(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM environments WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_sessions_for_environment(
    pool: &SqlitePool,
    environment_id: &str,
) -> Result<i64> {
    let count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions WHERE environment_id = ?")
            .bind(environment_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn record_slack_event(pool: &SqlitePool, event_id: &str) -> Result<bool> {
    let result =
        sqlx::query("INSERT OR IGNORE INTO slack_event_dedup (event_id, created_at) VALUES (?, ?)")
            .bind(event_id)
            .bind(Utc::now())
            .execute(pool)
            .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn get_session_by_thread(
    pool: &SqlitePool,
    team_id: &str,
    channel_id: &str,
    thread_ts: &str,
) -> Result<Option<Session>> {
    Ok(sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE team_id = ? AND channel_id = ? AND thread_ts = ?",
    )
    .bind(team_id)
    .bind(channel_id)
    .bind(thread_ts)
    .fetch_optional(pool)
    .await?)
}

pub async fn insert_session(
    pool: &SqlitePool,
    team_id: &str,
    channel_id: &str,
    thread_ts: &str,
    workspace_id: &str,
    workspace_path: &str,
    environment_id: Option<&str>,
    current_workflow_id: Option<&str>,
    status: &str,
) -> Result<Session> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO sessions (
            id, team_id, channel_id, thread_ts, workspace_id, workspace_path, environment_id, current_workflow_id, status, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(team_id)
    .bind(channel_id)
    .bind(thread_ts)
    .bind(workspace_id)
    .bind(workspace_path)
    .bind(environment_id)
    .bind(current_workflow_id)
    .bind(status)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    get_session(pool, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("session insert missing row"))
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Option<Session>> {
    Ok(
        sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn update_session_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    current_workflow_id: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE sessions SET status = ?, current_workflow_id = ?, updated_at = ? WHERE id = ?",
    )
    .bind(status)
    .bind(current_workflow_id)
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_task_run(
    pool: &SqlitePool,
    session_id: &str,
    trigger_message_ts: &str,
    workflow_id: Option<&str>,
    workflow_name: Option<&str>,
    runner_kind: &str,
    status: &str,
) -> Result<TaskRun> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO task_runs (
            id, session_id, trigger_message_ts, status, workflow_id, workflow_name, runner_kind, started_at, finished_at, exit_code, error_summary, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?)",
    )
    .bind(&id)
    .bind(session_id)
    .bind(trigger_message_ts)
    .bind(status)
    .bind(workflow_id)
    .bind(workflow_name)
    .bind(runner_kind)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    get_task_run(pool, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("task run insert missing row"))
}

pub async fn get_task_run(pool: &SqlitePool, id: &str) -> Result<Option<TaskRun>> {
    Ok(
        sqlx::query_as::<_, TaskRun>("SELECT * FROM task_runs WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_recent_task_runs(pool: &SqlitePool, limit: i64) -> Result<Vec<TaskRun>> {
    Ok(
        sqlx::query_as::<_, TaskRun>("SELECT * FROM task_runs ORDER BY created_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn list_recent_sessions(pool: &SqlitePool, limit: i64) -> Result<Vec<Session>> {
    Ok(
        sqlx::query_as::<_, Session>("SELECT * FROM sessions ORDER BY updated_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn list_task_runs_for_environment(
    pool: &SqlitePool,
    environment_id: &str,
) -> Result<Vec<TaskRun>> {
    Ok(sqlx::query_as::<_, TaskRun>(
        "SELECT tr.* FROM task_runs tr
         INNER JOIN sessions s ON s.id = tr.session_id
         WHERE s.environment_id = ?
         ORDER BY tr.created_at DESC",
    )
    .bind(environment_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_sessions_for_environment(
    pool: &SqlitePool,
    environment_id: &str,
) -> Result<Vec<Session>> {
    Ok(sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE environment_id = ? ORDER BY updated_at DESC",
    )
    .bind(environment_id)
    .fetch_all(pool)
    .await?)
}

pub async fn update_task_run_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    exit_code: Option<i64>,
    error_summary: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE task_runs SET status = ?, exit_code = ?, error_summary = ?, finished_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(status)
    .bind(exit_code)
    .bind(error_summary)
    .bind(Utc::now())
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn has_active_run(pool: &SqlitePool, session_id: &str) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM task_runs WHERE session_id = ? AND status IN ('queued', 'running', 'waiting_for_reply')",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn get_active_run_for_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<TaskRun>> {
    Ok(sqlx::query_as::<_, TaskRun>(
        "SELECT * FROM task_runs
         WHERE session_id = ? AND status IN ('queued', 'running', 'waiting_for_reply')
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn insert_task_message(
    pool: &SqlitePool,
    session_id: &str,
    task_run_id: Option<&str>,
    direction: &str,
    slack_user_id: Option<&str>,
    raw_payload: &str,
    resolved_payload: &str,
) -> Result<TaskMessage> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO task_messages (
            id, session_id, task_run_id, direction, slack_user_id, raw_payload, resolved_payload, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(session_id)
    .bind(task_run_id)
    .bind(direction)
    .bind(slack_user_id)
    .bind(raw_payload)
    .bind(resolved_payload)
    .bind(now)
    .execute(pool)
    .await?;
    get_task_message(pool, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("task message insert missing row"))
}

pub async fn get_task_message(pool: &SqlitePool, id: &str) -> Result<Option<TaskMessage>> {
    Ok(
        sqlx::query_as::<_, TaskMessage>("SELECT * FROM task_messages WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn get_task_messages_by_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<TaskMessage>> {
    Ok(sqlx::query_as::<_, TaskMessage>(
        "SELECT * FROM task_messages WHERE session_id = ? ORDER BY created_at ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_task_messages_by_run(
    pool: &SqlitePool,
    task_run_id: &str,
) -> Result<Vec<TaskMessage>> {
    Ok(sqlx::query_as::<_, TaskMessage>(
        "SELECT * FROM task_messages WHERE task_run_id = ? ORDER BY created_at ASC",
    )
    .bind(task_run_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_terminal_event(
    pool: &SqlitePool,
    task_run_id: &str,
    stream: &str,
    chunk: &str,
    sequence: i64,
) -> Result<TerminalEvent> {
    sqlx::query(
        "INSERT INTO terminal_events (task_run_id, stream, chunk, sequence, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(task_run_id)
    .bind(stream)
    .bind(chunk)
    .bind(sequence)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(sqlx::query_as::<_, TerminalEvent>(
        "SELECT * FROM terminal_events WHERE task_run_id = ? AND sequence = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(task_run_id)
    .bind(sequence)
    .fetch_one(pool)
    .await?)
}

pub async fn list_terminal_events_after(
    pool: &SqlitePool,
    task_run_id: &str,
    after_id: i64,
) -> Result<Vec<TerminalEvent>> {
    Ok(sqlx::query_as::<_, TerminalEvent>(
        "SELECT * FROM terminal_events WHERE task_run_id = ? AND id > ? ORDER BY id ASC",
    )
    .bind(task_run_id)
    .bind(after_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_policy_violation(
    pool: &SqlitePool,
    slack_user_id: &str,
    team_id: &str,
    channel_id: &str,
    thread_ts: &str,
    rule_type: &str,
    rule_id: &str,
    request_excerpt: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO policy_violations (
            id, slack_user_id, team_id, channel_id, thread_ts, rule_type, rule_id, request_excerpt, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(slack_user_id)
    .bind(team_id)
    .bind(channel_id)
    .bind(thread_ts)
    .bind(rule_type)
    .bind(rule_id)
    .bind(request_excerpt)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_recent_critical_violations(
    pool: &SqlitePool,
    slack_user_id: &str,
) -> Result<i64> {
    let since = Utc::now() - Duration::hours(24);
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM policy_violations WHERE slack_user_id = ? AND rule_type = 'critical_deny' AND created_at >= ?",
    )
    .bind(slack_user_id)
    .bind(since)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn insert_ban(pool: &SqlitePool, slack_user_id: &str, reason: &str) -> Result<()> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(24);
    sqlx::query("INSERT INTO bans (id, slack_user_id, reason, created_at, expires_at) VALUES (?, ?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string())
        .bind(slack_user_id)
        .bind(reason)
        .bind(now)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_active_ban(pool: &SqlitePool, slack_user_id: &str) -> Result<Option<Ban>> {
    Ok(sqlx::query_as::<_, Ban>(
        "SELECT * FROM bans WHERE slack_user_id = ? AND expires_at > ? ORDER BY expires_at DESC LIMIT 1",
    )
    .bind(slack_user_id)
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?)
}

pub async fn get_latest_run_for_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<TaskRun>> {
    Ok(sqlx::query_as::<_, TaskRun>(
        "SELECT * FROM task_runs WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_task_runs_for_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<TaskRun>> {
    Ok(sqlx::query_as::<_, TaskRun>(
        "SELECT * FROM task_runs WHERE session_id = ? ORDER BY created_at DESC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_task_run_with_session(
    pool: &SqlitePool,
    task_run_id: &str,
) -> Result<Option<(TaskRun, Session)>> {
    let run = get_task_run(pool, task_run_id).await?;
    if let Some(run) = run {
        let session = get_session(pool, &run.session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("task run session missing"))?;
        Ok(Some((run, session)))
    } else {
        Ok(None)
    }
}

pub async fn get_session_by_task_run(
    pool: &SqlitePool,
    task_run_id: &str,
) -> Result<Option<Session>> {
    let maybe_run = get_task_run(pool, task_run_id).await?;
    if let Some(run) = maybe_run {
        get_session(pool, &run.session_id).await
    } else {
        Ok(None)
    }
}
