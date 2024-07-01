-- Add migration script here
CREATE TABLE discord_id (
    discord_id NUMERIC PRIMARY KEY NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id)
);

-- Transfer existing discord_id data from users table to discord_id table
INSERT INTO discord_id (discord_id, user_id)
SELECT discord_id, id
FROM users;

-- Remove discord_id column from users table
ALTER TABLE users
DROP COLUMN discord_id;
