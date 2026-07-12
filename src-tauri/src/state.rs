use ls_core::{
    ai::ollama::OllamaBackend,
    models::{FileEntry, OrganizeAction},
};
use sqlx::SqlitePool;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

pub type Files = Arc<RwLock<HashMap<String, Vec<FileEntry>>>>;   // session_id → entries
pub type Actions = Arc<RwLock<Vec<OrganizeAction>>>;

pub struct AppState {
    pub pool: SqlitePool,
    pub files: Files,
    pub actions: Actions,
    pub settings: Arc<RwLock<AppSettings>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub ollama_url: String,
    pub text_model: String,
    pub vision_model: String,
    pub target_root: String,
    pub auto_classify: bool,
    pub auto_hash: bool,
    pub skip_hidden: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        let home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("LifeSort");
        Self {
            ollama_url: "http://localhost:11434".into(),
            text_model: "llama3".into(),
            vision_model: "llava".into(),
            target_root: home.to_string_lossy().into_owned(),
            auto_classify: true,
            auto_hash: true,
            skip_hidden: true,
        }
    }
}

impl AppState {
    pub fn ollama(&self) -> OllamaBackend {
        // blocking read; only called from async context with settings already known
        let s = self.settings.try_read().ok();
        if let Some(s) = s {
            OllamaBackend::new(s.ollama_url.clone(), s.text_model.clone(), s.vision_model.clone())
        } else {
            let def = AppSettings::default();
            OllamaBackend::new(def.ollama_url, def.text_model, def.vision_model)
        }
    }
}
