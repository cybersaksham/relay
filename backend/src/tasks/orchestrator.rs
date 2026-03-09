use std::sync::Arc;

use anyhow::Result;
use serde_json::json;
use tracing::{info, warn};

use crate::app_state::AppState;
use crate::db::models::{Environment, Session};
use crate::db::queries;
use crate::policies::evaluator::PolicyDecision;
use crate::runner::RunnerInput;
use crate::slack::formatter::{resolve_slack_text, resolved_payload_json};
use crate::slack::thread_context::normalize_thread;
use crate::slack::SlackEventEnvelope;
use crate::tasks::reply_service::persist_and_send_reply;
use crate::workflows::{matcher, renderer, selector};

pub async fn handle_slack_envelope(
    state: Arc<AppState>,
    envelope: SlackEventEnvelope,
) -> Result<()> {
    let event_id = envelope.payload.event_id;
    if !queries::record_slack_event(&state.db, &event_id).await? {
        return Ok(());
    }

    let event = envelope.payload.event;
    if event.event_type != "app_mention" {
        return Ok(());
    }

    let user_id = match event.user {
        Some(user_id) => user_id,
        None => return Ok(()),
    };
    let text = event.text.unwrap_or_default();
    let channel_id = match event.channel {
        Some(channel_id) => channel_id,
        None => return Ok(()),
    };
    let thread_ts = event.thread_ts.or(event.ts.clone()).unwrap_or_default();
    let trigger_message_ts = event.ts.unwrap_or_else(|| thread_ts.clone());
    let team_id = envelope
        .payload
        .team_id
        .unwrap_or_else(|| "unknown-team".to_string());

    if let Some(ban) = queries::get_active_ban(&state.db, &user_id).await? {
        state
            .slack
            .post_message(
                &channel_id,
                &thread_ts,
                &format!(
                    "Your access is temporarily blocked until {} because of repeated critical requests.",
                    ban.expires_at
                ),
            )
            .await?;
        return Ok(());
    }

    begin_processing_reaction(&state, &channel_id, &trigger_message_ts).await;

    let outcome = async {
        let messages = state.slack.fetch_thread(&channel_id, &thread_ts).await?;
        let thread = normalize_thread(&channel_id, &thread_ts, messages)?;
        let request_text = resolve_slack_text(&text);

        match state.policies.evaluate(&user_id, &request_text) {
            PolicyDecision::Allowed => {}
            PolicyDecision::NonMasterDenied(rule) => {
                state
                    .slack
                    .post_message(
                        &channel_id,
                        &thread_ts,
                        &format!(
                            "Request denied. Non-master users can only ask for approved tasks. Policy: {}.",
                            rule.title
                        ),
                    )
                    .await?;
                return Ok(RequestOutcome::Rejected);
            }
            PolicyDecision::CriticalDenied(rule) => {
                queries::insert_policy_violation(
                    &state.db,
                    &user_id,
                    &team_id,
                    &channel_id,
                    &thread_ts,
                    "critical_deny",
                    &rule.id,
                    &request_text.chars().take(500).collect::<String>(),
                )
                .await?;
                let count = queries::count_recent_critical_violations(&state.db, &user_id).await?;
                if count >= 2 {
                    queries::insert_ban(&state.db, &user_id, "Repeated critical deny requests").await?;
                }
                state
                    .slack
                    .post_message(
                        &channel_id,
                        &thread_ts,
                        &format!(
                            "Request denied. This falls under the critical deny policy: {}.",
                            rule.title
                        ),
                    )
                    .await?;
                return Ok(RequestOutcome::Rejected);
            }
        }

        let existing_session = state
            .sessions
            .find_by_thread(&team_id, &channel_id, &thread_ts)
            .await?;

        let environment =
            match resolve_environment(&state, existing_session.as_ref(), &request_text).await {
                Ok(environment) => environment,
                Err(error) => {
                    state
                        .slack
                        .post_message(&channel_id, &thread_ts, &error.to_string())
                        .await?;
                    return Ok(RequestOutcome::Rejected);
                }
            };
        let workflow = selector::select_workflow(
            &state.config,
            &state.workflows,
            &request_text,
            &thread,
            environment.as_ref(),
        )
        .await
        .or_else(|| matcher::match_workflow(&state.workflows, &request_text, environment.as_ref()));

        let session = match existing_session {
            Some(session) => {
                enforce_environment_binding(&session, environment.as_ref())?;
                session
            }
            None => {
                let prepared = if let Some(environment) = environment.as_ref() {
                    let source_path = state
                        .workspaces
                        .ensure_source_clone(
                            &environment.slug,
                            &environment.git_ssh_url,
                            &environment.default_branch,
                        )
                        .await?;
                    state
                        .workspaces
                        .prepare_repo_workspace(&environment.slug, &source_path, None)
                        .await?
                } else {
                    state.workspaces.prepare_general_workspace(None).await?
                };

                state
                    .sessions
                    .create(
                        &team_id,
                        &channel_id,
                        &thread_ts,
                        &prepared.workspace_id,
                        &prepared.workspace_path.display().to_string(),
                        environment.as_ref().map(|item| item.id.as_str()),
                        workflow.as_ref().map(|item| item.metadata.id.as_str()),
                        "idle",
                    )
                    .await?
            }
        };

        if state.sessions.has_active_run(&session.id).await? {
            persist_and_send_reply(
                &state,
                &session.id,
                None,
                &channel_id,
                &thread_ts,
                "This thread already has an active task run. Wait for it to finish before sending another request.",
            )
            .await?;
            return Ok(RequestOutcome::Rejected);
        }

        state
            .sessions
            .update_status(
                &session.id,
                "running",
                workflow.as_ref().map(|item| item.metadata.id.as_str()),
            )
            .await?;
        let task_run = queries::insert_task_run(
            &state.db,
            &session.id,
            &trigger_message_ts,
            workflow.as_ref().map(|item| item.metadata.id.as_str()),
            workflow.as_ref().map(|item| item.metadata.name.as_str()),
            state.runner.kind(),
            "running",
        )
        .await?;

        let raw_inbound = json!({
            "text": text,
            "user_id": user_id,
            "channel_id": channel_id,
            "thread_ts": thread_ts,
        })
        .to_string();
        queries::insert_task_message(
            &state.db,
            &session.id,
            Some(&task_run.id),
            "inbound",
            Some(&user_id),
            &raw_inbound,
            &resolved_payload_json(&request_text),
        )
        .await?;

        let prompt = renderer::render_prompt(
            workflow.as_ref(),
            environment.as_ref(),
            &thread,
            &session.workspace_path,
        );

        let output = state
            .runner
            .run(RunnerInput {
                task_run_id: task_run.id.clone(),
                workspace_path: session.workspace_path.clone(),
                prompt,
            })
            .await?;

        let reply_text = if output.status == "succeeded" && !output.stdout.trim().is_empty() {
            output.stdout.trim().to_string()
        } else if !output.stderr.trim().is_empty() {
            format!("Task failed.\n{}", output.stderr.trim())
        } else {
            format!("Task finished with status {}", output.status)
        };

        queries::update_task_run_status(
            &state.db,
            &task_run.id,
            &output.status,
            output.exit_code,
            (!output.stderr.trim().is_empty()).then_some(output.stderr.trim()),
        )
        .await?;
        state
            .sessions
            .update_status(
                &session.id,
                "idle",
                workflow.as_ref().map(|item| item.metadata.id.as_str()),
            )
            .await?;

        persist_and_send_reply(
            &state,
            &session.id,
            Some(&task_run.id),
            &channel_id,
            &thread_ts,
            &reply_text,
        )
        .await?;

        info!(task_run_id = %task_run.id, "completed task run");
        Ok(RequestOutcome::Completed)
    }
    .await;

    match outcome {
        Ok(RequestOutcome::Completed) => {
            finish_processing_reaction(&state, &channel_id, &trigger_message_ts, true).await;
            Ok(())
        }
        Ok(RequestOutcome::Rejected) => {
            finish_processing_reaction(&state, &channel_id, &trigger_message_ts, false).await;
            Ok(())
        }
        Err(error) => {
            finish_processing_reaction(&state, &channel_id, &trigger_message_ts, false).await;
            Err(error)
        }
    }
}

enum RequestOutcome {
    Completed,
    Rejected,
}

fn enforce_environment_binding(
    session: &Session,
    requested_environment: Option<&Environment>,
) -> Result<()> {
    match (&session.environment_id, requested_environment) {
        (Some(existing_id), Some(requested)) if existing_id != &requested.id => {
            anyhow::bail!(
                "This thread already has a workspace bound to a different environment. Start a new thread to use another environment."
            )
        }
        (None, Some(_)) => {
            anyhow::bail!(
                "This thread already has a general workspace. Start a new thread to use an environment."
            )
        }
        _ => Ok(()),
    }
}

async fn resolve_environment(
    state: &AppState,
    existing_session: Option<&Session>,
    request_text: &str,
) -> Result<Option<Environment>> {
    let requested_from_prompt = state.environments.resolve_from_prompt(request_text).await?;

    if let Some(session) = existing_session {
        if let Some(environment_id) = &session.environment_id {
            let bound = queries::get_environment(&state.db, environment_id).await?;
            if let (Some(requested), Some(bound)) = (&requested_from_prompt, &bound) {
                if requested.id != bound.id {
                    anyhow::bail!("This thread is already bound to a different environment. Start a new thread to switch environments.");
                }
            }
            return Ok(bound);
        }
        if requested_from_prompt.is_some() {
            anyhow::bail!("This thread started as a general workspace and cannot be rebound to an environment.");
        }
        return Ok(None);
    }

    if let Some(environment) = requested_from_prompt {
        return Ok(Some(environment));
    }

    if let Some(workflow) = matcher::match_workflow(&state.workflows, request_text, None) {
        if let Some(default_environment) = workflow.metadata.default_environment {
            return queries::get_environment_by_slug(&state.db, &default_environment).await;
        }
    }

    Ok(None)
}

async fn begin_processing_reaction(state: &AppState, channel_id: &str, message_ts: &str) {
    if let Err(error) = state
        .slack
        .add_reaction(channel_id, message_ts, "eyes")
        .await
    {
        warn!(
            ?error,
            channel_id, message_ts, "failed to add eyes reaction"
        );
    }
}

async fn finish_processing_reaction(
    state: &AppState,
    channel_id: &str,
    message_ts: &str,
    success: bool,
) {
    if let Err(error) = state
        .slack
        .remove_reaction(channel_id, message_ts, "eyes")
        .await
    {
        warn!(
            ?error,
            channel_id, message_ts, "failed to remove eyes reaction"
        );
    }

    let final_reaction = if success { "white-tick" } else { "x" };
    if let Err(error) = state
        .slack
        .add_reaction(channel_id, message_ts, final_reaction)
        .await
    {
        warn!(
            ?error,
            channel_id, message_ts, final_reaction, "failed to add final request reaction"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::enforce_environment_binding;
    use crate::db::models::{Environment, Session};
    use chrono::Utc;

    fn sample_session(environment_id: Option<&str>) -> Session {
        Session {
            id: "session-1".to_string(),
            team_id: "T1".to_string(),
            channel_id: "C1".to_string(),
            thread_ts: "123.456".to_string(),
            workspace_id: "workspace-1".to_string(),
            workspace_path: "/tmp/workspace-1".to_string(),
            environment_id: environment_id.map(str::to_string),
            current_workflow_id: None,
            status: "idle".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_environment(id: &str) -> Environment {
        Environment {
            id: id.to_string(),
            name: format!("Environment {id}"),
            slug: format!("env-{id}"),
            git_ssh_url: "git@github.com:example/repo.git".to_string(),
            default_branch: "master".to_string(),
            aliases: "[]".to_string(),
            enabled: true,
            source_sync_status: "ready".to_string(),
            source_sync_error: None,
            source_synced_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn rejects_switching_to_another_environment_in_same_thread() {
        let session = sample_session(Some("env-1"));
        let requested = sample_environment("env-2");

        let error = enforce_environment_binding(&session, Some(&requested))
            .expect_err("thread should stay bound to its original environment");

        assert!(error
            .to_string()
            .contains("Start a new thread to use another environment"));
    }

    #[test]
    fn rejects_rebinding_general_thread_to_environment() {
        let session = sample_session(None);
        let requested = sample_environment("env-1");

        let error = enforce_environment_binding(&session, Some(&requested))
            .expect_err("general thread should not switch environments");

        assert!(error
            .to_string()
            .contains("Start a new thread to use an environment"));
    }
}
