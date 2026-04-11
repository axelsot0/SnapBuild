import { useEffect, useCallback, useRef } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { motion, AnimatePresence } from "framer-motion";
import "./Sidebar.css";

import { openProject, createSnapshot, listSnapshots, deleteSnapshot } from "../../tauri/commands";
import { onSnapshotListChanged } from "../../tauri/events";
import { useProjectStore } from "../../stores/projectStore";
import { useGraphStore } from "../../stores/graphStore";
import { useUiStore } from "../../stores/uiStore";
import { useBreakpointStore } from "../../stores/breakpointStore";
import { shortId, timeAgo, formatBytes } from "../../lib/formatters";
import type { SnapshotRecord } from "../../tauri/types";

// ─── Filter chips config ─────────────────────────────────────────────────────
const FILTERS = [
  { id: "all",         label: "All" },
  { id: "ok",          label: "OK" },
  { id: "failed",      label: "Failed" },
  { id: "not_run",     label: "Pending" },
  { id: "breakpoints", label: "BP" },
] as const;

type FilterId = typeof FILTERS[number]["id"];

// ─── Snapshot list item ──────────────────────────────────────────────────────
function SnapshotItem({
  snap,
  isSelected,
  isBreakpoint,
  onSelect,
  onDelete,
}: {
  snap: SnapshotRecord;
  isSelected: boolean;
  isBreakpoint: boolean;
  onSelect: () => void;
  onDelete: (e: React.MouseEvent) => void;
}) {
  return (
    <div
      className={[
        "snapshot-item",
        isSelected    ? "snapshot-item--selected"    : "",
        isBreakpoint  ? "snapshot-item--breakpoint"  : "",
      ]
        .filter(Boolean)
        .join(" ")}
      onClick={onSelect}
      title={snap.snapshot_id}
    >
      {isBreakpoint ? (
        <span className="snapshot-item__bp-marker">◆</span>
      ) : (
        <span className={`snapshot-item__dot snapshot-item__dot--${snap.status}`} />
      )}

      <div className="snapshot-item__body">
        <div className="snapshot-item__id">{shortId(snap.snapshot_id)}</div>
        <div className="snapshot-item__meta">
          <span className="snapshot-item__time">
            {timeAgo(snap.created_at_unix_ms)}
          </span>
          <span>{snap.file_count} files</span>
          <span>{formatBytes(snap.total_size_bytes)}</span>
        </div>
      </div>

      <button
        className="snapshot-item__delete"
        onClick={onDelete}
        title="Delete snapshot"
        aria-label="Delete snapshot"
      >
        ×
      </button>
    </div>
  );
}

// ─── Sidebar ─────────────────────────────────────────────────────────────────
export function Sidebar() {
  const { projectInfo, snapshots, isLoading, error, setProject, setSnapshots, removeSnapshot, appendSnapshot, setLoading, setError } =
    useProjectStore();
  const { selectedSnapshotId, setSelected } = useGraphStore();
  const { activeFilter, sidebarSearchQuery, setFilter, setSidebarSearch } = useUiStore();
  const { isBreakpoint } = useBreakpointStore();

  const unlistenRef = useRef<(() => void) | null>(null);

  // ── Refresh snapshot list ────────────────────────────────────────────────
  const refreshSnapshots = useCallback(async () => {
    if (!projectInfo) return;
    try {
      const list = await listSnapshots();
      setSnapshots(list);
    } catch (e) {
      setError(String(e));
    }
  }, [projectInfo, setSnapshots, setError]);

  // ── Listen to Tauri events ───────────────────────────────────────────────
  useEffect(() => {
    let active = true;
    onSnapshotListChanged(() => {
      if (active) refreshSnapshots();
    }).then((unlisten) => {
      if (!active) unlisten();
      else unlistenRef.current = unlisten;
    });

    return () => {
      active = false;
      unlistenRef.current?.();
    };
  }, [refreshSnapshots]);

  // ── Load snapshots when project changes ─────────────────────────────────
  useEffect(() => {
    if (projectInfo) refreshSnapshots();
  }, [projectInfo, refreshSnapshots]);

  // ── Open Project ─────────────────────────────────────────────────────────
  const handleOpenProject = useCallback(async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Open Project Folder",
      });
      if (!selected || typeof selected !== "string") return;

      setLoading(true);
      setError(null);
      const info = await openProject(selected);
      setProject(info);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [setProject, setLoading, setError]);

  // ── Create Snapshot ──────────────────────────────────────────────────────
  const handleNewSnapshot = useCallback(async () => {
    if (!projectInfo) return;
    try {
      setLoading(true);
      const snap = await createSnapshot();
      appendSnapshot(snap);
      setSelected(snap.snapshot_id);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [projectInfo, appendSnapshot, setSelected, setLoading, setError]);

  // ── Delete Snapshot ──────────────────────────────────────────────────────
  const handleDelete = useCallback(
    async (e: React.MouseEvent, id: string) => {
      e.stopPropagation();
      try {
        await deleteSnapshot(id);
        removeSnapshot(id);
        if (selectedSnapshotId === id) setSelected(null);
      } catch (err) {
        setError(String(err));
      }
    },
    [removeSnapshot, selectedSnapshotId, setSelected, setError]
  );

  // ── Keyboard shortcut N → new snapshot ──────────────────────────────────
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (
        e.key === "n" &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        !(e.target instanceof HTMLInputElement) &&
        !(e.target instanceof HTMLTextAreaElement)
      ) {
        handleNewSnapshot();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [handleNewSnapshot]);

  // ── Filtered + searched snapshot list ───────────────────────────────────
  const filtered = snapshots.filter((s) => {
    if (activeFilter === "breakpoints") return isBreakpoint(s.snapshot_id);
    if (activeFilter !== "all" && s.status !== activeFilter) return false;
    if (sidebarSearchQuery) {
      return s.snapshot_id
        .toLowerCase()
        .includes(sidebarSearchQuery.toLowerCase());
    }
    return true;
  });

  // ─── No project open ───────────────────────────────────────────────────
  if (!projectInfo) {
    return (
      <aside className="sidebar">
        <div className="sidebar__open-area">
          <div className="sidebar__open-icon">📂</div>
          <div className="sidebar__open-title">No project open</div>
          <div className="sidebar__open-hint">
            Open a folder to start creating<br />snapshots of your work
          </div>
          <button
            className="sidebar__open-btn"
            onClick={handleOpenProject}
            disabled={isLoading}
          >
            {isLoading ? "Opening…" : "Open Project"}
          </button>
          {error && <div className="sidebar__error">{error}</div>}
        </div>
      </aside>
    );
  }

  // ─── Project open ──────────────────────────────────────────────────────
  return (
    <aside className="sidebar">
      {/* Header */}
      <div className="sidebar__header">
        <button
          className="sidebar__new-btn"
          onClick={handleNewSnapshot}
          disabled={isLoading}
          title="Create new snapshot (N)"
        >
          <span className="sidebar__new-btn__icon">+</span>
          New Snapshot
          <span className="sidebar__new-btn__shortcut">N</span>
        </button>

        {/* Search */}
        <div className="sidebar__search">
          <span className="sidebar__search__icon">🔍</span>
          <input
            className="sidebar__search__input"
            type="text"
            placeholder="Search snapshots…"
            value={sidebarSearchQuery}
            onChange={(e) => setSidebarSearch(e.target.value)}
            spellCheck={false}
          />
        </div>
      </div>

      {/* Filters */}
      <div className="sidebar__filters">
        {FILTERS.map((f) => (
          <button
            key={f.id}
            className={[
              "filter-chip",
              `filter-chip--${f.id}`,
              activeFilter === f.id ? "filter-chip--active" : "",
            ]
              .filter(Boolean)
              .join(" ")}
            onClick={() => setFilter(f.id as FilterId)}
          >
            {f.label}
          </button>
        ))}
      </div>

      {/* Error */}
      {error && <div className="sidebar__error">{error}</div>}

      {/* List */}
      <div className="sidebar__list">
        {isLoading && snapshots.length === 0 ? (
          <div className="sidebar__loading">
            <div className="sidebar__spinner" />
            Loading…
          </div>
        ) : filtered.length === 0 ? (
          <div className="sidebar__empty">
            <div className="sidebar__empty__icon">
              {activeFilter === "breakpoints" ? "◆" : "○"}
            </div>
            <div className="sidebar__empty__text">
              {activeFilter === "all" && snapshots.length === 0
                ? "No snapshots yet.\nPress N to create one."
                : activeFilter === "breakpoints"
                ? "No breakpoints set."
                : `No ${activeFilter} snapshots.`}
            </div>
          </div>
        ) : (
          <>
            <div className="sidebar__section-label">Snapshots</div>
            <AnimatePresence initial={false}>
              {filtered.map((snap) => (
                <motion.div
                  key={snap.snapshot_id}
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: "auto" }}
                  exit={{ opacity: 0, height: 0 }}
                  transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
                >
                  <SnapshotItem
                    snap={snap}
                    isSelected={selectedSnapshotId === snap.snapshot_id}
                    isBreakpoint={isBreakpoint(snap.snapshot_id)}
                    onSelect={() => setSelected(snap.snapshot_id)}
                    onDelete={(e) => handleDelete(e, snap.snapshot_id)}
                  />
                </motion.div>
              ))}
            </AnimatePresence>
          </>
        )}
      </div>

      {/* Count */}
      <div className="sidebar__count">
        {filtered.length} / {snapshots.length} snapshot
        {snapshots.length !== 1 ? "s" : ""}
      </div>
    </aside>
  );
}
