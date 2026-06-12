use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub id: String,
    pub hash: String,
    pub size: u64,
    pub file_ids: Vec<String>,
    pub keep_id: Option<String>,
    pub total_wasted_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateStrategy {
    KeepNewest,
    KeepOldest,
    KeepByPath(String),
    DeleteAll,
}
