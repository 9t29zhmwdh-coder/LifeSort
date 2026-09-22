use crate::error::LsResult;
use crate::state::{AppSettings, AppState};
use ls_core::ai::ollama::AiStatus;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> LsResult<AppSettings> {
    Ok(state.settings.read().await.clone())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, state: State<'_, Arc<AppState>>) -> LsResult<()> {
    state.save_settings(settings).await?;
    Ok(())
}

#[tauri::command]
pub async fn check_ollama(state: State<'_, Arc<AppState>>) -> LsResult<AiStatus> {
    Ok(state.ollama().await.status().await)
}
