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
}

/** Mirror of Rust `ProjectInfo` struct */
export interface ProjectInfo {
  path: string;
  name: string;
  store_root: string;
  snapshot_count: number;
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
