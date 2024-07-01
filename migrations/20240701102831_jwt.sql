-- Add migration script here
CREATE TABLE jwt (
    id SERIAL PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id),
    -- Token to get from email
    token TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- add email column to users table
CREATE EXTENSION plperlu;

CREATE FUNCTION valid_email(text)
    RETURNS boolean
    LANGUAGE plperlu
    IMMUTABLE LEAKPROOF STRICT AS
$$
    use Email::Valid;
    my $email = shift;
    Email::Valid->address($email) or die "Invalid email address: $email\n";
    return 'true';
$$;

CREATE DOMAIN email AS text NOT NULL
    CONSTRAINT valid_email CHECK (valid_email(VALUE));

ALTER TABLE users
    ADD COLUMN email email UNIQUE;