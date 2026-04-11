use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use tauri::{Emitter, State};

use crate::state::AppState;

/// A static flag so we only spawn one watcher thread per session.
static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start watching the project directory for file changes.
/// Emits `snapbuild://project-changed` events (debounced 2s).
/// Ignores `.snapbuild/`, `node_modules/`, `.git/`, `target/`.
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

/// Stop watching (no-op for now — will stop when app closes).
#[tauri::command]
pub fn stop_watcher() -> Result<(), String> {
    WATCHER_RUNNING.store(false, Ordering::SeqCst);
    Ok(())
}
