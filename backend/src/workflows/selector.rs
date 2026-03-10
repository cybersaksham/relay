use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::warn;
use uuid::Uuid;

use crate::config::SharedConfig;
use crate::db::models::Environment;
use crate::slack::thread_context::NormalizedThread;

use super::loader::{WorkflowDefinition, WorkflowRegistry};

pub async fn select_workflow(
    config: &SharedConfig,
    registry: &WorkflowRegistry,
    request_text: &str,
    thread: &NormalizedThread,
    environment: Option<&Environment>,
) -> Option<WorkflowDefinition> {
    let candidates = eligible_workflows(registry, environment);
    if candidates.is_empty() {
        return None;
    }

    match select_workflow_id(config, &candidates, request_text, thread, environment).await {
        Ok(Some(selected_id)) => candidates
            .into_iter()
            .find(|workflow| workflow.metadata.id == selected_id),
        Ok(None) => None,
        Err(error) => {
            warn!(?error, "workflow selection via codex failed");
            None
        }
    }
}

fn eligible_workflows(
    registry: &WorkflowRegistry,
    environment: Option<&Environment>,
) -> Vec<WorkflowDefinition> {
    registry
        .all()
        .into_iter()
        .filter(|workflow| match workflow.metadata.scope.as_str() {
            "global" => true,
            "environment" => {
                environment.is_some()
                    && workflow.metadata.environment_slug.as_deref()
                        == environment.map(|item| item.slug.as_str())
            }
            _ => false,
        })
        .collect()
}

async fn select_workflow_id(
    config: &SharedConfig,
    candidates: &[WorkflowDefinition],
    request_text: &str,
    thread: &NormalizedThread,
    environment: Option<&Environment>,
) -> Result<Option<String>> {
    let output_path = selector_output_path(&config.paths.relay_home);
    let prompt = build_selector_prompt(candidates, request_text, thread, environment);

    let mut command = Command::new(&config.codex.bin);
    command
        .arg("exec")
        .arg("--skip-git-repo-check")
        .arg("--ephemeral")
        .arg("--color")
        .arg("never")
        .arg("-o")
        .arg(&output_path)
        .args(&config.codex.default_args)
        .arg(prompt)
        .current_dir(&config.paths.relay_home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .context("failed to spawn codex workflow selector")?;
    let status = timeout(Duration::from_secs(90), child.wait())
        .await
        .map_err(|_| anyhow::anyhow!("codex workflow selector timed out after 90 seconds"))?
        .context("failed waiting for codex workflow selector")?;

    let selected = fs::read_to_string(&output_path)
        .await
        .unwrap_or_default()
        .trim()
        .to_string();
    let _ = fs::remove_file(&output_path).await;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "codex workflow selector exited unsuccessfully"
        ));
    }

    if selected.is_empty() || selected.eq_ignore_ascii_case("NONE") {
        return Ok(None);
    }

    Ok(Some(selected))
}

fn selector_output_path(relay_home: &PathBuf) -> String {
    relay_home
        .join(format!(".workflow-selection-{}.txt", Uuid::new_v4()))
        .display()
        .to_string()
}

fn build_selector_prompt(
    candidates: &[WorkflowDefinition],
    request_text: &str,
    thread: &NormalizedThread,
    environment: Option<&Environment>,
) -> String {
    let mut prompt = String::from(
        "Choose the single best workflow for this Slack thread.\n\
         You may only choose one of the listed workflow IDs or NONE.\n\
         Respond with exactly one token: either the workflow ID or NONE.\n\
         Do not explain your choice.\n\n",
    );

    if let Some(environment) = environment {
        prompt.push_str(&format!(
            "Resolved environment: {} ({})\n\n",
            environment.slug, environment.name
        ));
    } else {
        prompt.push_str("Resolved environment: none\n\n");
    }

    prompt.push_str(&format!(
        "Latest request: {}\n\n",
        compact_text(request_text, 320)
    ));
    prompt.push_str("Recent thread context:\n");
    for message in recent_relevant_messages(thread) {
        prompt.push_str(&format!(
            "- {}: {}\n",
            message.author_label,
            compact_text(&message.text, 220)
        ));
        if !message.attachments.is_empty() {
            let attachment_names = message
                .attachments
                .iter()
                .map(|attachment| attachment.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            prompt.push_str(&format!(
                "  attachments: {}\n",
                compact_text(&attachment_names, 220)
            ));
        }
    }
    prompt.push('\n');
    prompt.push_str("Available workflows:\n");

    for workflow in candidates {
        prompt.push_str(&format!(
            "- id: {}\n  name: {}\n  scope: {}\n  environment_slug: {}\n  trigger_phrases: {}\n  summary: {}\n",
            workflow.metadata.id,
            workflow.metadata.name,
            workflow.metadata.scope,
            workflow
                .metadata
                .environment_slug
                .as_deref()
                .unwrap_or(""),
            workflow.metadata.trigger_phrases.join(", "),
            compact_text(&workflow.metadata.instructions.join(" "), 180),
        ));
    }

    prompt
}

fn recent_relevant_messages(
    thread: &NormalizedThread,
) -> Vec<&crate::slack::thread_context::NormalizedThreadMessage> {
    let mut messages: Vec<_> = thread
        .messages
        .iter()
        .filter(|message| {
            message.author_id.is_some()
                && !message.text.trim().is_empty()
                && !looks_like_link_unfurl(&message.text)
        })
        .collect();

    if messages.len() > 4 {
        messages = messages.split_off(messages.len() - 4);
    }

    messages
}

fn looks_like_link_unfurl(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("added by github")
        || lower.contains("show more")
        || (lower.contains("please go through this checklist before your merge the pr")
            && lower.contains("added by github"))
}

fn compact_text(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= limit {
        collapsed
    } else {
        let mut compact = collapsed
            .chars()
            .take(limit.saturating_sub(1))
            .collect::<String>();
        compact.push('…');
        compact
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::eligible_workflows;
    use crate::db::models::Environment;
    use crate::workflows::loader::{WorkflowDefinition, WorkflowMetadata, WorkflowRegistry};
    use chrono::Utc;

    #[test]
    fn filters_to_global_and_matching_environment_workflows() {
        let registry = WorkflowRegistry::from_workflows(HashMap::from([
            (
                "global-review".to_string(),
                workflow("global-review", "global", None),
            ),
            (
                "newton-pr".to_string(),
                workflow("newton-pr", "environment", Some("newton-web")),
            ),
            (
                "other-env".to_string(),
                workflow("other-env", "environment", Some("other-web")),
            ),
        ]));

        let environment = Environment {
            id: "env-1".to_string(),
            name: "Newton Web".to_string(),
            slug: "newton-web".to_string(),
            git_ssh_url: "git@github.com:example/repo.git".to_string(),
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

        let filtered = eligible_workflows(&registry, Some(&environment));
        let mut ids: Vec<String> = filtered.into_iter().map(|item| item.metadata.id).collect();
        ids.sort();

        assert_eq!(
            ids,
            vec!["global-review".to_string(), "newton-pr".to_string()]
        );
    }

    fn workflow(id: &str, scope: &str, environment_slug: Option<&str>) -> WorkflowDefinition {
        WorkflowDefinition {
            metadata: WorkflowMetadata {
                id: id.to_string(),
                name: id.to_string(),
                scope: scope.to_string(),
                environment_slug: environment_slug.map(str::to_string),
                trigger_phrases: vec![],
                default_environment: None,
                instructions: vec![],
                response_mode: "reply".to_string(),
            },
            prompt: String::new(),
            root_dir: PathBuf::from("."),
        }
    }
}
