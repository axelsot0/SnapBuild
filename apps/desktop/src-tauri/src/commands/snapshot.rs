use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotRecord {
    pub snapshot_id: String,
    pub created_at_unix_ms: u64,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub parent_id: Option<String>,
    pub status: SnapshotStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    Ok,
    Failed,
    NotRun,
    Idle,
}

#[tauri::command]
pub fn create_snapshot(
    label: Option<String>,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<SnapshotRecord, String> {
    let project_path = state
        .project_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    // Get current HEAD as parent
    let parent_id = state.head_snapshot.lock().unwrap().clone();

    // Generate snapshot ID
    let snapshot_id = label.unwrap_or_else(|| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("snap-{:x}", ts)
    });

    let manifest =
        snapshot_engine::create_snapshot(&project_path, &store_root, &snapshot_id)
            .map_err(|e| format!("Snapshot failed: {:?}", e))?;

    // Track parent relationship
    if let Some(ref pid) = parent_id {
        state
            .parent_map
            .lock()
            .unwrap()
            .insert(snapshot_id.clone(), pid.clone());
    }

    // Update HEAD
    *state.head_snapshot.lock().unwrap() = Some(snapshot_id.clone());

    let record = SnapshotRecord {
        file_count: manifest.entries.len(),
        total_size_bytes: manifest.entries.iter().map(|e| e.size_bytes).sum(),
        created_at_unix_ms: manifest.created_at_unix_ms as u64,
        snapshot_id: manifest.snapshot_id,
        parent_id,
        status: SnapshotStatus::Idle,
    };

    // Notify all windows
    app.emit("snapbuild://snapshot-list-changed", ()).ok();

    Ok(record)
}

#[tauri::command]
pub fn list_snapshots(state: State<AppState>) -> Result<Vec<SnapshotRecord>, String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let snapshots_dir = store_root.join("snapshots");
    if !snapshots_dir.exists() {
        return Ok(vec![]);
    }

    let parent_map = state.parent_map.lock().unwrap().clone();

    let mut records: Vec<SnapshotRecord> = std::fs::read_dir(&snapshots_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "manifest")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let manifest_path = e.path();
            snapshot_engine::read_manifest(&manifest_path).ok().map(|m| {
                let parent_id = parent_map.get(&m.snapshot_id).cloned();
                SnapshotRecord {
                    file_count: m.entries.len(),
                    total_size_bytes: m.entries.iter().map(|e| e.size_bytes).sum(),
                    created_at_unix_ms: m.created_at_unix_ms as u64,
                    snapshot_id: m.snapshot_id,
                    parent_id,
                    status: SnapshotStatus::Idle,
                }
            })
        })
        .collect();

    // Sort by created_at descending (newest first)
    records.sort_by(|a, b| b.created_at_unix_ms.cmp(&a.created_at_unix_ms));

    Ok(records)
}

#[tauri::command]
pub fn delete_snapshot(
    snapshot_id: String,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let manifest_path = snapshot_engine::manifest_path(&store_root, &snapshot_id);
    if manifest_path.exists() {
        std::fs::remove_file(&manifest_path).map_err(|e| e.to_string())?;
    }

    state.parent_map.lock().unwrap().remove(&snapshot_id);

    // If deleted was HEAD, clear HEAD
    let mut head = state.head_snapshot.lock().unwrap();
    if head.as_deref() == Some(&snapshot_id) {
        *head = None;
    }

    app.emit("snapbuild://snapshot-list-changed", ()).ok();
    Ok(())
}

#[tauri::command]
pub fn restore_snapshot(
    snapshot_id: String,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;
    let project_path = state
        .project_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let manifest_path = snapshot_engine::manifest_path(&store_root, &snapshot_id);

    workspace_materializer::materialize_snapshot(&manifest_path, &store_root, &project_path)
        .map_err(|e| format!("Restore failed: {:?}", e))?;

    *state.head_snapshot.lock().unwrap() = Some(snapshot_id);
    app.emit("snapbuild://snapshot-list-changed", ()).ok();
    Ok(())
}

#[tauri::command]
pub fn materialize_snapshot(
    snapshot_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let manifest_path = snapshot_engine::manifest_path(&store_root, &snapshot_id);
    let temp_dir = std::env::temp_dir().join(format!("snapbuild-{}", &snapshot_id));
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    workspace_materializer::materialize_snapshot(&manifest_path, &store_root, &temp_dir)
        .map_err(|e| format!("Materialize failed: {:?}", e))?;

    Ok(temp_dir.to_string_lossy().to_string())
}
