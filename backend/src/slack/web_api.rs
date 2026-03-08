use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::SharedConfig;
use crate::slack::thread_context::SlackReplyMessage;

#[derive(Clone)]
pub struct SlackWebClient {
    config: SharedConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct SlackOpenConnectionResponse {
    pub ok: bool,
    pub url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackThreadResponse {
    ok: bool,
    messages: Option<Vec<SlackReplyMessage>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackPostMessageResponse {
    ok: bool,
    error: Option<String>,
}

impl SlackWebClient {
    pub fn new(config: SharedConfig) -> Self {
        Self {
            config,
            client: Client::builder().build().expect("http client"),
        }
    }

    pub async fn open_socket_connection(&self) -> Result<String> {
        let response: SlackOpenConnectionResponse = self
            .api_post_with_token("apps.connections.open", json!({}), true)
            .await?;
        if response.ok {
            response
                .url
                .ok_or_else(|| anyhow!("missing socket mode URL"))
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "socket mode failed".to_string())))
        }
    }

    pub async fn fetch_thread(
        &self,
        channel: &str,
        thread_ts: &str,
    ) -> Result<Vec<SlackReplyMessage>> {
        let response: SlackThreadResponse = self
            .api_get(
                "conversations.replies",
                &[("channel", channel), ("ts", thread_ts)],
            )
            .await?;
        if response.ok {
            Ok(response.messages.unwrap_or_default())
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "thread fetch failed".to_string())))
        }
    }

    pub async fn post_message(&self, channel: &str, thread_ts: &str, text: &str) -> Result<()> {
        let response: SlackPostMessageResponse = self
            .api_post_with_token(
                "chat.postMessage",
                json!({
                    "channel": channel,
                    "thread_ts": thread_ts,
                    "text": text,
                    "unfurl_links": false,
                    "unfurl_media": false
                }),
                false,
            )
            .await?;
        if response.ok {
            Ok(())
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "post message failed".to_string())))
        }
    }

    async fn api_post_with_token<T: DeserializeOwned>(
        &self,
        method: &str,
        body: Value,
        use_app_token: bool,
    ) -> Result<T> {
        self.client
            .post(format!("https://slack.com/api/{method}"))
            .bearer_auth(if use_app_token {
                &self.config.slack.app_token
            } else {
                &self.config.slack.bot_token
            })
            .json(&body)
            .send()
            .await
            .with_context(|| format!("slack POST {method} failed"))?
            .error_for_status()?
            .json()
            .await
            .context("failed to decode Slack response")
    }

    async fn api_get<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        self.client
            .get(format!("https://slack.com/api/{method}"))
            .bearer_auth(&self.config.slack.bot_token)
            .query(params)
            .send()
            .await
            .with_context(|| format!("slack GET {method} failed"))?
            .error_for_status()?
            .json()
            .await
            .context("failed to decode Slack response")
    }
}
