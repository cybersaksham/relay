use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::models::Ban;
use crate::db::queries;

pub async fn active_ban(pool: &SqlitePool, slack_user_id: &str) -> Result<Option<Ban>> {
    queries::get_active_ban(pool, slack_user_id).await
}
