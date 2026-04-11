import { create } from "zustand";

interface GraphState {
  // Viewport
  panX: number;
  panY: number;
  zoom: number;

  // Selection
  selectedSnapshotId: string | null;
  hoveredSnapshotId: string | null;

  // Actions
  setPan: (x: number, y: number) => void;
  setZoom: (zoom: number) => void;
  resetViewport: () => void;
  setSelected: (id: string | null) => void;
  setHovered: (id: string | null) => void;
}

const DEFAULT_ZOOM = 1;
const MIN_ZOOM = 0.3;
const MAX_ZOOM = 2.5;

export const useGraphStore = create<GraphState>((set) => ({
  panX: 0,
  panY: 0,
  zoom: DEFAULT_ZOOM,
  selectedSnapshotId: null,
  hoveredSnapshotId: null,

  setPan: (panX, panY) => set({ panX, panY }),

  setZoom: (zoom) =>
    set({ zoom: Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom)) }),

  resetViewport: () => set({ panX: 0, panY: 0, zoom: DEFAULT_ZOOM }),

  setSelected: (selectedSnapshotId) => set({ selectedSnapshotId }),

  setHovered: (hoveredSnapshotId) => set({ hoveredSnapshotId }),
}));

export { MIN_ZOOM, MAX_ZOOM };
