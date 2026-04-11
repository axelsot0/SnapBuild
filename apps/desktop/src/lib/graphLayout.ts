/**
 * Sugiyama-inspired DAG layout for the version graph.
 *
 * Input:  flat list of SnapshotRecord (each with optional parent_id)
 * Output: positioned nodes + edges ready for SVG rendering
 *
 * Axis X = time (left → right), Y = parallel branches (forks create new lanes)
 */

import type { SnapshotRecord } from "../tauri/types";

// ─── Types ───────────────────────────────────────────────────────────────────

export interface GraphNode {
  id: string;
  snapshot: SnapshotRecord;
  x: number;
  y: number;
  column: number; // generation index (left → right)
  lane: number;   // vertical lane
}

export interface GraphEdge {
  from: string;
  to: string;
  fromX: number;
  fromY: number;
  toX: number;
  toY: number;
}

export interface GraphLayout {
  nodes: GraphNode[];
  edges: GraphEdge[];
  width: number;
  height: number;
}

// ─── Config ──────────────────────────────────────────────────────────────────

export const NODE_WIDTH = 160;
export const NODE_HEIGHT = 72;
export const COL_GAP = 60;   // horizontal gap between columns
export const LANE_GAP = 40;  // vertical gap between lanes
const PADDING = 40;

// ─── Algorithm ───────────────────────────────────────────────────────────────

export function computeLayout(snapshots: SnapshotRecord[]): GraphLayout {
  if (snapshots.length === 0) {
    return { nodes: [], edges: [], width: 0, height: 0 };
  }

  // 1. Build adjacency: child → parent, parent → children
  const byId = new Map<string, SnapshotRecord>();
  const childrenOf = new Map<string, string[]>(); // parent → [children]

  for (const s of snapshots) {
    byId.set(s.snapshot_id, s);
    if (s.parent_id) {
      const kids = childrenOf.get(s.parent_id) ?? [];
      kids.push(s.snapshot_id);
      childrenOf.set(s.parent_id, kids);
    }
  }

  // 2. Find roots (nodes with no parent or whose parent isn't in the set)
  const roots = snapshots.filter(
    (s) => !s.parent_id || !byId.has(s.parent_id)
  );

  // Sort roots by creation time (oldest first → leftmost)
  roots.sort((a, b) => a.created_at_unix_ms - b.created_at_unix_ms);

  // 3. BFS to assign columns (generation) and lanes
  const columnMap = new Map<string, number>();
  const laneMap = new Map<string, number>();
  let nextLane = 0;

  // Process each root as its own lane seed
  const queue: { id: string; col: number; lane: number }[] = [];

  for (const root of roots) {
    queue.push({ id: root.snapshot_id, col: 0, lane: nextLane });
    nextLane++;
  }

  // Also find orphans not reachable from roots
  const visited = new Set<string>();

  while (queue.length > 0) {
    const { id, col, lane } = queue.shift()!;
    if (visited.has(id)) continue;
    visited.add(id);

    columnMap.set(id, col);
    laneMap.set(id, lane);

    const children = childrenOf.get(id) ?? [];
    // Sort children by creation time
    children.sort((a, b) => {
      const sa = byId.get(a)!;
      const sb = byId.get(b)!;
      return sa.created_at_unix_ms - sb.created_at_unix_ms;
    });

    for (let i = 0; i < children.length; i++) {
      if (visited.has(children[i])) continue;
      if (i === 0) {
        // First child continues the same lane
        queue.push({ id: children[i], col: col + 1, lane });
      } else {
        // Subsequent children fork into new lanes
        queue.push({ id: children[i], col: col + 1, lane: nextLane });
        nextLane++;
      }
    }
  }

  // Handle any unvisited nodes (disconnected)
  for (const s of snapshots) {
    if (!visited.has(s.snapshot_id)) {
      columnMap.set(s.snapshot_id, 0);
      laneMap.set(s.snapshot_id, nextLane);
      nextLane++;
    }
  }

  // 4. Compute pixel positions
  const nodes: GraphNode[] = [];
  const nodeMap = new Map<string, GraphNode>();

  for (const s of snapshots) {
    const col = columnMap.get(s.snapshot_id) ?? 0;
    const lane = laneMap.get(s.snapshot_id) ?? 0;

    const x = PADDING + col * (NODE_WIDTH + COL_GAP);
    const y = PADDING + lane * (NODE_HEIGHT + LANE_GAP);

    const node: GraphNode = {
      id: s.snapshot_id,
      snapshot: s,
      x,
      y,
      column: col,
      lane,
    };

    nodes.push(node);
    nodeMap.set(s.snapshot_id, node);
  }

  // 5. Generate edges
  const edges: GraphEdge[] = [];

  for (const s of snapshots) {
    if (s.parent_id && nodeMap.has(s.parent_id)) {
      const parent = nodeMap.get(s.parent_id)!;
      const child = nodeMap.get(s.snapshot_id)!;

      edges.push({
        from: parent.id,
        to: child.id,
        fromX: parent.x + NODE_WIDTH,
        fromY: parent.y + NODE_HEIGHT / 2,
        toX: child.x,
        toY: child.y + NODE_HEIGHT / 2,
      });
    }
  }

  // 6. Compute total dimensions
  const maxX = Math.max(...nodes.map((n) => n.x + NODE_WIDTH), 0);
  const maxY = Math.max(...nodes.map((n) => n.y + NODE_HEIGHT), 0);

  return {
    nodes,
    edges,
    width: maxX + PADDING,
    height: maxY + PADDING,
  };
}
