pub mod formatter;
pub mod message_manager;
pub mod socket_mode;
pub mod thread_context;
pub mod web_api;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackEventEnvelope {
    pub envelope_id: String,
    #[serde(rename = "type")]
    pub envelope_type: String,
    pub payload: SlackEnvelopePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackEnvelopePayload {
    pub event_id: String,
    pub team_id: Option<String>,
    pub event: SlackMessageEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessageEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub user: Option<String>,
    pub text: Option<String>,
    pub channel: Option<String>,
    pub ts: Option<String>,
    pub thread_ts: Option<String>,
}
