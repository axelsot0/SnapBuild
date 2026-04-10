# SnapBuild Roadmap — Week 1

## Goal

Ship a clickable vertical slice with real local snapshots and one visible run/restore demo path.

## Day 1 — Repo bootstrap
- Prepare monorepo folders (`apps/`, `crates/`, `docs/`).
- Finalize one-line value proposition and demo script.

## Day 2 — Desktop shell skeleton
- Set up Tauri + React shell.
- Add project-open workflow and empty timeline view.

## Day 3 — Snapshot engine core
- Implement file hashing and object deduplication.
- Create snapshot manifest model.

## Day 4 — Snapshot listing + timeline wiring
- Persist snapshot metadata.
- Render timeline cards with timestamp and status.

## Day 5 — Materialize + run + restore (happy path)
- Materialize selected snapshot into temp directory.
- Trigger a build command and capture result.
- Restore selected snapshot into workspace.

## Exit criteria
- User can create snapshots automatically.
- User can run a previous snapshot in a temp folder.
- User can restore a previous snapshot with one action.
