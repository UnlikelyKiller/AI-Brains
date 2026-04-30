-- Migration 0013: Relational Graph
-- Implements ADR-0009 using relational tables and recursive CTEs for multi-hop recall.

-- Central catalog of graph nodes to provide integer primary keys for fast join/recursion.
CREATE TABLE graph_node (
    node_id      INTEGER PRIMARY KEY,
    kind         TEXT NOT NULL CHECK (
        kind IN ('project', 'session', 'turn', 'memory', 'conflict', 'recipe')
    ),
    external_id  TEXT NOT NULL UNIQUE, -- The UUID from the domain model
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Edge table for relationship storage.
-- WITHOUT ROWID is used because we have a composite primary key.
CREATE TABLE graph_edge (
    src_id       INTEGER NOT NULL REFERENCES graph_node(node_id) ON DELETE CASCADE,
    label        TEXT NOT NULL CHECK (
        label IN (
            'IN_PROJECT',
            'IN_SESSION',
            'RECALLS',
            'SOURCE_FOR',
            'SYNTHESIZED_FROM',
            'CONFLICTS_WITH',
            'PART_OF_RECIPE'
        )
    ),
    dst_id       INTEGER NOT NULL REFERENCES graph_node(node_id) ON DELETE CASCADE,
    weight       REAL NOT NULL DEFAULT 1.0,
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (src_id, label, dst_id)
) WITHOUT ROWID;

-- Reverse traversal index for inbound lookups (e.g., Session <- Turn)
CREATE INDEX graph_edge_by_dst
    ON graph_edge(dst_id, label, src_id);
