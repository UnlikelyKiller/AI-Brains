# AI-Brains Conductor Registry

| Track | Name | Status | Owner | Spec | Description |
|-------|------|--------|-------|------|-------------|
| T61 | Nightly Synthesis Batch Limit | ✅ **Complete** | Hermes | [T61](tracks/trackT61-nightly-synthesis-limit/spec.md) | Fixed nightly hang by limiting synthesis to 50-memory batches |
| T62 | Semantic Search — Stored Embeddings | ✅ **Complete** | Hermes | [T62](tracks/trackT62-semantic-embeddings/spec.md) | Added embedding column, built backfill script, verified semantic recall |
| T63 | Nightly Embedding Integration | ✅ **Complete** | Hermes | [T63](tracks/trackT63-nightly-embedding-integration/spec.md) | Integrated embedding backfill into nightly pipeline; 50 memories auto-embedded per run |
| T64 | Stale Embedding Refresh + WAL Checkpointing | ✅ **Complete** | Hermes | [T64](tracks/trackT64-stale-refresh-wal/spec.md) | Added embedding timestamps, stale refresh, WAL checkpointing |
| T65 | Repo Alias Resolution | ✅ **Complete** | Hermes | [T65](tracks/trackT65-repo-alias-resolution/spec.md) | Auto-detect project IDs from aliases and git repos; scope recall per-repo |
| T66 | Graph-Augmented Recall + Graph Query CLI | ✅ **Code Complete** | Hermes+Claude | [T66](tracks/trackT66-graph-augmented-recall/spec.md) | All 3 phases implemented (recall augmentation, CLI queries, boost config). Pending `MemoryPinned`/`MemorySynthesized` events for full data |
| T67 | Memory Pinning Events | ✅ **Complete** | Hermes+Codex | [T67](tracks/trackT67-memory-pinning-events/spec.md) | Emit `MemoryPinned` events on recall so graph gets `RECALLS`/`SOURCE_FOR` edges |
| T68 | Memory Synthesis Events | ✅ **Complete** | Hermes+Codex | [T68](tracks/trackT68-memory-synthesis-events/spec.md) | Emit `MemorySynthesized` events during nightly so graph gets `SYNTHESIZED_FROM` edges |
| T69 | Live Graph Hook — Incremental Graph Updates | ✅ **Complete** | Claude+Codex | [T69](tracks/trackT69-live-graph-hook/spec.md) | Apply graph projector incrementally on each event append; eliminates need for manual `graph rebuild` after recall/nightly |
| T70 | ChangeGuard Symbol Bridge — Code-Aware Recall | ✅ **Complete** | Codex | [T70](tracks/trackT70-changeguard-symbol-bridge/spec.md) | Ingest ChangeGuard's code symbols (functions and routes) into AI-Brains during nightly so `recall` returns actual code structure |
| T71 | CI Tooling Reproducibility | ✅ **Complete** | Claude | [T71](tracks/trackT71-ci-tooling-reproducibility/spec.md) | All three tools install via cargo install --locked; full gate passes locally; scripts/dev-check.ps1 verifies presence + runs gate |

---

## Track Status Legend
- **Pending** — Requirements written, no implementation started
- **In Progress** — Active development
- **Complete** — All success criteria met, verified in production
- **Blocked** — External dependency preventing progress
- **Abandoned** — No longer relevant, archived for reference

## Adding a New Track
1. Create `tracks/trackTNN-<name>/spec.md` with problem statement, design, and verification
2. Add entry to table above with **Pending** status
3. Update to **In Progress** when implementation starts
4. Update to **Complete** when all success criteria are met
