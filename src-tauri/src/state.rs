use ls_core::{
    ai::ollama::OllamaBackend,
    db::queries,
    models::{ActionStatus, FileEntry, OrganizeAction},
};
use sqlx::SqlitePool;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

pub type Files = Arc<RwLock<HashMap<String, Vec<FileEntry>>>>; // session_id → entries
pub type Actions = Arc<RwLock<Vec<OrganizeAction>>>;

const SETTINGS_KEY: &str = "settings";

pub struct AppState {
    pub pool: SqlitePool,
    pub files: Files,
    pub actions: Actions,
    pub settings: Arc<RwLock<AppSettings>>,
    pub photos: Arc<RwLock<Vec<ls_photos::PhotoAsset>>>,
    /// Groups the model assigned, by asset id.
    pub photo_ai: Arc<RwLock<HashMap<String, ls_photos::GroupKey>>>,
    /// Set by the cancel button; the classification loop checks it per photo.
    pub photo_cancel: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
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
            text_model: ls_core::ai::DEFAULT_MODEL.into(),
            vision_model: ls_core::ai::DEFAULT_MODEL.into(),
            target_root: home.to_string_lossy().into_owned(),
            auto_classify: false,
            auto_hash: false,
            skip_hidden: true,
        }
    }
}

impl AppState {
    /// Settings and the undo journal from the database, so neither is lost
    /// when the window closes.
    pub async fn load(pool: SqlitePool) -> anyhow::Result<Self> {
        let settings = queries::get_setting(&pool, SETTINGS_KEY)
            .await?
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        let applied: Vec<OrganizeAction> = queries::list_actions(&pool)
            .await?
            .into_iter()
            .filter(|a| a.status == ActionStatus::Applied)
            .collect();
        Ok(Self {
            pool,
            files: Arc::default(),
            actions: Arc::new(RwLock::new(applied)),
            settings: Arc::new(RwLock::new(settings)),
            photos: Arc::default(),
            photo_ai: Arc::default(),
            photo_cancel: Arc::default(),
        })
    }

    pub async fn save_settings(&self, settings: AppSettings) -> anyhow::Result<()> {
        queries::set_setting(&self.pool, SETTINGS_KEY, &serde_json::to_string(&settings)?).await?;
        *self.settings.write().await = settings;
        Ok(())
    }

    pub async fn ollama(&self) -> OllamaBackend {
        let s = self.settings.read().await;
        OllamaBackend::new(s.ollama_url.clone(), s.text_model.clone(), s.vision_model.clone())
    }
}
