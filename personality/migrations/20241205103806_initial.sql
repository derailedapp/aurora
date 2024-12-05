CREATE TABLE IF NOT EXISTS actors (
    id TEXT NOT NULL PRIMARY KEY,
    display_name TEXT,
    avatar TEXT,
    banner TEXT,
    bio TEXT,
    status TEXT
);
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    flags INTEGER NOT NULL,
    theme TEXT NOT NULL,
    pickle TEXT NOT NULL,
    ed_key TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tracks (
    id TEXT NOT NULL PRIMARY KEY,
    author_id TEXT,
    content TEXT,
    -- abcde1234/post_id
    parent_id TEXT,
    FOREIGN KEY (author_id) REFERENCES actors(id) ON DELETE SET NULL
);
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE SET NULL
);