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
            commands::project::get_git_branch,
            // Snapshots
            commands::snapshot::create_snapshot,
            commands::snapshot::list_snapshots,
            commands::snapshot::delete_snapshot,
            commands::snapshot::restore_snapshot,
            commands::snapshot::materialize_snapshot,
            commands::snapshot::list_snapshot_files,
            commands::snapshot::compare_snapshots,
            // Build runner
            commands::runner::run_snapshot_build,
            commands::runner::get_build_history,
            // Tags & Colors
            commands::tags::add_tag,
            commands::tags::remove_tag,
            commands::tags::set_snapshot_color,
            commands::tags::list_tags,
            // Windows
            commands::window::open_window,
            // Watcher
            commands::watcher::start_watcher,
            commands::watcher::stop_watcher,
            commands::watcher::toggle_auto_snapshot,
            commands::watcher::get_auto_snapshot_enabled,
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
