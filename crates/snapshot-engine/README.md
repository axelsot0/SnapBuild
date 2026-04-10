# snapshot-engine

Local snapshot engine (implemented foundation).

## Current capabilities
- Walk project files with basic ignore rules.
- Hash file contents for deduplicated local storage.
- Store blobs in content-addressed layout (`objects/ab/cdef...`).
- Write/read snapshot manifests (`snapshots/<snapshot-id>.manifest`).
