-- Add migration script here
CREATE TABLE email_verification (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    data bytea NOT NULL,
    expiry_date timestamptz NOT NULL
);

-- add email column to users table
ALTER TABLE users
    -- emails are too chaotic to add any form of validation so the validation is send an email and hope
    ADD COLUMN email TEXT;