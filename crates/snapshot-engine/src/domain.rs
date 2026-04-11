#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    NotRun,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftState {
    pub id: String,
    pub project_id: String,
    pub source_fingerprint: String,
    pub started_at_unix_ms: u128,
    pub updated_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMeta {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub created_at_unix_ms: u128,
    pub source_fingerprint: String,
    pub dependency_fingerprint: String,
    pub git_branch: Option<String>,
    pub changed_files: Vec<String>,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub build_status: BuildStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRun {
    pub id: String,
    pub snapshot_id: String,
    pub command: String,
    pub started_at_unix_ms: u128,
    pub duration_ms: u128,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub metadata: Option<String>,
}
