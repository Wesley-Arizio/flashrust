CREATE TABLE IF NOT EXISTS credentials (
    id TEXT NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    active BOOLEAN DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT NOT NULL PRIMARY KEY,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    expires_at INTEGER NOT NULL,
    credential_id TEXT NOT NULL,
    active BOOLEAN DEFAULT TRUE,
    FOREIGN KEY (credential_id) REFERENCES credentials(id) ON DELETE CASCADE
);