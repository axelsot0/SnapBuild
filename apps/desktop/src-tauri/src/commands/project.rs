use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub store_root: String,
    pub snapshot_count: usize,
    pub git_branch: Option<String>,
}

#[tauri::command]
pub fn open_project(path: String, state: State<AppState>) -> Result<ProjectInfo, String> {
    let project_path = PathBuf::from(&path);
    if !project_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !project_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let store_root = project_path.join(".snapbuild");
    std::fs::create_dir_all(&store_root).map_err(|e| e.to_string())?;

    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    // Initialize SQLite database
    let db_path = store_root.join("snapbuild.db");
    snapshot_engine::sqlite_persistence::init_schema(&db_path)
        .map_err(|e| format!("SQLite init failed: {:?}", e))?;

    // Generate a stable project ID from the path
    let project_id = format!("proj-{:x}", {
        let mut hash: u64 = 5381;
        for b in path.as_bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(*b as u64);
        }
        hash
    });

    // Detect git branch
    let git_branch = snapshot_engine::detect_git_branch(&project_path);

    // Save/update project in SQLite
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    snapshot_engine::sqlite_persistence::save_project(
        &db_path,
        &project_id,
        &name,
        &path,
        git_branch.as_deref(),
        created_at,
    )
    .map_err(|e| format!("Save project failed: {:?}", e))?;

    // Rebuild parent_map from SQLite edges
    let edges = snapshot_engine::sqlite_persistence::list_snapshot_edges(&db_path, &project_id)
        .unwrap_or_default();
    let mut parent_map = state.parent_map.lock().unwrap();
    parent_map.clear();
    for (parent_id, child_id) in edges {
        parent_map.insert(child_id, parent_id);
    }
    drop(parent_map);

    // Count existing snapshots (from disk as source of truth for file count)
    let snapshot_count = {
        let snapshots_dir = store_root.join("snapshots");
        if snapshots_dir.exists() {
            std::fs::read_dir(&snapshots_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "manifest")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        }
    };

    *state.project_path.lock().unwrap() = Some(project_path);
    *state.store_root.lock().unwrap() = Some(store_root.clone());
    *state.db_path.lock().unwrap() = Some(db_path);
    *state.project_id.lock().unwrap() = Some(project_id);

    Ok(ProjectInfo {
        path,
        name,
        store_root: store_root.to_string_lossy().to_string(),
        snapshot_count,
        git_branch,
    })
}

#[tauri::command]
pub fn get_project_info(state: State<AppState>) -> Option<ProjectInfo> {
    let project_path = state.project_path.lock().unwrap().clone()?;
    let store_root = state.store_root.lock().unwrap().clone()?;

    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    let git_branch = snapshot_engine::detect_git_branch(&project_path);

    let snapshot_count = {
        let snapshots_dir = store_root.join("snapshots");
        if snapshots_dir.exists() {
            std::fs::read_dir(&snapshots_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "manifest")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        }
    };

    Some(ProjectInfo {
        path: project_path.to_string_lossy().to_string(),
        name,
        store_root: store_root.to_string_lossy().to_string(),
        snapshot_count,
        git_branch,
    })
}

#[tauri::command]
pub fn get_git_branch(state: State<AppState>) -> Option<String> {
    let project_path = state.project_path.lock().unwrap().clone()?;
    snapshot_engine::detect_git_branch(&project_path)
}
