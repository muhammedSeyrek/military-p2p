
CREATE TABLE commanders (
    id               UUID PRIMARY KEY,
    full_name        TEXT NOT NULL,
    email            TEXT NOT NULL UNIQUE,
    rank             TEXT NOT NULL,
    public_key_pem   TEXT NOT NULL,
    unit_id          UUID REFERENCES units(id) ON DELETE SET NULL,
    network_address  TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_commanders_email ON commanders(email);
CREATE INDEX idx_commanders_rank ON commanders(rank);
CREATE INDEX idx_commanders_unit ON commanders(unit_id);