import { invoke } from "@tauri-apps/api/core";
import type {
  ProjectInfo,
  SnapshotRecord,
  BuildResult,
  BuildRunRecord,
  FileEntry,
  DiffResult,
  TagInfo,
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

/** Get the current git branch of the open project. */
export async function getGitBranch(): Promise<string | null> {
  return invoke<string | null>("get_git_branch");
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

/** List all files in a snapshot manifest. */
export async function listSnapshotFiles(
  snapshotId: string
): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("list_snapshot_files", { snapshotId });
}

/** Compare two snapshots, returning added/removed/modified files. */
export async function compareSnapshots(
  idA: string,
  idB: string
): Promise<DiffResult> {
  return invoke<DiffResult>("compare_snapshots", { idA, idB });
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

/** Enable or disable auto-snapshot on file change. */
export async function toggleAutoSnapshot(enabled: boolean): Promise<void> {
  return invoke<void>("toggle_auto_snapshot", { enabled });
}

/** Get current auto-snapshot enabled state. */
export async function getAutoSnapshotEnabled(): Promise<boolean> {
  return invoke<boolean>("get_auto_snapshot_enabled");
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

/** Get build history for a snapshot. */
export async function getBuildHistory(
  snapshotId: string
): Promise<BuildRunRecord[]> {
  return invoke<BuildRunRecord[]>("get_build_history", { snapshotId });
}

// ─── Tags & Colors ──────────────────────────────────────────────────────────

/** Add a tag to a snapshot. */
export async function addTag(
  snapshotId: string,
  tagName: string,
  color?: string
): Promise<void> {
  return invoke<void>("add_tag", {
    snapshotId,
    tagName,
    color: color ?? null,
  });
}

/** Remove a tag from a snapshot. */
export async function removeTag(
  snapshotId: string,
  tagName: string
): Promise<void> {
  return invoke<void>("remove_tag", { snapshotId, tagName });
}

/** Set the color of a snapshot node. */
export async function setSnapshotColor(
  snapshotId: string,
  color: string | null
): Promise<void> {
  return invoke<void>("set_snapshot_color", { snapshotId, color });
}

/** List all tags in the project. */
export async function listTags(): Promise<TagInfo[]> {
  return invoke<TagInfo[]>("list_tags");
}
