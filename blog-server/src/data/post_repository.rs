use crate::domain::error::DomainError;
use crate::domain::post::Post;
use crate::presentation::dto::UpdatePostRequest;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, user: Post) -> Result<Post, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Post>, DomainError>;
    async fn update_post(
        &self,
        id: Uuid,
        author_id: Uuid,
        update: UpdatePostRequest,
    ) -> Result<Option<Post>, DomainError>;
    async fn delete_post(&self, id: Uuid, author_id: Uuid) -> Result<(), DomainError>;
    async fn get_posts(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError>;
}

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, post: Post) -> Result<Post, DomainError> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO posts (id, author_id, title, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            "#,
        )
        .bind(post.id)
        .bind(post.author_id)
        .bind(&post.title)
        .bind(&post.content)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to create post: {}", e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        info!(post_id = %post.id, author_id = %post.author_id, "post created");
        Ok(post)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Post>, DomainError> {
        sqlx::query_as::<_, Post>(
            r#"
            SELECT id, author_id, title, content, created_at, updated_at
            FROM posts WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("db error find_by_id {}: {}", id, e);
            DomainError::Internal(e.to_string())
        })
    }

    async fn update_post(
        &self,
        id: Uuid,
        author_id: Uuid,
        update: UpdatePostRequest,
    ) -> Result<Option<Post>, DomainError> {
        let now = Utc::now();
        let post = sqlx::query_as::<_, Post>(
            r#"
            UPDATE posts
            SET
                title = COALESCE($1, title),
                content = COALESCE($2, content),
                updated_at = $3
            WHERE id = $4 AND author_id = $5
            RETURNING id, author_id, title, content, created_at, updated_at
            "#,
        )
        .bind(update.title)
        .bind(update.content)
        .bind(now)
        .bind(id)
        .bind(author_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to update post {}: {}", id, e);
            DomainError::Internal(e.to_string())
        })?;

        if post.is_some() {
            info!(post_id = %id, "post updated");
        }

        Ok(post)
    }

    async fn delete_post(&self, id: Uuid, author_id: Uuid) -> Result<(), DomainError> {
        let deleted = sqlx::query("DELETE FROM posts WHERE id = $1 AND author_id = $2")
            .bind(id)
            .bind(author_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if deleted.rows_affected() == 0 {
            // Проверяем существование
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1)")
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;

            return if exists {
                Err(DomainError::Forbidden)
            } else {
                Err(DomainError::PostNotFound(id))
            };
        }

        info!(post_id = %id, "post deleted");
        Ok(())
    }

    async fn get_posts(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError> {
        let limit = limit.unwrap_or(10).min(100) as i64;
        let offset = offset.unwrap_or(0) as i64;

        sqlx::query_as::<_, Post>(
            r#"
        SELECT id, author_id, title, content, created_at, updated_at
        FROM posts
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool) // ← fetch_all, not fetch_optional!
        .await
        .map_err(|e| {
            error!("db error while fetching posts: {}", e);
            DomainError::Internal(e.to_string())
        })
    }
}
