-- migrate up
CREATE TABLE users
(
    id            UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    login         TEXT        NOT NULL UNIQUE CHECK (trim(login) <> ''),
    email         TEXT        NOT NULL UNIQUE CHECK (trim(email) <> ''),
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_login ON users (login);
CREATE INDEX idx_users_email ON users (email);
