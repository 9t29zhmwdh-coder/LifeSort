use crate::error::LsResult;
use crate::state::AppState;
use ls_core::{ai::AiBackend, classifier, db::queries, models::Classification};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn classify_file(
    file_id: String,
    session_id: String,
    state: State<'_, Arc<AppState>>,
) -> LsResult<Option<Classification>> {
    let mut map = state.files.write().await;
    let entries = match map.get_mut(&session_id) {
        Some(e) => e,
        None => return Ok(None),
    };
    let entry = match entries.iter_mut().find(|e| e.id == file_id) {
        Some(e) => e,
        None => return Ok(None),
    };

    let ollama = state.ollama();
    let ai_available = ollama.is_available().await;
    let ai: Option<&dyn AiBackend> = if ai_available { Some(&ollama) } else { None };

    let cls = classifier::classify_entry(entry, ai).await;
    entry.classification = Some(cls.clone());
    entry.tags = cls.tags.clone();

    // Persist
    let _ = queries::insert_file(&state.pool, entry).await;

    Ok(Some(cls))
}

#[tauri::command]
pub async fn classify_batch(
    session_id: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> LsResult<usize> {
    let ollama = state.ollama();
    let ai_available = ollama.is_available().await;

    let entries_snapshot: Vec<_> = {
        let map = state.files.read().await;
        map.get(&session_id).cloned().unwrap_or_default()
    };
    let total = entries_snapshot.len();
    let pool_clone = state.pool.clone();
    let files_clone = state.files.clone();
    let sid = session_id.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let mut done = 0usize;
        // Collect results without holding the write lock during AI calls
        let mut results: Vec<(String, ls_core::models::Classification)> = vec![];
        for entry in &entries_snapshot {
            if entry.classification.is_some() {
                done += 1;
                continue;
            }
            // AI requires owning the entry briefly
            let cls = if ai_available {
                let ollama = OllamaLocal::new_from_state_async().await;
                classifier::classify_entry(entry, Some(&ollama)).await
            } else {
                classifier::classify_entry(entry, None).await
            };
            results.push((entry.id.clone(), cls));
            done += 1;
            let _ = app_clone.emit("classify://progress", (done, total));
        }

        // Write back
        let mut map = files_clone.write().await;
        if let Some(entries) = map.get_mut(&sid) {
            for entry in entries.iter_mut() {
                if let Some(pos) = results.iter().position(|(id, _)| id == &entry.id) {
                    let (_, cls) = results.remove(pos);
                    entry.tags = cls.tags.clone();
                    entry.classification = Some(cls);
                    let _ = queries::insert_file(&pool_clone, entry).await;
                }
            }
        }
        let _ = app_clone.emit("classify://done", done);
    });

    Ok(total)
}

// Local helper; avoids passing Arc<AppState> into spawned task
struct OllamaLocal(ls_core::ai::ollama::OllamaBackend);
impl OllamaLocal {
    async fn new_from_state_async() -> ls_core::ai::ollama::OllamaBackend {
        ls_core::ai::ollama::OllamaBackend::new(
            "http://localhost:11434".into(),
            "llama3".into(),
            "llava".into(),
        )
    }
}
impl std::ops::Deref for OllamaLocal {
    type Target = ls_core::ai::ollama::OllamaBackend;
    fn deref(&self) -> &Self::Target { &self.0 }
}
