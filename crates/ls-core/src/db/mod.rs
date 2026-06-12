pub mod queries;

use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;

pub async fn open(db_path: &Path) -> Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    sqlx::migrate!("src/db/migrations").run(&pool).await?;
    Ok(pool)
}
