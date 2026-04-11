import { create } from "zustand";
import type { BuildResult } from "../tauri/types";

const MAX_BLOCKS = 50;

export interface TerminalLine {
  text: string;
  isStderr: boolean;
}

export interface TerminalBlock {
  id: string; // snapshot_id
  command: string;
  lines: TerminalLine[];
  result: BuildResult | null; // null = still running
  collapsed: boolean;
  startedAt: number; // Date.now()
}

interface TerminalState {
  blocks: TerminalBlock[];

  // Actions
  startBlock: (snapshotId: string, command: string) => void;
  appendLine: (snapshotId: string, line: string, isStderr: boolean) => void;
  finishBlock: (result: BuildResult) => void;
  toggleCollapsed: (id: string) => void;
  clearAll: () => void;
}

export const useTerminalStore = create<TerminalState>((set) => ({
  blocks: [],

  startBlock: (id, command) =>
    set((state) => {
      const newBlock: TerminalBlock = {
        id,
        command,
        lines: [],
        result: null,
        collapsed: false,
        startedAt: Date.now(),
      };
      const blocks = [newBlock, ...state.blocks].slice(0, MAX_BLOCKS);
      return { blocks };
    }),

  appendLine: (snapshotId, line, isStderr) =>
    set((state) => ({
      blocks: state.blocks.map((b) =>
        b.id === snapshotId
          ? { ...b, lines: [...b.lines, { text: line, isStderr }] }
          : b
      ),
    })),

  finishBlock: (result) =>
    set((state) => ({
      blocks: state.blocks.map((b) =>
        b.id === result.snapshot_id ? { ...b, result } : b
      ),
    })),

  toggleCollapsed: (id) =>
    set((state) => ({
      blocks: state.blocks.map((b) =>
        b.id === id ? { ...b, collapsed: !b.collapsed } : b
      ),
    })),

  clearAll: () => set({ blocks: [] }),
}));
