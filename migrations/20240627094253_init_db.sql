-- Add migration script here
CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    elo_rating REAL NOT NULL,
    elo_deviation REAL NOT NULL,
    elo_volatility REAL NOT NULL
) STRICT;