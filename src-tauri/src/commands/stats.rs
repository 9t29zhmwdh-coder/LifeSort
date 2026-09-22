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
    /// Keyed by category id; the UI translates.
    pub by_category: HashMap<String, usize>,
    pub classified: usize,
    pub duplicate_count: usize,
    pub wasted_bytes: u64,
}

#[tauri::command]
pub async fn get_stats(session_id: String, state: State<'_, Arc<AppState>>) -> LsResult<ScanStats> {
    let map = state.files.read().await;
    let entries = map.get(&session_id).map(Vec::as_slice).unwrap_or_default();

    let mut by_kind: HashMap<String, usize> = HashMap::new();
    let mut by_category: HashMap<String, usize> = HashMap::new();
    let mut groups: HashMap<&str, (u64, u64)> = HashMap::new(); // id → (size, members)
    for e in entries {
        *by_kind.entry(kind_key(&e.kind).into()).or_default() += 1;
        if let Some(ref cls) = e.classification {
            *by_category.entry(cls.category.key()).or_default() += 1;
        }
        if let Some(ref g) = e.duplicate_group_id {
            let slot = groups.entry(g.as_str()).or_insert((e.size, 0));
            slot.1 += 1;
        }
    }

    Ok(ScanStats {
        total_files: entries.len(),
        total_size_bytes: entries.iter().map(|e| e.size).sum(),
        by_kind,
        by_category,
        classified: entries.iter().filter(|e| e.classification.is_some()).count(),
        duplicate_count: groups.values().map(|(_, n)| (*n as usize).saturating_sub(1)).sum(),
        wasted_bytes: groups.values().map(|(size, n)| size * n.saturating_sub(1)).sum(),
    })
}

fn kind_key(kind: &FileKind) -> &'static str {
    match kind {
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
    }
}
