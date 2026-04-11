import { create } from "zustand";

interface BreakpointState {
  // Set of snapshot IDs that are marked as breakpoints
  breakpointIds: Set<string>;

  // Actions
  toggle: (id: string) => void;
  add: (id: string) => void;
  remove: (id: string) => void;
  isBreakpoint: (id: string) => boolean;
  clear: () => void;
}

export const useBreakpointStore = create<BreakpointState>((set, get) => ({
  breakpointIds: new Set(),

  toggle: (id) =>
    set((state) => {
      const next = new Set(state.breakpointIds);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return { breakpointIds: next };
    }),

  add: (id) =>
    set((state) => {
      const next = new Set(state.breakpointIds);
      next.add(id);
      return { breakpointIds: next };
    }),

  remove: (id) =>
    set((state) => {
      const next = new Set(state.breakpointIds);
      next.delete(id);
      return { breakpointIds: next };
    }),

  isBreakpoint: (id) => get().breakpointIds.has(id),

  clear: () => set({ breakpointIds: new Set() }),
}));
