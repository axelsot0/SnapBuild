use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppState {
    pub project_path: Mutex<Option<PathBuf>>,
    pub store_root: Mutex<Option<PathBuf>>,
    /// Maps snapshot_id -> parent_id (tracked in memory, not in manifest)
    pub parent_map: Mutex<HashMap<String, String>>,
    /// Current HEAD snapshot id
    pub head_snapshot: Mutex<Option<String>>,
}
