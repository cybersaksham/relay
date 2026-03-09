use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::api::tasks as task_api;
use crate::app_state::AppState;
use crate::db::models::Environment;
use crate::environments::service::{
    CreateEnvironmentInput, DeleteEnvironmentResponse, EnvironmentWithPaths,
};

#[derive(Debug, Serialize)]
pub struct EnvironmentDetailResponse {
    pub environment: Environment,
    pub source_path: String,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Environment>>, (StatusCode, String)> {
    state
        .environments
        .list()
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEnvironmentInput>,
) -> Result<Json<EnvironmentWithPaths>, (StatusCode, String)> {
    state
        .environments
        .create(payload)
        .await
        .map(Json)
        .map_err(user_error)
}

pub async fn update(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEnvironmentInput>,
) -> Result<Json<EnvironmentWithPaths>, (StatusCode, String)> {
    state
        .environments
        .update(&id, payload)
        .await
        .map(Json)
        .map_err(user_error)
}

pub async fn delete(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<DeleteEnvironmentResponse>, (StatusCode, String)> {
    state
        .environments
        .delete(&id)
        .await
        .map(Json)
        .map_err(user_error)
}

pub async fn refresh(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<EnvironmentWithPaths>, (StatusCode, String)> {
    state
        .environments
        .refresh_source(&id)
        .await
        .map(Json)
        .map_err(refresh_error)
}

pub async fn get(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<EnvironmentDetailResponse>, (StatusCode, String)> {
    match state.environments.get(&id).await.map_err(internal_error)? {
        Some(environment) => Ok(Json(EnvironmentDetailResponse {
            source_path: state
                .environments
                .source_path_for_slug(&environment.slug)
                .display()
                .to_string(),
            environment,
        })),
        None => Err((StatusCode::NOT_FOUND, "environment not found".to_string())),
    }
}

pub async fn tasks(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<task_api::SessionSummaryResponse>>, (StatusCode, String)> {
    task_api::summarize_sessions_for_environment(&state, &id)
        .await
        .map(Json)
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn user_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, error.to_string())
}

fn refresh_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    let message = error.to_string();
    if message.contains("already in progress") {
        (StatusCode::CONFLICT, message)
    } else {
        (StatusCode::BAD_REQUEST, message)
    }
}
