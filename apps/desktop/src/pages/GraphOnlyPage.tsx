import { GraphCanvas } from "../components/graph/GraphCanvas";
import { Titlebar } from "../components/shell/Titlebar";

export function GraphOnlyPage() {
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
        terminalCollapsed={true}
      />
      <div style={{ flex: 1, overflow: "hidden" }}>
        <GraphCanvas />
      </div>
    </div>
  );
}
