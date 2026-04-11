use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use tauri::{Emitter, Manager, State};

use crate::state::AppState;

/// A static flag so we only spawn one watcher thread per session.
static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start watching the project directory for file changes.
/// Emits `snapbuild://project-changed` events (debounced 2s).
/// Ignores `.snapbuild/`, `node_modules/`, `.git/`, `target/`.
/// When auto-snapshot is enabled, automatically creates snapshots on change.
#[tauri::command]
pub fn start_watcher(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        return Ok(()); // already running
    }

    let project_path = state
        .project_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let app_handle = app.clone();
    let path = project_path.clone();

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer = match new_debouncer(Duration::from_secs(2), tx) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Watcher init failed: {:?}", e);
                WATCHER_RUNNING.store(false, Ordering::SeqCst);
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(&path, RecursiveMode::Recursive) {
            eprintln!("Watch start failed: {:?}", e);
            WATCHER_RUNNING.store(false, Ordering::SeqCst);
            return;
        }

        let ignored = [".snapbuild", "node_modules", ".git", "target", "__pycache__"];

        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    let dominated_by_ignored = events.iter().all(|ev| {
                        let p = ev.path.to_string_lossy();
                        ignored.iter().any(|ig| p.contains(ig))
                    });

                    if !dominated_by_ignored && !events.is_empty() {
                        app_handle
                            .emit("snapbuild://project-changed", ())
                            .ok();

                        // Auto-snapshot if enabled
                        let app_state = app_handle.state::<AppState>();
                        let auto_enabled = *app_state.auto_snapshot_enabled.lock().unwrap();
                        if auto_enabled {
                            try_auto_snapshot(&app_handle, &path);
                        }
                    }
                }
                Ok(Err(errs)) => {
                    eprintln!("Watcher errors: {:?}", errs);
                }
                Err(_) => {
                    // Channel closed, watcher dropped
                    break;
                }
            }
        }

        WATCHER_RUNNING.store(false, Ordering::SeqCst);
    });

    Ok(())
}

/// Attempt auto-snapshot creation using fingerprint dedup
fn try_auto_snapshot(app: &tauri::AppHandle, project_path: &std::path::Path) {
    let state = app.state::<AppState>();

    let store_root = match state.store_root.lock().unwrap().clone() {
        Some(s) => s,
        None => return,
    };

    // Compute current source fingerprint from all project files
    // We use an empty file list which will produce a hash of empty string,
    // so instead we use the dependency_fingerprint as a quick proxy
    let current_fp = snapshot_engine::dependency_fingerprint(project_path)
        .unwrap_or_default();

    if current_fp.is_empty() {
        return;
    }

    let last_fp = state.last_source_fingerprint.lock().unwrap().clone();
    let should_snap = snapshot_engine::auto_snapshot::should_create_snapshot(
        snapshot_engine::auto_snapshot::SnapshotTrigger::OnSave,
        true,
        &current_fp,
        last_fp.as_deref(),
    );

    if !should_snap {
        return;
    }

    // Generate auto snapshot ID
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let snapshot_id = format!("auto-{:x}", ts);

    // Get parent
    let parent_id = state.head_snapshot.lock().unwrap().clone();

    match snapshot_engine::create_snapshot(project_path, &store_root, &snapshot_id) {
        Ok(manifest) => {
            // Track parent
            if let Some(ref pid) = parent_id {
                state
                    .parent_map
                    .lock()
                    .unwrap()
                    .insert(snapshot_id.clone(), pid.clone());
            }

            // Update HEAD
            *state.head_snapshot.lock().unwrap() = Some(snapshot_id.clone());

            // Update last fingerprint
            *state.last_source_fingerprint.lock().unwrap() = Some(current_fp);

            // Persist to SQLite
            let db_path = state.db_path.lock().unwrap().clone();
            let project_id = state.project_id.lock().unwrap().clone();
            if let (Some(db), Some(pid)) = (db_path, project_id) {
                let git_branch = snapshot_engine::detect_git_branch(project_path);
                let dep_fp = snapshot_engine::dependency_fingerprint(project_path)
                    .unwrap_or_default();
                let file_paths: Vec<String> = manifest.entries.iter().map(|e| e.path.clone()).collect();
                let src_fp = snapshot_engine::source_fingerprint_for_files(project_path, &file_paths)
                    .unwrap_or_default();

                let meta = snapshot_engine::domain::SnapshotMeta {
                    id: snapshot_id.clone(),
                    parent_id: parent_id.clone(),
                    name: snapshot_id.clone(),
                    created_at_unix_ms: manifest.created_at_unix_ms,
                    source_fingerprint: src_fp,
                    dependency_fingerprint: dep_fp,
                    git_branch,
                    changed_files: file_paths,
                    favorite: false,
                    tags: vec![],
                    color: None,
                    build_status: snapshot_engine::domain::BuildStatus::NotRun,
                };
                snapshot_engine::sqlite_persistence::save_snapshot(&db, &pid, &meta).ok();
                if let Some(ref parent) = parent_id {
                    snapshot_engine::sqlite_persistence::save_snapshot_edge(&db, parent, &snapshot_id).ok();
                }
            }

            // Notify all windows
            app.emit("snapbuild://snapshot-list-changed", ()).ok();
        }
        Err(e) => {
            eprintln!("Auto-snapshot failed: {:?}", e);
        }
    }
}

/// Stop watching (no-op for now — will stop when app closes).
#[tauri::command]
pub fn stop_watcher() -> Result<(), String> {
    WATCHER_RUNNING.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn toggle_auto_snapshot(enabled: bool, state: State<AppState>) -> Result<(), String> {
    *state.auto_snapshot_enabled.lock().unwrap() = enabled;
    Ok(())
}

#[tauri::command]
pub fn get_auto_snapshot_enabled(state: State<AppState>) -> bool {
    *state.auto_snapshot_enabled.lock().unwrap()
}
