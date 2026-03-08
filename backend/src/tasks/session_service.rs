use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::models::{Session, TaskRun};
use crate::db::queries;

#[derive(Clone)]
pub struct SessionService {
    pool: SqlitePool,
}

impl SessionService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_thread(
        &self,
        team_id: &str,
        channel_id: &str,
        thread_ts: &str,
    ) -> Result<Option<Session>> {
        queries::get_session_by_thread(&self.pool, team_id, channel_id, thread_ts).await
    }

    pub async fn create(
        &self,
        team_id: &str,
        channel_id: &str,
        thread_ts: &str,
        workspace_id: &str,
        workspace_path: &str,
        environment_id: Option<&str>,
        workflow_id: Option<&str>,
        status: &str,
    ) -> Result<Session> {
        queries::insert_session(
            &self.pool,
            team_id,
            channel_id,
            thread_ts,
            workspace_id,
            workspace_path,
            environment_id,
            workflow_id,
            status,
        )
        .await
    }

    pub async fn update_status(
        &self,
        session_id: &str,
        status: &str,
        workflow_id: Option<&str>,
    ) -> Result<()> {
        queries::update_session_status(&self.pool, session_id, status, workflow_id).await
    }

    pub async fn has_active_run(&self, session_id: &str) -> Result<bool> {
        queries::has_active_run(&self.pool, session_id).await
    }

    pub async fn latest_run(&self, session_id: &str) -> Result<Option<TaskRun>> {
        queries::get_latest_run_for_session(&self.pool, session_id).await
    }
}
