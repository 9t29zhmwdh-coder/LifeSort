use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{
    db::queries,
    models::FileEntry,
    scanner::{self, ScanOptions},
};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

#[derive(Serialize)]
pub struct ScanSession {
    pub id: String,
    pub path: String,
    pub file_count: usize,
}

#[tauri::command]
pub async fn scan_directory(
    path: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> LsResult<ScanSession> {
    let session_id = Uuid::new_v4().to_string();
    queries::insert_session(&state.pool, &session_id, &path).await?;

    let settings = state.settings.read().await.clone();
    let opts = ScanOptions {
        skip_hidden: settings.skip_hidden,
        ..Default::default()
    };
    let path_clone = path.clone();
    let session_clone = session_id.clone();
    let pool_clone = state.pool.clone();
    let files_clone = state.files.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let mut batch: Vec<FileEntry> = vec![];
        let _ = scanner::scan_directory(
            std::path::Path::new(&path_clone),
            &session_clone,
            &opts,
            |entry| {
                batch.push(entry);
                if batch.len().is_multiple_of(50) {
                    let _ = app_clone.emit("scan://progress", batch.len());
                }
            },
        );
        let count = batch.len();
        // Persist to DB
        for entry in &batch {
            let _ = queries::insert_file(&pool_clone, entry).await;
        }
        let _ = queries::update_session_count(&pool_clone, &session_clone, count as i64).await;
        let mut map = files_clone.write().await;
        map.insert(session_clone.clone(), batch);
        let _ = app_clone.emit("scan://done", (session_clone, count));
    });

    Ok(ScanSession { id: session_id, path, file_count: 0 })
}

#[tauri::command]
pub async fn get_scan_results(
    session_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<FileEntry>> {
    let map = state.files.read().await;
    if let Some(entries) = map.get(&session_id) {
        return Ok(entries.clone());
    }
    // Fallback to DB
    let entries = queries::list_files_by_session(&state.pool, &session_id).await?;
    Ok(entries)
}
