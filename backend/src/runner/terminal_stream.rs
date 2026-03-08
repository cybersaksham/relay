use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::queries;

pub async fn persist_chunk(
    pool: &SqlitePool,
    task_run_id: &str,
    stream: &str,
    chunk: &str,
    sequence: i64,
) -> Result<()> {
    queries::insert_terminal_event(pool, task_run_id, stream, chunk, sequence).await?;
    Ok(())
}
