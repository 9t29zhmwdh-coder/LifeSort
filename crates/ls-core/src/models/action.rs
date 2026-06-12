use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Move,
    Copy,
    Delete,
    Tag,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Pending,
    Applied,
    Skipped,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeAction {
    pub id: String,
    pub file_id: String,
    pub file_name: String,
    pub kind: ActionKind,
    pub source_path: String,
    pub target_path: Option<String>,
    pub reason: String,
    pub status: ActionStatus,
    pub undoable: bool,
}
