-- FriendBox username/password auth migration.
-- Paste and run this in DBeaver while connected to friendbox_db.

ALTER TABLE players
    ALTER COLUMN device_id DROP NOT NULL;

ALTER TABLE players
    ADD COLUMN IF NOT EXISTS username TEXT;

ALTER TABLE players
    ADD COLUMN IF NOT EXISTS password_hash TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_players_username
    ON players(username)
    WHERE username IS NOT NULL;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO friendbox_user;
GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO friendbox_user;
