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

#[derive(Debug, Deserialize)]
struct SlackBasicResponse {
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackAuthTestResponse {
    ok: bool,
    user_id: Option<String>,
    bot_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackHistoryResponse {
    ok: bool,
    messages: Option<Vec<SlackFetchedMessage>>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SlackAuthIdentity {
    pub user_id: String,
    pub bot_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackFetchedMessage {
    pub ts: String,
    pub user: Option<String>,
    pub bot_id: Option<String>,
    pub text: Option<String>,
    pub thread_ts: Option<String>,
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

    pub async fn auth_identity(&self) -> Result<SlackAuthIdentity> {
        let response: SlackAuthTestResponse = self
            .api_post_with_token("auth.test", json!({}), false)
            .await?;
        if response.ok {
            Ok(SlackAuthIdentity {
                user_id: response
                    .user_id
                    .ok_or_else(|| anyhow!("auth.test did not return user_id"))?,
                bot_id: response.bot_id,
            })
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "auth.test failed".to_string())))
        }
    }

    pub async fn fetch_message(
        &self,
        channel: &str,
        ts: &str,
        thread_ts: Option<&str>,
    ) -> Result<SlackFetchedMessage> {
        if let Some(thread_ts) = thread_ts.filter(|thread_ts| *thread_ts != ts) {
            return self.fetch_thread_message(channel, thread_ts, ts).await;
        }

        let response: SlackHistoryResponse = self
            .api_get(
                "conversations.history",
                &[
                    ("channel", channel),
                    ("oldest", ts),
                    ("latest", ts),
                    ("inclusive", "true"),
                    ("limit", "1"),
                ],
            )
            .await?;
        if response.ok {
            response
                .messages
                .unwrap_or_default()
                .into_iter()
                .find(|message| message.ts == ts)
                .ok_or_else(|| anyhow!("message not found"))
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "message fetch failed".to_string())))
        }
    }

    async fn fetch_thread_message(
        &self,
        channel: &str,
        thread_ts: &str,
        ts: &str,
    ) -> Result<SlackFetchedMessage> {
        let response: SlackHistoryResponse = self
            .api_get(
                "conversations.replies",
                &[("channel", channel), ("ts", thread_ts)],
            )
            .await?;

        if response.ok {
            response
                .messages
                .unwrap_or_default()
                .into_iter()
                .find(|message| message.ts == ts)
                .ok_or_else(|| anyhow!("message not found"))
        } else {
            Err(anyhow!(response.error.unwrap_or_else(|| {
                "thread message fetch failed".to_string()
            })))
        }
    }

    pub async fn update_message(&self, channel: &str, ts: &str, text: &str) -> Result<()> {
        let response: SlackPostMessageResponse = self
            .api_post_with_token(
                "chat.update",
                json!({
                    "channel": channel,
                    "ts": ts,
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
                .unwrap_or_else(|| "update message failed".to_string())))
        }
    }

    pub async fn delete_message(&self, channel: &str, ts: &str) -> Result<()> {
        let response: SlackBasicResponse = self
            .api_post_with_token(
                "chat.delete",
                json!({
                    "channel": channel,
                    "ts": ts
                }),
                false,
            )
            .await?;
        if response.ok {
            Ok(())
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "delete message failed".to_string())))
        }
    }

    pub async fn add_reaction(&self, channel: &str, timestamp: &str, name: &str) -> Result<()> {
        match self.add_reaction_name(channel, timestamp, name).await {
            Ok(()) => Ok(()),
            Err(error) if name == "white-tick" && error.to_string() == "invalid_name" => {
                self.add_reaction_name(channel, timestamp, "white_check_mark")
                    .await
            }
            Err(error) => Err(error),
        }
    }

    pub async fn remove_reaction(&self, channel: &str, timestamp: &str, name: &str) -> Result<()> {
        let response: SlackBasicResponse = self
            .api_post_with_token(
                "reactions.remove",
                json!({
                    "channel": channel,
                    "timestamp": timestamp,
                    "name": name
                }),
                false,
            )
            .await?;

        if response.ok {
            return Ok(());
        }

        let error = response
            .error
            .unwrap_or_else(|| "reaction remove failed".to_string());
        if error == "no_reaction" {
            return Ok(());
        }

        Err(anyhow!(error))
    }

    async fn add_reaction_name(&self, channel: &str, timestamp: &str, name: &str) -> Result<()> {
        let response: SlackBasicResponse = self
            .api_post_with_token(
                "reactions.add",
                json!({
                    "channel": channel,
                    "timestamp": timestamp,
                    "name": name
                }),
                false,
            )
            .await?;

        if response.ok {
            Ok(())
        } else {
            Err(anyhow!(response
                .error
                .unwrap_or_else(|| "reaction add failed".to_string())))
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
