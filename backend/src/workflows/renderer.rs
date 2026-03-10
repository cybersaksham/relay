use crate::db::models::Environment;
use crate::slack::thread_context::{NormalizedThread, NormalizedThreadAttachment};

use super::loader::WorkflowDefinition;

pub fn render_prompt(
    workflow: Option<&WorkflowDefinition>,
    environment: Option<&Environment>,
    thread: &NormalizedThread,
    workspace_path: &str,
) -> String {
    let mut sections = Vec::new();

    if let Some(workflow) = workflow {
        sections.push(format!(
            "Workflow: {}\nInstructions:\n{}\n",
            workflow.metadata.name, workflow.prompt
        ));
    }

    if let Some(environment) = environment {
        sections.push(format!(
            "Environment: {}\nRepo: {}\nDefault branch: {}\n",
            environment.slug, environment.git_ssh_url, environment.default_branch
        ));
    }

    sections.push(format!("Workspace: {}\n", workspace_path));
    sections.push("Slack thread context:".to_string());
    for message in &thread.messages {
        let mut block = format!("- {}: {}", message.author_label, message.text);
        if !message.attachments.is_empty() {
            for attachment in &message.attachments {
                block.push_str(&format!(
                    "\n  - attachment: {}",
                    render_attachment(attachment)
                ));
            }
        }
        sections.push(block);
    }

    sections.join("\n")
}

fn render_attachment(attachment: &NormalizedThreadAttachment) -> String {
    let mut parts = vec![attachment.name.clone()];

    if let Some(mimetype) = &attachment.mimetype {
        parts.push(mimetype.clone());
    } else if let Some(filetype) = &attachment.filetype {
        parts.push(filetype.clone());
    }

    if let Some(local_path) = &attachment.local_path {
        parts.push(format!("local_path={local_path}"));
    }

    parts.join(" | ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Utc;

    use super::render_prompt;
    use crate::db::models::Environment;
    use crate::slack::thread_context::{
        NormalizedThread, NormalizedThreadAttachment, NormalizedThreadMessage,
    };
    use crate::workflows::loader::{WorkflowDefinition, WorkflowMetadata};

    #[test]
    fn render_prompt_includes_attachment_local_paths() {
        let workflow = WorkflowDefinition {
            metadata: WorkflowMetadata {
                id: "iconland-add-icon".to_string(),
                name: "Iconland Add Icon".to_string(),
                scope: "environment".to_string(),
                environment_slug: Some("grauity".to_string()),
                trigger_phrases: vec![],
                default_environment: Some("grauity".to_string()),
                instructions: vec![],
                response_mode: "reply".to_string(),
            },
            prompt: "Follow the icon workflow".to_string(),
            root_dir: PathBuf::from("/tmp/iconland-add-icon"),
        };
        let environment = Environment {
            id: "env-1".to_string(),
            name: "Grauity".to_string(),
            slug: "grauity".to_string(),
            git_ssh_url: "git@github.com:example/grauity.git".to_string(),
            default_branch: "master".to_string(),
            aliases: "[]".to_string(),
            enabled: true,
            source_sync_status: "ready".to_string(),
            source_sync_error: None,
            source_synced_at: Some(Utc::now()),
            source_setup_script: None,
            workspace_setup_script: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let thread = NormalizedThread {
            channel_id: "C123".to_string(),
            thread_ts: "1000.01".to_string(),
            messages: vec![NormalizedThreadMessage {
                ts: "1000.01".to_string(),
                author_id: Some("U123".to_string()),
                author_label: "U123".to_string(),
                text: "These are the icons for loader".to_string(),
                attachments: vec![NormalizedThreadAttachment {
                    id: Some("F123".to_string()),
                    name: "loadingicon.zip".to_string(),
                    mimetype: Some("application/zip".to_string()),
                    filetype: Some("zip".to_string()),
                    download_url: Some("https://example.com/loadingicon.zip".to_string()),
                    local_path: Some(
                        "/tmp/workspace/.git/relay-thread-context/1000_01/loadingicon.zip"
                            .to_string(),
                    ),
                }],
            }],
        };

        let prompt = render_prompt(
            Some(&workflow),
            Some(&environment),
            &thread,
            "/tmp/workspace",
        );

        assert!(prompt.contains("loadingicon.zip | application/zip"));
        assert!(prompt.contains("local_path=/tmp/workspace/.git/relay-thread-context"));
    }
}
