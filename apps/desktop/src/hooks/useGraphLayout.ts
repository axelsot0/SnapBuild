import { useMemo } from "react";
import { computeLayout, type GraphLayout } from "../lib/graphLayout";
import type { SnapshotRecord } from "../tauri/types";

/**
 * Memoized hook that recomputes the DAG layout only when
 * the snapshot list actually changes.
 */
export function useGraphLayout(snapshots: SnapshotRecord[]): GraphLayout {
  return useMemo(() => computeLayout(snapshots), [snapshots]);
}
