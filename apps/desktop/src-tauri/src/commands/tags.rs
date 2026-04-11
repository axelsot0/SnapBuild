use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagInfo {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[tauri::command]
pub fn add_tag(
    snapshot_id: String,
    tag_name: String,
    color: Option<String>,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let db_path = state
        .db_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No database initialized")?;

    snapshot_engine::sqlite_persistence::add_tag(
        &db_path,
        &snapshot_id,
        &tag_name,
        color.as_deref(),
    )
    .map_err(|e| format!("Add tag failed: {:?}", e))?;

    app.emit("snapbuild://snapshot-list-changed", ()).ok();
    Ok(())
}

#[tauri::command]
pub fn remove_tag(
    snapshot_id: String,
    tag_name: String,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let db_path = state
        .db_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No database initialized")?;

    snapshot_engine::sqlite_persistence::remove_tag(&db_path, &snapshot_id, &tag_name)
        .map_err(|e| format!("Remove tag failed: {:?}", e))?;

    app.emit("snapbuild://snapshot-list-changed", ()).ok();
    Ok(())
}

#[tauri::command]
pub fn set_snapshot_color(
    snapshot_id: String,
    color: Option<String>,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let db_path = state
        .db_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No database initialized")?;

    snapshot_engine::sqlite_persistence::set_snapshot_color(
        &db_path,
        &snapshot_id,
        color.as_deref(),
    )
    .map_err(|e| format!("Set color failed: {:?}", e))?;

    app.emit("snapbuild://snapshot-list-changed", ()).ok();
    Ok(())
}

#[tauri::command]
pub fn list_tags(state: State<AppState>) -> Result<Vec<TagInfo>, String> {
    let db_path = state
        .db_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No database initialized")?;

    let tags = snapshot_engine::sqlite_persistence::list_all_tags(&db_path)
        .map_err(|e| format!("List tags failed: {:?}", e))?;

    Ok(tags
        .into_iter()
        .map(|(id, name, color)| TagInfo { id, name, color })
        .collect())
}
