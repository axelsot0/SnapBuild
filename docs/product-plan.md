# SnapBuild Product Plan (MVP-first)

## 1) What SnapBuild is
SnapBuild is a desktop app to explore local project states, run builds from those states, and restore safely with low friction.

Core product idea:
- editor
- state canvas
- build history

## 2) Problems to solve (now)
1. Experiment without polluting Git history.
2. See attempts as visual states, not shell command memory.
3. Understand which states built/failed and lineage.
4. Restore useful states in one click.

If a feature does not push one of these outcomes, it is out of scope.

## 3) Out of scope (now)
- Replacing Git.
- Replacing VS Code.
- Heavy Docker/Nix-like storage runtime.
- Kernel-level overlay filesystem.
- Snapshot-per-keystroke behavior.

## 4) Real MVP definition
"Edit a project, generate useful states automatically, visualize them in an ordered canvas, run one, restore one, and keep everything persisted across app restarts."

Must-have MVP features:
- Embedded editor.
- Change detection.
- Auto-snapshots with healthy rules.
- Manual snapshots.
- Editable names.
- Favorites.
- Tags/colors.
- Build status per snapshot.
- Auto-layout canvas.
- Persistence after app close.
- Minimal Git context.
- Functional run and restore.

## 5) Product model
### A) Draft state
Ephemeral editing state used to group in-progress changes.

### B) Snapshot
Persistent visible state with metadata:
- id
- name
- parent_id
- timestamp
- source_fingerprint
- git_branch
- touched_files
- favorite
- tags/color
- build_status
- build_history
- dependency_fingerprint

### C) Build run
Each run is tied to one snapshot:
- command
- started_at
- duration
- success/failure
- stdout/stderr
- metadata

## 6) Auto-snapshot rules
Create snapshots when:
- user saves
- user clicks run
- 30-60s idle after real edits
- user creates manual snapshot
- user restores state
- user closes project/app

Before creating:
1. compute source fingerprint
2. if equal to latest persisted snapshot, skip
3. if equal to existing historical snapshot, reuse/connect instead of duplicate

## 7) UX layout
- Left: project, branch, git state, snapshot list, filters
- Center: auto-layout DAG canvas
- Right: snapshot inspector + actions (Run/Restore/Compare/Favorite)
- Bottom: run terminal blocks linked to snapshots

## 8) Technical priorities (P0)
1. SQLite DAG persistence (`projects`, `snapshots`, `snapshot_edges`, `build_runs`, `tags`, `snapshot_tags`).
2. Distinct `source_fingerprint` and `dependency_fingerprint`.
3. Dependency layer based on lockfiles.
4. Smarter run model (materialized source + dependency layer reuse).

## 9) Defer (post-MVP)
- zstd compression
- delta packs
- advanced overlay/COW
- advanced symlink/junction strategy
- remote cache / multi-machine sync

## 10) MVP demo success
- Open project.
- Edit in SnapBuild.
- Observe useful new state.
- Name it.
- Run it and see build result.
- Restore previous state.
- Close app and reopen with state preserved.

## 11) Core product decision
SnapBuild is not "a snapshot storage system".
SnapBuild is a UI to understand and control work states.
