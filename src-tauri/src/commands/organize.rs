use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{
    db::queries,
    models::{ActionStatus, OrganizeAction},
    organizer::{self, ConflictStrategy, OrganizerConfig},
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn propose_actions(
    session_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<OrganizeAction>> {
    let settings = state.settings.read().await.clone();
    let config = OrganizerConfig {
        target_root: std::path::PathBuf::from(&settings.target_root),
        dry_run: true,
        on_conflict: ConflictStrategy::Rename,
    };
    let map = state.files.read().await;
    let entries = map.get(&session_id).cloned().unwrap_or_default();
    drop(map);

    let actions = organizer::propose_actions(&entries, &config);
    // Cache
    let mut stored = state.actions.write().await;
    stored.extend(actions.clone());
    Ok(actions)
}

#[tauri::command]
pub async fn execute_action(
    action_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<ActionStatus> {
    let mut stored = state.actions.write().await;
    let action = stored
        .iter_mut()
        .find(|a| a.id == action_id)
        .ok_or_else(|| anyhow::anyhow!("Aktion nicht gefunden"))?;
    if let Err(e) = organizer::execute_action(action) {
        action.status = ActionStatus::Failed(e.to_string());
    }
    let status = action.status.clone();
    queries::insert_action(&state.pool, action).await?;
    Ok(status)
}

#[tauri::command]
pub async fn execute_all(
    session_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<(String, ActionStatus)>> {
    let mut stored = state.actions.write().await;
    let mut results = vec![];
    for action in stored.iter_mut().filter(|a| a.status == ActionStatus::Pending) {
        if let Err(e) = organizer::execute_action(action) {
            action.status = ActionStatus::Failed(e.to_string());
        }
        results.push((action.id.clone(), action.status.clone()));
        let _ = queries::insert_action(&state.pool, action).await;
    }
    Ok(results)
}

#[tauri::command]
pub async fn undo_action(
    action_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<bool> {
    let mut stored = state.actions.write().await;
    let action = stored
        .iter_mut()
        .find(|a| a.id == action_id)
        .ok_or_else(|| anyhow::anyhow!("Aktion nicht gefunden"))?;
    let ok = organizer::undo_action(action)?;
    if ok {
        queries::update_action_status(&state.pool, &action.id, &action.status).await?;
    }
    Ok(ok)
}

#[tauri::command]
pub async fn list_actions(state: State<'_, Arc<AppState>>) -> LsResult<Vec<OrganizeAction>> {
    let stored = state.actions.read().await;
    Ok(stored.clone())
}
