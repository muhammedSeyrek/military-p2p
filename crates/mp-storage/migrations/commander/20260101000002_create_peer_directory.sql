CREATE TABLE peer_directory (
    commander_id       UUID PRIMARY KEY,
    full_name          TEXT NOT NULL,
    email              TEXT NOT NULL UNIQUE,
    rank               TEXT NOT NULL,
    public_key_pem     TEXT NOT NULL,
    network_address    TEXT NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_peer_email ON peer_directory(email);