-- Add migration script here
-- Used to check if the verification token has been used
ALTER TABLE email_verification
ADD COLUMN used BOOLEAN NOT NULL DEFAULT FALSE;