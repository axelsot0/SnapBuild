/** Mirror of Rust `SnapshotStatus` enum */
export type SnapshotStatus = "ok" | "failed" | "not_run" | "idle";

/** Mirror of Rust `SnapshotRecord` struct */
export interface SnapshotRecord {
  snapshot_id: string;
  created_at_unix_ms: number;
  file_count: number;
  total_size_bytes: number;
  parent_id: string | null;
  status: SnapshotStatus;
  git_branch: string | null;
  source_fingerprint: string | null;
  tags: string[];
  color: string | null;
}

/** Mirror of Rust `ProjectInfo` struct */
export interface ProjectInfo {
  path: string;
  name: string;
  store_root: string;
  snapshot_count: number;
  git_branch: string | null;
}

/** Mirror of Rust `BuildResult` struct */
export interface BuildResult {
  snapshot_id: string;
  command: string;
  exit_code: number;
  duration_ms: number;
  success: boolean;
}

/** Mirror of Rust `BuildChunk` struct (streamed event payload) */
export interface BuildChunk {
  snapshot_id: string;
  line: string;
  is_stderr: boolean;
}

/** Mirror of Rust `BuildRunRecord` struct */
export interface BuildRunRecord {
  id: string;
  snapshot_id: string;
  command: string;
  started_at_unix_ms: number;
  duration_ms: number;
  success: boolean;
}

/** Mirror of Rust `BuildStatusChanged` event payload */
export interface BuildStatusChanged {
  snapshot_id: string;
  status: string;
}

/** Mirror of Rust `FileEntry` struct */
export interface FileEntry {
  path: string;
  blob_hash: string;
  size_bytes: number;
}

/** Mirror of Rust `ModifiedFile` struct */
export interface ModifiedFile {
  path: string;
  old_hash: string;
  new_hash: string;
  old_size: number;
  new_size: number;
}

/** Mirror of Rust `DiffResult` struct */
export interface DiffResult {
  added: FileEntry[];
  removed: FileEntry[];
  modified: ModifiedFile[];
}

/** Mirror of Rust `TagInfo` struct */
export interface TagInfo {
  id: string;
  name: string;
  color: string | null;
}
