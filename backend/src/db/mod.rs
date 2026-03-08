pub mod models;
pub mod queries;

use std::path::PathBuf;

use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, SqlitePool};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        ensure_sqlite_path(url).await?;
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

async fn ensure_sqlite_path(url: &str) -> anyhow::Result<()> {
    if !url.starts_with("sqlite:") {
        return Ok(());
    }

    let raw_path = url.trim_start_matches("sqlite:");
    if raw_path.is_empty() || raw_path == ":memory:" {
        return Ok(());
    }

    let path_without_query = raw_path.split('?').next().unwrap_or_default();
    let normalized = if let Some(stripped) = path_without_query.strip_prefix("//") {
        stripped
    } else {
        path_without_query
    };

    if normalized.is_empty() {
        return Ok(());
    }

    let path = PathBuf::from(normalized);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        tokio::fs::create_dir_all(parent).await?;
    }

    if tokio::fs::metadata(&path).await.is_err() {
        tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .await?;
    }

    Ok(())
}
