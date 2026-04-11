import { useEffect, useState, useCallback } from "react";
import { startWatcher } from "../tauri/commands";
import { onProjectChanged } from "../tauri/events";
import { useProjectStore } from "../stores/projectStore";

/**
 * Starts the Rust file watcher when a project is open.
 * Returns `hasChanges` (true after a file change is detected,
 * reset when a new snapshot is created).
 */
export function useFileWatcher() {
  const projectInfo = useProjectStore((s) => s.projectInfo);
  const snapshotCount = useProjectStore((s) => s.snapshots.length);
  const [hasChanges, setHasChanges] = useState(false);

  // Start watcher when project opens
  useEffect(() => {
    if (!projectInfo) return;
    startWatcher().catch(() => {}); // fire-and-forget
  }, [projectInfo]);

  // Listen for change events
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | null = null;

    onProjectChanged(() => {
      if (active) setHasChanges(true);
    }).then((u) => {
      if (!active) u();
      else unlisten = u;
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  // Reset hasChanges when a new snapshot is created
  useEffect(() => {
    setHasChanges(false);
  }, [snapshotCount]);

  const clearChanges = useCallback(() => setHasChanges(false), []);

  return { hasChanges, clearChanges };
}
