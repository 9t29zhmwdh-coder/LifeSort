use crate::error::LsResult;
use crate::state::{AppSettings, AppState};
use ls_core::ai::AiBackend;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> LsResult<AppSettings> {
    Ok(state.settings.read().await.clone())
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, Arc<AppState>>,
) -> LsResult<()> {
    *state.settings.write().await = settings;
    Ok(())
}

#[tauri::command]
pub async fn check_ollama(state: State<'_, Arc<AppState>>) -> LsResult<bool> {
    let ok = state.ollama().is_available().await;
    Ok(ok)
}

#[tauri::command]
pub async fn list_plugins() -> LsResult<Vec<String>> {
    Ok(vec![]) // Plugins are registered at startup; this returns their names
}
