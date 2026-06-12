use crate::error::LsResult;
use crate::state::AppState;
use ls_core::models::FileKind;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tauri::State;

#[derive(Serialize)]
pub struct ScanStats {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub by_kind: HashMap<String, usize>,
    pub by_category: HashMap<String, usize>,
    pub classified: usize,
    pub duplicate_count: usize,
    pub wasted_bytes: u64,
}

#[tauri::command]
pub async fn get_stats(
    session_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<ScanStats> {
    let map = state.files.read().await;
    let entries = map.get(&session_id).cloned().unwrap_or_default();
    drop(map);

    let total_files = entries.len();
    let total_size_bytes: u64 = entries.iter().map(|e| e.size).sum();
    let classified = entries.iter().filter(|e| e.classification.is_some()).count();
    let duplicate_count = entries.iter().filter(|e| e.duplicate_group_id.is_some()).count();

    let mut by_kind: HashMap<String, usize> = HashMap::new();
    let mut by_category: HashMap<String, usize> = HashMap::new();

    for e in &entries {
        let k = match e.kind {
            FileKind::Photo => "photo",
            FileKind::Pdf => "pdf",
            FileKind::Document => "document",
            FileKind::Video => "video",
            FileKind::Audio => "audio",
            FileKind::Archive => "archive",
            FileKind::Installer => "installer",
            FileKind::Code => "code",
            FileKind::Font => "font",
            FileKind::Unknown => "unknown",
        };
        *by_kind.entry(k.into()).or_default() += 1;
        if let Some(ref cls) = e.classification {
            *by_category.entry(cls.category.display_name().into()).or_default() += 1;
        }
    }

    Ok(ScanStats {
        total_files,
        total_size_bytes,
        by_kind,
        by_category,
        classified,
        duplicate_count,
        wasted_bytes: 0,
    })
}
