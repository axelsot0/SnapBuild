import { create } from "zustand";

type FilterChip = "all" | "ok" | "failed" | "not_run" | "breakpoints";

interface UiState {
  // Panel visibility
  inspectorVisible: boolean;
  terminalVisible: boolean;
  commandPaletteOpen: boolean;

  // Sidebar
  activeFilter: FilterChip;
  sidebarSearchQuery: string;

  // Actions
  toggleInspector: () => void;
  toggleTerminal: () => void;
  setInspectorVisible: (v: boolean) => void;
  setTerminalVisible: (v: boolean) => void;
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  toggleCommandPalette: () => void;
  setFilter: (filter: FilterChip) => void;
  setSidebarSearch: (q: string) => void;
}

export const useUiStore = create<UiState>((set) => ({
  inspectorVisible: true,
  terminalVisible: true,
  commandPaletteOpen: false,
  activeFilter: "all",
  sidebarSearchQuery: "",

  toggleInspector: () =>
    set((state) => ({ inspectorVisible: !state.inspectorVisible })),

  toggleTerminal: () =>
    set((state) => ({ terminalVisible: !state.terminalVisible })),

  setInspectorVisible: (inspectorVisible) => set({ inspectorVisible }),

  setTerminalVisible: (terminalVisible) => set({ terminalVisible }),

  openCommandPalette: () => set({ commandPaletteOpen: true }),

  closeCommandPalette: () => set({ commandPaletteOpen: false }),

  toggleCommandPalette: () =>
    set((state) => ({ commandPaletteOpen: !state.commandPaletteOpen })),

  setFilter: (activeFilter) => set({ activeFilter }),

  setSidebarSearch: (sidebarSearchQuery) => set({ sidebarSearchQuery }),
}));
