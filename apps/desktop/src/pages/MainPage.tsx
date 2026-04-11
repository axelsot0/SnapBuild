import { AppShell } from "../components/shell/AppShell";
import { Sidebar } from "../components/sidebar/Sidebar";
import { GraphCanvas } from "../components/graph/GraphCanvas";
import { InspectorPanel } from "../components/inspector/InspectorPanel";
import { TerminalPanel } from "../components/terminal/TerminalPanel";
import { CommandPalette } from "../components/command-palette/CommandPalette";

export function MainPage() {
  return (
    <AppShell
      sidebar={<Sidebar />}
      graph={<GraphCanvas />}
      inspector={<InspectorPanel />}
      terminal={<TerminalPanel />}
      commandPalette={<CommandPalette />}
    />
  );
}
