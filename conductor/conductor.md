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
| T72 | Status & Doc Reconciliation | ✅ **Complete** | Claude | [T72](tracks/trackT72-status-reconciliation/spec.md) | Rewrote Docs/status.md to T71 reality; restored bridge docs to .agents skill; archived stale root artifacts; documented cargo audit quirk in ci-tooling.md |
| T73 | Idempotent `init` | ✅ **Complete** | Claude | [T73](tracks/trackT73-init-safety/spec.md) | `init` refuses on populated vault unless `--force`; structured JSON error envelope on refusal; 2 new tests |
| T74 | Graph Health Smoke Test | ✅ **Complete** | Claude | [T74](tracks/trackT74-graph-health-smoke/spec.md) | nextest smoke that runs init → ingest → pin → recall → `graph update` and asserts nodes/edges ≥ 1, status live |
| T75 | OPERATIONS.md Modernization | ✅ **Complete** | Claude | [T75](tracks/trackT75-operations-modernization/spec.md) | Rewrote OPERATIONS.md to cover daemon, forget, safety sync, sync query, bridge, schedule, restore, and the full env-var surface |
| T76 | CLI Polish (project list + backup restore) | ✅ **Complete** | Claude | [T76](tracks/trackT76-cli-polish/spec.md) | Widened `project list` name column with hint header; added `--force` and `--dry-run` to `backup restore`; 2 new tests |
| T77 | forget --memory-id validation | ✅ **Complete** | Claude | [T77](tracks/trackT77-forget-memory-id-validation/spec.md) | `forget --memory-id=<unknown>` exits 1 with "Memory <id> not found." instead of silently no-op'ing |
| T78 | daemon schedule schtasks quoting | ✅ **Complete** | Claude | [T78](tracks/trackT78-daemon-schedule-quoting/spec.md) | `render_daemon_logon_command` uses single-quote wrapping so schtasks accepts paths with spaces |
| T79 | nightly --skip-import | ✅ **Complete** | Claude | [T79](tracks/trackT79-nightly-skip-import/spec.md) | Opt-out flag for `antigravity_import` in `nightly`; prevents cross-vault contamination on isolated/CI vaults |
| T80 | --no-project-context flag | ✅ **Complete** | Claude | [T80](tracks/trackT80-no-project-context-flag/spec.md) | Global escape hatch so `main()` does not auto-clear `AI_BRAINS_*` env vars when no `.env` exists in cwd |
| T81 | --quiet silences bridge warnings | ✅ **Complete** | Claude | [T81](tracks/trackT81-quiet-bridge-warnings/spec.md) | `recall --quiet`, `preflight --quiet`, `sync query --quiet` suppress the "ChangeGuard bridge query failed" warning |
| T82 | honor context --new-project | ✅ **Complete** | Claude | [T82](tracks/trackT82-context-new-project/spec.md) | `context --new-project` rotates the project_id and prints "Rotating project ID from <old> to fresh UUID." |
| T83 | JSON schemas for agy-hook & sync pull | ✅ **Complete** | Claude | [T83](tracks/trackT83-schemas-for-cli-commands/spec.md) | `agy-hook --schema` and `sync pull --schema` print JSON Schema 2020-12 documents; schemas at `Docs/schemas/` |
| T84 | Self-Healing / Auto-Restart Tooling | ✅ **Complete** | Claude | [T84](tracks/trackT84-self-healing-auto-restart/spec.md) | `daemon update` stops daemon gracefully (force if unresponsive), runs `cargo install`, restarts; `Build-AIBrains.ps1` does the same |
| T85 | Configuration-Based Backend URL and Port Status Checks | ✅ **Complete** | Claude | [T85](tracks/trackT85-config-based-port-status/spec.md) | `daemon status` reads `AI_BRAINS_MODEL_URL`/`AI_BRAINS_EMBEDDING_URL`, parses host:port, probes those; defaults to Ollama :11434 and llama.cpp :8080 |
| T86 | Structured Stdin for Pipeline Tooling | ✅ **Complete** | Claude | [T86](tracks/trackT86-structured-stdin/spec.md) | `recall -` reads query from stdin; `preflight --stdin` reads JSON `{"scope":[...],"max_words":N}` from stdin; TTY guard prevents hanging |
| T87 | Bridge:Vault Result Ratio in Recall | ⏳ **Pending** | — | [T87](tracks/trackT87-bridge-vault-recall-ratio/spec.md) | Cap bridge at ceil(limit/2) results; reserve remainder for vault FTS+semantic so vault memories surface |
| T88 | Fix `pin` to Print Projection `memory_id` | ⏳ **Pending** | — | [T88](tracks/trackT88-pin-prints-memory-id/spec.md) | Print `turn_id` (not `event_id`) in pin success message so `forget --memory-id` works with the reported UUID |
| T89 | `project set-alias` Command | ⏳ **Pending** | — | [T89](tracks/trackT89-project-set-alias/spec.md) | New command sets human-readable alias; unlocks `project detect` and `project resolve` |
| T90 | FTS5 Query Sanitization | ⏳ **Pending** | — | [T90](tracks/trackT90-fts5-query-sanitization/spec.md) | Wrap tokens in double-quotes before passing to FTS5/bridge; prevents syntax errors on `.`, `*`, `(`, etc. |
| T91 | Strip ANSI Before Ledger Search in `sync query` | ⏳ **Pending** | — | [T91](tracks/trackT91-strip-ansi-sync-query/spec.md) | Apply `strip_ansi_escapes` to query before passing to changeguard ledger search |
| T92 | Debug and Fix `sync pull --hotspots/--ledger` | ⏳ **Pending** | — | [T92](tracks/trackT92-sync-pull-hotspots-debug/spec.md) | Investigate NDJSON field mismatch between changeguard output and sync pull parser; fix to sync > 0 records |
| T93 | `project detect` Fallback to `.env` Project ID | ⏳ **Pending** | — | [T93](tracks/trackT93-project-detect-env-fallback/spec.md) | Fall back to `AI_BRAINS_PROJECT_ID` from `.env` when slug matching fails; exit 1 with clear message if neither works |
| UX | Friendly default project name | ✅ **Complete** | Claude | [UX](tracks/trackUX-friendly-default-project-name/spec.md) | Default name is `(no alias) — <8-char-uuid-prefix>` instead of `Project <full-uuid>`; full id still in dedicated column |
| Docs | WORKFLOWS.md cookbook | ✅ **Complete** | Claude | [Docs/WORKFLOWS.md](../Docs/WORKFLOWS.md) | 6 end-to-end recipes: setup, Antigravity import, hygiene, backup, code-search, daemon/nightly |


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
