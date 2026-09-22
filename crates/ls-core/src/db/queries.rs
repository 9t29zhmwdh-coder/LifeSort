//! Persistent state: the move journal, so undo survives a restart, and the
//! settings. Scan results stay in memory; they are cheap to recreate and
//! would be stale after a restart anyway.

use crate::models::{ActionKind, ActionStatus, OrganizeAction};
use anyhow::Result;
use sqlx::SqlitePool;

fn status_str(status: &ActionStatus) -> &'static str {
    match status {
        ActionStatus::Pending => "pending",
        ActionStatus::Applied => "applied",
        ActionStatus::Skipped => "skipped",
        ActionStatus::Failed(_) => "failed",
    }
}

// ── Actions ───────────────────────────────────────────────────

pub async fn insert_action(pool: &SqlitePool, action: &OrganizeAction) -> Result<()> {
    let kind = format!("{:?}", action.kind).to_lowercase();
    // Debug formatting turned Failed("…") into `failed("…")`, which read back
    // as pending.
    let status = status_str(&action.status);
    let undoable = if action.undoable { 1i64 } else { 0 };
    sqlx::query!(
        "INSERT OR REPLACE INTO organize_actions(id, file_id, file_name, kind, source_path, target_path, reason, status, undoable)
         VALUES(?,?,?,?,?,?,?,?,?)",
        action.id, action.file_id, action.file_name, kind,
        action.source_path, action.target_path, action.reason, status, undoable
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_action_status(pool: &SqlitePool, id: &str, status: &ActionStatus) -> Result<()> {
    let s = status_str(status);
    sqlx::query!("UPDATE organize_actions SET status=? WHERE id=?", s, id)
        .execute(pool).await?;
    Ok(())
}

pub async fn list_actions(pool: &SqlitePool) -> Result<Vec<OrganizeAction>> {
    let rows = sqlx::query!(
        "SELECT * FROM organize_actions ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| OrganizeAction {
        id: r.id.unwrap_or_default(),
        file_id: r.file_id,
        file_name: r.file_name,
        kind: match r.kind.as_str() {
            "move"   => ActionKind::Move,
            "delete" => ActionKind::Delete,
            "copy"   => ActionKind::Copy,
            "rename" => ActionKind::Rename,
            _        => ActionKind::Tag,
        },
        source_path: r.source_path,
        target_path: r.target_path,
        reason: r.reason,
        status: match r.status.as_str() {
            "applied"  => ActionStatus::Applied,
            "skipped"  => ActionStatus::Skipped,
            "failed"   => ActionStatus::Failed(String::new()),
            _          => ActionStatus::Pending,
        },
        undoable: r.undoable != 0,
    }).collect())
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row = sqlx::query!("SELECT value FROM app_settings WHERE key=?", key)
        .fetch_optional(pool).await?;
    Ok(row.map(|r| r.value))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query!("INSERT OR REPLACE INTO app_settings(key, value) VALUES(?,?)", key, value)
        .execute(pool).await?;
    Ok(())
}
