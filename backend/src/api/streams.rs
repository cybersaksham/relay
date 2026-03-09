use std::convert::Infallible;
use std::sync::Arc;

use async_stream::stream;
use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use tokio::time::{sleep, Duration};

use crate::app_state::AppState;
use crate::db::queries;

pub async fn terminal_stream(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let mut after_id = 0_i64;
        loop {
            match queries::list_terminal_events_after(&state.db, &id, after_id).await {
                Ok(events) => {
                    for event in events {
                        after_id = event.id;
                        let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                        yield Ok(Event::default().event("terminal").data(payload));
                    }
                }
                Err(error) => {
                    yield Ok(Event::default().event("error").data(error.to_string()));
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn events_stream(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let mut last_status = String::new();
        loop {
            match queries::get_task_run(&state.db, &id).await {
                Ok(Some(run)) => {
                    if run.status != last_status {
                        last_status = run.status.clone();
                        let payload = serde_json::to_string(&run).unwrap_or_else(|_| "{}".to_string());
                        yield Ok(Event::default().event("status").data(payload));
                    }
                }
                Ok(None) => {
                    yield Ok(Event::default().event("error").data("task not found"));
                    break;
                }
                Err(error) => {
                    yield Ok(Event::default().event("error").data(error.to_string()));
                    break;
                }
            }
            sleep(Duration::from_secs(2)).await;
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn environment_sync_stream(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let mut after_id = 0_i64;
        loop {
            match queries::list_environment_sync_events_after(&state.db, &id, after_id).await {
                Ok(events) => {
                    for event in events {
                        after_id = event.id;
                        let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                        yield Ok(Event::default().event("sync").data(payload));
                    }
                }
                Err(error) => {
                    yield Ok(Event::default().event("error").data(error.to_string()));
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
