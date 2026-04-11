import { useRef, useCallback } from "react";
import "./GraphCanvas.css";

import { useProjectStore } from "../../stores/projectStore";
import { useGraphStore } from "../../stores/graphStore";
import { useGraphLayout } from "../../hooks/useGraphLayout";
import { useGraphInteraction } from "../../hooks/useGraphInteraction";
import { SnapshotNode } from "./SnapshotNode";
import { SnapshotEdge } from "./SnapshotEdge";
import { Minimap } from "./Minimap";

export function GraphCanvas() {
  const { projectInfo, snapshots } = useProjectStore();
  const { panX, panY, zoom, selectedSnapshotId, hoveredSnapshotId, setSelected, setHovered } =
    useGraphStore();

  const layout = useGraphLayout(snapshots);
  const svgRef = useRef<SVGSVGElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const { handleMouseDown, handleMouseMove, handleMouseUp, handleWheel } =
    useGraphInteraction(svgRef, layout.nodes);

  // IDs connected to the hovered node (for edge highlighting)
  const hoveredEdgeIds = new Set<string>();
  if (hoveredSnapshotId) {
    for (const e of layout.edges) {
      if (e.from === hoveredSnapshotId || e.to === hoveredSnapshotId) {
        hoveredEdgeIds.add(`${e.from}-${e.to}`);
      }
    }
  }

  // Get container dimensions for minimap viewport calculation
  const getContainerSize = useCallback(() => {
    if (containerRef.current) {
      return {
        width: containerRef.current.clientWidth,
        height: containerRef.current.clientHeight,
      };
    }
    return { width: 800, height: 600 };
  }, []);

  // ─── Empty states ─────────────────────────────────────────────
  if (!projectInfo) {
    return (
      <div className="graph-empty">
        <div className="graph-empty__icon">&#x2B21;</div>
        <div className="graph-empty__title">Version Graph</div>
        <div className="graph-empty__hint">
          Open a project using the sidebar to begin
        </div>
      </div>
    );
  }

  if (snapshots.length === 0) {
    return (
      <div className="graph-empty">
        <div className="graph-empty__title">{projectInfo.name}</div>
        <div className="graph-empty__hint">
          Press <kbd className="graph-empty__kbd">N</kbd> or click{" "}
          <strong style={{ color: "var(--color-text-primary)" }}>
            New Snapshot
          </strong>{" "}
          to start
        </div>
      </div>
    );
  }

  // ─── Graph ────────────────────────────────────────────────────
  const containerSize = getContainerSize();

  return (
    <div className="graph-canvas" ref={containerRef}>
      <svg
        ref={svgRef}
        width="100%"
        height="100%"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onWheel={handleWheel}
      >
        {/* Dot grid pattern */}
        <defs>
          <pattern
            id="dot-grid"
            width={24}
            height={24}
            patternUnits="userSpaceOnUse"
            patternTransform={`translate(${panX % (24 * zoom)}, ${panY % (24 * zoom)}) scale(${zoom})`}
          >
            <circle cx={12} cy={12} r={0.7} className="graph-canvas__dots" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#dot-grid)" />

        {/* Transformed group for pan & zoom */}
        <g transform={`translate(${panX}, ${panY}) scale(${zoom})`}>
          {/* Edges (render behind nodes) */}
          {layout.edges.map((e) => (
            <SnapshotEdge
              key={`${e.from}-${e.to}`}
              edge={e}
              highlighted={hoveredEdgeIds.has(`${e.from}-${e.to}`)}
            />
          ))}

          {/* Nodes */}
          {layout.nodes.map((n) => (
            <SnapshotNode
              key={n.id}
              node={n}
              isSelected={selectedSnapshotId === n.id}
              isHovered={hoveredSnapshotId === n.id}
              onSelect={() => setSelected(n.id)}
              onHover={(h) => setHovered(h ? n.id : null)}
            />
          ))}
        </g>
      </svg>

      {/* Minimap */}
      <Minimap
        layout={layout}
        panX={panX}
        panY={panY}
        zoom={zoom}
        viewportWidth={containerSize.width}
        viewportHeight={containerSize.height}
        selectedId={selectedSnapshotId}
      />
    </div>
  );
}
