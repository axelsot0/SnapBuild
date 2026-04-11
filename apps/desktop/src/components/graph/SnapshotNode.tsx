import type { GraphNode } from "../../lib/graphLayout";
import { shortId, timeAgo } from "../../lib/formatters";
import { useBreakpointStore } from "../../stores/breakpointStore";

interface SnapshotNodeProps {
  node: GraphNode;
  isSelected: boolean;
  isHovered: boolean;
  onSelect: () => void;
  onHover: (hovered: boolean) => void;
}

export function SnapshotNode({
  node,
  isSelected,
  isHovered: _isHovered,
  onSelect,
  onHover,
}: SnapshotNodeProps) {
  const { isBreakpoint } = useBreakpointStore();
  const bp = isBreakpoint(node.id);
  const snap = node.snapshot;

  const cardClass = [
    "graph-node__card",
    isSelected ? "graph-node__card--selected" : "",
    bp ? "graph-node__card--breakpoint" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <foreignObject
      x={node.x}
      y={node.y}
      width={160}
      height={72}
      className="graph-node"
    >
      <div
        className={cardClass}
        onClick={(e) => {
          e.stopPropagation();
          onSelect();
        }}
        onMouseEnter={() => onHover(true)}
        onMouseLeave={() => onHover(false)}
      >
        {/* Top row */}
        <div className="graph-node__top">
          {bp ? (
            <span className="graph-node__bp-diamond">◆</span>
          ) : (
            <span className={`graph-node__dot graph-node__dot--${snap.status}`} />
          )}
          <span className="graph-node__id">{shortId(snap.snapshot_id)}</span>
        </div>

        {/* Meta row */}
        <div className="graph-node__meta">
          <span className="graph-node__meta-time">
            {timeAgo(snap.created_at_unix_ms)}
          </span>
          <span>{snap.file_count} files</span>
        </div>

        {/* Hover actions */}
        <div className="graph-node__actions">
          <button className="graph-node__action-btn" title="Run build">
            ▶
          </button>
          <button className="graph-node__action-btn" title="Restore">
            ↩
          </button>
        </div>
      </div>
    </foreignObject>
  );
}
