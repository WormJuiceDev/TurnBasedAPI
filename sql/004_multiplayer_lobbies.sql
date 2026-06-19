-- FriendBox multiplayer lobby flow.
-- Paste and run this in DBeaver while connected to friendbox_db after 003.

CREATE INDEX IF NOT EXISTS idx_game_invites_game_id
    ON game_invites(game_id);

CREATE INDEX IF NOT EXISTS idx_games_owner_status
    ON games(owner_player_id, status, updated_at DESC);

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO friendbox_user;
GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO friendbox_user;
