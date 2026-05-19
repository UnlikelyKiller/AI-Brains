-- Add tx_id and project_id to core projections for ChangeGuard linking
ALTER TABLE project_projection ADD COLUMN tx_id TEXT;
ALTER TABLE session_projection ADD COLUMN tx_id TEXT;
ALTER TABLE turn_projection ADD COLUMN project_id TEXT;
ALTER TABLE turn_projection ADD COLUMN tx_id TEXT;
ALTER TABLE memory_projection ADD COLUMN tx_id TEXT;

CREATE INDEX idx_project_tx ON project_projection(tx_id);
CREATE INDEX idx_session_tx ON session_projection(tx_id);
CREATE INDEX idx_turn_project ON turn_projection(project_id);
CREATE INDEX idx_turn_tx ON turn_projection(tx_id);
CREATE INDEX idx_memory_tx ON memory_projection(tx_id);
