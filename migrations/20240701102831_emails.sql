-- Add migration script here
-- Note to future self: keep in old email verification rows to prevent reusing email verification tokens
CREATE TABLE email_verification (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    data bytea NOT NULL,
    expiry_date timestamptz NOT NULL
);

CREATE TABLE email_login (
    email text PRIMARY KEY NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id)
);