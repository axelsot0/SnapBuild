use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use crate::state::AppState;
use snapshot_engine::domain::{BuildStatus, SnapshotMeta};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotRecord {
    pub snapshot_id: String,
    pub created_at_unix_ms: u64,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub parent_id: Option<String>,
    pub status: SnapshotStatus,
    pub git_branch: Option<String>,
    pub source_fingerprint: Option<String>,
    pub tags: Vec<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    Ok,
    Failed,
    NotRun,
    Idle,
}

fn build_status_to_snapshot_status(bs: &BuildStatus) -> SnapshotStatus {
    match bs {
        BuildStatus::Success => SnapshotStatus::Ok,
        BuildStatus::Failed => SnapshotStatus::Failed,
        BuildStatus::Running => SnapshotStatus::NotRun,
        BuildStatus::NotRun => SnapshotStatus::Idle,
    }
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

    // Track parent relationship in memory
    if let Some(ref pid) = parent_id {
        state
            .parent_map
            .lock()
            .unwrap()
            .insert(snapshot_id.clone(), pid.clone());
    }

    // Update HEAD
    *state.head_snapshot.lock().unwrap() = Some(snapshot_id.clone());

    // Compute fingerprints and detect git branch
    let git_branch = snapshot_engine::detect_git_branch(&project_path);
    let file_paths: Vec<String> = manifest.entries.iter().map(|e| e.path.clone()).collect();
    let source_fp = snapshot_engine::source_fingerprint_for_files(&project_path, &file_paths)
        .unwrap_or_default();
    let dep_fp = snapshot_engine::dependency_fingerprint(&project_path).unwrap_or_default();

    // Update last source fingerprint for auto-snapshot dedup
    *state.last_source_fingerprint.lock().unwrap() = Some(source_fp.clone());

    // Persist to SQLite
    let db_path = state.db_path.lock().unwrap().clone();
    let project_id = state.project_id.lock().unwrap().clone();
    if let (Some(db), Some(pid)) = (db_path, project_id) {
        let changed_files: Vec<String> = manifest.entries.iter().map(|e| e.path.clone()).collect();
        let meta = SnapshotMeta {
            id: snapshot_id.clone(),
            parent_id: parent_id.clone(),
            name: snapshot_id.clone(),
            created_at_unix_ms: manifest.created_at_unix_ms,
            source_fingerprint: source_fp.clone(),
            dependency_fingerprint: dep_fp,
            git_branch: git_branch.clone(),
            changed_files,
            favorite: false,
            tags: vec![],
            color: None,
            build_status: BuildStatus::NotRun,
        };
        snapshot_engine::sqlite_persistence::save_snapshot(&db, &pid, &meta).ok();

        if let Some(ref parent) = parent_id {
            snapshot_engine::sqlite_persistence::save_snapshot_edge(&db, parent, &snapshot_id).ok();
        }
    }

    let record = SnapshotRecord {
        file_count: manifest.entries.len(),
        total_size_bytes: manifest.entries.iter().map(|e| e.size_bytes).sum(),
        created_at_unix_ms: manifest.created_at_unix_ms as u64,
        snapshot_id: manifest.snapshot_id,
        parent_id,
        status: SnapshotStatus::Idle,
        git_branch,
        source_fingerprint: Some(source_fp),
        tags: vec![],
        color: None,
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

    let db_path = state.db_path.lock().unwrap().clone();
    let project_id = state.project_id.lock().unwrap().clone();

    // Try SQLite first for richer metadata
    if let (Some(db), Some(pid)) = (db_path, project_id) {
        let db_snapshots = snapshot_engine::sqlite_persistence::list_snapshots(&db, &pid)
            .unwrap_or_default();

        if !db_snapshots.is_empty() {
            let mut records: Vec<SnapshotRecord> = db_snapshots
                .into_iter()
                .map(|meta| {
                    // Read manifest from disk to get accurate file_count and total_size
                    let manifest_path = snapshot_engine::manifest_path(&store_root, &meta.id);
                    let (file_count, total_size_bytes) = snapshot_engine::read_manifest(&manifest_path)
                        .map(|m| {
                            (
                                m.entries.len(),
                                m.entries.iter().map(|e| e.size_bytes).sum::<u64>(),
                            )
                        })
                        .unwrap_or((0, 0));

                    // Load tags for this snapshot
                    let tags = snapshot_engine::sqlite_persistence::list_snapshot_tags(&db, &meta.id)
                        .unwrap_or_default();

                    SnapshotRecord {
                        snapshot_id: meta.id,
                        created_at_unix_ms: meta.created_at_unix_ms as u64,
                        file_count,
                        total_size_bytes,
                        parent_id: meta.parent_id,
                        status: build_status_to_snapshot_status(&meta.build_status),
                        git_branch: meta.git_branch,
                        source_fingerprint: Some(meta.source_fingerprint),
                        tags,
                        color: meta.color,
                    }
                })
                .collect();

            records.sort_by(|a, b| b.created_at_unix_ms.cmp(&a.created_at_unix_ms));
            return Ok(records);
        }
    }

    // Fallback: read from manifest files on disk
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
                    git_branch: None,
                    source_fingerprint: None,
                    tags: vec![],
                    color: None,
                }
            })
        })
        .collect();

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

// ─── File listing & Compare ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub blob_hash: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifiedFile {
    pub path: String,
    pub old_hash: String,
    pub new_hash: String,
    pub old_size: u64,
    pub new_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffResult {
    pub added: Vec<FileEntry>,
    pub removed: Vec<FileEntry>,
    pub modified: Vec<ModifiedFile>,
}

#[tauri::command]
pub fn list_snapshot_files(
    snapshot_id: String,
    state: State<AppState>,
) -> Result<Vec<FileEntry>, String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let manifest_path = snapshot_engine::manifest_path(&store_root, &snapshot_id);
    let manifest = snapshot_engine::read_manifest(&manifest_path)
        .map_err(|e| format!("Read manifest failed: {:?}", e))?;

    Ok(manifest
        .entries
        .into_iter()
        .map(|e| FileEntry {
            path: e.path,
            blob_hash: e.blob_hash,
            size_bytes: e.size_bytes,
        })
        .collect())
}

#[tauri::command]
pub fn compare_snapshots(
    id_a: String,
    id_b: String,
    state: State<AppState>,
) -> Result<DiffResult, String> {
    let store_root = state
        .store_root
        .lock()
        .unwrap()
        .clone()
        .ok_or("No project open")?;

    let manifest_a = snapshot_engine::read_manifest(&snapshot_engine::manifest_path(&store_root, &id_a))
        .map_err(|e| format!("Read manifest A failed: {:?}", e))?;
    let manifest_b = snapshot_engine::read_manifest(&snapshot_engine::manifest_path(&store_root, &id_b))
        .map_err(|e| format!("Read manifest B failed: {:?}", e))?;

    let map_a: std::collections::HashMap<&str, &snapshot_engine::SnapshotEntry> =
        manifest_a.entries.iter().map(|e| (e.path.as_str(), e)).collect();
    let map_b: std::collections::HashMap<&str, &snapshot_engine::SnapshotEntry> =
        manifest_b.entries.iter().map(|e| (e.path.as_str(), e)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    // Files in B but not in A = added
    for (path, entry) in &map_b {
        if !map_a.contains_key(path) {
            added.push(FileEntry {
                path: path.to_string(),
                blob_hash: entry.blob_hash.clone(),
                size_bytes: entry.size_bytes,
            });
        }
    }

    // Files in A but not in B = removed
    for (path, entry) in &map_a {
        if !map_b.contains_key(path) {
            removed.push(FileEntry {
                path: path.to_string(),
                blob_hash: entry.blob_hash.clone(),
                size_bytes: entry.size_bytes,
            });
        }
    }

    // Files in both but with different hashes = modified
    for (path, entry_a) in &map_a {
        if let Some(entry_b) = map_b.get(path) {
            if entry_a.blob_hash != entry_b.blob_hash {
                modified.push(ModifiedFile {
                    path: path.to_string(),
                    old_hash: entry_a.blob_hash.clone(),
                    new_hash: entry_b.blob_hash.clone(),
                    old_size: entry_a.size_bytes,
                    new_size: entry_b.size_bytes,
                });
            }
        }
    }

    added.sort_by(|a, b| a.path.cmp(&b.path));
    removed.sort_by(|a, b| a.path.cmp(&b.path));
    modified.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(DiffResult { added, removed, modified })
}
