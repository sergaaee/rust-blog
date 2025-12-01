-- Add up migration script here
CREATE TABLE posts
(
    id         UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    author_id  UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,

    title      TEXT        NOT NULL CHECK (trim(title) <> ''),
    content    TEXT        NOT NULL CHECK (trim(content) <> ''),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_title CHECK (length(trim(title)) >= 1),
    CONSTRAINT valid_content CHECK (length(trim(content)) >= 1)
);

-- Автоматическое обновление updated_at
CREATE OR REPLACE FUNCTION update_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_posts_update_timestamp
    BEFORE UPDATE
    ON posts
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at();

CREATE INDEX idx_posts_author_id ON posts (author_id);
CREATE INDEX idx_posts_created_at ON posts (created_at DESC);
CREATE INDEX idx_posts_author_created ON posts (author_id, created_at DESC);