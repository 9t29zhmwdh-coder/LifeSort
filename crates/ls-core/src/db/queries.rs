#![allow(unused_imports)]
use crate::models::{
    ActionKind, ActionStatus, Classification, ClassifierKind, FileEntry, FileKind,
    OrganizeAction,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

// ── Session ───────────────────────────────────────────────────

pub async fn insert_session(pool: &SqlitePool, id: &str, path: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query!(
        "INSERT INTO scan_sessions(id, path, created_at) VALUES(?,?,?)",
        id, path, now
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_session_count(pool: &SqlitePool, id: &str, count: i64) -> Result<()> {
    sqlx::query!(
        "UPDATE scan_sessions SET file_count=?, status='done' WHERE id=?",
        count, id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ── File Entries ──────────────────────────────────────────────

pub async fn insert_file(pool: &SqlitePool, entry: &FileEntry) -> Result<()> {
    let kind = format!("{:?}", entry.kind).to_lowercase();
    let tags = serde_json::to_string(&entry.tags)?;
    let (category, confidence, classified_by, extracted_date, extracted_amount, extracted_sender,
         ai_summary, subcategory) = if let Some(ref c) = entry.classification {
        (
            Some(format!("{:?}", c.category)),
            Some(c.confidence),
            Some(format!("{:?}", c.classified_by)),
            c.extracted_date.map(|d| d.to_string()),
            c.extracted_amount,
            c.extracted_sender.clone(),
            c.ai_summary.clone(),
            c.subcategory.clone(),
        )
    } else {
        (None, None, None, None, None, None, None, None)
    };
    let (w, h) = entry.dimensions.map(|(w, h)| (Some(w as i64), Some(h as i64))).unwrap_or((None, None));
    let modified_at = entry.modified_at.to_rfc3339();
    let exif_date = entry.exif_date.map(|d| d.to_rfc3339());
    let created_at = entry.created_at.map(|d| d.to_rfc3339());

    sqlx::query!(
        r#"INSERT OR REPLACE INTO file_entries
        (id, scan_session_id, path, name, extension, size, mime_type, kind, hash,
         created_at, modified_at, exif_date, width, height,
         category, subcategory, confidence, tags, extracted_date, extracted_amount,
         extracted_sender, ai_summary, classified_by, duplicate_group_id)
        VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        entry.id, entry.scan_session_id, entry.path, entry.name, entry.extension,
        { let sz = entry.size as i64; sz }, entry.mime_type, kind, entry.hash,
        created_at, modified_at, exif_date, w, h,
        category, subcategory, confidence, tags, extracted_date, extracted_amount,
        extracted_sender, ai_summary, classified_by, entry.duplicate_group_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_files_by_session(pool: &SqlitePool, session_id: &str) -> Result<Vec<FileEntry>> {
    let rows = sqlx::query!(
        "SELECT * FROM file_entries WHERE scan_session_id=? ORDER BY modified_at DESC",
        session_id
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|r| {
        let kind = match r.kind.as_str() {
            "photo" => FileKind::Photo,
            "pdf" => FileKind::Pdf,
            "video" => FileKind::Video,
            "audio" => FileKind::Audio,
            "archive" => FileKind::Archive,
            "installer" => FileKind::Installer,
            "document" => FileKind::Document,
            "code" => FileKind::Code,
            _ => FileKind::Unknown,
        };
        let tags: Vec<String> = r.tags.as_deref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default();
        Ok(FileEntry {
            id: r.id.unwrap_or_default(),
            scan_session_id: r.scan_session_id,
            path: r.path,
            name: r.name,
            extension: r.extension,
            size: r.size as u64,
            mime_type: r.mime_type,
            kind,
            hash: r.hash,
            created_at: None,
            modified_at: Utc::now(),
            exif_date: None,
            dimensions: match (r.width, r.height) {
                (Some(w), Some(h)) => Some((w as u32, h as u32)),
                _ => None,
            },
            classification: None,
            tags,
            duplicate_group_id: r.duplicate_group_id,
        })
    }).collect()
}

// ── Actions ───────────────────────────────────────────────────

pub async fn insert_action(pool: &SqlitePool, action: &OrganizeAction) -> Result<()> {
    let kind = format!("{:?}", action.kind).to_lowercase();
    let status = format!("{:?}", action.status).to_lowercase();
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
    let s = match status {
        ActionStatus::Pending  => "pending",
        ActionStatus::Applied  => "applied",
        ActionStatus::Skipped  => "skipped",
        ActionStatus::Failed(_) => "failed",
    };
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
