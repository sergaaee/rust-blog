use crate::domain::error::DomainError;
use crate::domain::user::User;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
}

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: User) -> Result<User, DomainError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to create user: {}", e);
            if e.as_database_error()
                .and_then(|db| db.constraint())
                .map(|c| c.contains("users_email"))
                == Some(true)
            {
                DomainError::UserAlreadyExists("email already registered".to_string())
            } else {
                DomainError::Internal(format!("database error: {}", e))
            }
        })?;

        info!(user_id = %user.id, email = %user.email, "user created");
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to find user by email {}: {}", email, e);
            DomainError::Internal(format!("database error: {}", e))
        })
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to find user by username {}: {}", username, e);
            DomainError::Internal(format!("database error: {}", e))
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to find user by id {}: {}", id, e);
            DomainError::Internal(format!("database error: {}", e))
        })
    }
}
