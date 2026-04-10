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
  README.md
```

## Next planning checkpoints

Before coding deeply, lock these 4 items:

1. Value proposition in one sentence.
2. 90-second demo flow.
3. Minimum architecture.
4. First-week implementation roadmap.
