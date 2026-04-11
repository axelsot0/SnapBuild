use crate::domain::{BuildRun, BuildStatus, SnapshotMeta};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum SqlitePersistenceError {
    Io(std::io::Error),
    SqliteFailed(String),
    ParseInt(std::num::ParseIntError),
}

impl From<std::io::Error> for SqlitePersistenceError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::num::ParseIntError> for SqlitePersistenceError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

pub fn init_schema(db_path: &Path) -> Result<(), SqlitePersistenceError> {
    let sql = r#"
    PRAGMA journal_mode=WAL;

    CREATE TABLE IF NOT EXISTS projects (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      root_path TEXT NOT NULL,
      git_branch TEXT,
      created_at_unix_ms INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS snapshots (
      id TEXT PRIMARY KEY,
      project_id TEXT NOT NULL,
      parent_id TEXT,
      name TEXT NOT NULL,
      created_at_unix_ms INTEGER NOT NULL,
      source_fingerprint TEXT NOT NULL,
      dependency_fingerprint TEXT NOT NULL,
      git_branch TEXT,
      changed_files_json TEXT NOT NULL,
      favorite INTEGER NOT NULL,
      color TEXT,
      build_status TEXT NOT NULL,
      FOREIGN KEY(project_id) REFERENCES projects(id)
    );

    CREATE TABLE IF NOT EXISTS snapshot_edges (
      parent_id TEXT NOT NULL,
      child_id TEXT NOT NULL,
      PRIMARY KEY(parent_id, child_id)
    );

    CREATE TABLE IF NOT EXISTS build_runs (
      id TEXT PRIMARY KEY,
      snapshot_id TEXT NOT NULL,
      command TEXT NOT NULL,
      started_at_unix_ms INTEGER NOT NULL,
      duration_ms INTEGER NOT NULL,
      success INTEGER NOT NULL,
      stdout TEXT NOT NULL,
      stderr TEXT NOT NULL,
      metadata TEXT
    );

    CREATE TABLE IF NOT EXISTS tags (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      color TEXT
    );

    CREATE TABLE IF NOT EXISTS snapshot_tags (
      snapshot_id TEXT NOT NULL,
      tag_id TEXT NOT NULL,
      PRIMARY KEY(snapshot_id, tag_id)
    );
    "#;

    run_sql(db_path, sql).map(|_| ())
}

pub fn save_project(
    db_path: &Path,
    id: &str,
    name: &str,
    root_path: &str,
    git_branch: Option<&str>,
    created_at_unix_ms: u128,
) -> Result<(), SqlitePersistenceError> {
    let sql = format!(
        "INSERT OR REPLACE INTO projects (id,name,root_path,git_branch,created_at_unix_ms) VALUES ('{}','{}','{}',{},{});",
        esc(id),
        esc(name),
        esc(root_path),
        opt_text(git_branch),
        created_at_unix_ms
    );

    run_sql(db_path, &sql).map(|_| ())
}

pub fn save_snapshot(
    db_path: &Path,
    project_id: &str,
    snapshot: &SnapshotMeta,
) -> Result<(), SqlitePersistenceError> {
    let changed_files_json = format!(
        "[{}]",
        snapshot
            .changed_files
            .iter()
            .map(|f| format!("\"{}\"", esc(f)))
            .collect::<Vec<_>>()
            .join(",")
    );

    let sql = format!(
        "INSERT OR REPLACE INTO snapshots (id,project_id,parent_id,name,created_at_unix_ms,source_fingerprint,dependency_fingerprint,git_branch,changed_files_json,favorite,color,build_status) VALUES ('{}','{}',{},'{}',{},'{}','{}',{},'{}',{},{},'{}');",
        esc(&snapshot.id),
        esc(project_id),
        opt_text(snapshot.parent_id.as_deref()),
        esc(&snapshot.name),
        snapshot.created_at_unix_ms,
        esc(&snapshot.source_fingerprint),
        esc(&snapshot.dependency_fingerprint),
        opt_text(snapshot.git_branch.as_deref()),
        esc(&changed_files_json),
        if snapshot.favorite { 1 } else { 0 },
        opt_text(snapshot.color.as_deref()),
        build_status_to_str(&snapshot.build_status),
    );

    run_sql(db_path, &sql).map(|_| ())
}

pub fn save_snapshot_edge(
    db_path: &Path,
    parent_id: &str,
    child_id: &str,
) -> Result<(), SqlitePersistenceError> {
    let sql = format!(
        "INSERT OR IGNORE INTO snapshot_edges (parent_id,child_id) VALUES ('{}','{}');",
        esc(parent_id),
        esc(child_id)
    );
    run_sql(db_path, &sql).map(|_| ())
}

pub fn save_build_run(db_path: &Path, run: &BuildRun) -> Result<(), SqlitePersistenceError> {
    let sql = format!(
        "INSERT OR REPLACE INTO build_runs (id,snapshot_id,command,started_at_unix_ms,duration_ms,success,stdout,stderr,metadata) VALUES ('{}','{}','{}',{}, {}, {}, '{}', '{}', {});",
        esc(&run.id),
        esc(&run.snapshot_id),
        esc(&run.command),
        run.started_at_unix_ms,
        run.duration_ms,
        if run.success { 1 } else { 0 },
        esc(&run.stdout),
        esc(&run.stderr),
        opt_text(run.metadata.as_deref()),
    );
    run_sql(db_path, &sql).map(|_| ())
}

pub fn count_rows(db_path: &Path, table: &str) -> Result<i64, SqlitePersistenceError> {
    let sql = format!("SELECT COUNT(*) FROM {};", table);
    let out = run_sql(db_path, &sql)?;
    Ok(out.trim().parse::<i64>()?)
}

fn run_sql(db_path: &Path, sql: &str) -> Result<String, SqlitePersistenceError> {
    let output = Command::new("sqlite3").arg(db_path).arg(sql).output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(SqlitePersistenceError::SqliteFailed(err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn esc(value: &str) -> String {
    value.replace('"', "\"\"").replace('\'', "''")
}

fn opt_text(value: Option<&str>) -> String {
    value
        .map(|v| format!("'{}'", esc(v)))
        .unwrap_or_else(|| "NULL".to_string())
}

fn build_status_to_str(status: &BuildStatus) -> &'static str {
    match status {
        BuildStatus::NotRun => "not_run",
        BuildStatus::Running => "running",
        BuildStatus::Success => "success",
        BuildStatus::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BuildStatus;

    #[test]
    fn creates_schema_and_persists_dag_rows() {
        let db = std::env::temp_dir().join(format!(
            "snapbuild-persistence-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        init_schema(&db).unwrap();

        save_project(&db, "p1", "demo", "/tmp/demo", Some("main"), 1).unwrap();

        let snapshot = SnapshotMeta {
            id: "s1".to_string(),
            parent_id: None,
            name: "First".to_string(),
            created_at_unix_ms: 2,
            source_fingerprint: "srcfp".to_string(),
            dependency_fingerprint: "depfp".to_string(),
            git_branch: Some("main".to_string()),
            changed_files: vec!["src/main.ts".to_string()],
            favorite: true,
            tags: vec!["stable".to_string()],
            color: Some("green".to_string()),
            build_status: BuildStatus::Success,
        };
        save_snapshot(&db, "p1", &snapshot).unwrap();
        save_snapshot_edge(&db, "s1", "s1").unwrap();

        let run = BuildRun {
            id: "r1".to_string(),
            snapshot_id: "s1".to_string(),
            command: "cargo test".to_string(),
            started_at_unix_ms: 3,
            duration_ms: 10,
            success: true,
            stdout: "ok".to_string(),
            stderr: "".to_string(),
            metadata: None,
        };
        save_build_run(&db, &run).unwrap();

        assert_eq!(count_rows(&db, "projects").unwrap(), 1);
        assert_eq!(count_rows(&db, "snapshots").unwrap(), 1);
        assert_eq!(count_rows(&db, "snapshot_edges").unwrap(), 1);
        assert_eq!(count_rows(&db, "build_runs").unwrap(), 1);

        let _ = std::fs::remove_file(db);
    }
}
