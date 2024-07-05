-- Add migration script here
CREATE TABLE games (
    id INTEGER PRIMARY KEY NOT NULL,
    white_id INTEGER NOT NULL REFERENCES users(id),
    black_id INTEGER NOT NULL REFERENCES users(id),
    -- 0 = draw, 1 = white, 2 = black, null = ongoing
    outcome INTEGER,
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