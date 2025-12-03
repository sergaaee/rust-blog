use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, error::ErrorUnauthorized};
use futures_util::future::{Ready, ready};
use uuid::Uuid;

use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::domain::error::DomainError;
use crate::infrastructure::security::JwtKeys;

pub fn ensure_owner(item_id: &Uuid, user_id: &Uuid) -> Result<(), DomainError> {
    if item_id != user_id {
        Err(DomainError::Unauthorized)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>() {
            Some(user) => ready(Ok(user.clone())),
            None => ready(Err(ErrorUnauthorized("missing authenticated user"))),
        }
    }
}

pub async fn extract_user_from_token(
    token: &str,
    keys: &JwtKeys,
    auth_service: &AuthService<PostgresUserRepository>,
) -> Result<AuthenticatedUser, Error> {
    let claims = keys
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| ErrorUnauthorized("invalid token"))?;

    let user = auth_service
        .get_user(user_id)
        .await
        .map_err(|_| ErrorUnauthorized("user not found"))?;

    Ok(AuthenticatedUser {
        id: user.id,
        username: user.username,
    })
}
