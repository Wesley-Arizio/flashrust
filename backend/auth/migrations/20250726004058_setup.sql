-- Add migration script here
CREATE TABLE IF NOT EXISTS credentials (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    active BOOLEAN DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    credential_id UUID NOT NULL,
    active BOOLEAN DEFAULT TRUE,
    CONSTRAINT fk_credentials FOREIGN KEY (credential_id) REFERENCES credentials(id) ON DELETE CASCADE
);