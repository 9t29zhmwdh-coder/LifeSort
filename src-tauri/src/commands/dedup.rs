use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{dedup, models::DuplicateGroup};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn find_duplicates(
    session_id: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<DuplicateGroup>> {
    let files_clone = state.files.clone();
    let sid = session_id.clone();
    let app_clone = app.clone();

    // Hash computation is CPU-bound — run in blocking thread
    let groups = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        let mut entries = rt.block_on(async {
            let map = files_clone.read().await;
            map.get(&sid).cloned().unwrap_or_default()
        });
        let _ = app_clone.emit("dedup://hashing", entries.len());
        dedup::compute_hashes(&mut entries);
        // Write hashes back
        rt.block_on(async {
            let mut map = files_clone.write().await;
            if let Some(stored) = map.get_mut(&sid) {
                for e in stored.iter_mut() {
                    if let Some(updated) = entries.iter().find(|u| u.id == e.id) {
                        e.hash = updated.hash.clone();
                    }
                }
            }
        });
        dedup::find_duplicate_groups(&entries)
    }).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    app.emit("dedup://done", groups.len())?;
    Ok(groups)
}

#[tauri::command]
pub async fn resolve_duplicate(
    group: DuplicateGroup,
    keep_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<String>> {
    let map = state.files.read().await;
    let all_entries: Vec<_> = map.values().flatten().collect();
    let to_delete: Vec<String> = group
        .file_ids
        .iter()
        .filter(|id| **id != keep_id)
        .filter_map(|id| all_entries.iter().find(|e| &e.id == id))
        .map(|e| e.path.clone())
        .collect();
    drop(map);

    let mut deleted = vec![];
    for path in &to_delete {
        if let Ok(()) = std::fs::remove_file(path) {
            deleted.push(path.clone());
        }
    }
    Ok(deleted)
}
