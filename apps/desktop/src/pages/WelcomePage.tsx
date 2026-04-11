import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import "./WelcomePage.css";

import { openProject } from "../tauri/commands";
import { useProjectStore } from "../stores/projectStore";

interface RecentProject {
  name: string;
  path: string;
  snapshotCount: number;
}

interface WelcomePageProps {
  onProjectOpened: () => void;
}

export function WelcomePage({ onProjectOpened }: WelcomePageProps) {
  const { setProject, setLoading, setError, isLoading, error } = useProjectStore();
  const [recents, setRecents] = useState<RecentProject[]>([]);

  // Load recents from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem("snapbuild-recents");
      if (stored) setRecents(JSON.parse(stored));
    } catch { /* ignore */ }
  }, []);

  const saveRecent = useCallback((name: string, path: string, count: number) => {
    setRecents((prev) => {
      const filtered = prev.filter((r) => r.path !== path);
      const next = [{ name, path, snapshotCount: count }, ...filtered].slice(0, 5);
      localStorage.setItem("snapbuild-recents", JSON.stringify(next));
      return next;
    });
  }, []);

  const handleOpen = useCallback(async (path?: string) => {
    try {
      const selected = path ?? await openDialog({
        directory: true,
        multiple: false,
        title: "Open Project Folder",
      });
      if (!selected || typeof selected !== "string") return;

      setLoading(true);
      setError(null);
      const info = await openProject(selected);
      setProject(info);
      saveRecent(info.name, info.path, info.snapshot_count);
      onProjectOpened();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [setProject, setLoading, setError, saveRecent, onProjectOpened]);

  // Ctrl+O shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "o") {
        e.preventDefault();
        handleOpen();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [handleOpen]);

  // Window controls
  const appWindow = getCurrentWindow();

  return (
    <div className="welcome">
      {/* Titlebar */}
      <div className="welcome__titlebar" data-tauri-drag-region>
        <button className="welcome__titlebar__btn welcome__titlebar__btn--close" onClick={() => appWindow.close()} />
        <button className="welcome__titlebar__btn welcome__titlebar__btn--min" onClick={() => appWindow.minimize()} />
        <button className="welcome__titlebar__btn welcome__titlebar__btn--max" onClick={() => appWindow.toggleMaximize()} />
      </div>

      {/* Content */}
      <div className="welcome__content">
        {/* Brand */}
        <div className="welcome__brand">
          <div className="welcome__logo">&#x2B21;</div>
          <div className="welcome__title">SnapBuild</div>
          <div className="welcome__subtitle">
            Snapshot-based version control for your projects.
            Create, compare, restore and build any point in time.
          </div>
        </div>

        {/* Open button */}
        <div className="welcome__actions">
          <button
            className="welcome__open-btn"
            onClick={() => handleOpen()}
            disabled={isLoading}
          >
            {isLoading ? "Opening..." : "Open Project"}
          </button>
          <div className="welcome__shortcut-hint">
            <kbd>Ctrl</kbd> + <kbd>O</kbd> to open
          </div>
        </div>

        {/* Error */}
        {error && (
          <div style={{
            color: "var(--color-status-fail)",
            fontSize: "var(--font-size-xs)",
            maxWidth: 280,
            textAlign: "center",
          }}>
            {error}
          </div>
        )}

        {/* Recent projects */}
        {recents.length > 0 && (
          <div className="welcome__recents">
            <div className="welcome__recents-label">Recent Projects</div>
            {recents.map((r) => (
              <div
                key={r.path}
                className="welcome__recent-item"
                onClick={() => handleOpen(r.path)}
              >
                <span className="welcome__recent-icon">&#x1F4C1;</span>
                <div className="welcome__recent-info">
                  <div className="welcome__recent-name">{r.name}</div>
                  <div className="welcome__recent-path">{r.path}</div>
                </div>
                <span className="welcome__recent-count">
                  {r.snapshotCount} snap{r.snapshotCount !== 1 ? "s" : ""}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="welcome__footer">
        SnapBuild v0.1.0
      </div>
    </div>
  );
}
