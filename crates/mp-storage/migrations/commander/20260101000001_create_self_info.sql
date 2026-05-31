
CREATE TABLE self_info (
    id                 UUID PRIMARY KEY,
    commander_id       UUID NOT NULL UNIQUE,
    full_name          TEXT NOT NULL,
    email              TEXT NOT NULL UNIQUE,
    rank               TEXT NOT NULL,
    password_hash      TEXT NOT NULL,
    private_key_pem    TEXT NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);