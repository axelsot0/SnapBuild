pub mod commands;
pub mod state;

use tauri::Manager;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Project
            commands::project::open_project,
            commands::project::get_project_info,
            // Snapshots
            commands::snapshot::create_snapshot,
            commands::snapshot::list_snapshots,
            commands::snapshot::delete_snapshot,
            commands::snapshot::restore_snapshot,
            commands::snapshot::materialize_snapshot,
            // Build runner
            commands::runner::run_snapshot_build,
            // Windows
            commands::window::open_window,
            // Watcher
            commands::watcher::start_watcher,
            commands::watcher::stop_watcher,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_webview_window("main")
                .unwrap()
                .open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running SnapBuild");
}
