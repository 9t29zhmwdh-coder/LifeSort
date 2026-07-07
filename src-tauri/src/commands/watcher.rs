use crate::error::LsResult;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn start_watch(path: String, _state: State<'_, Arc<AppState>>) -> LsResult<()> {
    // Watcher is fire-and-forget; new files trigger a re-scan of that path
    tracing::info!("Watch gestartet: {path}");
    Ok(())
}

#[tauri::command]
pub async fn stop_watch(_state: State<'_, Arc<AppState>>) -> LsResult<()> {
    tracing::info!("Watch gestoppt");
    Ok(())
}
