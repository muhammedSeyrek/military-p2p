
CREATE TABLE operations (
    id                    UUID PRIMARY KEY,
    name                  TEXT NOT NULL,
    encrypted_aes_key     BYTEA NOT NULL,
    merkle_root           BYTEA NOT NULL,
    leaf_hash             BYTEA NOT NULL,
    total_parts           INTEGER NOT NULL CHECK (total_parts > 0),
    part_index            INTEGER NOT NULL CHECK (part_index >= 0),
    received_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT merkle_root_size CHECK (octet_length(merkle_root) = 32),
    CONSTRAINT leaf_hash_size CHECK (octet_length(leaf_hash) = 32)
);

CREATE INDEX idx_op_name ON operations(name);
CREATE INDEX idx_op_received_at ON operations(received_at DESC);