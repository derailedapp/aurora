ALTER TABLE channels
ADD position INTEGER NOT NULL;
-- DM channels and GCs
ALTER TABLE channels
ALTER COLUMN guild_id DROP NOT NULL;