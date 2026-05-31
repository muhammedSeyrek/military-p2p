
CREATE TABLE operation_recipients (
    id             UUID PRIMARY KEY,
    operation_id   UUID NOT NULL REFERENCES operations(id) ON DELETE CASCADE,
    commander_id   UUID NOT NULL REFERENCES commanders(id),
    leaf_hash      BYTEA NOT NULL,
    part_index     INTEGER NOT NULL CHECK (part_index >= 0),

    CONSTRAINT leaf_hash_size CHECK (octet_length(leaf_hash) = 32),
    UNIQUE (operation_id, commander_id),
    UNIQUE (operation_id, part_index)
);

CREATE INDEX idx_recipients_op ON operation_recipients(operation_id);
CREATE INDEX idx_recipients_cmd ON operation_recipients(commander_id);