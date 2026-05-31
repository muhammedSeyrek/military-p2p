
CREATE TABLE units (
    id            UUID PRIMARY KEY,
    corps_number  INTEGER NOT NULL,
    name          TEXT NOT NULL,
    unit_type     TEXT NOT NULL,
    location      TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_units_corps ON units(corps_number);
CREATE INDEX idx_units_location ON units(location);