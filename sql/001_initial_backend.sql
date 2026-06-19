-- FriendBox initial backend schema
-- Paste and run this in DBeaver while connected to friendbox_db.
--
-- Backend boundary:
-- The server stores authenticated players, sessions, games, opaque game state,
-- and opaque turn payloads. It does not enforce game rules, decide legal moves,
-- calculate winners, or interpret the JSON gameplay data.

CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY,
    device_id TEXT UNIQUE,
    username TEXT UNIQUE,
    password_hash TEXT,
    display_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS player_sessions (
    token TEXT PRIMARY KEY,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_player_sessions_player_id
    ON player_sessions(player_id);

CREATE TABLE IF NOT EXISTS games (
    id UUID PRIMARY KEY,
    owner_player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    client_game_type TEXT NOT NULL DEFAULT 'friendbox',
    status TEXT NOT NULL DEFAULT 'active',
    state JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_games_owner_player_id
    ON games(owner_player_id);

CREATE INDEX IF NOT EXISTS idx_games_updated_at
    ON games(updated_at DESC);

CREATE TABLE IF NOT EXISTS game_participants (
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (game_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_game_participants_player_id
    ON game_participants(player_id);

CREATE TABLE IF NOT EXISTS turns (
    id UUID PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    client_turn_id TEXT,
    sequence_no INTEGER NOT NULL,
    turn_payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (game_id, sequence_no),
    UNIQUE (game_id, client_turn_id)
);

CREATE INDEX IF NOT EXISTS idx_turns_game_id_sequence_no
    ON turns(game_id, sequence_no);
