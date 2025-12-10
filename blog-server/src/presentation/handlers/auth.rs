use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::domain::error::DomainError;
use crate::presentation::dto::{AuthResponse, LoginRequest, RegisterRequest};
use actix_web::{HttpResponse, Responder, Scope, post, web};
use std::sync::Arc;
use tracing::info;

pub fn scope() -> Scope {
    web::scope("/auth")
        .service(register)
        .service(login)
        .service(token)
}

#[post("/register")]
async fn register(
    service: web::Data<Arc<AuthService<PostgresUserRepository>>>,
    payload: web::Json<RegisterRequest>,
) -> Result<impl Responder, DomainError> {
    let user = service.register(&payload.0).await?;

    info!(user_id = %user.id, email = %user.email, "user registered");

    let log_req = LoginRequest {
        username: user.username,
        password: payload.password.clone(),
    };

    let expires_in = 3600 * 24; // 24 часа
    let jwt = service.login(&log_req).await?;

    info!(username = %log_req.username, "user logged in");

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token: jwt,
        expires_in,
        token_type: "Bearer".to_string(),
    }))
}

#[post("/login")]
async fn login(
    service: web::Data<Arc<AuthService<PostgresUserRepository>>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, DomainError> {
    dotenvy::dotenv().ok();
    let expires_in: i64 = std::env::var("ACCESS_TOKEN_EXPIRATION_SECS")
        .expect("Missing access token expiration time")
        .parse()
        .expect("ACCESS_TOKEN_EXPIRATION_SECS must be an integer");

    let jwt = service.login(&payload.0).await?;

    info!(username = %payload.username, "user logged in");

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token: jwt,
        expires_in,
        token_type: "Bearer".to_string(),
    }))
}

#[post("/token")]
async fn token(
    service: web::Data<Arc<AuthService<PostgresUserRepository>>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, DomainError> {
    dotenvy::dotenv().ok();
    let expires_in: i64 = std::env::var("ACCESS_TOKEN_EXPIRATION_SECS")
        .expect("Missing access token expiration time")
        .parse()
        .expect("ACCESS_TOKEN_EXPIRATION_SECS must be an integer");

    let jwt = service.login(&payload.0).await?;
    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token: jwt,
        expires_in,
        token_type: "Bearer".to_string(),
    }))
}
