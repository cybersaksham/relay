use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::app_state::AppState;
use crate::db::models::{Session, TaskMessage, TaskRun};
use crate::db::queries;

#[derive(Debug, Serialize)]
pub struct TaskDetailResponse {
    pub run: TaskRun,
    pub session: Session,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub environment_count: i64,
    pub recent_tasks: Vec<TaskRun>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DashboardResponse>, (StatusCode, String)> {
    let environment_count = state.environments.count().await.map_err(internal_error)?;
    let recent_tasks = queries::list_recent_task_runs(&state.db, 25)
        .await
        .map_err(internal_error)?;
    Ok(Json(DashboardResponse {
        environment_count,
        recent_tasks,
    }))
}

pub async fn get(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<TaskDetailResponse>, (StatusCode, String)> {
    match queries::get_task_run_with_session(&state.db, &id)
        .await
        .map_err(internal_error)?
    {
        Some((run, session)) => Ok(Json(TaskDetailResponse { run, session })),
        None => Err((StatusCode::NOT_FOUND, "task not found".to_string())),
    }
}

pub async fn messages(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TaskMessage>>, (StatusCode, String)> {
    let maybe_session = queries::get_session_by_task_run(&state.db, &id)
        .await
        .map_err(internal_error)?;
    let session =
        maybe_session.ok_or_else(|| (StatusCode::NOT_FOUND, "task not found".to_string()))?;
    queries::get_task_messages_by_session(&state.db, &session.id)
        .await
        .map(Json)
        .map_err(internal_error)
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
