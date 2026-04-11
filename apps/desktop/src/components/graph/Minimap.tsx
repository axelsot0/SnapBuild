import type { GraphLayout } from "../../lib/graphLayout";

interface MinimapProps {
  layout: GraphLayout;
  panX: number;
  panY: number;
  zoom: number;
  viewportWidth: number;
  viewportHeight: number;
  selectedId: string | null;
}

const MINIMAP_W = 160;
const MINIMAP_H = 90;

export function Minimap({
  layout,
  panX,
  panY,
  zoom,
  viewportWidth,
  viewportHeight,
  selectedId,
}: MinimapProps) {
  if (layout.nodes.length === 0 || layout.width === 0) return null;

  // Scale factor to fit the entire graph into the minimap
  const scaleX = MINIMAP_W / layout.width;
  const scaleY = MINIMAP_H / layout.height;
  const scale = Math.min(scaleX, scaleY) * 0.9;

  // Offset to center graph in minimap
  const offsetX = (MINIMAP_W - layout.width * scale) / 2;
  const offsetY = (MINIMAP_H - layout.height * scale) / 2;

  // Viewport rectangle (what the user currently sees)
  const vpX = (-panX / zoom) * scale + offsetX;
  const vpY = (-panY / zoom) * scale + offsetY;
  const vpW = (viewportWidth / zoom) * scale;
  const vpH = (viewportHeight / zoom) * scale;

  return (
    <div className="graph-minimap">
      <svg width={MINIMAP_W} height={MINIMAP_H}>
        {/* Nodes as small rectangles */}
        {layout.nodes.map((n) => (
          <rect
            key={n.id}
            x={n.x * scale + offsetX}
            y={n.y * scale + offsetY}
            width={160 * scale}
            height={72 * scale}
            className={`graph-minimap__node ${
              n.id === selectedId
                ? "graph-minimap__node--selected"
                : "graph-minimap__node--default"
            }`}
          />
        ))}

        {/* Viewport indicator */}
        <rect
          x={vpX}
          y={vpY}
          width={vpW}
          height={vpH}
          className="graph-minimap__viewport"
        />
      </svg>
    </div>
  );
}
