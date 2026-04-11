import { useState, useCallback } from "react";
import "./InspectorPanel.css";

import { useProjectStore } from "../../stores/projectStore";
import { useGraphStore } from "../../stores/graphStore";
import { useBreakpointStore } from "../../stores/breakpointStore";
import { useTerminalStore } from "../../stores/terminalStore";
import { shortId, timeAgo, formatBytes } from "../../lib/formatters";
import { restoreSnapshot, runSnapshotBuild, deleteSnapshot } from "../../tauri/commands";
import type { SnapshotRecord } from "../../tauri/types";

export function InspectorPanel() {
  const { snapshots, removeSnapshot } = useProjectStore();
  const { selectedSnapshotId, setSelected } = useGraphStore();
  const { isBreakpoint, toggle: toggleBreakpoint } = useBreakpointStore();
  const { startBlock } = useTerminalStore();
  const [toast, setToast] = useState<string | null>(null);
  const [runCommand, setRunCommand] = useState("npm run build");

  const selected: SnapshotRecord | undefined = snapshots.find(
    (s) => s.snapshot_id === selectedSnapshotId
  );

  // ── Toast helper ──────────────────────────────────────────────
  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 1500);
  }, []);

  // ── Copy ID ───────────────────────────────────────────────────
  const handleCopyId = useCallback(() => {
    if (!selected) return;
    navigator.clipboard.writeText(selected.snapshot_id);
    showToast("Copied!");
  }, [selected, showToast]);

  // ── Restore ───────────────────────────────────────────────────
  const handleRestore = useCallback(async () => {
    if (!selected) return;
    try {
      await restoreSnapshot(selected.snapshot_id);
      showToast("Restored!");
    } catch (e) {
      showToast(`Error: ${e}`);
    }
  }, [selected, showToast]);

  // ── Run build ─────────────────────────────────────────────────
  const handleRun = useCallback(async () => {
    if (!selected || !runCommand.trim()) return;
    const parts = runCommand.trim().split(/\s+/);
    const cmd = parts[0];
    const args = parts.slice(1);
    startBlock(selected.snapshot_id, runCommand.trim());
    try {
      await runSnapshotBuild(selected.snapshot_id, cmd, args);
    } catch (e) {
      showToast(`Run failed: ${e}`);
    }
  }, [selected, runCommand, startBlock, showToast]);

  // ── Delete ────────────────────────────────────────────────────
  const handleDelete = useCallback(async () => {
    if (!selected) return;
    try {
      await deleteSnapshot(selected.snapshot_id);
      removeSnapshot(selected.snapshot_id);
      setSelected(null);
    } catch (e) {
      showToast(`Delete failed: ${e}`);
    }
  }, [selected, removeSnapshot, setSelected, showToast]);

  // ── Empty state ───────────────────────────────────────────────
  if (!selected) {
    return (
      <aside className="inspector">
        <div className="inspector__empty">
          <div className="inspector__empty__icon">&#x229E;</div>
          <div>Select a snapshot<br />to inspect</div>
        </div>
      </aside>
    );
  }

  const bp = isBreakpoint(selected.snapshot_id);

  return (
    <aside className="inspector" style={{ position: "relative" }}>
      {/* Header */}
      <div className="inspector__header">
        <span
          className={`inspector__header__dot inspector__header__dot--${selected.status}`}
        />
        <span className="inspector__header__id" title={selected.snapshot_id}>
          {shortId(selected.snapshot_id)}
        </span>
        <button
          className="inspector__header__close"
          onClick={() => setSelected(null)}
          title="Close inspector"
        >
          &#x2715;
        </button>
      </div>

      {/* Run command input */}
      <div className="inspector__run-bar">
        <span className="inspector__run-bar__prompt">$</span>
        <input
          className="inspector__run-bar__input"
          type="text"
          value={runCommand}
          onChange={(e) => setRunCommand(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") handleRun(); }}
          placeholder="Command to run…"
          spellCheck={false}
        />
        <button
          className="inspector__run-bar__go"
          onClick={handleRun}
          title="Run this command"
        >
          &#x25B6;
        </button>
      </div>

      {/* Action buttons */}
      <div className="inspector__actions">
        <button
          className="inspector__action-btn inspector__action-btn--accent"
          onClick={handleRun}
          title="Run build on this snapshot"
        >
          &#x25B6; Run
        </button>
        <button
          className="inspector__action-btn"
          onClick={handleRestore}
          title="Restore workspace to this snapshot"
        >
          &#x21A9; Restore
        </button>
        <button
          className={`inspector__action-btn inspector__action-btn--bp ${
            bp ? "inspector__action-btn--bp-active" : ""
          }`}
          onClick={() => toggleBreakpoint(selected.snapshot_id)}
          title={bp ? "Remove breakpoint" : "Mark as breakpoint"}
        >
          &#x25C6; {bp ? "Unmark BP" : "Breakpoint"}
        </button>
        <button
          className="inspector__action-btn"
          onClick={handleCopyId}
          title="Copy full snapshot ID"
        >
          Copy ID
        </button>
        <button
          className="inspector__action-btn inspector__action-btn--danger"
          onClick={handleDelete}
          title="Delete this snapshot"
        >
          &#x2715; Delete
        </button>
      </div>

      {/* Metadata */}
      <div className="inspector__meta">
        <div className="inspector__meta-grid">
          <span className="inspector__meta-label">ID</span>
          <span className="inspector__meta-value">{selected.snapshot_id}</span>

          <span className="inspector__meta-label">Created</span>
          <span className="inspector__meta-value inspector__meta-value--secondary">
            {timeAgo(selected.created_at_unix_ms)}
            <span style={{ marginLeft: 6, fontSize: 10, color: "var(--color-text-muted)" }}>
              ({new Date(selected.created_at_unix_ms).toLocaleTimeString()})
            </span>
          </span>

          <span className="inspector__meta-label">Files</span>
          <span className="inspector__meta-value">{selected.file_count}</span>

          <span className="inspector__meta-label">Size</span>
          <span className="inspector__meta-value">
            {formatBytes(selected.total_size_bytes)}
          </span>

          <span className="inspector__meta-label">Parent</span>
          <span className="inspector__meta-value inspector__meta-value--secondary">
            {selected.parent_id ? shortId(selected.parent_id) : "—"}
          </span>

          <span className="inspector__meta-label">Status</span>
          <span className="inspector__meta-value">{selected.status}</span>
        </div>
      </div>

      {/* Files section placeholder — will show real diff in future */}
      <div className="inspector__section">Snapshot Files</div>
      <div className="inspector__files">
        <div className="inspector__file-item" style={{ color: "var(--color-text-muted)", fontFamily: "var(--font-ui)" }}>
          {selected.file_count} file{selected.file_count !== 1 ? "s" : ""} in this snapshot
        </div>
      </div>

      {/* Toast */}
      {toast && <div className="inspector__toast">{toast}</div>}
    </aside>
  );
}
