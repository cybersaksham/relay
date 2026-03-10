use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::formatter::resolve_slack_text;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlackThreadFile {
    pub id: Option<String>,
    pub name: Option<String>,
    pub mimetype: Option<String>,
    pub filetype: Option<String>,
    pub url_private: Option<String>,
    pub url_private_download: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedThreadAttachment {
    pub id: Option<String>,
    pub name: String,
    pub mimetype: Option<String>,
    pub filetype: Option<String>,
    pub download_url: Option<String>,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedThreadMessage {
    pub ts: String,
    pub author_id: Option<String>,
    pub author_label: String,
    pub text: String,
    pub attachments: Vec<NormalizedThreadAttachment>,
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
    #[serde(default)]
    pub files: Vec<SlackThreadFile>,
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
                attachments: message
                    .files
                    .into_iter()
                    .map(|file| NormalizedThreadAttachment {
                        id: file.id,
                        name: file
                            .name
                            .filter(|name| !name.trim().is_empty())
                            .unwrap_or_else(|| "attachment".to_string()),
                        mimetype: file.mimetype,
                        filetype: file.filetype,
                        download_url: file.url_private_download.or(file.url_private),
                        local_path: None,
                    })
                    .collect(),
            }
        })
        .collect();

    Ok(NormalizedThread {
        channel_id: channel_id.to_string(),
        thread_ts: thread_ts.to_string(),
        messages,
    })
}

#[cfg(test)]
mod tests {
    use super::{normalize_thread, SlackReplyMessage, SlackThreadFile};

    #[test]
    fn normalizes_file_attachments_into_thread_context() {
        let thread = normalize_thread(
            "C123",
            "1000.01",
            vec![SlackReplyMessage {
                ts: "1000.01".to_string(),
                user: Some("U123".to_string()),
                bot_id: None,
                text: Some("Here is the icon zip".to_string()),
                files: vec![SlackThreadFile {
                    id: Some("F123".to_string()),
                    name: Some("loadingicon.zip".to_string()),
                    mimetype: Some("application/zip".to_string()),
                    filetype: Some("zip".to_string()),
                    url_private: None,
                    url_private_download: Some(
                        "https://files.slack.com/files-pri/T1-F123/download/loadingicon.zip"
                            .to_string(),
                    ),
                }],
            }],
        )
        .expect("thread should normalize");

        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].attachments.len(), 1);
        assert_eq!(thread.messages[0].attachments[0].name, "loadingicon.zip");
        assert_eq!(
            thread.messages[0].attachments[0].download_url.as_deref(),
            Some("https://files.slack.com/files-pri/T1-F123/download/loadingicon.zip")
        );
        assert_eq!(thread.messages[0].attachments[0].local_path, None);
    }
}
