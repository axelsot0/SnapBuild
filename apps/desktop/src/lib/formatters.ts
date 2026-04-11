/** Format bytes to human-readable string: 1.2 MB, 345 KB, etc. */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const val = bytes / Math.pow(k, i);
  return `${val < 10 ? val.toFixed(1) : Math.round(val)} ${sizes[i]}`;
}

/** Truncate a snapshot ID to its short form for display. */
export function shortId(id: string): string {
  // "snap-18f3a2b" → "18f3a2b" (strip prefix, take last 7 chars of hex part)
  const parts = id.split("-");
  if (parts.length >= 2) {
    const hex = parts.slice(1).join("-");
    return hex.slice(-7);
  }
  return id.slice(0, 7);
}

/** Time ago string from a unix ms timestamp. */
export function timeAgo(unixMs: number): string {
  const diff = Date.now() - unixMs;
  const secs = Math.floor(diff / 1000);
  if (secs < 5) return "just now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

/** Format a duration in ms to a human string: 1.2s, 340ms */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}
