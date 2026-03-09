use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{error, info};

use crate::db::models::Environment;
use crate::db::queries;
use crate::workspaces::session_workspace::WorkspaceManager;

use super::git;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateEnvironmentInput {
    pub name: String,
    pub slug: String,
    pub git_ssh_url: String,
    pub default_branch: String,
    pub aliases: Vec<String>,
    pub enabled: Option<bool>,
    pub source_setup_script: Option<String>,
    pub workspace_setup_script: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentWithPaths {
    pub environment: Environment,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteEnvironmentResponse {
    pub deleted_id: String,
}

#[derive(Clone)]
pub struct EnvironmentService {
    pool: SqlitePool,
    workspace_manager: Arc<WorkspaceManager>,
}

impl EnvironmentService {
    pub fn new(pool: SqlitePool, workspace_manager: Arc<WorkspaceManager>) -> Self {
        Self {
            pool,
            workspace_manager,
        }
    }

    pub async fn list(&self) -> Result<Vec<Environment>> {
        queries::list_environments(&self.pool).await
    }

    pub async fn count(&self) -> Result<i64> {
        queries::count_environments(&self.pool).await
    }

    pub async fn get(&self, id: &str) -> Result<Option<Environment>> {
        queries::get_environment(&self.pool, id).await
    }

    pub async fn create(&self, input: CreateEnvironmentInput) -> Result<EnvironmentWithPaths> {
        let source_setup_script = normalize_script(input.source_setup_script.as_deref());
        let workspace_setup_script = normalize_script(input.workspace_setup_script.as_deref());

        if queries::get_environment_by_slug(&self.pool, &input.slug)
            .await?
            .is_some()
        {
            return Err(anyhow!("environment slug already exists"));
        }

        git::validate_remote_access(&input.git_ssh_url, &input.default_branch).await?;

        let aliases = serde_json::to_string(&input.aliases)?;
        let environment = queries::insert_environment(
            &self.pool,
            &input.name,
            &input.slug,
            &input.git_ssh_url,
            &input.default_branch,
            &aliases,
            input.enabled.unwrap_or(true),
            source_setup_script.as_deref(),
            workspace_setup_script.as_deref(),
        )
        .await?;

        self.spawn_source_sync(environment.clone());

        Ok(EnvironmentWithPaths {
            source_path: self
                .source_path_for_slug(&environment.slug)
                .display()
                .to_string(),
            environment,
        })
    }

    pub async fn update(
        &self,
        id: &str,
        input: CreateEnvironmentInput,
    ) -> Result<EnvironmentWithPaths> {
        let source_setup_script = normalize_script(input.source_setup_script.as_deref());
        let workspace_setup_script = normalize_script(input.workspace_setup_script.as_deref());
        let existing = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow!("environment not found"))?;

        if queries::get_environment_by_slug_excluding_id(&self.pool, &input.slug, id)
            .await?
            .is_some()
        {
            return Err(anyhow!("environment slug already exists"));
        }

        git::validate_remote_access(&input.git_ssh_url, &input.default_branch).await?;

        let aliases = serde_json::to_string(&input.aliases)?;
        let environment = queries::update_environment(
            &self.pool,
            id,
            &input.name,
            &input.slug,
            &input.git_ssh_url,
            &input.default_branch,
            &aliases,
            input.enabled.unwrap_or(true),
            source_setup_script.as_deref(),
            workspace_setup_script.as_deref(),
        )
        .await?;

        self.spawn_source_sync_with_previous(existing.slug, environment.clone());

        Ok(EnvironmentWithPaths {
            source_path: self
                .source_path_for_slug(&environment.slug)
                .display()
                .to_string(),
            environment,
        })
    }

    pub async fn delete(&self, id: &str) -> Result<DeleteEnvironmentResponse> {
        let environment = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow!("environment not found"))?;

        if queries::count_sessions_for_environment(&self.pool, id).await? > 0 {
            return Err(anyhow!(
                "environment cannot be deleted because tasks already reference it"
            ));
        }

        queries::delete_environment(&self.pool, id).await?;
        self.workspace_manager
            .delete_source_clone(&environment.slug)
            .await?;

        Ok(DeleteEnvironmentResponse {
            deleted_id: environment.id,
        })
    }

    pub async fn refresh_source(&self, id: &str) -> Result<EnvironmentWithPaths> {
        let environment = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow!("environment not found"))?;

        if environment.source_sync_status == "syncing" {
            return Err(anyhow!("environment source sync already in progress"));
        }

        queries::update_environment_source_status(&self.pool, id, "syncing", None, None).await?;
        let refreshed = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow!("environment not found"))?;
        self.spawn_source_sync(refreshed.clone());

        Ok(EnvironmentWithPaths {
            source_path: self
                .source_path_for_slug(&refreshed.slug)
                .display()
                .to_string(),
            environment: refreshed,
        })
    }

    pub async fn resolve_from_prompt(&self, prompt: &str) -> Result<Option<Environment>> {
        let prompt = prompt.to_lowercase();
        let environments = self.list().await?;
        let mut matches = Vec::new();

        for environment in environments {
            let aliases: Vec<String> =
                serde_json::from_str(&environment.aliases).unwrap_or_default();
            if prompt.contains(&environment.slug.to_lowercase())
                || prompt.contains(&environment.name.to_lowercase())
                || aliases
                    .iter()
                    .any(|alias| prompt.contains(&alias.to_lowercase()))
            {
                matches.push(environment);
            }
        }

        if matches.len() == 1 {
            Ok(matches.into_iter().next())
        } else {
            Ok(None)
        }
    }

    pub fn source_path_for_slug(&self, slug: &str) -> PathBuf {
        self.workspace_manager.source_path(slug)
    }

    fn spawn_source_sync(&self, environment: Environment) {
        self.spawn_source_sync_with_previous(environment.slug.clone(), environment);
    }

    fn spawn_source_sync_with_previous(&self, previous_slug: String, environment: Environment) {
        let pool = self.pool.clone();
        let workspace_manager = self.workspace_manager.clone();
        tokio::spawn(async move {
            let mut sync_sequence = 0_i64;
            if let Err(error) = queries::clear_environment_sync_events(&pool, &environment.id).await
            {
                error!(environment_id = %environment.id, ?error, "failed to clear environment sync events");
            }
            append_sync_log(
                &pool,
                &environment.id,
                "stdout",
                "Starting source cache sync...",
                &mut sync_sequence,
            )
            .await;
            append_sync_log(
                &pool,
                &environment.id,
                "stdout",
                &format!(
                    "Syncing {} on branch {}",
                    environment.git_ssh_url, environment.default_branch
                ),
                &mut sync_sequence,
            )
            .await;

            let sync_result = workspace_manager
                .reset_source_clone(
                    &previous_slug,
                    &environment.slug,
                    &environment.git_ssh_url,
                    &environment.default_branch,
                )
                .await;

            match sync_result {
                Ok(source_path) => {
                    append_sync_log(
                        &pool,
                        &environment.id,
                        "stdout",
                        &format!("Source cache ready at {}", source_path.display()),
                        &mut sync_sequence,
                    )
                    .await;
                    if let Some(source_setup_script) =
                        normalize_script(environment.source_setup_script.as_deref())
                    {
                        append_sync_log(
                            &pool,
                            &environment.id,
                            "stdout",
                            "[source setup] running configured setup script",
                            &mut sync_sequence,
                        )
                        .await;
                        let hook_result = workspace_manager
                            .run_shell_hook(&source_path, &source_setup_script)
                            .await;
                        append_sync_output(
                            &pool,
                            &environment.id,
                            &hook_result.stdout,
                            &hook_result.stderr,
                            &mut sync_sequence,
                        )
                        .await;
                        if !hook_result.succeeded() {
                            let summary = summarize_hook_failure(
                                "Source setup script failed",
                                &hook_result.stderr,
                                &hook_result.stdout,
                            );
                            error!(
                                environment_id = %environment.id,
                                environment_slug = %environment.slug,
                                source_path = %source_path.display(),
                                exit_code = ?hook_result.exit_code,
                                timed_out = hook_result.timed_out,
                                stdout = %hook_result.stdout,
                                stderr = %hook_result.stderr,
                                "source setup hook failed"
                            );
                            append_sync_log(
                                &pool,
                                &environment.id,
                                "stderr",
                                &summary,
                                &mut sync_sequence,
                            )
                            .await;
                            if let Err(error) = queries::update_environment_source_status(
                                &pool,
                                &environment.id,
                                "failed",
                                Some(&summary),
                                None,
                            )
                            .await
                            {
                                error!(environment_id = %environment.id, ?error, "failed to mark environment as failed");
                            }
                            return;
                        }
                        append_sync_log(
                            &pool,
                            &environment.id,
                            "stdout",
                            "[source setup] completed successfully",
                            &mut sync_sequence,
                        )
                        .await;
                        if !hook_result.stdout.trim().is_empty()
                            || !hook_result.stderr.trim().is_empty()
                        {
                            info!(
                                environment_id = %environment.id,
                                environment_slug = %environment.slug,
                                source_path = %source_path.display(),
                                stdout = %hook_result.stdout,
                                stderr = %hook_result.stderr,
                                "source setup hook completed"
                            );
                        }
                    }

                    info!(
                        environment_id = %environment.id,
                        environment_slug = %environment.slug,
                        source_path = %source_path.display(),
                        "environment source clone synced"
                    );
                    append_sync_log(
                        &pool,
                        &environment.id,
                        "stdout",
                        "Environment source cache sync completed.",
                        &mut sync_sequence,
                    )
                    .await;
                    if let Err(error) = queries::update_environment_source_status(
                        &pool,
                        &environment.id,
                        "ready",
                        None,
                        Some(Utc::now()),
                    )
                    .await
                    {
                        error!(environment_id = %environment.id, ?error, "failed to mark environment as ready");
                    }
                }
                Err(sync_error) => {
                    let summary = compact_error_excerpt(&sync_error.to_string());
                    error!(
                        environment_id = %environment.id,
                        environment_slug = %environment.slug,
                        ?sync_error,
                        "failed to sync environment source clone"
                    );
                    append_sync_log(
                        &pool,
                        &environment.id,
                        "stderr",
                        &format!("Source sync failed: {summary}"),
                        &mut sync_sequence,
                    )
                    .await;
                    if let Err(error) = queries::update_environment_source_status(
                        &pool,
                        &environment.id,
                        "failed",
                        Some(&summary),
                        None,
                    )
                    .await
                    {
                        error!(environment_id = %environment.id, ?error, "failed to mark environment as failed");
                    }
                }
            }
        });
    }
}

fn normalize_script(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
}

fn compact_error_excerpt(message: &str) -> String {
    if message.trim().is_empty() {
        return "hook command failed without stderr output".to_string();
    }
    message
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(220)
        .collect()
}

fn summarize_hook_failure(prefix: &str, stderr: &str, stdout: &str) -> String {
    let detail = if !stderr.trim().is_empty() {
        compact_error_excerpt(stderr)
    } else if !stdout.trim().is_empty() {
        compact_error_excerpt(stdout)
    } else {
        "no output captured".to_string()
    };
    format!("{prefix}: {detail}")
}

async fn append_sync_log(
    pool: &SqlitePool,
    environment_id: &str,
    stream: &str,
    message: &str,
    sequence: &mut i64,
) {
    let chunk = if message.ends_with('\n') {
        message.to_string()
    } else {
        format!("{message}\n")
    };
    if let Err(error) =
        queries::insert_environment_sync_event(pool, environment_id, stream, &chunk, *sequence)
            .await
    {
        error!(environment_id = %environment_id, ?error, "failed to insert sync event");
    }
    *sequence += 1;
}

async fn append_sync_output(
    pool: &SqlitePool,
    environment_id: &str,
    stdout: &str,
    stderr: &str,
    sequence: &mut i64,
) {
    for line in stdout.lines() {
        append_sync_log(pool, environment_id, "stdout", line, sequence).await;
    }
    for line in stderr.lines() {
        append_sync_log(pool, environment_id, "stderr", line, sequence).await;
    }
}
