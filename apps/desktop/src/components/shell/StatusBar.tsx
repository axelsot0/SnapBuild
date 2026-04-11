import "./StatusBar.css";
import { useProjectStore } from "../../stores/projectStore";
import { useFileWatcher } from "../../hooks/useFileWatcher";

export function StatusBar() {
  const { projectInfo, snapshots } = useProjectStore();
  const { hasChanges } = useFileWatcher();

  const projectName = projectInfo?.name ?? null;
  const snapshotCount = snapshots.length;

  return (
    <div className="status-bar">
      <div className="status-bar__left">
        <span
          className="status-bar__item status-bar__item--clickable"
          title={projectInfo?.path ?? "No project open"}
        >
          {projectName ? `📁 ${projectName}` : "~ No project open"}
        </span>
      </div>

      <div className="status-bar__center">
        <span className="status-bar__item">
          {snapshotCount} snapshot{snapshotCount !== 1 ? "s" : ""}
        </span>
        {hasChanges && (
          <span className="status-bar__item status-bar__changes">
            {"⚠"} Unsaved changes
          </span>
        )}
      </div>

      <div className="status-bar__right">
        <span className="status-bar__item">
          <div className={`status-dot${projectInfo ? " status-dot--active" : ""}${hasChanges ? " status-dot--changes" : ""}`} />
          {hasChanges ? "Changed" : projectInfo ? "Ready" : "Idle"}
        </span>
        <span className="status-bar__item">
          <kbd className="kbd">Ctrl+K</kbd> commands
        </span>
      </div>
    </div>
  );
}
