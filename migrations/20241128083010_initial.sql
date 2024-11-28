CREATE TABLE servers (
    id TEXT NOT NULL PRIMARY KEY,
    -- example.com
    name TEXT NOT NULL,
    -- example.com/api | api.example.com
    api_url TEXT NOT NULL
);
CREATE TABLE users (
    -- @12345abcde@example.com
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    password TEXT,
    flags INTEGER,
    server_id TEXT,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);
CREATE TABLE user_settings (
    id TEXT PRIMARY KEY,
    theme TEXT NOT NULL,
    FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE guilds (
    -- !12345abcde@example.com
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    server_id TEXT,
    permissions BIGINT,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);
CREATE TABLE guild_members (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    server_id TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    PRIMARY KEY (guild_id, user_id)
);
CREATE TABLE channels (
    -- !12345abcde#12345abcde@example.com
    id TEXT PRIMARY KEY,
    server_id TEXT,
    guild_id TEXT NOT NULL,
    name TEXT NOT NULL,
    last_message_id TEXT,
    FOREIGN KEY (guild_id) REFERENCES guilds(id) ON DELETE CASCADE,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);
-- Messages are only cached by servers not stored.
CREATE TABLE messages (
    -- !12345abcde#12345abcde+12345abcde@example.com
    id TEXT PRIMARY KEY,
    author_id TEXT,
    channel_id TEXT NOT NULL,
    content TEXT NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
)