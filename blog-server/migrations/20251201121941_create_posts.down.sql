-- Add down migration script here
DROP TRIGGER IF EXISTS trg_posts_update_timestamp ON posts;
DROP FUNCTION IF EXISTS update_updated_at();
DROP TABLE posts;
