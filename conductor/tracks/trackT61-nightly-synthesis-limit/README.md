# Track T61: Limit Nightly Synthesis Batch Size

## Problem
The nightly intelligence sweep hangs indefinitely because `MemorySynthesizer::run_synthesis()` fetches **ALL** memories at the source level (8,447 level-0 memories) and processes them in chunks of 5, with each chunk requiring ~70s (synthesis + verification). Total: ~32 hours.

## Solution
Add a `limit` parameter to `QueryStore::get_memories_by_level()` and cap synthesis to a reasonable batch (e.g., most recent 100 memories).

## Files to Change
1. `crates/ai-brains-store/src/lib.rs` — Update `QueryStore` trait signature
2. `crates/ai-brains-store/src/query_store.rs` — Add LIMIT to SQL, order by recency
3. `crates/ai-brains-brain/src/memory_synthesis.rs` — Pass batch size limit
4. `crates/ai-brains-brain/src/lib.rs` — Update `run_synthesis` call

## Testing
- Run `ai-brains nightly --status` before and after
- Run a truncated nightly and verify it completes in < 5 minutes
