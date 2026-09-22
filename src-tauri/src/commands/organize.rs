use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{
    db::queries,
    models::{ActionStatus, FolderLang, OrganizeAction},
    organizer::{self, OrganizerConfig},
};
use std::sync::Arc;
use tauri::State;

/// Fresh proposals for the session. Replaces earlier unexecuted proposals
/// instead of piling them up; executed moves stay for undo.
#[tauri::command]
pub async fn propose_actions(
    session_id: String,
    lang: FolderLang,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<OrganizeAction>> {
    let config = OrganizerConfig {
        target_root: std::path::PathBuf::from(&state.settings.read().await.target_root),
        folder_lang: lang,
    };
    let entries = state.files.read().await.get(&session_id).cloned().unwrap_or_default();
    let proposed = organizer::propose_actions(&entries, &config);

    let mut stored = state.actions.write().await;
    stored.retain(|a| a.status == ActionStatus::Applied);
    stored.extend(proposed.clone());
    Ok(proposed)
}

/// Runs one move, journals it, and points the in-memory entry at the new
/// place so a second proposal round starts from where the file really is.
async fn run_one(state: &AppState, action_id: &str) -> LsResult<OrganizeAction> {
    let mut stored = state.actions.write().await;
    let action = stored
        .iter_mut()
        .find(|a| a.id == action_id)
        .ok_or_else(|| anyhow::anyhow!("unknown action {action_id}"))?;
    if action.status != ActionStatus::Pending {
        return Ok(action.clone());
    }
    if let Err(e) = organizer::execute_action(action) {
        action.status = ActionStatus::Failed(e.to_string());
    }
    let result = action.clone();
    drop(stored);

    queries::insert_action(&state.pool, &result).await?;
    if let (ActionStatus::Applied, Some(target)) = (&result.status, &result.target_path) {
        for entries in state.files.write().await.values_mut() {
            if let Some(e) = entries.iter_mut().find(|e| e.id == result.file_id) {
                e.path = target.clone();
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn execute_action(action_id: String, state: State<'_, Arc<AppState>>) -> LsResult<OrganizeAction> {
    run_one(&state, &action_id).await
}

#[tauri::command]
pub async fn undo_action(action_id: String, state: State<'_, Arc<AppState>>) -> LsResult<OrganizeAction> {
    let mut stored = state.actions.write().await;
    let action = stored
        .iter_mut()
        .find(|a| a.id == action_id)
        .ok_or_else(|| anyhow::anyhow!("unknown action {action_id}"))?;
    organizer::undo_action(action)?;
    let result = action.clone();
    drop(stored);
    queries::update_action_status(&state.pool, &result.id, &result.status).await?;
    for entries in state.files.write().await.values_mut() {
        if let Some(e) = entries.iter_mut().find(|e| e.id == result.file_id) {
            e.path = result.source_path.clone();
        }
    }
    Ok(result)
}

/// Everything proposed or done, newest journal entries included, so undo
/// works for moves from an earlier session too.
#[tauri::command]
pub async fn list_actions(state: State<'_, Arc<AppState>>) -> LsResult<Vec<OrganizeAction>> {
    Ok(state.actions.read().await.clone())
}
