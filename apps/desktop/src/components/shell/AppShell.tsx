import { useState, useCallback, useRef } from "react";
import "./AppShell.css";
import { Titlebar } from "./Titlebar";
import { StatusBar } from "./StatusBar";

interface AppShellProps {
  sidebar: React.ReactNode;
  graph: React.ReactNode;
  inspector: React.ReactNode;
  terminal: React.ReactNode;
  commandPalette?: React.ReactNode;
}

const MIN_TERMINAL_HEIGHT = 80;
const MAX_TERMINAL_HEIGHT = 400;
const DEFAULT_TERMINAL_HEIGHT = 220;

export function AppShell({
  sidebar,
  graph,
  inspector,
  terminal,
  commandPalette,
}: AppShellProps) {
  const [terminalHeight, setTerminalHeight] = useState(DEFAULT_TERMINAL_HEIGHT);
  const [terminalCollapsed, setTerminalCollapsed] = useState(false);
  const [inspectorVisible, setInspectorVisible] = useState(true);
  const isDragging = useRef(false);
  const dragStartY = useRef(0);
  const dragStartHeight = useRef(0);

  const handleResizeStart = useCallback(
    (e: React.MouseEvent) => {
      isDragging.current = true;
      dragStartY.current = e.clientY;
      dragStartHeight.current = terminalHeight;

      const onMove = (ev: MouseEvent) => {
        if (!isDragging.current) return;
        const delta = dragStartY.current - ev.clientY;
        const newHeight = Math.min(
          MAX_TERMINAL_HEIGHT,
          Math.max(MIN_TERMINAL_HEIGHT, dragStartHeight.current + delta)
        );
        setTerminalHeight(newHeight);
        setTerminalCollapsed(false);
      };

      const onUp = () => {
        isDragging.current = false;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };

      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    },
    [terminalHeight]
  );

  return (
    <div className="app-shell">
      <Titlebar
        onToggleInspector={() => setInspectorVisible((v) => !v)}
        onToggleTerminal={() => setTerminalCollapsed((v) => !v)}
        inspectorVisible={inspectorVisible}
        terminalCollapsed={terminalCollapsed}
      />

      <div
        className={`app-shell__body${
          !inspectorVisible ? " app-shell__body--inspector-hidden" : ""
        }`}
      >
        {sidebar}

        <div className="app-shell__main">
          <div className="app-shell__graph-area">{graph}</div>

          <div
            className="terminal-resize-handle"
            onMouseDown={handleResizeStart}
          />

          <div
            className={`app-shell__terminal${
              terminalCollapsed ? " app-shell__terminal--collapsed" : ""
            }`}
            style={{ height: terminalCollapsed ? 0 : terminalHeight }}
          >
            {terminal}
          </div>
        </div>

        {inspectorVisible && inspector}
      </div>

      <StatusBar />

      {commandPalette}
    </div>
  );
}
