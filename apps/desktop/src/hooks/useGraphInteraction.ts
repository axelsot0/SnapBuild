import { useCallback, useRef, useEffect } from "react";
import { useGraphStore, MIN_ZOOM, MAX_ZOOM } from "../stores/graphStore";
import type { GraphNode } from "../lib/graphLayout";

const ZOOM_FACTOR = 0.08;

/**
 * Handles pan (mouse drag), zoom (scroll wheel) and keyboard navigation
 * for the graph canvas.
 */
export function useGraphInteraction(
  svgRef: React.RefObject<SVGSVGElement | null>,
  nodes: GraphNode[]
) {
  const { panX, panY, zoom, setPan, setZoom, selectedSnapshotId, setSelected } =
    useGraphStore();

  const isPanning = useRef(false);
  const panStartMouse = useRef({ x: 0, y: 0 });
  const panStartOffset = useRef({ x: 0, y: 0 });

  // ── Mouse pan ─────────────────────────────────────────────────
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      // Only pan on left-click on the SVG background
      if (e.button !== 0) return;
      if ((e.target as HTMLElement).closest(".graph-node")) return;

      isPanning.current = true;
      panStartMouse.current = { x: e.clientX, y: e.clientY };
      panStartOffset.current = { x: panX, y: panY };
      (e.target as HTMLElement).style.cursor = "grabbing";
    },
    [panX, panY]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isPanning.current) return;
      const dx = e.clientX - panStartMouse.current.x;
      const dy = e.clientY - panStartMouse.current.y;
      setPan(panStartOffset.current.x + dx, panStartOffset.current.y + dy);
    },
    [setPan]
  );

  const handleMouseUp = useCallback(
    (e: React.MouseEvent) => {
      isPanning.current = false;
      (e.target as HTMLElement).style.cursor = "";
    },
    []
  );

  // ── Scroll zoom ───────────────────────────────────────────────
  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault();
      const delta = e.deltaY > 0 ? -ZOOM_FACTOR : ZOOM_FACTOR;
      const newZoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom + delta));

      // Zoom toward cursor position
      if (svgRef.current) {
        const rect = svgRef.current.getBoundingClientRect();
        const cx = e.clientX - rect.left;
        const cy = e.clientY - rect.top;
        const scale = newZoom / zoom;
        const newPanX = cx - scale * (cx - panX);
        const newPanY = cy - scale * (cy - panY);
        setPan(newPanX, newPanY);
      }

      setZoom(newZoom);
    },
    [zoom, panX, panY, setPan, setZoom, svgRef]
  );

  // ── Keyboard navigation ───────────────────────────────────────
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      )
        return;

      // Arrow keys navigate between nodes
      if (
        ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)
      ) {
        e.preventDefault();
        if (nodes.length === 0) return;

        const current = nodes.find((n) => n.id === selectedSnapshotId);
        if (!current) {
          setSelected(nodes[0].id);
          return;
        }

        let target: GraphNode | undefined;

        switch (e.key) {
          case "ArrowRight":
            // Next column, same lane (or closest)
            target = nodes
              .filter((n) => n.column > current.column)
              .sort(
                (a, b) =>
                  a.column - b.column ||
                  Math.abs(a.lane - current.lane) -
                    Math.abs(b.lane - current.lane)
              )[0];
            break;
          case "ArrowLeft":
            target = nodes
              .filter((n) => n.column < current.column)
              .sort(
                (a, b) =>
                  b.column - a.column ||
                  Math.abs(a.lane - current.lane) -
                    Math.abs(b.lane - current.lane)
              )[0];
            break;
          case "ArrowDown":
            target = nodes
              .filter((n) => n.lane > current.lane)
              .sort(
                (a, b) =>
                  a.lane - b.lane ||
                  Math.abs(a.column - current.column) -
                    Math.abs(b.column - current.column)
              )[0];
            break;
          case "ArrowUp":
            target = nodes
              .filter((n) => n.lane < current.lane)
              .sort(
                (a, b) =>
                  b.lane - a.lane ||
                  Math.abs(a.column - current.column) -
                    Math.abs(b.column - current.column)
              )[0];
            break;
        }

        if (target) setSelected(target.id);
      }

      // Space → center on selected node
      if (e.key === " " && selectedSnapshotId) {
        e.preventDefault();
        const node = nodes.find((n) => n.id === selectedSnapshotId);
        if (node && svgRef.current) {
          const rect = svgRef.current.getBoundingClientRect();
          setPan(
            rect.width / 2 - (node.x + 80) * zoom,
            rect.height / 2 - (node.y + 36) * zoom
          );
        }
      }

      // F → fit all nodes
      if (e.key === "f" && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        fitAll();
      }
    };

    const fitAll = () => {
      if (nodes.length === 0 || !svgRef.current) return;
      const rect = svgRef.current.getBoundingClientRect();

      const minX = Math.min(...nodes.map((n) => n.x));
      const minY = Math.min(...nodes.map((n) => n.y));
      const maxX = Math.max(...nodes.map((n) => n.x + 160));
      const maxY = Math.max(...nodes.map((n) => n.y + 72));

      const contentW = maxX - minX + 80;
      const contentH = maxY - minY + 80;

      const scaleX = rect.width / contentW;
      const scaleY = rect.height / contentH;
      const newZoom = Math.min(Math.max(Math.min(scaleX, scaleY), MIN_ZOOM), MAX_ZOOM);

      setPan(
        (rect.width - contentW * newZoom) / 2 - minX * newZoom + 40 * newZoom,
        (rect.height - contentH * newZoom) / 2 - minY * newZoom + 40 * newZoom
      );
      setZoom(newZoom);
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [nodes, selectedSnapshotId, setSelected, setPan, setZoom, zoom, svgRef]);

  return {
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    handleWheel,
  };
}
