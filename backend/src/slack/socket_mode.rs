use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::app_state::AppState;
use crate::slack::SlackEventEnvelope;
use crate::tasks::orchestrator::handle_slack_envelope;

pub async fn run_socket_mode(state: Arc<AppState>) -> Result<()> {
    if state.config.slack.app_token.is_empty() || state.config.slack.bot_token.is_empty() {
        warn!("Slack tokens missing; socket mode worker disabled");
        return Ok(());
    }

    loop {
        match open_and_process(state.clone()).await {
            Ok(()) => sleep(Duration::from_secs(3)).await,
            Err(error) => {
                error!(?error, "socket mode loop failed");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn open_and_process(state: Arc<AppState>) -> Result<()> {
    let socket_url = state.slack.open_socket_connection().await?;
    let (ws_stream, _) = connect_async(socket_url).await?;
    info!("socket mode connected");
    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        let message = message?;
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text)?;
            if value.get("type").and_then(Value::as_str) == Some("hello") {
                continue;
            }

            if let Some(envelope_id) = value.get("envelope_id").and_then(Value::as_str) {
                write
                    .send(Message::Text(json!({ "envelope_id": envelope_id }).to_string().into()))
                    .await?;
            }

            if value.get("payload").is_some() && value.get("type").and_then(Value::as_str) == Some("events_api") {
                let envelope: SlackEventEnvelope = serde_json::from_value(value)?;
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_slack_envelope(state_clone, envelope).await {
                        error!(?error, "failed to handle slack envelope");
                    }
                });
            }
        }
    }

    Ok(())
}
