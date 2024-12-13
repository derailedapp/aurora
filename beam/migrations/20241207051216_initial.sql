CREATE TABLE actors (
    id TEXT NOT NULL PRIMARY KEY,
    display_name TEXT,
    avatar TEXT,
    banner TEXT,
    bio TEXT,
    status TEXT
);
CREATE TABLE followings (
    user_id TEXT NOT NULL,
    other_user_id TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (other_user_id) REFERENCES actors(id) ON DELETE CASCADE
);
CREATE TABLE tracks (
    id TEXT NOT NULL PRIMARY KEY,
    author_id TEXT NOT NULL,
    content TEXT,
    ts BIGINT NOT NULL,
    original_ts BIGINT NOT NULL,
    parent_id TEXT,
    FOREIGN KEY (parent_id) REFERENCES tracks(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES actors(id) ON DELETE CASCADE
);
CREATE TABLE track_topics (
    id TEXT NOT NULL,
    topic TEXT NOT NULL,
    FOREIGN KEY (id) REFERENCES tracks(id) ON DELETE CASCADE,
    PRIMARY KEY (id, topic)
);