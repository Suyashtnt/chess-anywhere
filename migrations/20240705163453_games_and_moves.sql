-- Add migration script here
CREATE TABLE games (
    id INTEGER PRIMARY KEY NOT NULL,
    white_id INTEGER NOT NULL REFERENCES users(id),
    black_id INTEGER NOT NULL REFERENCES users(id),
    -- W = white win, B = black win, D = draw, Null = ongoing
    outcome TEXT CHECK(outcome IN ('W', 'B', 'D')),
    created_at INTEGER NOT NULL DEFAULT (unixepoch('now'))
) STRICT;

CREATE TABLE moves (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL REFERENCES games(id),
    player_id INTEGER NOT NULL REFERENCES users(id),
    move_number INTEGER NOT NULL,
    move TEXT NOT NULL,
    played_at INTEGER NOT NULL DEFAULT (unixepoch('now'))
) STRICT;
