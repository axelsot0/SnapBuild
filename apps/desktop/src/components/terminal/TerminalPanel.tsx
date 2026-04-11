import { useEffect, useRef, useCallback } from "react";
import "./TerminalPanel.css";

import { useTerminalStore, type TerminalBlock as TBlock } from "../../stores/terminalStore";
import { onBuildOutputChunk, onBuildComplete } from "../../tauri/events";
import { formatDuration } from "../../lib/formatters";

// ─── Single block ────────────────────────────────────────────────────────────
function TerminalBlock({ block }: { block: TBlock }) {
  const { toggleCollapsed } = useTerminalStore();
  const bodyRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom as new lines arrive
  useEffect(() => {
    if (bodyRef.current && !block.collapsed) {
      bodyRef.current.scrollTop = bodyRef.current.scrollHeight;
    }
  }, [block.lines.length, block.collapsed]);

  const isRunning = block.result === null;
  const isSuccess = block.result?.success === true;

  const blockClass = [
    "terminal-block",
    isRunning
      ? "terminal-block--running"
      : isSuccess
      ? "terminal-block--success"
      : "terminal-block--failed",
  ].join(" ");

  const badgeClass = isRunning
    ? "terminal-block__badge terminal-block__badge--running"
    : isSuccess
    ? "terminal-block__badge terminal-block__badge--success"
    : "terminal-block__badge terminal-block__badge--failed";

  const badgeText = isRunning
    ? "RUNNING"
    : isSuccess
    ? `EXIT 0`
    : `EXIT ${block.result?.exit_code}`;

  const handleCopyAll = useCallback(() => {
    const text = block.lines.map((l) => l.text).join("\n");
    navigator.clipboard.writeText(text);
  }, [block.lines]);

  const handleCopyLine = useCallback((text: string) => {
    navigator.clipboard.writeText(text);
  }, []);

  return (
    <div className={blockClass}>
      <div
        className="terminal-block__header"
        onClick={() => toggleCollapsed(block.id)}
      >
        <span
          className={`terminal-block__chevron${
            !block.collapsed ? " terminal-block__chevron--open" : ""
          }`}
        >
          &#x25B6;
        </span>
        <span className="terminal-block__command">$ {block.command}</span>
        <span className={badgeClass}>{badgeText}</span>
        {block.result && (
          <span className="terminal-block__duration">
            {formatDuration(block.result.duration_ms)}
          </span>
        )}
        {/* Copy all button in header */}
        <button
          className="terminal-block__copy-all"
          onClick={(e) => { e.stopPropagation(); handleCopyAll(); }}
          title="Copy all output"
        >
          Copy
        </button>
      </div>

      <div
        ref={bodyRef}
        className={`terminal-block__body${
          block.collapsed ? " terminal-block__body--collapsed" : ""
        }`}
      >
        {block.lines.length === 0 && !isRunning ? (
          <div className="terminal-block__no-output">No output</div>
        ) : (
          block.lines.map((line, i) => (
            <div
              key={i}
              className={`terminal-line${
                line.isStderr ? " terminal-line--stderr" : ""
              }`}
            >
              <span className="terminal-line__text">{line.text}</span>
              <button
                className="terminal-line__copy"
                onClick={() => handleCopyLine(line.text)}
                title="Copy line"
              >
                &#x2398;
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// ─── Terminal Panel ──────────────────────────────────────────────────────────
export function TerminalPanel() {
  const { blocks, appendLine, finishBlock, clearAll } = useTerminalStore();

  // Subscribe to Tauri build events
  useEffect(() => {
    let active = true;

    const unsubs: (() => void)[] = [];

    onBuildOutputChunk((chunk) => {
      if (active) {
        appendLine(chunk.snapshot_id, chunk.line, chunk.is_stderr);
      }
    }).then((u) => {
      if (!active) u();
      else unsubs.push(u);
    });

    onBuildComplete((result) => {
      if (active) {
        finishBlock(result);
      }
    }).then((u) => {
      if (!active) u();
      else unsubs.push(u);
    });

    return () => {
      active = false;
      unsubs.forEach((u) => u());
    };
  }, [appendLine, finishBlock]);

  if (blocks.length === 0) {
    return (
      <div className="terminal-panel">
        <div className="terminal-panel__empty">
          <span className="terminal-panel__empty__prompt">$</span>
          Terminal output will appear here
        </div>
      </div>
    );
  }

  return (
    <div className="terminal-panel" style={{ position: "relative" }}>
      <button className="terminal-panel__clear" onClick={clearAll}>
        Clear
      </button>
      <div className="terminal-panel__scroll">
        {blocks.map((block) => (
          <TerminalBlock key={block.id} block={block} />
        ))}
      </div>
    </div>
  );
}
