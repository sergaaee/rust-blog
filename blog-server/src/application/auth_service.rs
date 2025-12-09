use std::sync::Arc;

use tracing::instrument;

use crate::data::user_repository::UserRepository;
use crate::domain::{error::DomainError, user::User};
use crate::infrastructure::security::{JwtKeys, hash_password, verify_password};
use crate::presentation::dto::{LoginRequest, RegisterRequest};

#[derive(Clone)]
pub struct AuthService<R: UserRepository + 'static> {
    repo: Arc<R>,
    keys: JwtKeys,
}

impl<R> AuthService<R>
where
    R: UserRepository + 'static,
{
    pub fn new(repo: Arc<R>, keys: JwtKeys) -> Self {
        Self { repo, keys }
    }

    pub fn keys(&self) -> &JwtKeys {
        &self.keys
    }

    pub async fn get_user(&self, id: uuid::Uuid) -> Result<User, DomainError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::UserNotFound(id))
    }

    #[instrument(skip(self))]
    pub async fn register(
        &self,
        user: &RegisterRequest
    ) -> Result<User, DomainError> {
        let hash =
            hash_password(user.password.as_str()).map_err(|err| DomainError::Internal(err.to_string()))?;
        let user = User::new(user.username.to_string(), user.email.as_str().to_lowercase(), hash);
        self.repo.create(user).await
    }

    #[instrument(skip(self))]
    pub async fn login(&self, login: &LoginRequest) -> Result<String, DomainError> {
        let user = self
            .repo
            .find_by_email(&login.email.as_str().to_lowercase())
            .await?
            .ok_or(DomainError::Unauthorized)?;

        let valid = verify_password(login.password.as_str(), &user.password_hash)
            .map_err(|_| DomainError::Unauthorized)?;
        if !valid {
            return Err(DomainError::Unauthorized);
        }

        self.keys
            .generate_token(user.id)
            .map_err(|err| DomainError::Internal(err.to_string()))
    }
}
