-- Add migration script here
CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) NOT NULL UNIQUE,
    discord_id NUMERIC UNIQUE,
    elo_rating DOUBLE PRECISION NOT NULL,
    elo_deviation DOUBLE PRECISION NOT NULL,
    elo_volatility DOUBLE PRECISION NOT NULL
)