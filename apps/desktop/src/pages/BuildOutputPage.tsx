import { TerminalPanel } from "../components/terminal/TerminalPanel";
import { Titlebar } from "../components/shell/Titlebar";

export function BuildOutputPage() {
  return (
    <div style={{
      width: "100vw",
      height: "100vh",
      display: "flex",
      flexDirection: "column",
      background: "var(--color-bg-base)",
    }}>
      <Titlebar
        onToggleInspector={() => {}}
        onToggleTerminal={() => {}}
        inspectorVisible={false}
        terminalCollapsed={false}
      />
      <div style={{ flex: 1, overflow: "hidden" }}>
        <TerminalPanel />
      </div>
    </div>
  );
}
