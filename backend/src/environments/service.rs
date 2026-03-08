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

    pub async fn tasks(&self, environment_id: &str) -> Result<Vec<crate::db::models::TaskRun>> {
        queries::list_task_runs_for_environment(&self.pool, environment_id).await
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
            if let Err(error) = queries::update_environment_source_status(
                &pool,
                &environment.id,
                "syncing",
                None,
                None,
            )
            .await
            {
                error!(environment_id = %environment.id, ?error, "failed to mark environment as syncing");
            }

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
                    info!(
                        environment_id = %environment.id,
                        environment_slug = %environment.slug,
                        source_path = %source_path.display(),
                        "environment source clone synced"
                    );
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
                    error!(
                        environment_id = %environment.id,
                        environment_slug = %environment.slug,
                        ?sync_error,
                        "failed to sync environment source clone"
                    );
                    if let Err(error) = queries::update_environment_source_status(
                        &pool,
                        &environment.id,
                        "failed",
                        Some(&sync_error.to_string()),
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
