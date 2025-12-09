-- migrate up
CREATE TABLE users
(
    id            UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    username         TEXT        NOT NULL UNIQUE CHECK (trim(username) <> ''),
    email         TEXT        NOT NULL UNIQUE CHECK (trim(email) <> ''),
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_email ON users (email);
