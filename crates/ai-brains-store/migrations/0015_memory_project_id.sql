ALTER TABLE memory_projection ADD COLUMN project_id TEXT;
CREATE INDEX idx_memory_project ON memory_projection(project_id);
