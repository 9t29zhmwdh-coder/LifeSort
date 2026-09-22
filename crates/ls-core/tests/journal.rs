//! The move journal is what makes undo possible after a restart.

use ls_core::db::{self, queries};
use ls_core::models::{ActionKind, ActionStatus, OrganizeAction};

fn action(id: &str, status: ActionStatus) -> OrganizeAction {
    OrganizeAction {
        id: id.into(),
        file_id: "f".into(),
        file_name: "a.pdf".into(),
        kind: ActionKind::Move,
        source_path: "/from/a.pdf".into(),
        target_path: Some("/to/a.pdf".into()),
        reason: "Documents/Letters".into(),
        status,
        undoable: true,
    }
}

#[tokio::test]
async fn journal_and_settings_survive_reopening() {
    let dir = std::env::temp_dir().join(format!("lifesort-journal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("j.db");

    let pool = db::open(&path).await.unwrap();
    queries::insert_action(&pool, &action("1", ActionStatus::Applied)).await.unwrap();
    queries::insert_action(&pool, &action("2", ActionStatus::Failed("disk full".into()))).await.unwrap();
    queries::set_setting(&pool, "settings", "{\"x\":1}").await.unwrap();
    drop(pool);

    let pool = db::open(&path).await.unwrap();
    let mut actions = queries::list_actions(&pool).await.unwrap();
    actions.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(actions[0].status, ActionStatus::Applied);
    assert_eq!(actions[0].target_path.as_deref(), Some("/to/a.pdf"));
    // Was read back as pending before, which offered a failed move for re-execution.
    assert!(matches!(actions[1].status, ActionStatus::Failed(_)));
    assert_eq!(queries::get_setting(&pool, "settings").await.unwrap().as_deref(), Some("{\"x\":1}"));
    let _ = std::fs::remove_dir_all(&dir);
}
