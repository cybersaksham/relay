use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serde_json::json;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, warn};

use crate::app_state::AppState;
use crate::config::SharedConfig;
use crate::db::models::{Environment, Session};
use crate::db::queries;
use crate::policies::evaluator::PolicyDecision;
use crate::runner::{RunnerInput, RunnerOutput};
use crate::slack::formatter::{resolve_slack_text, resolved_payload_json};
use crate::slack::thread_context::normalize_thread;
use crate::slack::SlackEventEnvelope;
use crate::tasks::reply_service::persist_and_send_reply;
use crate::workflows::loader::{WorkflowDefinition, WorkflowRegistry};
use crate::workflows::{matcher, renderer, selector};

const PLAYWRIGHT_TASK_WORKFLOW_ID: &str = "playwright-task";
const PLAYWRIGHT_CLI_BLOCKED_MESSAGE: &str =
    "Browser CLI prerequisites/network are unavailable right now; task was blocked before execution.";

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
        if !state.policies.is_master(&user_id) {
            state
                .slack
                .post_message(
                    &channel_id,
                    &thread_ts,
                    &format!(
                        "<@{}>, This bot is enabled only for Saksham right now. We will roll out for others soon!!",
                        user_id
                    ),
                )
                .await?;
            return Ok(RequestOutcome::Rejected);
        }

        let messages = state.slack.fetch_thread(&channel_id, &thread_ts).await?;
        let thread = normalize_thread(&channel_id, &thread_ts, messages)?;
        let request_text = resolve_slack_text(&text);
        let execution_policy = classify_request_execution_policy(&request_text);

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
        let workflow = if let Some(pinned) = pin_playwright_workflow(&state.workflows, &execution_policy)
        {
            Some(pinned)
        } else {
            selector::select_workflow(
                &state.config,
                &state.workflows,
                &request_text,
                &thread,
                environment.as_ref(),
            )
            .await
            .or_else(|| {
                matcher::match_workflow(&state.workflows, &request_text, environment.as_ref())
            })
        };

        let mut workspace_setup_script_to_run: Option<String> = None;
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
                if prepared.created {
                    workspace_setup_script_to_run = environment
                        .as_ref()
                        .and_then(|item| normalize_script(item.workspace_setup_script.as_deref()));
                }

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

        if should_post_task_kickoff_message(environment.as_ref()) {
            let kickoff_message = build_task_kickoff_message(
                &state.config,
                &session,
                environment.as_ref(),
                workflow.as_ref(),
            );
            persist_and_send_reply(
                &state,
                &session.id,
                Some(&task_run.id),
                &channel_id,
                &thread_ts,
                &kickoff_message,
            )
            .await?;
        }

        if let Some(workspace_setup_script) = workspace_setup_script_to_run.as_deref() {
            let hook_result = state
                .workspaces
                .run_shell_hook(Path::new(&session.workspace_path), workspace_setup_script)
                .await;
            persist_workspace_hook_terminal_output(&state.db, &task_run.id, &hook_result).await?;

            if !hook_result.succeeded() {
                let hook_error_summary =
                    summarize_hook_failure_for_task(&hook_result.stderr, &hook_result.stdout);
                queries::update_task_run_status(
                    &state.db,
                    &task_run.id,
                    "failed",
                    hook_result.exit_code,
                    Some(&hook_error_summary),
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
                    &format!("Task failed before execution. {hook_error_summary}"),
                )
                .await?;
                return Ok(RequestOutcome::Rejected);
            }
        }

        if is_playwright_task_workflow(workflow.as_ref())
            && execution_policy.explicit_cli_playwright_requested
        {
            if let Err(error) = run_playwright_cli_preflight(&state.config).await {
                let error_summary = error.summary();
                queries::update_task_run_status(
                    &state.db,
                    &task_run.id,
                    "blocked",
                    None,
                    Some(&error_summary),
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
                    PLAYWRIGHT_CLI_BLOCKED_MESSAGE,
                )
                .await?;
                return Ok(RequestOutcome::Rejected);
            }
        }

        let prompt = renderer::render_prompt(
            workflow.as_ref(),
            environment.as_ref(),
            &thread,
            &session.workspace_path,
        );
        let prompt = apply_browser_execution_directive(prompt, &execution_policy);

        let output = state
            .runner
            .run(RunnerInput {
                task_run_id: task_run.id.clone(),
                workspace_path: session.workspace_path.clone(),
                prompt,
                timeout_seconds: is_playwright_task_workflow(workflow.as_ref())
                    .then_some(state.config.codex.browser_task_timeout_seconds),
            })
            .await?;

        let reply_text = build_reply_text(&output);
        let error_summary = summarize_error_for_storage(&output);

        queries::update_task_run_status(
            &state.db,
            &task_run.id,
            &output.status,
            output.exit_code,
            error_summary.as_deref(),
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

#[derive(Debug, Clone, Default)]
struct RequestExecutionPolicy {
    is_browser_task: bool,
    explicit_playwright_requested: bool,
    explicit_cli_playwright_requested: bool,
}

#[derive(Debug)]
enum CliPreflightError {
    MissingNpx,
    TimedOut(String),
    WrapperUnavailable(String),
    NetworkUnavailable(String),
}

impl CliPreflightError {
    fn summary(&self) -> String {
        match self {
            Self::MissingNpx => {
                "Playwright CLI preflight failed: npx is not available.".to_string()
            }
            Self::TimedOut(step) => {
                format!("Playwright CLI preflight timed out while {step}.")
            }
            Self::WrapperUnavailable(message) => {
                format!("Playwright CLI preflight failed: {message}")
            }
            Self::NetworkUnavailable(message) => {
                format!("Playwright CLI preflight network failure: {message}")
            }
        }
    }
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

fn classify_request_execution_policy(request_text: &str) -> RequestExecutionPolicy {
    let normalized = request_text.to_lowercase();
    let is_browser_task = [
        "playwright",
        "browser",
        "ui test",
        "screenshot",
        "open url",
        "navigate",
    ]
    .iter()
    .any(|phrase| normalized.contains(phrase));
    let explicit_playwright_requested = normalized.contains("playwright");
    let explicit_cli_playwright_requested =
        normalized.contains("playwright cli") || normalized.contains("cli skill");

    RequestExecutionPolicy {
        is_browser_task,
        explicit_playwright_requested,
        explicit_cli_playwright_requested,
    }
}

fn pin_playwright_workflow(
    registry: &WorkflowRegistry,
    policy: &RequestExecutionPolicy,
) -> Option<WorkflowDefinition> {
    if policy.explicit_playwright_requested {
        registry.get(PLAYWRIGHT_TASK_WORKFLOW_ID)
    } else {
        None
    }
}

fn is_playwright_task_workflow(workflow: Option<&WorkflowDefinition>) -> bool {
    workflow
        .map(|item| item.metadata.id.as_str() == PLAYWRIGHT_TASK_WORKFLOW_ID)
        .unwrap_or(false)
}

fn apply_browser_execution_directive(prompt: String, policy: &RequestExecutionPolicy) -> String {
    if !policy.is_browser_task {
        return prompt;
    }

    let mut directive = String::from(
        "Execution policy for browser tasks:\n\
         - Prefer Playwright MCP/browser tools by default.\n\
         - Use Playwright CLI only if the request explicitly asks for CLI.\n",
    );
    if policy.explicit_cli_playwright_requested {
        directive
            .push_str("- This request explicitly asked for Playwright CLI; CLI path is allowed.\n");
    }
    directive.push('\n');
    directive.push_str(&prompt);
    directive
}

async fn run_playwright_cli_preflight(config: &SharedConfig) -> Result<(), CliPreflightError> {
    let timeout_seconds = config.codex.playwright_cli_preflight_timeout_seconds;

    let mut npx_check = Command::new("sh");
    npx_check
        .arg("-lc")
        .arg("command -v npx >/dev/null 2>&1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let npx_status = timeout(Duration::from_secs(timeout_seconds), npx_check.status())
        .await
        .map_err(|_| CliPreflightError::TimedOut("checking npx".to_string()))?
        .map_err(|error| CliPreflightError::WrapperUnavailable(error.to_string()))?;
    if !npx_status.success() {
        return Err(CliPreflightError::MissingNpx);
    }

    let mut wrapper_check = Command::new(&config.codex.playwright_cli_wrapper);
    wrapper_check
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let output = timeout(Duration::from_secs(timeout_seconds), wrapper_check.output())
        .await
        .map_err(|_| CliPreflightError::TimedOut("checking Playwright CLI wrapper".to_string()))?
        .map_err(|error| CliPreflightError::WrapperUnavailable(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let excerpt = compact_error_excerpt(&stderr);
    if contains_network_prereq_failure(&stderr) {
        Err(CliPreflightError::NetworkUnavailable(excerpt))
    } else {
        Err(CliPreflightError::WrapperUnavailable(excerpt))
    }
}

fn contains_network_prereq_failure(message: &str) -> bool {
    let normalized = message.to_lowercase();
    [
        "enotfound",
        "eai_again",
        "etimedout",
        "registry.npmjs.org",
        "network request failed",
        "npm error network",
    ]
    .iter()
    .any(|pattern| normalized.contains(pattern))
}

fn compact_error_excerpt(message: &str) -> String {
    if message.trim().is_empty() {
        return "preflight check failed without stderr output".to_string();
    }
    message
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(220)
        .collect()
}

fn normalize_script(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
}

fn summarize_hook_failure_for_task(stderr: &str, stdout: &str) -> String {
    let detail = if !stderr.trim().is_empty() {
        compact_error_excerpt(stderr)
    } else if !stdout.trim().is_empty() {
        compact_error_excerpt(stdout)
    } else {
        "workspace setup script failed with no output".to_string()
    };
    format!("Workspace setup script failed: {detail}")
}

async fn persist_workspace_hook_terminal_output(
    pool: &sqlx::SqlitePool,
    task_run_id: &str,
    result: &crate::workspaces::session_workspace::HookRunResult,
) -> Result<()> {
    let mut sequence = 0_i64;
    let header = "[workspace setup] Running workspace setup script\n";
    queries::insert_terminal_event(pool, task_run_id, "stdout", header, sequence).await?;
    sequence += 1;

    for line in result.stdout.lines() {
        queries::insert_terminal_event(pool, task_run_id, "stdout", &format!("{line}\n"), sequence)
            .await?;
        sequence += 1;
    }

    for line in result.stderr.lines() {
        queries::insert_terminal_event(pool, task_run_id, "stderr", &format!("{line}\n"), sequence)
            .await?;
        sequence += 1;
    }

    if result.timed_out {
        queries::insert_terminal_event(
            pool,
            task_run_id,
            "stderr",
            "[workspace setup] script timed out\n",
            sequence,
        )
        .await?;
    } else {
        let status_line = if result.succeeded() {
            "[workspace setup] completed successfully\n"
        } else {
            "[workspace setup] failed\n"
        };
        queries::insert_terminal_event(pool, task_run_id, "stdout", status_line, sequence).await?;
    }

    Ok(())
}

fn build_task_kickoff_message(
    config: &SharedConfig,
    session: &Session,
    environment: Option<&Environment>,
    workflow: Option<&WorkflowDefinition>,
) -> String {
    let environment_label = environment
        .map(|item| item.name.as_str())
        .unwrap_or("general");
    let workflow_label = workflow
        .map(|item| item.metadata.name.as_str())
        .unwrap_or("Generic run");
    let task_url = format!(
        "{}/tasks/{}",
        config.server.portal_base_url.trim_end_matches('/'),
        session.id
    );
    format!(
        "Kicked off the task in environment `{environment_label}` using workflow `{workflow_label}`.\nOpen task: {task_url}"
    )
}

fn should_post_task_kickoff_message(environment: Option<&Environment>) -> bool {
    environment.is_some()
}

fn build_reply_text(output: &RunnerOutput) -> String {
    match output.status.as_str() {
        "cancelled" => "Task cancelled manually.".to_string(),
        "timed_out" => "Task timed out before completion.".to_string(),
        "succeeded" if !output.stdout.trim().is_empty() => output.stdout.trim().to_string(),
        _ if contains_network_prereq_failure(&output.stderr) => {
            "Task failed due to browser runtime prerequisites/network availability.".to_string()
        }
        _ if !output.stderr.trim().is_empty() => {
            format!(
                "Task failed.\n{}",
                compact_error_excerpt(output.stderr.trim())
            )
        }
        _ => format!("Task finished with status {}", output.status),
    }
}

fn summarize_error_for_storage(output: &RunnerOutput) -> Option<String> {
    match output.status.as_str() {
        "succeeded" => None,
        "cancelled" => Some("Task was cancelled manually.".to_string()),
        "timed_out" => Some("Task execution timed out before completion.".to_string()),
        _ if contains_network_prereq_failure(&output.stderr) => {
            Some("Browser runtime prerequisites/network availability issue.".to_string())
        }
        _ if !output.stderr.trim().is_empty() => Some(compact_error_excerpt(output.stderr.trim())),
        _ => Some(format!("Task finished with status {}", output.status)),
    }
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
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::{enforce_environment_binding, should_post_task_kickoff_message};
    use crate::db::models::{Environment, Session};
    use crate::workflows::loader::{WorkflowDefinition, WorkflowMetadata, WorkflowRegistry};
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
            source_setup_script: None,
            workspace_setup_script: None,
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

    #[test]
    fn classifies_browser_and_cli_playwright_requests() {
        let policy = super::classify_request_execution_policy(
            "Use Playwright CLI skill to open URL and count cards",
        );
        assert!(policy.is_browser_task);
        assert!(policy.explicit_playwright_requested);
        assert!(policy.explicit_cli_playwright_requested);
    }

    #[test]
    fn classifies_non_browser_requests() {
        let policy = super::classify_request_execution_policy("Please review PR #123");
        assert!(!policy.is_browser_task);
        assert!(!policy.explicit_playwright_requested);
        assert!(!policy.explicit_cli_playwright_requested);
    }

    #[test]
    fn pins_playwright_workflow_for_explicit_playwright_prompt() {
        let registry = WorkflowRegistry::from_workflows(HashMap::from([(
            "playwright-task".to_string(),
            WorkflowDefinition {
                metadata: WorkflowMetadata {
                    id: "playwright-task".to_string(),
                    name: "Playwright Task".to_string(),
                    scope: "global".to_string(),
                    environment_slug: None,
                    trigger_phrases: vec!["playwright".to_string()],
                    default_environment: None,
                    instructions: vec!["Use browser automation".to_string()],
                    response_mode: "reply".to_string(),
                },
                prompt: "Prompt".to_string(),
                root_dir: PathBuf::from(".workflows/global/playwright-task"),
            },
        )]));
        let policy = super::classify_request_execution_policy("Run this with playwright.");

        let workflow =
            super::pin_playwright_workflow(&registry, &policy).expect("workflow should be pinned");
        assert_eq!(workflow.metadata.id, "playwright-task");
    }

    #[test]
    fn network_precheck_error_signatures_detected() {
        assert!(super::contains_network_prereq_failure(
            "npm error code ENOTFOUND registry.npmjs.org"
        ));
        assert!(super::contains_network_prereq_failure(
            "request failed with EAI_AGAIN"
        ));
        assert!(!super::contains_network_prereq_failure(
            "permission denied opening wrapper"
        ));
    }

    #[test]
    fn applies_browser_execution_directive_for_browser_tasks() {
        let policy =
            super::classify_request_execution_policy("Use playwright to take a screenshot");
        let prompt = super::apply_browser_execution_directive("Base prompt".to_string(), &policy);
        assert!(prompt.contains("Prefer Playwright MCP/browser tools by default"));
        assert!(prompt.contains("Base prompt"));
    }

    #[test]
    fn browser_directive_allows_cli_when_explicitly_requested() {
        let policy = super::classify_request_execution_policy("Use Playwright CLI skill");
        let prompt = super::apply_browser_execution_directive("Base prompt".to_string(), &policy);
        assert!(prompt.contains("CLI path is allowed"));
    }

    #[test]
    fn skips_kickoff_message_for_general_threads() {
        assert!(!should_post_task_kickoff_message(None));
        assert!(should_post_task_kickoff_message(Some(&sample_environment(
            "env-1"
        ))));
    }
}
