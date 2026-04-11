import { invoke } from "@tauri-apps/api/core";
import type {
  ProjectInfo,
  SnapshotRecord,
  BuildResult,
} from "./types";

// ─── Project ────────────────────────────────────────────────────────────────

/** Open a project folder. Returns project info. */
export async function openProject(path: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>("open_project", { path });
}

/** Get info about the currently open project. Returns null if none open. */
export async function getProjectInfo(): Promise<ProjectInfo | null> {
  return invoke<ProjectInfo | null>("get_project_info");
}

// ─── Snapshots ──────────────────────────────────────────────────────────────

/** Create a new snapshot. `label` is optional custom ID. */
export async function createSnapshot(
  label?: string
): Promise<SnapshotRecord> {
  return invoke<SnapshotRecord>("create_snapshot", { label: label ?? null });
}

/** List all snapshots for the open project, newest first. */
export async function listSnapshots(): Promise<SnapshotRecord[]> {
  return invoke<SnapshotRecord[]>("list_snapshots");
}

/** Permanently delete a snapshot manifest. */
export async function deleteSnapshot(snapshotId: string): Promise<void> {
  return invoke<void>("delete_snapshot", { snapshotId });
}

/** Restore a snapshot back into the project workspace. */
export async function restoreSnapshot(snapshotId: string): Promise<void> {
  return invoke<void>("restore_snapshot", { snapshotId });
}

/** Materialize a snapshot into a temp directory. Returns the temp path. */
export async function materializeSnapshot(
  snapshotId: string
): Promise<string> {
  return invoke<string>("materialize_snapshot", { snapshotId });
}

// ─── Windows ────────────────────────────────────────────────────────────────

/** Open a new detached window. Types: "graph", "build", "compare" */
export async function openWindow(
  windowType: string,
  labelSuffix?: string
): Promise<void> {
  return invoke<void>("open_window", {
    windowType,
    labelSuffix: labelSuffix ?? null,
  });
}

// ─── Watcher ────────────────────────────────────────────────────────────────

/** Start watching the project directory for file changes. */
export async function startWatcher(): Promise<void> {
  return invoke<void>("start_watcher");
}

/** Stop the file watcher. */
export async function stopWatcher(): Promise<void> {
  return invoke<void>("stop_watcher");
}

// ─── Build runner ───────────────────────────────────────────────────────────

/**
 * Materialize the snapshot and run `command args` inside it.
 * Streams stdout/stderr as `snapbuild://build-output-chunk` events.
 * Emits `snapbuild://build-complete` on exit.
 */
export async function runSnapshotBuild(
  snapshotId: string,
  command: string,
  args: string[] = []
): Promise<BuildResult> {
  return invoke<BuildResult>("run_snapshot_build", {
    snapshotId,
    command,
    args,
  });
}
