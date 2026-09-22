//! Persistent state: the move journal, so undo survives a restart, and the
//! settings. Scan results stay in memory; they are cheap to recreate and
//! would be stale after a restart anyway.

use crate::models::{ActionKind, ActionStatus, OrganizeAction};
use anyhow::Result;
use sqlx::{Row, SqlitePool};

fn status_str(status: &ActionStatus) -> &'static str {
    match status {
        ActionStatus::Pending => "pending",
        ActionStatus::Applied => "applied",
        ActionStatus::Skipped => "skipped",
        ActionStatus::Failed(_) => "failed",
    }
}

// Plain runtime queries instead of `sqlx::query!`: the macro needs a prepared
// database at compile time (DATABASE_URL), so a plain `cargo build` from a
// fresh clone failed. The round trip is covered by tests/journal.rs.

// ── Actions ───────────────────────────────────────────────────

pub async fn insert_action(pool: &SqlitePool, action: &OrganizeAction) -> Result<()> {
    let kind = format!("{:?}", action.kind).to_lowercase();
    // Debug formatting turned Failed("…") into `failed("…")`, which read back
    // as pending.
    let status = status_str(&action.status);
    sqlx::query(
        "INSERT OR REPLACE INTO organize_actions(id, file_id, file_name, kind, source_path, target_path, reason, status, undoable)
         VALUES(?,?,?,?,?,?,?,?,?)",
    )
    .bind(&action.id)
    .bind(&action.file_id)
    .bind(&action.file_name)
    .bind(kind)
    .bind(&action.source_path)
    .bind(&action.target_path)
    .bind(&action.reason)
    .bind(status)
    .bind(action.undoable as i64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_action_status(pool: &SqlitePool, id: &str, status: &ActionStatus) -> Result<()> {
    sqlx::query("UPDATE organize_actions SET status=? WHERE id=?")
        .bind(status_str(status))
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_actions(pool: &SqlitePool) -> Result<Vec<OrganizeAction>> {
    let rows = sqlx::query(
        "SELECT id, file_id, file_name, kind, source_path, target_path, reason, status, undoable
         FROM organize_actions ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let kind: String = r.try_get("kind")?;
            let status: String = r.try_get("status")?;
            Ok(OrganizeAction {
                id: r.try_get("id")?,
                file_id: r.try_get("file_id")?,
                file_name: r.try_get("file_name")?,
                kind: match kind.as_str() {
                    "move" => ActionKind::Move,
                    "delete" => ActionKind::Delete,
                    "copy" => ActionKind::Copy,
                    "rename" => ActionKind::Rename,
                    _ => ActionKind::Tag,
                },
                source_path: r.try_get("source_path")?,
                target_path: r.try_get("target_path")?,
                reason: r.try_get("reason")?,
                status: match status.as_str() {
                    "applied" => ActionStatus::Applied,
                    "skipped" => ActionStatus::Skipped,
                    "failed" => ActionStatus::Failed(String::new()),
                    _ => ActionStatus::Pending,
                },
                undoable: r.try_get::<i64, _>("undoable")? != 0,
            })
        })
        .collect()
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let value = sqlx::query_scalar("SELECT value FROM app_settings WHERE key=?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(value)
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO app_settings(key, value) VALUES(?,?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}
