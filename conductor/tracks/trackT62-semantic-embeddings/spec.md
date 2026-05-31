# Track T62: Semantic Search — Stored Embeddings

## Problem Statement
Semantic search (`recall --semantic`) structurally fails because:
1. **Migration `0018_memory_embedding.sql` not applied** — `memory_projection` table lacks `embedding BLOB` column
2. **No embedding generation pipeline** — 8,447 pinned memories have `embedding IS NULL`; Nomic server on 8083 is ready but unused for bulk population
3. **Nightly doesn't generate embeddings** — synthesis creates new memories but doesn't populate embeddings for them

## Goals
1. Apply migration `0018` to add `embedding` column
2. Build batch embedding generation for existing memories
3. Integrate embedding generation into nightly pipeline
4. Verify `--semantic` recall returns results

## Success Criteria
- `recall "deployment" --semantic --limit 3` returns ≥1 hit with `source: "semantic"`
- Nightly completes synthesis AND populates embeddings for synthesized memories
- ≥100 memories have non-null embeddings within 24 hours

## Technical Design

### Migration Application
- Let the built-in migration runner handle it by running `preflight` or `recall` from Windows PowerShell (not WSL)
- Migration runner tracks state in `schema_migrations` table
- `0018_memory_embedding.sql`: `ALTER TABLE memory_projection ADD COLUMN embedding BLOB;`

### Embedding Generation Strategy
- **Option A: Extend nightly** — After synthesis, generate embeddings for all new level-1 memories + backfill level-0
- **Option B: Standalone batch job** — One-time pass over all pinned memories, generate embeddings via Nomic API
- **Recommended: Option A + partial B** — Nightly handles new memories going forward; standalone script handles one-time backfill of top 500 most recent

### Nomic API
- Endpoint: `POST http://localhost:8083/v1/embeddings`
- Model: `nomic-embed-text-v1.5`
- Input: memory content string
- Output: 768-dim float32 vector
- Store: Little-endian f32 bytes in `embedding BLOB`

## Implementation Plan
### Phase 1: Apply Migration (5 min)
1. Run Windows binary to trigger migration runner
2. Verify column exists via PRAGMA

### Phase 2: Build Batch Embedder (30 min)
1. Create `scripts/backfill-embeddings.py`
2. Query memories with `embedding IS NULL`
3. Call Nomic API for each (or batch)
4. Store BLOBs back to SQLite
5. Run from Windows PowerShell (avoid WSL WAL lock)

### Phase 3: Integrate into Nightly (15 min)
1. After `synthesize_cluster`, generate embedding for the synthesized memory
2. After nightly completes, backfill any remaining null embeddings for recent memories

### Phase 4: Verification (10 min)
1. Run `recall --semantic` with known query
2. Confirm `source: "semantic"` appears in results
3. Compare with FTS5-only results

## Files
- `crates/ai-brains-store/migrations/0018_memory_embedding.sql` (already exists)
- `scripts/backfill-embeddings.py` (new)
- `crates/ai-brains-brain/src/memory_synthesis.rs` (modify)
- `crates/ai-brains-cli/src/commands/nightly.rs` (modify)

## Verification
```powershell
# After Phase 1
ai-brains --vault-path C:\devi-brainsault.db recall "test" --limit 1
# Should not crash with "no such column: embedding"

# After Phase 2
python3 C:\dev\AI-Brains\scriptsackfill-embeddings.py --limit 100
# Should report 100 embeddings generated and stored

# After Phase 4
ai-brains --vault-path C:\devi-brainsault.db recall "KinLedger" --semantic --limit 3
# Should include results with source: "semantic"
```
