import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import Fuse from "fuse.js";
import "./CommandPalette.css";

import { useProjectStore } from "../../stores/projectStore";
import { useUiStore } from "../../stores/uiStore";
import { useGraphStore } from "../../stores/graphStore";
import { createSnapshot } from "../../tauri/commands";
import { shortId } from "../../lib/formatters";

// ─── Command registry ────────────────────────────────────────────────────────
interface PaletteEntry {
  id: string;
  icon: string;
  label: string;
  shortcut?: string;
  group: "command" | "snapshot";
  action: () => void;
}

export function CommandPalette() {
  const { commandPaletteOpen, closeCommandPalette, toggleInspector, toggleTerminal } = useUiStore();
  const { snapshots, appendSnapshot } = useProjectStore();
  const { setSelected } = useGraphStore();
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // ── Build command list ────────────────────────────────────────
  const commands = useMemo<PaletteEntry[]>(() => {
    const cmds: PaletteEntry[] = [
      {
        id: "cmd-new-snapshot",
        icon: "+",
        label: "New Snapshot",
        shortcut: "N",
        group: "command",
        action: async () => {
          try {
            const snap = await createSnapshot();
            appendSnapshot(snap);
            setSelected(snap.snapshot_id);
          } catch (_e) { /* handled elsewhere */ }
        },
      },
      {
        id: "cmd-toggle-inspector",
        icon: "⊞",
        label: "Toggle Inspector Panel",
        shortcut: "I",
        group: "command",
        action: toggleInspector,
      },
      {
        id: "cmd-toggle-terminal",
        icon: ">_",
        label: "Toggle Terminal Panel",
        shortcut: "`",
        group: "command",
        action: toggleTerminal,
      },
      {
        id: "cmd-fit-graph",
        icon: "⊡",
        label: "Fit Graph to View",
        shortcut: "F",
        group: "command",
        action: () => {
          // Dispatch keyboard event for the graph handler
          window.dispatchEvent(
            new KeyboardEvent("keydown", { key: "f", bubbles: true })
          );
        },
      },
    ];

    // Add snapshot entries
    for (const snap of snapshots.slice(0, 20)) {
      cmds.push({
        id: `snap-${snap.snapshot_id}`,
        icon: "●",
        label: `Go to ${shortId(snap.snapshot_id)}`,
        group: "snapshot",
        action: () => setSelected(snap.snapshot_id),
      });
    }

    return cmds;
  }, [snapshots, appendSnapshot, setSelected, toggleInspector, toggleTerminal]);

  // ── Fuzzy search ──────────────────────────────────────────────
  const fuse = useMemo(
    () =>
      new Fuse(commands, {
        keys: ["label", "id"],
        threshold: 0.4,
        includeScore: true,
      }),
    [commands]
  );

  const results = useMemo(() => {
    if (!query.trim()) return commands;
    return fuse.search(query).map((r) => r.item);
  }, [query, commands, fuse]);

  // Reset active index when results change
  useEffect(() => setActiveIndex(0), [results]);

  // ── Open / close ──────────────────────────────────────────────
  useEffect(() => {
    if (commandPaletteOpen) {
      setQuery("");
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [commandPaletteOpen]);

  // Global Ctrl+K handler
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        if (commandPaletteOpen) closeCommandPalette();
        else useUiStore.getState().openCommandPalette();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [commandPaletteOpen, closeCommandPalette]);

  // ── Execute ───────────────────────────────────────────────────
  const executeItem = useCallback(
    (item: PaletteEntry) => {
      closeCommandPalette();
      item.action();
    },
    [closeCommandPalette]
  );

  // ── Keyboard nav inside palette ───────────────────────────────
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        closeCommandPalette();
        return;
      }
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setActiveIndex((i) => Math.min(i + 1, results.length - 1));
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setActiveIndex((i) => Math.max(i - 1, 0));
        return;
      }
      if (e.key === "Enter" && results[activeIndex]) {
        e.preventDefault();
        executeItem(results[activeIndex]);
      }
    },
    [results, activeIndex, closeCommandPalette, executeItem]
  );

  if (!commandPaletteOpen) return null;

  // ── Group results ─────────────────────────────────────────────
  const commandItems = results.filter((r) => r.group === "command");
  const snapshotItems = results.filter((r) => r.group === "snapshot");

  let globalIdx = -1; // track index across groups for active highlight

  return (
    <div className="cmd-palette-overlay" onClick={closeCommandPalette}>
      <div className="cmd-palette" onClick={(e) => e.stopPropagation()}>
        {/* Input */}
        <div className="cmd-palette__input-wrap">
          <span className="cmd-palette__input-icon">&#x2318;</span>
          <input
            ref={inputRef}
            className="cmd-palette__input"
            type="text"
            placeholder="Type a command or snapshot…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            spellCheck={false}
          />
          <span className="cmd-palette__input-hint">
            <kbd>Esc</kbd> close
          </span>
        </div>

        {/* Results */}
        <div className="cmd-palette__results">
          {results.length === 0 ? (
            <div className="cmd-palette__empty">
              No results for "{query}"
            </div>
          ) : (
            <>
              {commandItems.length > 0 && (
                <>
                  <div className="cmd-palette__group">Commands</div>
                  {commandItems.map((item) => {
                    globalIdx++;
                    const idx = globalIdx;
                    return (
                      <div
                        key={item.id}
                        className={`cmd-palette__item${
                          idx === activeIndex ? " cmd-palette__item--active" : ""
                        }`}
                        onClick={() => executeItem(item)}
                        onMouseEnter={() => setActiveIndex(idx)}
                      >
                        <span className="cmd-palette__item-icon">{item.icon}</span>
                        <span className="cmd-palette__item-label">{item.label}</span>
                        {item.shortcut && (
                          <span className="cmd-palette__item-shortcut">
                            <kbd>{item.shortcut}</kbd>
                          </span>
                        )}
                      </div>
                    );
                  })}
                </>
              )}

              {snapshotItems.length > 0 && (
                <>
                  <div className="cmd-palette__group">Snapshots</div>
                  {snapshotItems.map((item) => {
                    globalIdx++;
                    const idx = globalIdx;
                    return (
                      <div
                        key={item.id}
                        className={`cmd-palette__item${
                          idx === activeIndex ? " cmd-palette__item--active" : ""
                        }`}
                        onClick={() => executeItem(item)}
                        onMouseEnter={() => setActiveIndex(idx)}
                      >
                        <span className="cmd-palette__item-icon">{item.icon}</span>
                        <span className="cmd-palette__item-label">{item.label}</span>
                      </div>
                    );
                  })}
                </>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
