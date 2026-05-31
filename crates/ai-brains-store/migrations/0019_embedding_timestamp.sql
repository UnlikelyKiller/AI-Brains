-- Add embedding_generated_at column for tracking stale embeddings
ALTER TABLE memory_projection ADD COLUMN embedding_generated_at TEXT;

-- Update existing embedded memories to set timestamp if missing
UPDATE memory_projection
SET embedding_generated_at = updated_at
WHERE embedding IS NOT NULL
  AND embedding_generated_at IS NULL;
