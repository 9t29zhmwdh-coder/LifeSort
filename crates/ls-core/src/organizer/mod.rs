use crate::models::{ActionKind, ActionStatus, FileEntry, FolderLang, OrganizeAction};
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct OrganizerConfig {
    pub target_root: PathBuf,
    pub folder_lang: FolderLang,
}

/// Build a list of proposed move actions for classified files.
///
/// Two files with the same name heading for the same folder get distinct
/// targets here already, so the preview shows what will really happen.
pub fn propose_actions(entries: &[FileEntry], config: &OrganizerConfig) -> Vec<OrganizeAction> {
    let mut reserved: HashSet<PathBuf> = HashSet::new();
    entries
        .iter()
        .filter_map(|entry| {
            let cls = entry.classification.as_ref()?;
            let folder = cls.category.folder_path(cls.extracted_date, config.folder_lang);
            let target_dir = config.target_root.join(&folder);

            let source = Path::new(&entry.path);
            if source.parent() == Some(target_dir.as_path()) {
                return None;
            }

            let target_path = free_target(&target_dir.join(&entry.name), &reserved);
            reserved.insert(target_path.clone());

            Some(OrganizeAction {
                id: Uuid::new_v4().to_string(),
                file_id: entry.id.clone(),
                file_name: entry.name.clone(),
                kind: ActionKind::Move,
                source_path: entry.path.clone(),
                target_path: Some(target_path.to_string_lossy().into_owned()),
                reason: folder,
                status: ActionStatus::Pending,
                undoable: true,
            })
        })
        .collect()
}

/// First path of `name`, `name (2)`, `name (3)` … that neither exists on disk
/// nor was handed out already.
fn free_target(wanted: &Path, reserved: &HashSet<PathBuf>) -> PathBuf {
    let taken = |p: &Path| p.exists() || reserved.contains(p);
    if !taken(wanted) {
        return wanted.to_path_buf();
    }
    let stem = wanted.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = wanted.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let dir = wanted.parent().unwrap_or_else(|| Path::new(""));
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|p| !taken(p))
        .expect("an unbounded counter always finds a free name")
}

/// Execute a single move. Never overwrites: if the planned target has been
/// taken since the proposal, the file gets the next free name and the action
/// records where it really went, so undo finds it.
pub fn execute_action(action: &mut OrganizeAction) -> Result<()> {
    if action.kind != ActionKind::Move {
        action.status = ActionStatus::Skipped;
        return Ok(());
    }
    let src = PathBuf::from(&action.source_path);
    if !src.is_file() {
        bail!("source no longer exists: {}", src.display());
    }
    let planned = PathBuf::from(action.target_path.as_deref().context("move without target")?);
    let target = free_target(&planned, &HashSet::new());
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    move_file(&src, &target)?;
    action.target_path = Some(target.to_string_lossy().into_owned());
    action.status = ActionStatus::Applied;
    Ok(())
}

/// Moves the file back. Refuses when something now occupies the original
/// place, because putting the file back would destroy that other file.
pub fn undo_action(action: &mut OrganizeAction) -> Result<bool> {
    if !action.undoable || action.status != ActionStatus::Applied || action.kind != ActionKind::Move {
        return Ok(false);
    }
    let Some(ref tgt) = action.target_path else { return Ok(false) };
    let (from, to) = (Path::new(tgt), Path::new(&action.source_path));
    if to.exists() {
        bail!("original location is occupied again: {}", to.display());
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    move_file(from, to)?;
    action.status = ActionStatus::Pending;
    Ok(true)
}

/// `rename`, with copy and delete as fallback when source and target sit on
/// different volumes (an external disk, a network share).
fn move_file(src: &Path, dst: &Path) -> Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::CrossesDevices => {
            let copied = std::fs::copy(src, dst)?;
            let original = std::fs::metadata(src)?.len();
            if copied != original {
                let _ = std::fs::remove_file(dst);
                bail!("copy incomplete ({copied} of {original} bytes), original kept");
            }
            std::fs::remove_file(src)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, Classification, ClassifierKind, FileKind};
    use chrono::Utc;

    fn tempdir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ls-org-{tag}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn entry(path: &Path, category: Category) -> FileEntry {
        FileEntry {
            id: Uuid::new_v4().to_string(),
            path: path.to_string_lossy().into_owned(),
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            extension: None,
            size: 0,
            mime_type: String::new(),
            kind: FileKind::Photo,
            hash: None,
            created_at: None,
            modified_at: Utc::now(),
            exif_date: None,
            dimensions: None,
            camera: None,
            screenshot_marker: false,
            classification: Some(Classification::simple(category, 1.0, &[], ClassifierKind::Rules)),
            tags: vec![],
            scan_session_id: String::new(),
            duplicate_group_id: None,
        }
    }

    fn config(root: &Path) -> OrganizerConfig {
        OrganizerConfig { target_root: root.to_path_buf(), folder_lang: FolderLang::En }
    }

    /// Two IMG_0001.JPG from different folders used to land on the same
    /// target, and the second `rename` silently replaced the first photo.
    #[test]
    fn same_name_never_overwrites() {
        let root = tempdir("same");
        for sub in ["a", "b"] {
            std::fs::create_dir_all(root.join(sub)).unwrap();
            std::fs::write(root.join(sub).join("IMG_0001.JPG"), sub).unwrap();
        }
        let entries = vec![
            entry(&root.join("a/IMG_0001.JPG"), Category::PhotoScreenshot),
            entry(&root.join("b/IMG_0001.JPG"), Category::PhotoScreenshot),
        ];
        let mut actions = propose_actions(&entries, &config(&root.join("out")));
        for a in &mut actions {
            execute_action(a).unwrap();
        }
        let dir = root.join("out/Photos/Screenshots");
        assert_eq!(std::fs::read_to_string(dir.join("IMG_0001.JPG")).unwrap(), "a");
        assert_eq!(std::fs::read_to_string(dir.join("IMG_0001 (2).JPG")).unwrap(), "b");
    }

    /// A file that appears at the target after the proposal must survive too.
    #[test]
    fn target_taken_after_proposal_gets_next_name() {
        let root = tempdir("late");
        std::fs::write(root.join("x.pdf"), "new").unwrap();
        let mut actions = propose_actions(&[entry(&root.join("x.pdf"), Category::Contract)], &config(&root));
        let dir = root.join("Documents/Contracts");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.pdf"), "already there").unwrap();

        execute_action(&mut actions[0]).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("x.pdf")).unwrap(), "already there");
        assert_eq!(std::fs::read_to_string(dir.join("x (2).pdf")).unwrap(), "new");
        assert!(actions[0].target_path.as_deref().unwrap().ends_with("x (2).pdf"));
    }

    #[test]
    fn undo_restores_and_refuses_to_overwrite() {
        let root = tempdir("undo");
        let src = root.join("doc.pdf");
        std::fs::write(&src, "content").unwrap();
        let mut action = propose_actions(&[entry(&src, Category::Letter)], &config(&root)).remove(0);
        execute_action(&mut action).unwrap();
        assert!(!src.exists());

        assert!(undo_action(&mut action).unwrap());
        assert_eq!(std::fs::read_to_string(&src).unwrap(), "content");

        execute_action(&mut action).unwrap();
        std::fs::write(&src, "someone else").unwrap();
        assert!(undo_action(&mut action).is_err());
        assert_eq!(std::fs::read_to_string(&src).unwrap(), "someone else");
    }

    #[test]
    fn missing_source_fails_instead_of_pretending() {
        let root = tempdir("gone");
        let mut action = propose_actions(&[entry(&root.join("gone.pdf"), Category::Letter)], &config(&root)).remove(0);
        assert!(execute_action(&mut action).is_err());
        assert_eq!(action.status, ActionStatus::Pending);
    }
}
