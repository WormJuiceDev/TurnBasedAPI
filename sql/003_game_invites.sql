-- FriendBox game invitation flow.
-- Paste and run this in DBeaver while connected to friendbox_db.

CREATE TABLE IF NOT EXISTS game_invites (
    id UUID PRIMARY KEY,
    inviter_player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    invitee_player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    client_game_type TEXT NOT NULL DEFAULT 'friendbox',
    status TEXT NOT NULL DEFAULT 'pending',
    initial_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    game_id UUID REFERENCES games(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,
    CHECK (status IN ('pending', 'accepted', 'declined')),
    CHECK (inviter_player_id <> invitee_player_id)
);

CREATE INDEX IF NOT EXISTS idx_game_invites_invitee_status
    ON game_invites(invitee_player_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_game_invites_inviter
    ON game_invites(inviter_player_id, created_at DESC);

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO friendbox_user;
GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO friendbox_user;
