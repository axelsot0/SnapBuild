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
    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    // Count existing snapshots
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

    Ok(ProjectInfo {
        path,
        name,
        store_root: store_root.to_string_lossy().to_string(),
        snapshot_count,
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
    })
}
