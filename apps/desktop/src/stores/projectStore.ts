import { create } from "zustand";
import type { ProjectInfo, SnapshotRecord } from "../tauri/types";

interface ProjectState {
  // Data
  projectInfo: ProjectInfo | null;
  snapshots: SnapshotRecord[];
  isLoading: boolean;
  error: string | null;

  // Actions
  setProject: (info: ProjectInfo) => void;
  setSnapshots: (snapshots: SnapshotRecord[]) => void;
  appendSnapshot: (snapshot: SnapshotRecord) => void;
  removeSnapshot: (id: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

export const useProjectStore = create<ProjectState>((set) => ({
  projectInfo: null,
  snapshots: [],
  isLoading: false,
  error: null,

  setProject: (info) => set({ projectInfo: info, error: null }),

  setSnapshots: (snapshots) => set({ snapshots }),

  appendSnapshot: (snapshot) =>
    set((state) => ({
      snapshots: [snapshot, ...state.snapshots],
    })),

  removeSnapshot: (id) =>
    set((state) => ({
      snapshots: state.snapshots.filter((s) => s.snapshot_id !== id),
    })),

  setLoading: (isLoading) => set({ isLoading }),

  setError: (error) => set({ error }),

  reset: () =>
    set({
      projectInfo: null,
      snapshots: [],
      isLoading: false,
      error: null,
    }),
}));
