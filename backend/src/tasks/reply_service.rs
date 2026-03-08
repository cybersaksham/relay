use anyhow::Result;

use crate::app_state::AppState;
use crate::db::queries;
use crate::slack::formatter::resolved_payload_json;

pub async fn persist_and_send_reply(
    state: &AppState,
    session_id: &str,
    task_run_id: Option<&str>,
    channel_id: &str,
    thread_ts: &str,
    text: &str,
) -> Result<()> {
    let raw = serde_json::json!({ "text": text }).to_string();
    let resolved = resolved_payload_json(text);

    queries::insert_task_message(
        &state.db,
        session_id,
        task_run_id,
        "outbound",
        None,
        &raw,
        &resolved,
    )
    .await?;

    state.slack.post_message(channel_id, thread_ts, text).await?;
    Ok(())
}
