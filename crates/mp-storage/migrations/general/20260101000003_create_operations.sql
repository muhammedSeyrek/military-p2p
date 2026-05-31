
CREATE TABLE operations (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL,
    merkle_root     BYTEA NOT NULL,
    total_parts     INTEGER NOT NULL CHECK (total_parts > 0),
    status          TEXT NOT NULL DEFAULT 'dispatched',
    created_by      UUID NOT NULL REFERENCES commanders(id),
    dispatched_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT merkle_root_size CHECK (octet_length(merkle_root) = 32)
);

CREATE INDEX idx_operations_name ON operations(name);
CREATE INDEX idx_operations_status ON operations(status);
CREATE INDEX idx_operations_created_at ON operations(created_at DESC);