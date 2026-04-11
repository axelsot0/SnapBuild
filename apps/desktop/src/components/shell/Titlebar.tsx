import "./Titlebar.css";

interface TitlebarProps {
  onToggleInspector: () => void;
  onToggleTerminal: () => void;
  inspectorVisible: boolean;
  terminalCollapsed: boolean;
}

export function Titlebar({
  onToggleInspector,
  onToggleTerminal,
  inspectorVisible,
  terminalCollapsed,
}: TitlebarProps) {
  const handleClose = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().close();
  };

  const handleMinimize = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().minimize();
  };

  const handleMaximize = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().toggleMaximize();
  };

  return (
    <div className="titlebar">
      <div className="titlebar__drag-region" data-tauri-drag-region />

      <div className="titlebar__left">
        <div className="titlebar__logo" />
        <span className="titlebar__project-name">SnapBuild</span>
      </div>

      <div className="titlebar__center">
        <span className="titlebar__app-name">snapshot · build · restore</span>
      </div>

      <div className="titlebar__right">
        <button
          className={`titlebar__action${inspectorVisible ? " titlebar__action--active" : ""}`}
          onClick={onToggleInspector}
          title="Toggle Inspector (I)"
        >
          ⊞
        </button>
        <button
          className={`titlebar__action${!terminalCollapsed ? " titlebar__action--active" : ""}`}
          onClick={onToggleTerminal}
          title="Toggle Terminal (`)"
        >
          ⌨
        </button>

        <div className="titlebar__divider" />

        <div className="titlebar__window-controls">
          <div className="window-btn window-btn--close" onClick={handleClose} title="Close" />
          <div className="window-btn window-btn--minimize" onClick={handleMinimize} title="Minimize" />
          <div className="window-btn window-btn--maximize" onClick={handleMaximize} title="Maximize" />
        </div>
      </div>
    </div>
  );
}
