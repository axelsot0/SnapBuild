use snapshot_engine::{blob_path, read_manifest, SnapshotError};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum MaterializerError {
    Snapshot(SnapshotError),
    Io(std::io::Error),
}

impl From<SnapshotError> for MaterializerError {
    fn from(value: SnapshotError) -> Self {
        Self::Snapshot(value)
    }
}

impl From<std::io::Error> for MaterializerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn materialize_snapshot(
    manifest_path: &Path,
    store_root: &Path,
    target_dir: &Path,
) -> Result<(), MaterializerError> {
    let manifest = read_manifest(manifest_path)?;
    let objects_dir = store_root.join("objects");

    fs::create_dir_all(target_dir)?;

    for entry in manifest.entries {
        let source_blob = blob_path(&objects_dir, &entry.blob_hash);
        let output_path = target_dir.join(entry.path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source_blob, output_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use snapshot_engine::{create_snapshot, manifest_path};

    #[test]
    fn materializes_files_from_snapshot() {
        let base = std::env::temp_dir().join(format!(
            "snapbuild-materializer-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        let project = base.join("project");
        let store = base.join(".snapbuild");
        let target = base.join("materialized");

        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("src/app.ts"), "export const x = 1;").unwrap();

        create_snapshot(&project, &store, "snap-a").unwrap();
        materialize_snapshot(&manifest_path(&store, "snap-a"), &store, &target).unwrap();

        let content = fs::read_to_string(target.join("src/app.ts")).unwrap();
        assert_eq!(content, "export const x = 1;");

        let _ = fs::remove_dir_all(base);
    }
}
