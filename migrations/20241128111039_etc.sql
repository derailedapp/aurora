CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE TABLE guild_invites (
    -- ~12345abcde@example.com
    id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    FOREIGN KEY (guild_id) REFERENCES guilds(id)
);