mod commands;
mod error;
mod state;

use commands::*;
use ls_core::db;
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri::async_runtime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("no app data dir");
            let db_path = data_dir.join("lifesort.db");

            let app_state = async_runtime::block_on(async {
                let pool = db::open(&db_path).await?;
                AppState::load(pool).await
            })
            .expect("DB init failed");
            app.manage(Arc::new(app_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scanner::scan_directory,
            scanner::get_scan_results,
            classify::classify_batch,
            dedup::find_duplicates,
            dedup::resolve_duplicate,
            organize::propose_actions,
            organize::execute_action,
            organize::undo_action,
            organize::list_actions,
            settings::get_settings,
            settings::save_settings,
            settings::check_ollama,
            stats::get_stats,
            photos::platform,
            photos::photos_access,
            photos::photos_scan,
            photos::photos_groups,
            photos::photos_classify,
            photos::photos_cancel,
            photos::photos_add_album,
            photos::photos_open_app,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri error");
}

fn main() {
    run()
}
