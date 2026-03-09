pub mod environments;
pub mod slack;
pub mod streams;
pub mod tasks;

use std::sync::Arc;

use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde_json::json;

use crate::app_state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/api/environments",
            get(environments::list).post(environments::create),
        )
        .route(
            "/api/environments/:id",
            get(environments::get)
                .put(environments::update)
                .delete(environments::delete),
        )
        .route("/api/environments/:id/refresh", post(environments::refresh))
        .route("/api/environments/:id/tasks", get(environments::tasks))
        .route("/api/slack/messages/lookup", post(slack::lookup_message))
        .route(
            "/api/slack/messages",
            axum::routing::put(slack::update_message).delete(slack::delete_message),
        )
        .route("/api/tasks", get(tasks::list))
        .route("/api/tasks/:id", get(tasks::get))
        .route("/api/tasks/:id/messages", get(tasks::messages))
        .route("/api/tasks/:id/cancel", post(tasks::cancel))
        .route(
            "/api/tasks/:id/workspace-terminal/ws",
            get(tasks::terminal_socket),
        )
        .route(
            "/api/tasks/:id/terminal/stream",
            get(streams::terminal_stream),
        )
        .route("/api/tasks/:id/events/stream", get(streams::events_stream))
        .route(
            "/api/environments/:id/sync/stream",
            get(streams::environment_sync_stream),
        )
        .with_state(state)
}

async fn healthz(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "service": "relay-backend",
        "base_url": state.config.server.base_url,
    }))
}
