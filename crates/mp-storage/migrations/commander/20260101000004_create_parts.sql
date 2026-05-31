
CREATE TABLE parts (
    id                  UUID PRIMARY KEY,
    operation_id        UUID NOT NULL REFERENCES operations(id) ON DELETE CASCADE,
    part_index          INTEGER NOT NULL CHECK (part_index >= 0),
    ciphertext_chunk    BYTEA NOT NULL,

    UNIQUE (operation_id, part_index)
);

CREATE INDEX idx_parts_op ON parts(operation_id);