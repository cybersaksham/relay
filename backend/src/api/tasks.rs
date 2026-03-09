use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::db::models::{Session, TaskMessage, TaskRun};
use crate::db::queries;
use crate::workspaces::session_workspace::WorkspaceGitDiff;

#[derive(Debug, Serialize)]
pub struct SessionSummaryResponse {
    pub session: Session,
    pub latest_run: Option<TaskRun>,
    pub run_count: usize,
}

#[derive(Debug, Serialize)]
pub struct TaskDetailResponse {
    pub session: Session,
    pub latest_run: Option<TaskRun>,
    pub runs: Vec<TaskRun>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub environment_count: i64,
    pub recent_sessions: Vec<SessionSummaryResponse>,
}

#[derive(Debug, Serialize)]
pub struct CancelTaskResponse {
    pub task_run_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceGitFileActionRequest {
    pub path: String,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DashboardResponse>, (StatusCode, String)> {
    let environment_count = state.environments.count().await.map_err(internal_error)?;
    let recent_sessions = build_session_summaries(
        &state.db,
        queries::list_recent_sessions(&state.db, 25)
            .await
            .map_err(internal_error)?,
    )
    .await
    .map_err(internal_error)?;
    Ok(Json(DashboardResponse {
        environment_count,
        recent_sessions,
    }))
}

pub async fn get(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<TaskDetailResponse>, (StatusCode, String)> {
    match queries::get_session(&state.db, &id)
        .await
        .map_err(internal_error)?
    {
        Some(session) => {
            let runs = queries::list_task_runs_for_session(&state.db, &session.id)
                .await
                .map_err(internal_error)?;
            let latest_run = runs.first().cloned();
            Ok(Json(TaskDetailResponse {
                session,
                latest_run,
                runs,
            }))
        }
        None => Err((StatusCode::NOT_FOUND, "task thread not found".to_string())),
    }
}

pub async fn messages(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TaskMessage>>, (StatusCode, String)> {
    let session = queries::get_session(&state.db, &id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "task thread not found".to_string()))?;
    queries::get_task_messages_by_session(&state.db, &session.id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn cancel(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CancelTaskResponse>, (StatusCode, String)> {
    let session = queries::get_session(&state.db, &id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "task thread not found".to_string()))?;

    let active_run = queries::get_active_run_for_session(&state.db, &session.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                "No running request found in this thread.".to_string(),
            )
        })?;

    let cancelled = state
        .runner
        .cancel(&active_run.id)
        .await
        .map_err(internal_error)?;
    if !cancelled {
        return Err((
            StatusCode::CONFLICT,
            "Unable to cancel this request because it is no longer active.".to_string(),
        ));
    }

    Ok(Json(CancelTaskResponse {
        task_run_id: active_run.id,
        status: "cancellation_requested".to_string(),
    }))
}

pub async fn terminal_socket(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, (StatusCode, String)> {
    let session = queries::get_session(&state.db, &id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "task thread not found".to_string()))?;

    state
        .terminals
        .ensure_started(&session.id, &session.workspace_path)
        .await
        .map_err(internal_error)?;

    Ok(ws.on_upgrade(move |socket| {
        let state = state.clone();
        async move {
            if let Err(error) = state
                .terminals
                .handle_socket(socket, session.id, session.workspace_path)
                .await
            {
                tracing::warn!(?error, "workspace terminal socket closed with error");
            }
        }
    }))
}

pub async fn workspace_git_diff(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<WorkspaceGitDiff>, (StatusCode, String)> {
    let session = get_session_for_task(&state, &id).await?;
    state
        .workspaces
        .inspect_git_diff(std::path::Path::new(&session.workspace_path))
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn stage_workspace_git_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WorkspaceGitFileActionRequest>,
) -> Result<Json<WorkspaceGitDiff>, (StatusCode, String)> {
    let session = get_session_for_task(&state, &id).await?;
    let workspace_path = std::path::Path::new(&session.workspace_path);
    state
        .workspaces
        .stage_git_file(workspace_path, &payload.path)
        .await
        .map_err(internal_error)?;
    state
        .workspaces
        .inspect_git_diff(workspace_path)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn revert_workspace_git_file(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WorkspaceGitFileActionRequest>,
) -> Result<Json<WorkspaceGitDiff>, (StatusCode, String)> {
    let session = get_session_for_task(&state, &id).await?;
    let workspace_path = std::path::Path::new(&session.workspace_path);
    state
        .workspaces
        .revert_git_file(workspace_path, &payload.path)
        .await
        .map_err(internal_error)?;
    state
        .workspaces
        .inspect_git_diff(workspace_path)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn summarize_sessions_for_environment(
    state: &Arc<AppState>,
    environment_id: &str,
) -> Result<Vec<SessionSummaryResponse>, (StatusCode, String)> {
    build_session_summaries(
        &state.db,
        queries::list_sessions_for_environment(&state.db, environment_id)
            .await
            .map_err(internal_error)?,
    )
    .await
    .map_err(internal_error)
}

async fn build_session_summaries(
    pool: &sqlx::SqlitePool,
    sessions: Vec<Session>,
) -> anyhow::Result<Vec<SessionSummaryResponse>> {
    let mut summaries = Vec::with_capacity(sessions.len());
    for session in sessions {
        let runs = queries::list_task_runs_for_session(pool, &session.id).await?;
        let latest_run = runs.first().cloned();
        summaries.push(SessionSummaryResponse {
            session,
            latest_run,
            run_count: runs.len(),
        });
    }
    Ok(summaries)
}

async fn get_session_for_task(
    state: &Arc<AppState>,
    id: &str,
) -> Result<Session, (StatusCode, String)> {
    queries::get_session(&state.db, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "task thread not found".to_string()))
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
