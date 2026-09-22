use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{dedup, models::DuplicateGroup};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn find_duplicates(session_id: String, state: State<'_, Arc<AppState>>) -> LsResult<Vec<DuplicateGroup>> {
    let mut entries = state.files.read().await.get(&session_id).cloned().unwrap_or_default();
    let (entries, groups) = tokio::task::spawn_blocking(move || {
        dedup::compute_hashes(&mut entries);
        let groups = dedup::find_duplicate_groups(&mut entries);
        (entries, groups)
    })
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Merge hashes and group ids back instead of replacing the list, so a
    // classification that finished meanwhile is kept.
    if let Some(stored) = state.files.write().await.get_mut(&session_id) {
        for e in stored.iter_mut() {
            if let Some(u) = entries.iter().find(|u| u.id == e.id) {
                e.hash = u.hash.clone();
                e.duplicate_group_id = u.duplicate_group_id.clone();
            }
        }
    }
    Ok(groups)
}

/// Moves every copy except `keep_id` to the system trash, where the user can
/// still restore it. Returns the paths that were moved.
#[tauri::command]
pub async fn resolve_duplicate(
    session_id: String,
    group: DuplicateGroup,
    keep_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Vec<String>> {
    if !group.file_ids.contains(&keep_id) {
        return Err(anyhow::anyhow!("the file to keep is not part of this group").into());
    }
    let mut files = state.files.write().await;
    let Some(entries) = files.get_mut(&session_id) else { return Ok(vec![]) };

    let candidates: Vec<String> = group
        .file_ids
        .iter()
        .filter(|id| **id != keep_id)
        .filter_map(|id| entries.iter().find(|e| &e.id == id).map(|e| e.path.clone()))
        .collect();
    let hash = group.hash.clone();
    let trashed = tokio::task::spawn_blocking(move || {
        candidates
            .into_iter()
            // Only trash a file whose content is still what was hashed.
            .filter(|p| ls_core::dedup::hasher::hash_file(p).as_deref() == Some(hash.as_str()))
            .filter(|p| trash_context().delete(p).is_ok())
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    entries.retain(|e| !trashed.contains(&e.path));
    if let Some(kept) = entries.iter_mut().find(|e| e.id == keep_id) {
        kept.duplicate_group_id = None;
    }
    Ok(trashed)
}

/// On macOS the crate defaults to asking Finder via AppleScript, which needs
/// an Automation permission and blocks until the user answers a prompt that
/// may sit behind the window. NSFileManager needs no permission.
fn trash_context() -> trash::TrashContext {
    #[allow(unused_mut)]
    let mut ctx = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        ctx.set_delete_method(DeleteMethod::NsFileManager);
    }
    ctx
}
