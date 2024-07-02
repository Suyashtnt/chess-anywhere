-- Add migration script here
-- How did I forget this
ALTER TABLE email_verification
ADD COLUMN email text NOT NULL;