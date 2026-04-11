# SnapBuild 4-Week Execution Roadmap (P0/P1/P2)

## Week 1 — Product order + persistence base
- P0: finalize snapshot/draft/build-run domain model.
- P0: define SQLite schema for DAG + tags + build runs.
- P1: wire persistence layer in backend abstractions.
- P1: branch + minimal git context wiring.
- P2: docs and demo script updates.

## Week 2 — Run/Restore that matters
- P0: source fingerprint and dependency fingerprint split.
- P0: dependency layer resolution from lockfiles.
- P0: run association to both fingerprints.
- P1: restore robustness and safety backup.
- P2: compare basics.

## Week 3 — Live editing experience
- P0: embedded editor integration.
- P0: auto-snapshot triggers + debounce/idle policy.
- P1: draft-state promotion to persisted snapshot.
- P1: canvas interactions and inspector edits.
- P2: richer tags/colors UX.

## Week 4 — hardening and polish
- P0: end-to-end demo reliability.
- P1: improved materialization caching strategy.
- P1: run history UX and logs.
- P2: storage/runtime optimizations after validation.

## Priority scale
- P0: required for MVP demo acceptance.
- P1: high impact but can be narrowed.
- P2: polish/optimization.
