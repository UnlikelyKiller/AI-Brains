-- Migration 0018: Memory Embedding
-- Add embedding column to memory_projection for semantic search support.
ALTER TABLE memory_projection ADD COLUMN embedding BLOB;
