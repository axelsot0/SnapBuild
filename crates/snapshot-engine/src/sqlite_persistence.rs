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

// ─── Read / Query functions ────────────────────────────────────────────────

pub fn list_snapshots(
    db_path: &Path,
    project_id: &str,
) -> Result<Vec<SnapshotMeta>, SqlitePersistenceError> {
    let sql = format!(
        "SELECT id,parent_id,name,created_at_unix_ms,source_fingerprint,dependency_fingerprint,git_branch,changed_files_json,favorite,color,build_status FROM snapshots WHERE project_id='{}' ORDER BY created_at_unix_ms DESC;",
        esc(project_id)
    );
    let out = run_sql(db_path, &sql)?;
    let mut results = Vec::new();
    for line in out.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(11, '|').collect();
        if cols.len() < 11 {
            continue;
        }
        let changed_files = parse_changed_files_json(cols[7]);
        results.push(SnapshotMeta {
            id: cols[0].to_string(),
            parent_id: none_if_empty(cols[1]),
            name: cols[2].to_string(),
            created_at_unix_ms: cols[3].parse::<u128>().unwrap_or(0),
            source_fingerprint: cols[4].to_string(),
            dependency_fingerprint: cols[5].to_string(),
            git_branch: none_if_empty(cols[6]),
            changed_files,
            favorite: cols[8] == "1",
            color: none_if_empty(cols[9]),
            build_status: str_to_build_status(cols[10]),
            tags: vec![],
        });
    }
    Ok(results)
}

pub fn get_snapshot(
    db_path: &Path,
    snapshot_id: &str,
) -> Result<Option<SnapshotMeta>, SqlitePersistenceError> {
    let sql = format!(
        "SELECT id,parent_id,name,created_at_unix_ms,source_fingerprint,dependency_fingerprint,git_branch,changed_files_json,favorite,color,build_status FROM snapshots WHERE id='{}';",
        esc(snapshot_id)
    );
    let out = run_sql(db_path, &sql)?;
    let line = out.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let cols: Vec<&str> = line.splitn(11, '|').collect();
    if cols.len() < 11 {
        return Ok(None);
    }
    let changed_files = parse_changed_files_json(cols[7]);
    Ok(Some(SnapshotMeta {
        id: cols[0].to_string(),
        parent_id: none_if_empty(cols[1]),
        name: cols[2].to_string(),
        created_at_unix_ms: cols[3].parse::<u128>().unwrap_or(0),
        source_fingerprint: cols[4].to_string(),
        dependency_fingerprint: cols[5].to_string(),
        git_branch: none_if_empty(cols[6]),
        changed_files,
        favorite: cols[8] == "1",
        color: none_if_empty(cols[9]),
        build_status: str_to_build_status(cols[10]),
        tags: vec![],
    }))
}

pub fn list_snapshot_edges(
    db_path: &Path,
    project_id: &str,
) -> Result<Vec<(String, String)>, SqlitePersistenceError> {
    let sql = format!(
        "SELECT e.parent_id,e.child_id FROM snapshot_edges e INNER JOIN snapshots s ON s.id=e.child_id WHERE s.project_id='{}';",
        esc(project_id)
    );
    let out = run_sql(db_path, &sql)?;
    let mut edges = Vec::new();
    for line in out.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(2, '|').collect();
        if cols.len() == 2 {
            edges.push((cols[0].to_string(), cols[1].to_string()));
        }
    }
    Ok(edges)
}

pub fn list_build_runs(
    db_path: &Path,
    snapshot_id: &str,
) -> Result<Vec<BuildRun>, SqlitePersistenceError> {
    let sql = format!(
        "SELECT id,snapshot_id,command,started_at_unix_ms,duration_ms,success,stdout,stderr,metadata FROM build_runs WHERE snapshot_id='{}' ORDER BY started_at_unix_ms DESC;",
        esc(snapshot_id)
    );
    let out = run_sql(db_path, &sql)?;
    let mut runs = Vec::new();
    for line in out.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(9, '|').collect();
        if cols.len() < 9 {
            continue;
        }
        runs.push(BuildRun {
            id: cols[0].to_string(),
            snapshot_id: cols[1].to_string(),
            command: cols[2].to_string(),
            started_at_unix_ms: cols[3].parse::<u128>().unwrap_or(0),
            duration_ms: cols[4].parse::<u128>().unwrap_or(0),
            success: cols[5] == "1",
            stdout: cols[6].to_string(),
            stderr: cols[7].to_string(),
            metadata: none_if_empty(cols[8]),
        });
    }
    Ok(runs)
}

pub fn get_latest_build_run(
    db_path: &Path,
    snapshot_id: &str,
) -> Result<Option<BuildRun>, SqlitePersistenceError> {
    let sql = format!(
        "SELECT id,snapshot_id,command,started_at_unix_ms,duration_ms,success,stdout,stderr,metadata FROM build_runs WHERE snapshot_id='{}' ORDER BY started_at_unix_ms DESC LIMIT 1;",
        esc(snapshot_id)
    );
    let out = run_sql(db_path, &sql)?;
    let line = out.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let cols: Vec<&str> = line.splitn(9, '|').collect();
    if cols.len() < 9 {
        return Ok(None);
    }
    Ok(Some(BuildRun {
        id: cols[0].to_string(),
        snapshot_id: cols[1].to_string(),
        command: cols[2].to_string(),
        started_at_unix_ms: cols[3].parse::<u128>().unwrap_or(0),
        duration_ms: cols[4].parse::<u128>().unwrap_or(0),
        success: cols[5] == "1",
        stdout: cols[6].to_string(),
        stderr: cols[7].to_string(),
        metadata: none_if_empty(cols[8]),
    }))
}

pub fn list_projects(
    db_path: &Path,
) -> Result<Vec<(String, String, String)>, SqlitePersistenceError> {
    let sql = "SELECT id,name,root_path FROM projects ORDER BY created_at_unix_ms DESC;";
    let out = run_sql(db_path, sql)?;
    let mut projects = Vec::new();
    for line in out.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(3, '|').collect();
        if cols.len() == 3 {
            projects.push((cols[0].to_string(), cols[1].to_string(), cols[2].to_string()));
        }
    }
    Ok(projects)
}

pub fn update_snapshot_status(
    db_path: &Path,
    snapshot_id: &str,
    status: &str,
) -> Result<(), SqlitePersistenceError> {
    let sql = format!(
        "UPDATE snapshots SET build_status='{}' WHERE id='{}';",
        esc(status),
        esc(snapshot_id)
    );
    run_sql(db_path, &sql).map(|_| ())
}

pub fn list_snapshot_tags(
    db_path: &Path,
    snapshot_id: &str,
) -> Result<Vec<String>, SqlitePersistenceError> {
    let sql = format!(
        "SELECT t.name FROM tags t INNER JOIN snapshot_tags st ON st.tag_id=t.id WHERE st.snapshot_id='{}';",
        esc(snapshot_id)
    );
    let out = run_sql(db_path, &sql)?;
    Ok(out.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect())
}

pub fn add_tag(
    db_path: &Path,
    snapshot_id: &str,
    tag_name: &str,
    tag_color: Option<&str>,
) -> Result<(), SqlitePersistenceError> {
    let tag_id = format!("tag-{}", tag_name.to_lowercase().replace(' ', "-"));
    let sql = format!(
        "INSERT OR IGNORE INTO tags (id,name,color) VALUES ('{}','{}',{});\nINSERT OR IGNORE INTO snapshot_tags (snapshot_id,tag_id) VALUES ('{}','{}');",
        esc(&tag_id),
        esc(tag_name),
        opt_text(tag_color),
        esc(snapshot_id),
        esc(&tag_id)
    );
    run_sql(db_path, &sql).map(|_| ())
}

pub fn remove_tag(
    db_path: &Path,
    snapshot_id: &str,
    tag_name: &str,
) -> Result<(), SqlitePersistenceError> {
    let sql = format!(
        "DELETE FROM snapshot_tags WHERE snapshot_id='{}' AND tag_id IN (SELECT id FROM tags WHERE name='{}');",
        esc(snapshot_id),
        esc(tag_name)
    );
    run_sql(db_path, &sql).map(|_| ())
}

pub fn set_snapshot_color(
    db_path: &Path,
    snapshot_id: &str,
    color: Option<&str>,
) -> Result<(), SqlitePersistenceError> {
    let sql = format!(
        "UPDATE snapshots SET color={} WHERE id='{}';",
        opt_text(color),
        esc(snapshot_id)
    );
    run_sql(db_path, &sql).map(|_| ())
}

pub fn list_all_tags(
    db_path: &Path,
) -> Result<Vec<(String, String, Option<String>)>, SqlitePersistenceError> {
    let sql = "SELECT id,name,color FROM tags ORDER BY name;";
    let out = run_sql(db_path, sql)?;
    let mut tags = Vec::new();
    for line in out.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(3, '|').collect();
        if cols.len() >= 2 {
            tags.push((
                cols[0].to_string(),
                cols[1].to_string(),
                cols.get(2).and_then(|c| none_if_empty(c)),
            ));
        }
    }
    Ok(tags)
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

fn str_to_build_status(s: &str) -> BuildStatus {
    match s {
        "running" => BuildStatus::Running,
        "success" => BuildStatus::Success,
        "failed" => BuildStatus::Failed,
        _ => BuildStatus::NotRun,
    }
}

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() || s == "NULL" {
        None
    } else {
        Some(s.to_string())
    }
}

fn parse_changed_files_json(json: &str) -> Vec<String> {
    // Simple JSON array parser for ["file1","file2"] format
    let trimmed = json.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() {
        return vec![];
    }
    trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
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
