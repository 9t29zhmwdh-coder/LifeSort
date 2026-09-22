use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{
    models::FileEntry,
    scanner::{self, ScanOptions},
};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct ScanSession {
    pub id: String,
    pub path: String,
    pub file_count: usize,
}

#[derive(Serialize, Clone)]
pub struct ScanDone {
    pub id: String,
    pub count: usize,
    pub error: Option<String>,
}

/// Starts the scan and returns at once; `scan://done` follows when the walk
/// is finished. The UI waits for that event instead of guessing a delay.
#[tauri::command]
pub async fn scan_directory(
    path: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> LsResult<ScanSession> {
    if scanner::is_package(std::path::Path::new(&path)) {
        return Err(anyhow::anyhow!(
            "LifeSort does not reorganise the inside of an app package or a Photos library; moving files there would break it"
        ).into());
    }
    let session_id = Uuid::new_v4().to_string();
    let opts = ScanOptions {
        skip_hidden: state.settings.read().await.skip_hidden,
        ..Default::default()
    };
    let (root, sid, files) = (path.clone(), session_id.clone(), state.files.clone());

    tokio::spawn(async move {
        let progress = app.clone();
        let walk = tokio::task::spawn_blocking(move || {
            let mut batch: Vec<FileEntry> = vec![];
            let result = scanner::scan_directory(std::path::Path::new(&root), &sid, &opts, |entry| {
                batch.push(entry);
                if batch.len().is_multiple_of(50) {
                    let _ = progress.emit("scan://progress", batch.len());
                }
            });
            (sid, batch, result.err().map(|e| e.to_string()))
        })
        .await;
        let (sid, batch, error) = match walk {
            Ok(done) => done,
            Err(e) => return eprintln!("scan task failed: {e}"),
        };
        let count = batch.len();
        files.write().await.insert(sid.clone(), batch);
        let _ = app.emit("scan://done", ScanDone { id: sid, count, error });
    });

    Ok(ScanSession { id: session_id, path, file_count: 0 })
}

#[tauri::command]
pub async fn get_scan_results(session_id: String, state: State<'_, Arc<AppState>>) -> LsResult<Vec<FileEntry>> {
    Ok(state.files.read().await.get(&session_id).cloned().unwrap_or_default())
}
