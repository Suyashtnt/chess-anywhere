-- Add migration script here
-- Note to future self: keep in old email verification rows to prevent reusing email verification tokens
CREATE TABLE email_verification (
    id INTEGER PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    used INTEGER NOT NULL DEFAULT FALSE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    data BLOB NOT NULL,
    expiry_date INTEGER NOT NULL
) STRICT;

CREATE TABLE email_login (
    email TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id),
    UNIQUE(email, user_id)
) STRICT;