use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::formatter::resolve_slack_text;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedThreadMessage {
    pub ts: String,
    pub author_id: Option<String>,
    pub author_label: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedThread {
    pub channel_id: String,
    pub thread_ts: String,
    pub messages: Vec<NormalizedThreadMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackReplyMessage {
    pub ts: String,
    pub user: Option<String>,
    pub bot_id: Option<String>,
    pub text: Option<String>,
}

pub fn normalize_thread(
    channel_id: &str,
    thread_ts: &str,
    messages: Vec<SlackReplyMessage>,
) -> Result<NormalizedThread> {
    let messages = messages
        .into_iter()
        .map(|message| {
            let author_label = message
                .user
                .clone()
                .or(message.bot_id.clone())
                .unwrap_or_else(|| "system".to_string());
            NormalizedThreadMessage {
                ts: message.ts,
                author_id: message.user,
                author_label,
                text: resolve_slack_text(message.text.as_deref().unwrap_or_default()),
            }
        })
        .collect();

    Ok(NormalizedThread {
        channel_id: channel_id.to_string(),
        thread_ts: thread_ts.to_string(),
        messages,
    })
}
