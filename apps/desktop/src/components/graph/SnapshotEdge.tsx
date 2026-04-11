import type { GraphEdge } from "../../lib/graphLayout";

interface SnapshotEdgeProps {
  edge: GraphEdge;
  highlighted: boolean;
}

/**
 * Cubic bezier SVG path connecting parent → child nodes.
 * The control points create a smooth horizontal-first curve.
 */
export function SnapshotEdge({ edge, highlighted }: SnapshotEdgeProps) {
  const dx = (edge.toX - edge.fromX) * 0.5;

  // Horizontal-first cubic bezier
  const d = [
    `M ${edge.fromX} ${edge.fromY}`,
    `C ${edge.fromX + dx} ${edge.fromY}`,
    `  ${edge.toX - dx} ${edge.toY}`,
    `  ${edge.toX} ${edge.toY}`,
  ].join(" ");

  return (
    <path
      d={d}
      className={`graph-edge${highlighted ? " graph-edge--highlighted" : ""}`}
    />
  );
}
