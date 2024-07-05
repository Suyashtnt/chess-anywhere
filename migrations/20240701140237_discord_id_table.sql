-- Add migration script here
CREATE TABLE discord_id (
    discord_id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id),
    UNIQUE(discord_id, user_id)
);