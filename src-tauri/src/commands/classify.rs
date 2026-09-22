use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{
    ai::{ollama::AiStatus, AiBackend},
    classifier,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[derive(serde::Serialize)]
pub struct ClassifyStart {
    pub total: usize,
    pub ai: AiStatus,
}

/// Classifies every file of the session in the background. Each result is
/// written back as soon as it exists, so the file view fills while the run
/// is going and a closed window loses nothing already done.
#[tauri::command]
pub async fn classify_batch(
    session_id: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> LsResult<ClassifyStart> {
    let ollama = state.ollama().await;
    let ai = ollama.status().await;
    let pending: Vec<_> = state
        .files
        .read()
        .await
        .get(&session_id)
        .map(|v| v.iter().filter(|e| e.classification.is_none()).cloned().collect())
        .unwrap_or_default();
    let total = pending.len();
    let use_ai = ai == AiStatus::Ready;
    let files = state.files.clone();

    tokio::spawn(async move {
        for (done, entry) in pending.iter().enumerate() {
            let backend: Option<&dyn AiBackend> = if use_ai { Some(&ollama) } else { None };
            let cls = classifier::classify_entry(entry, backend).await;
            if let Some(stored) = files.write().await.get_mut(&session_id).and_then(|v| v.iter_mut().find(|e| e.id == entry.id)) {
                stored.tags = cls.tags.clone();
                stored.classification = Some(cls);
            }
            let _ = app.emit("classify://progress", (done + 1, total));
        }
        let _ = app.emit("classify://done", total);
    });

    Ok(ClassifyStart { total, ai })
}
