import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BuildChunk, BuildResult } from "./types";

// ─── Event names ────────────────────────────────────────────────────────────

export const EVENTS = {
  SNAPSHOT_LIST_CHANGED: "snapbuild://snapshot-list-changed",
  BUILD_OUTPUT_CHUNK: "snapbuild://build-output-chunk",
  BUILD_COMPLETE: "snapbuild://build-complete",
  PROJECT_CHANGED: "snapbuild://project-changed",
} as const;

// ─── Typed listeners ────────────────────────────────────────────────────────

/** Called whenever the snapshot list changes (create / delete / restore). */
export function onSnapshotListChanged(
  cb: () => void
): Promise<UnlistenFn> {
  return listen(EVENTS.SNAPSHOT_LIST_CHANGED, cb);
}

/** Called for each stdout/stderr line emitted during a build run. */
export function onBuildOutputChunk(
  cb: (chunk: BuildChunk) => void
): Promise<UnlistenFn> {
  return listen<BuildChunk>(EVENTS.BUILD_OUTPUT_CHUNK, (e) => cb(e.payload));
}

/** Called when a build process exits. */
export function onBuildComplete(
  cb: (result: BuildResult) => void
): Promise<UnlistenFn> {
  return listen<BuildResult>(EVENTS.BUILD_COMPLETE, (e) => cb(e.payload));
}

/** Called when the file watcher detects workspace changes. */
export function onProjectChanged(
  cb: () => void
): Promise<UnlistenFn> {
  return listen(EVENTS.PROJECT_CHANGED, cb);
}
