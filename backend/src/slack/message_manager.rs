use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Serialize;
use url::Url;

use crate::slack::formatter::resolve_slack_text;
use crate::slack::web_api::{SlackAuthIdentity, SlackFetchedMessage, SlackWebClient};

#[derive(Debug, Clone, Serialize)]
pub struct ManagedSlackMessage {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: Option<String>,
    pub text: String,
    pub raw_text: String,
    pub author_user_id: Option<String>,
    pub author_bot_id: Option<String>,
}

pub struct SlackMessageManager {
    slack: SlackWebClient,
}

impl SlackMessageManager {
    pub fn new(slack: SlackWebClient) -> Self {
        Self { slack }
    }

    pub async fn fetch_message_by_permalink(&self, permalink: &str) -> Result<ManagedSlackMessage> {
        let target = parse_slack_permalink(permalink)?;
        let identity = self.slack.auth_identity().await?;
        let message = self
            .slack
            .fetch_message(
                &target.channel_id,
                &target.message_ts,
                target.thread_ts.as_deref(),
            )
            .await?;

        ensure_message_owned_by_bot(&message, &identity)?;

        Ok(ManagedSlackMessage {
            channel_id: target.channel_id,
            ts: message.ts,
            thread_ts: message.thread_ts,
            text: resolve_slack_text(message.text.as_deref().unwrap_or_default()),
            raw_text: message.text.unwrap_or_default(),
            author_user_id: message.user,
            author_bot_id: message.bot_id,
        })
    }

    pub async fn update_message(
        &self,
        channel_id: &str,
        ts: &str,
        thread_ts: Option<&str>,
        text: &str,
    ) -> Result<ManagedSlackMessage> {
        let updated_text = text.trim();
        if updated_text.is_empty() {
            return Err(anyhow!("message text cannot be empty"));
        }

        self.slack.update_message(channel_id, ts, updated_text).await?;
        let identity = self.slack.auth_identity().await?;
        let message = self.slack.fetch_message(channel_id, ts, thread_ts).await?;
        ensure_message_owned_by_bot(&message, &identity)?;

        Ok(ManagedSlackMessage {
            channel_id: channel_id.to_string(),
            ts: message.ts,
            thread_ts: message.thread_ts,
            text: resolve_slack_text(message.text.as_deref().unwrap_or_default()),
            raw_text: message.text.unwrap_or_default(),
            author_user_id: message.user,
            author_bot_id: message.bot_id,
        })
    }

    pub async fn delete_message(
        &self,
        channel_id: &str,
        ts: &str,
        thread_ts: Option<&str>,
    ) -> Result<()> {
        let identity = self.slack.auth_identity().await?;
        let message = self.slack.fetch_message(channel_id, ts, thread_ts).await?;
        ensure_message_owned_by_bot(&message, &identity)?;
        self.slack.delete_message(channel_id, ts).await
    }
}

#[derive(Debug)]
struct SlackPermalinkTarget {
    channel_id: String,
    message_ts: String,
    thread_ts: Option<String>,
}

fn parse_slack_permalink(permalink: &str) -> Result<SlackPermalinkTarget> {
    let trimmed = permalink.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("slack message link is required"));
    }

    let url = Url::parse(trimmed).context("invalid Slack message link")?;
    let segments: Vec<_> = url
        .path_segments()
        .ok_or_else(|| anyhow!("invalid Slack message link"))?
        .collect();

    if segments.len() < 3 || segments[0] != "archives" {
        return Err(anyhow!("Slack link must point to a message permalink"));
    }

    let channel_id = segments[1].to_string();
    let raw_message_id = segments[2];
    let captures = Regex::new(r"^p(\d{16})$")
        .expect("valid regex")
        .captures(raw_message_id)
        .ok_or_else(|| anyhow!("Slack link must include a valid message timestamp"))?;
    let digits = captures
        .get(1)
        .map(|value| value.as_str())
        .ok_or_else(|| anyhow!("Slack link must include a valid message timestamp"))?;

    let (seconds, micros) = digits.split_at(10);
    let thread_ts = url
        .query_pairs()
        .find_map(|(key, value)| (key == "thread_ts").then(|| value.into_owned()))
        .filter(|value| !value.trim().is_empty());

    Ok(SlackPermalinkTarget {
        channel_id,
        message_ts: format!("{seconds}.{micros}"),
        thread_ts,
    })
}

fn ensure_message_owned_by_bot(
    message: &SlackFetchedMessage,
    identity: &SlackAuthIdentity,
) -> Result<()> {
    let is_bot_user = message.user.as_deref() == Some(identity.user_id.as_str());
    let is_bot_id = identity
        .bot_id
        .as_deref()
        .zip(message.bot_id.as_deref())
        .map(|(expected, actual)| expected == actual)
        .unwrap_or(false);

    if is_bot_user || is_bot_id {
        Ok(())
    } else {
        Err(anyhow!(
            "message was not posted by the bot configured via SLACK_BOT_TOKEN"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_slack_permalink;

    #[test]
    fn parses_standard_slack_permalink() {
        let target = parse_slack_permalink(
            "https://example.slack.com/archives/C12345678/p1736451111222233",
        )
        .expect("target");

        assert_eq!(target.channel_id, "C12345678");
        assert_eq!(target.message_ts, "1736451111.222233");
        assert_eq!(target.thread_ts, None);
    }

    #[test]
    fn parses_thread_reply_permalink() {
        let target = parse_slack_permalink(
            "https://example.slack.com/archives/C12345678/p1773055938193069?thread_ts=1773055935.359359&cid=C12345678",
        )
        .expect("target");

        assert_eq!(target.channel_id, "C12345678");
        assert_eq!(target.message_ts, "1773055938.193069");
        assert_eq!(target.thread_ts.as_deref(), Some("1773055935.359359"));
    }

    #[test]
    fn rejects_non_message_permalink() {
        let error = parse_slack_permalink("https://example.slack.com/messages")
            .expect_err("should reject invalid permalink");

        assert!(error.to_string().contains("message permalink"));
    }
}
