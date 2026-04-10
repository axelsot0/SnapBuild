# SnapBuild Roadmap — 2 Weeks

## Outcome
A demo-ready desktop app where developers can snapshot, run from snapshot, and restore in one click.

## Week 1 (foundation)
- Repo and app bootstrap.
- Snapshot engine basics (hashing + deduplicated object store).
- Timeline view wired to local snapshot metadata.
- First happy path for run + restore.

## Week 2 (demo hardening)
- Better project-type detection and command presets.
- Build status persistence (`ok`, `failed`, `not_run`) with durations.
- Snapshot compare basics (changed files + run result deltas).
- UX polish and demo script rehearsal.

## Demo definition of done
- Automatic snapshots from project edits.
- Timeline with clear status colors.
- `Run snapshot` executes from temp workspace.
- `Restore` returns workspace to selected snapshot.
