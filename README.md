# SnapBuild

Build any moment of your project instantly.

SnapBuild is a desktop app that lets developers create fast local snapshots of a project, restore any state in one click, and run builds from past moments without juggling branches, stashes, or duplicate folders.

## MVP focus

- **Desktop app:** Tauri + React.
- **Local engine:** create snapshots, list snapshots, restore snapshot, and materialize snapshots into temp folders.
- **Visible demo:** timeline of snapshots, **Run snapshot**, and **Restore** actions.

## Initial repository structure

```text
SnapBuild/
  apps/
    desktop/
  crates/
    snapshot-engine/
    build-runner/
    workspace-materializer/
  docs/
    mvp-spec.md
  Cargo.toml
  README.md
```

## Development status (week 1)

Implemented backend foundations in Rust:

- `snapshot-engine`: CAS storage + plain manifest files + SQLite DAG persistence module.
- `workspace-materializer`: reconstruct snapshot into temp folder.
- `build-runner`: execute command and capture result metadata.

Run all tests:

```bash
cargo test
```

## Product planning docs

- `docs/product-plan.md`
- `docs/roadmap-4-weeks.md`
