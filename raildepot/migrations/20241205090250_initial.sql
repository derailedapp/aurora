CREATE TABLE IF NOT EXISTS identifiers (
    id TEXT NOT NULL PRIMARY KEY,
    handle TEXT,
    server TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS public_keys (
    id TEXT NOT NULL,
    key TEXT NOT NULL,
    FOREIGN KEY (id) REFERENCES identifiers(id) ON DELETE CASCADE,
    PRIMARY KEY (id, key)
);