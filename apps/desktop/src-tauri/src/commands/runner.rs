use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildResult {
    pub snapshot_id: String,
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildRunRecord {
    pub id: String,
    pub snapshot_id: String,
    pub command: String,
    pub started_at_unix_ms: u64,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildStatusChanged {
    pub snapshot_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildChunk {
    pub snapshot_id: String,
    pub line: String,
    pub is_stderr: bool,
}

/// Materialize a snapshot into a temp directory and run the given command
/// inside it, streaming stdout/stderr as `snapbuild://build-output-chunk`
/// events. Emits `snapbuild://build-complete` when the process exits.
#[tauri::command]
pub fn run_snapshot_build(
    snapshot_id: String,
    command: String,
    args: Vec<String>,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<BuildResult, String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    // Materialize the snapshot into a temp directory
    let temp_dir = std::env::temp_dir().join(format!("snapbuild-run-{}", &snapshot_id));
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let manifest_path = snapshot_engine::manifest_path(&store_root, &snapshot_id);
    workspace_materializer::materialize_snapshot(&manifest_path, &store_root, &temp_dir)
        .map_err(|e| format!("Materialize failed: {:?}", e))?;

    // Spawn the process with piped I/O
    let started = std::time::Instant::now();

    // On Windows, spawn via cmd.exe so that .cmd/.bat scripts (npm, npx, etc.)
    // and PATH resolution work correctly.
    #[cfg(target_os = "windows")]
    let mut child = Command::new("cmd")
        .arg("/C")
        .arg(&command)
        .args(&args)
        .current_dir(&temp_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", command, e))?;

    #[cfg(not(target_os = "windows"))]
    let mut child = Command::new(&command)
        .args(&args)
        .current_dir(&temp_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", command, e))?;

    // Accumulators for stdout/stderr (for persistence)
    let stdout_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf = Arc::new(Mutex::new(String::new()));

    // Stream stdout on a background thread
    let stdout_pipe = child.stdout.take().expect("stdout pipe missing");
    let sid_out = snapshot_id.clone();
    let app_out = app.clone();
    let stdout_buf_out = stdout_buf.clone();
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        for line in reader.lines().flatten() {
            {
                let mut buf = stdout_buf_out.lock().unwrap();
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(&line);
            }
            app_out
                .emit(
                    "snapbuild://build-output-chunk",
                    BuildChunk {
                        snapshot_id: sid_out.clone(),
                        line,
                        is_stderr: false,
                    },
                )
                .ok();
        }
    });

    // Stream stderr on a background thread
    let stderr_pipe = child.stderr.take().expect("stderr pipe missing");
    let sid_err = snapshot_id.clone();
    let app_err = app.clone();
    let stderr_buf_err = stderr_buf.clone();
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr_pipe);
        for line in reader.lines().flatten() {
            {
                let mut buf = stderr_buf_err.lock().unwrap();
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(&line);
            }
            app_err
                .emit(
                    "snapbuild://build-output-chunk",
                    BuildChunk {
                        snapshot_id: sid_err.clone(),
                        line,
                        is_stderr: true,
                    },
                )
                .ok();
        }
    });

    // Wait for process to exit, then join streaming threads
    let status = child.wait().map_err(|e| e.to_string())?;
    stdout_thread.join().ok();
    stderr_thread.join().ok();

    let duration_ms = started.elapsed().as_millis() as u64;
    let exit_code = status.code().unwrap_or(-1);

    let full_command = std::iter::once(command.as_str())
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");

    let result = BuildResult {
        snapshot_id: snapshot_id.clone(),
        command: full_command.clone(),
        exit_code,
        duration_ms,
        success: status.success(),
    };

    // Persist build run to SQLite
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db) = db_path {
        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            - duration_ms as u128;

        let build_run = snapshot_engine::domain::BuildRun {
            id: format!("run-{:x}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
            snapshot_id: snapshot_id.clone(),
            command: full_command,
            started_at_unix_ms: started_at,
            duration_ms: duration_ms as u128,
            success: status.success(),
            stdout: stdout_buf.lock().unwrap().clone(),
            stderr: stderr_buf.lock().unwrap().clone(),
            metadata: None,
        };
        snapshot_engine::sqlite_persistence::save_build_run(&db, &build_run).ok();

        // Update snapshot build status
        let status_str = if status.success() { "success" } else { "failed" };
        snapshot_engine::sqlite_persistence::update_snapshot_status(&db, &snapshot_id, status_str).ok();

        // Emit build status changed event
        app.emit("snapbuild://build-status-changed", BuildStatusChanged {
            snapshot_id: snapshot_id.clone(),
            status: status_str.to_string(),
        }).ok();
    }

    // Notify all windows that a build finished
    app.emit("snapbuild://build-complete", result.clone()).ok();

    Ok(result)
}

#[tauri::command]
pub fn get_build_history(
    snapshot_id: String,
    state: State<AppState>,
) -> Result<Vec<BuildRunRecord>, String> {
    let db_path = state
        .db_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No database initialized")?;

    let runs = snapshot_engine::sqlite_persistence::list_build_runs(&db_path, &snapshot_id)
        .map_err(|e| format!("Query failed: {:?}", e))?;

    Ok(runs
        .into_iter()
        .map(|r| BuildRunRecord {
            id: r.id,
            snapshot_id: r.snapshot_id,
            command: r.command,
            started_at_unix_ms: r.started_at_unix_ms as u64,
            duration_ms: r.duration_ms as u64,
            success: r.success,
        })
        .collect())
}
