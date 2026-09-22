//! The Apple Photos mode: read the library, group what takes space, let the
//! model spot memes and photographed documents, collect groups in albums.
//! Nothing here deletes or moves a photo.

use crate::error::LsResult;
use crate::state::AppState;
use base64::Engine;
use ls_core::ai::{ollama::AiStatus, AiBackend};
use ls_core::models::Category;
use ls_photos::{Access, AssetKind, Group, GroupKey};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Preview size for the model, the same as for files in the folder mode.
const PREVIEW_EDGE: u32 = 1024;

#[tauri::command]
pub fn platform() -> &'static str {
    std::env::consts::OS
}

#[tauri::command]
pub async fn photos_access(request: bool) -> LsResult<Access> {
    let status = tokio::task::spawn_blocking(move || {
        let now = ls_photos::access();
        if request && now == Access::NotDetermined {
            ls_photos::request_access()
        } else {
            now
        }
    })
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(status)
}

#[derive(Serialize, Clone)]
pub struct PhotosDone {
    pub count: usize,
    pub bytes: u64,
    pub error: Option<String>,
}

/// Reads the whole library in the background; `photos://progress` reports
/// (done, total) and `photos://done` the result.
#[tauri::command]
pub async fn photos_scan(app: AppHandle, state: State<'_, Arc<AppState>>) -> LsResult<()> {
    let (photos, ai) = (state.photos.clone(), state.photo_ai.clone());
    tokio::spawn(async move {
        let progress = app.clone();
        let result = tokio::task::spawn_blocking(move || {
            ls_photos::list_assets(|done, total| {
                let _ = progress.emit("photos://progress", (done, total));
            })
        })
        .await;
        let done = match result {
            Ok(Ok(assets)) => {
                let summary = PhotosDone { count: assets.len(), bytes: assets.iter().map(|a| a.bytes).sum(), error: None };
                *photos.write().await = assets;
                ai.write().await.clear();
                summary
            }
            Ok(Err(e)) => PhotosDone { count: 0, bytes: 0, error: Some(e.to_string()) },
            Err(e) => PhotosDone { count: 0, bytes: 0, error: Some(e.to_string()) },
        };
        let _ = app.emit("photos://done", done);
    });
    Ok(())
}

#[derive(Serialize)]
pub struct GroupView {
    #[serde(flatten)]
    pub group: Group,
    /// The ten largest members, for a quick look before creating the album.
    pub top: Vec<ls_photos::PhotoAsset>,
}

#[tauri::command]
pub async fn photos_groups(state: State<'_, Arc<AppState>>) -> LsResult<Vec<GroupView>> {
    let assets = state.photos.read().await;
    let ai = state.photo_ai.read().await;
    let groups = ls_photos::group_assets(&assets, &ai);
    Ok(groups
        .into_iter()
        .map(|group| {
            let top = group
                .ids
                .iter()
                .take(10)
                .filter_map(|id| assets.iter().find(|a| &a.id == id).cloned())
                .collect();
            GroupView { group, top }
        })
        .collect())
}

/// Which model answer puts a photo into which group. Everything else is an
/// ordinary photo and stays out of the cleanup groups.
fn group_for(category: Category) -> Option<GroupKey> {
    match category {
        Category::PhotoMeme => Some(GroupKey::Memes),
        Category::PhotoDocument => Some(GroupKey::PhotographedDocuments),
        Category::PhotoScreenshot => Some(GroupKey::Screenshots),
        _ => None,
    }
}

#[derive(Serialize)]
pub struct PhotosClassifyStart {
    pub total: usize,
    pub ai: AiStatus,
}

/// Lets the vision model look at every photo that is not already known as a
/// screenshot, burst frame or favourite. Runs in the background, one photo
/// at a time, and stops at the next photo after `photos_cancel`.
#[tauri::command]
pub async fn photos_classify(app: AppHandle, state: State<'_, Arc<AppState>>) -> LsResult<PhotosClassifyStart> {
    let ollama = state.ollama().await;
    let ai = ollama.status().await;
    if ai != AiStatus::Ready {
        return Ok(PhotosClassifyStart { total: 0, ai });
    }
    let done_ids = state.photo_ai.read().await.keys().cloned().collect::<std::collections::HashSet<_>>();
    let todo: Vec<String> = state
        .photos
        .read()
        .await
        .iter()
        .filter(|a| a.kind == AssetKind::Image && !a.screenshot && !a.burst_extra && !a.favorite)
        .filter(|a| !done_ids.contains(&a.id))
        .map(|a| a.id.clone())
        .collect();
    let total = todo.len();
    let (results, cancel) = (state.photo_ai.clone(), state.photo_cancel.clone());
    cancel.store(false, Ordering::Relaxed);

    tokio::spawn(async move {
        for (i, id) in todo.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let fetch_id = id.clone();
            let jpeg = tokio::task::spawn_blocking(move || ls_photos::thumbnail_jpeg(&fetch_id, PREVIEW_EDGE))
                .await
                .ok()
                .flatten();
            if let Some(jpeg) = jpeg {
                let b64 = base64::engine::general_purpose::STANDARD.encode(jpeg);
                if let Ok(c) = ollama.classify_image(&b64).await {
                    if let Some(key) = group_for(c.category) {
                        results.write().await.insert(id, key);
                    }
                }
            }
            let _ = app.emit("photos://classify-progress", (i + 1, total));
        }
        let _ = app.emit("photos://classify-done", total);
    });
    Ok(PhotosClassifyStart { total, ai })
}

#[tauri::command]
pub fn photos_cancel(state: State<'_, Arc<AppState>>) {
    state.photo_cancel.store(true, Ordering::Relaxed);
}

/// Collects a group in an album with the given title, adding to an existing
/// album of that name instead of creating a second one.
#[tauri::command]
pub async fn photos_add_album(key: GroupKey, title: String, state: State<'_, Arc<AppState>>) -> LsResult<usize> {
    let ids = {
        let assets = state.photos.read().await;
        let ai = state.photo_ai.read().await;
        ls_photos::group_assets(&assets, &ai)
            .into_iter()
            .find(|g| g.key == key)
            .map(|g| g.ids)
            .unwrap_or_default()
    };
    let added = tokio::task::spawn_blocking(move || ls_photos::add_to_album(&title, &ids))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))??;
    Ok(added)
}

#[tauri::command]
pub fn photos_open_app() -> LsResult<()> {
    std::process::Command::new("/usr/bin/open").args(["-a", "Photos"]).spawn()?;
    Ok(())
}
