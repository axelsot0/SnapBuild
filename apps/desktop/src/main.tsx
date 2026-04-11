import React, { useState } from "react";
import ReactDOM from "react-dom/client";
import "./design-system/index.css";

import { WelcomePage } from "./pages/WelcomePage";
import { MainPage } from "./pages/MainPage";
import { GraphOnlyPage } from "./pages/GraphOnlyPage";
import { BuildOutputPage } from "./pages/BuildOutputPage";
import { useProjectStore } from "./stores/projectStore";

/** Read the `?window=` query param to determine which page to render */
function getWindowType(): string | null {
  const params = new URLSearchParams(window.location.search);
  return params.get("window");
}

function App() {
  const windowType = getWindowType();

  // Detached windows render their specific page directly
  if (windowType === "graph") return <GraphOnlyPage />;
  if (windowType === "build") return <BuildOutputPage />;

  // Main window: Welcome → Main flow
  return <MainWindow />;
}

function MainWindow() {
  const projectInfo = useProjectStore((s) => s.projectInfo);
  const [entered, setEntered] = useState(false);

  if (!projectInfo && !entered) {
    return <WelcomePage onProjectOpened={() => setEntered(true)} />;
  }

  return <MainPage />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
