use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::slack::message_manager::SlackMessageManager;

#[derive(Debug, Deserialize)]
pub struct LookupSlackMessageRequest {
    pub permalink: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSlackMessageRequest {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: Option<String>,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSlackMessageRequest {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DeleteSlackMessageResponse {
    pub deleted: bool,
}

pub async fn lookup_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LookupSlackMessageRequest>,
) -> Result<Json<crate::slack::message_manager::ManagedSlackMessage>, (StatusCode, String)> {
    SlackMessageManager::new(state.slack.as_ref().clone())
        .fetch_message_by_permalink(&payload.permalink)
        .await
        .map(Json)
        .map_err(user_error)
}

pub async fn update_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateSlackMessageRequest>,
) -> Result<Json<crate::slack::message_manager::ManagedSlackMessage>, (StatusCode, String)> {
    SlackMessageManager::new(state.slack.as_ref().clone())
        .update_message(
            &payload.channel_id,
            &payload.ts,
            payload.thread_ts.as_deref(),
            &payload.text,
        )
        .await
        .map(Json)
        .map_err(user_error)
}

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteSlackMessageRequest>,
) -> Result<Json<DeleteSlackMessageResponse>, (StatusCode, String)> {
    SlackMessageManager::new(state.slack.as_ref().clone())
        .delete_message(&payload.channel_id, &payload.ts, payload.thread_ts.as_deref())
        .await
        .map(|_| Json(DeleteSlackMessageResponse { deleted: true }))
        .map_err(user_error)
}

fn user_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, error.to_string())
}
