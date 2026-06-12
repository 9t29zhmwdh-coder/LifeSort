mod commands;
mod error;
mod state;

use commands::*;
use ls_core::db;
use state::{AppSettings, AppState};
use std::{collections::HashMap, sync::Arc};
use tauri::Manager;
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("no app data dir");
            let db_path = data_dir.join("lifesort.db");

            let rt = tokio::runtime::Handle::current();
            let pool = rt.block_on(db::open(&db_path)).expect("DB init failed");

            let app_state = Arc::new(AppState {
                pool,
                files: Arc::new(RwLock::new(HashMap::new())),
                actions: Arc::new(RwLock::new(vec![])),
                settings: Arc::new(RwLock::new(AppSettings::default())),
            });
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scanner::scan_directory,
            scanner::get_scan_results,
            classify::classify_file,
            classify::classify_batch,
            dedup::find_duplicates,
            dedup::resolve_duplicate,
            organize::propose_actions,
            organize::execute_action,
            organize::execute_all,
            organize::undo_action,
            organize::list_actions,
            settings::get_settings,
            settings::save_settings,
            settings::check_ollama,
            settings::list_plugins,
            stats::get_stats,
            watcher::start_watch,
            watcher::stop_watch,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri error");
}

fn main() {
    run()
}
