use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SnapshotError {
    Io(std::io::Error),
    Parse(String),
    PathOutsideProject(String),
}

impl From<std::io::Error> for SnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub path: String,
    pub blob_hash: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotManifest {
    pub snapshot_id: String,
    pub created_at_unix_ms: u128,
    pub entries: Vec<SnapshotEntry>,
}

pub fn create_snapshot(
    project_root: &Path,
    store_root: &Path,
    snapshot_id: &str,
) -> Result<SnapshotManifest, SnapshotError> {
    let objects_dir = store_root.join("objects");
    let snapshots_dir = store_root.join("snapshots");
    fs::create_dir_all(&objects_dir)?;
    fs::create_dir_all(&snapshots_dir)?;

    let mut files = Vec::new();
    collect_files(project_root, project_root, &mut files)?;

    let mut entries = Vec::new();
    for abs_path in files {
        let rel_path = abs_path
            .strip_prefix(project_root)
            .map_err(|_| SnapshotError::PathOutsideProject(abs_path.display().to_string()))?;

        let data = fs::read(&abs_path)?;
        let hash = hash_bytes(&data);
        let blob = blob_path(&objects_dir, &hash);
        if !blob.exists() {
            if let Some(parent) = blob.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(blob, &data)?;
        }

        entries.push(SnapshotEntry {
            path: normalize_path(rel_path),
            blob_hash: hash,
            size_bytes: data.len() as u64,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let manifest = SnapshotManifest {
        snapshot_id: snapshot_id.to_string(),
        created_at_unix_ms: unix_ms(),
        entries,
    };

    write_manifest(&manifest, &snapshots_dir)?;
    Ok(manifest)
}

pub fn read_manifest(manifest_path: &Path) -> Result<SnapshotManifest, SnapshotError> {
    let content = fs::read_to_string(manifest_path)?;
    parse_manifest(&content)
}

pub fn manifest_path(store_root: &Path, snapshot_id: &str) -> PathBuf {
    store_root.join("snapshots").join(format!("{snapshot_id}.manifest"))
}

pub fn blob_path(objects_dir: &Path, hash: &str) -> PathBuf {
    let (prefix, suffix) = hash.split_at(2);
    objects_dir.join(prefix).join(suffix)
}

fn write_manifest(manifest: &SnapshotManifest, snapshots_dir: &Path) -> Result<(), SnapshotError> {
    let mut body = format!("{}|{}\n", manifest.snapshot_id, manifest.created_at_unix_ms);
    for entry in &manifest.entries {
        body.push_str(&format!(
            "{}\t{}\t{}\n",
            entry.path, entry.blob_hash, entry.size_bytes
        ));
    }
    fs::write(
        snapshots_dir.join(format!("{}.manifest", manifest.snapshot_id)),
        body,
    )?;
    Ok(())
}

fn parse_manifest(content: &str) -> Result<SnapshotManifest, SnapshotError> {
    let mut lines = content.lines();
    let header = lines
        .next()
        .ok_or_else(|| SnapshotError::Parse("missing manifest header".to_string()))?;
    let mut header_parts = header.split('|');
    let snapshot_id = header_parts
        .next()
        .ok_or_else(|| SnapshotError::Parse("missing snapshot id".to_string()))?
        .to_string();
    let created_at_unix_ms = header_parts
        .next()
        .ok_or_else(|| SnapshotError::Parse("missing timestamp".to_string()))?
        .parse::<u128>()
        .map_err(|_| SnapshotError::Parse("invalid timestamp".to_string()))?;

    let mut entries = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut p = line.split('\t');
        let path = p
            .next()
            .ok_or_else(|| SnapshotError::Parse("invalid entry path".to_string()))?
            .to_string();
        let blob_hash = p
            .next()
            .ok_or_else(|| SnapshotError::Parse("invalid entry hash".to_string()))?
            .to_string();
        let size_bytes = p
            .next()
            .ok_or_else(|| SnapshotError::Parse("invalid entry size".to_string()))?
            .parse::<u64>()
            .map_err(|_| SnapshotError::Parse("invalid entry size number".to_string()))?;
        entries.push(SnapshotEntry {
            path,
            blob_hash,
            size_bytes,
        });
    }

    Ok(SnapshotManifest {
        snapshot_id,
        created_at_unix_ms,
        entries,
    })
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<(), SnapshotError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if should_ignore(&file_name) {
            continue;
        }

        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else if path.is_file() {
            if path.starts_with(root) {
                out.push(path);
            }
        }
    }
    Ok(())
}

fn should_ignore(name: &str) -> bool {
    [".git", "node_modules", "build", "dist", ".snapbuild"]
        .iter()
        .any(|n| n == &name)
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_snapshot_and_manifest() {
        let base = std::env::temp_dir().join(format!("snapbuild-test-{}", unix_ms()));
        let project = base.join("project");
        let store = base.join(".snapbuild");
        fs::create_dir_all(project.join("src")).unwrap();

        fs::write(project.join("a.txt"), "hello").unwrap();
        fs::write(project.join("src/main.js"), "console.log('x')").unwrap();

        let snapshot = create_snapshot(&project, &store, "snap-1").unwrap();
        assert_eq!(snapshot.entries.len(), 2);

        let manifest = read_manifest(&manifest_path(&store, "snap-1")).unwrap();
        assert_eq!(manifest.snapshot_id, "snap-1");

        for entry in manifest.entries {
            let blob = blob_path(&store.join("objects"), &entry.blob_hash);
            assert!(blob.exists());
        }

        let _ = fs::remove_dir_all(base);
    }
}
