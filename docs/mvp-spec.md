# SnapBuild MVP Spec (v0.2)

## Product promise

**Build any moment of your project instantly.**

SnapBuild is a desktop app for running experiments safely: snapshot, run, compare, and restore project states without branch/stash overhead.

## Target user (initial)

- Frontend developers.
- Indie hackers shipping quickly.
- Small teams that frequently break local builds while experimenting.

## Demo flow (90 seconds)

1. Open project in SnapBuild.
2. Edit a few files and produce a broken state.
3. Timeline shows automatic snapshots.
4. Select a previous stable snapshot.
5. Click **Run snapshot** (materialized in temp dir).
6. Click **Restore** to go back to a known-good state.

## MVP scope

### Included
- Automatic local snapshots (incremental + deduplicated).
- Snapshot timeline with status (`ok`, `failed`, `not_run`).
- Run selected snapshots from temp workspaces.
- Restore selected snapshots into main workspace.

### Not included (yet)
- Git replacement.
- Kernel/filesystem driver work.
- Team collaboration and remote snapshot sync.
- CI/CD deep integrations.

## Minimum architecture

- **Desktop shell:** Tauri.
- **UI:** React + TypeScript.
- **Core services:** Rust crates.
- **Storage:** local CAS-style objects + snapshot metadata.

## First implementation modules

- `snapshot-engine`: hashing, object storage, snapshot manifests.
- `workspace-materializer`: reconstruct snapshot into temp folder.
- `build-runner`: execute builds/tests and record results.
- `apps/desktop`: timeline UI + run/restore controls.
