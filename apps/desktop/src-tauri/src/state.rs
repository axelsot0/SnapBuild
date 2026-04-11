use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppState {
    pub project_path: Mutex<Option<PathBuf>>,
    pub store_root: Mutex<Option<PathBuf>>,
    /// Path to the SQLite database file
    pub db_path: Mutex<Option<PathBuf>>,
    /// Project identifier for SQLite queries
    pub project_id: Mutex<Option<String>>,
    /// Maps child_snapshot_id -> parent_snapshot_id
    pub parent_map: Mutex<HashMap<String, String>>,
    /// Current HEAD snapshot id
    pub head_snapshot: Mutex<Option<String>>,
    /// Whether auto-snapshot on file change is enabled
    pub auto_snapshot_enabled: Mutex<bool>,
    /// Last computed source fingerprint (for dedup in auto-snapshot)
    pub last_source_fingerprint: Mutex<Option<String>>,
}
