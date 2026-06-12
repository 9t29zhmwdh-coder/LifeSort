pub mod rules;

use crate::models::{ActionKind, ActionStatus, FileEntry, OrganizeAction};
use anyhow::Result;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct OrganizerConfig {
    pub target_root: PathBuf,
    pub dry_run: bool,
    pub on_conflict: ConflictStrategy,
}

#[derive(Debug, Clone)]
pub enum ConflictStrategy {
    Skip,
    Rename,
    Overwrite,
}

/// Build a list of proposed move actions for classified files.
pub fn propose_actions(entries: &[FileEntry], config: &OrganizerConfig) -> Vec<OrganizeAction> {
    entries
        .iter()
        .filter(|e| e.classification.is_some())
        .filter_map(|entry| {
            let cls = entry.classification.as_ref()?;
            let folder = cls.category.folder_path(cls.extracted_date);
            let target_dir = config.target_root.join(&folder);
            let target_path = target_dir.join(&entry.name);

            // Skip if already in target
            let source = Path::new(&entry.path);
            if source.parent() == Some(target_dir.as_path()) {
                return None;
            }

            Some(OrganizeAction {
                id: Uuid::new_v4().to_string(),
                file_id: entry.id.clone(),
                file_name: entry.name.clone(),
                kind: ActionKind::Move,
                source_path: entry.path.clone(),
                target_path: Some(target_path.to_string_lossy().into_owned()),
                reason: format!(
                    "{} → {}",
                    cls.category.display_name(),
                    folder
                ),
                status: ActionStatus::Pending,
                undoable: true,
            })
        })
        .collect()
}

/// Execute a single action.
pub fn execute_action(action: &mut OrganizeAction) -> Result<()> {
    match action.kind {
        ActionKind::Move => {
            let src = Path::new(&action.source_path);
            let tgt = Path::new(action.target_path.as_deref().unwrap_or(""));
            if let Some(parent) = tgt.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(src, tgt)?;
            action.status = ActionStatus::Applied;
        }
        ActionKind::Delete => {
            std::fs::remove_file(&action.source_path)?;
            action.status = ActionStatus::Applied;
        }
        ActionKind::Copy => {
            let src = Path::new(&action.source_path);
            let tgt = Path::new(action.target_path.as_deref().unwrap_or(""));
            if let Some(parent) = tgt.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(src, tgt)?;
            action.status = ActionStatus::Applied;
        }
        _ => {
            action.status = ActionStatus::Skipped;
        }
    }
    Ok(())
}

pub fn undo_action(action: &mut OrganizeAction) -> Result<bool> {
    if !action.undoable || action.status != ActionStatus::Applied {
        return Ok(false);
    }
    if action.kind == ActionKind::Move {
        if let Some(ref tgt) = action.target_path {
            std::fs::rename(tgt, &action.source_path)?;
            action.status = ActionStatus::Pending;
            return Ok(true);
        }
    }
    Ok(false)
}
