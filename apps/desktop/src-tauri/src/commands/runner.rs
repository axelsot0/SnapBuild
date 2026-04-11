use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
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

    // Stream stdout on a background thread
    let stdout_pipe = child.stdout.take().expect("stdout pipe missing");
    let sid_out = snapshot_id.clone();
    let app_out = app.clone();
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        for line in reader.lines().flatten() {
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
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr_pipe);
        for line in reader.lines().flatten() {
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
        command: full_command,
        exit_code,
        duration_ms,
        success: status.success(),
    };

    // Notify all windows that a build finished
    app.emit("snapbuild://build-complete", result.clone()).ok();

    Ok(result)
}
