use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::domain::error::DomainError;
use crate::presentation::dto::{AuthResponse, LoginRequest, RegisterRequest};
use actix_web::{HttpResponse, Responder, Scope, post, web};
use tracing::info;

pub fn scope() -> Scope {
    web::scope("")
        .service(register)
        .service(login)
        .service(token)
}

#[post("/auth/register")]
async fn register(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<RegisterRequest>,
) -> Result<impl Responder, DomainError> {
    let user = service
        .register(
            payload.username.clone(),
            payload.email.clone(),
            payload.password.clone(),
        )
        .await?;

    info!(user_id = %user.id, email = %user.email, "user registered");

    Ok(HttpResponse::Created().json(serde_json::json!({
        "username": user.username,
        "user_id": user.id,
        "email": user.email
    })))
}

#[post("/auth/login")]
async fn login(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, DomainError> {
    let expires_in = 3600; // 1 час
    let jwt = service
        .login(payload.email.as_str(), &payload.password)
        .await?;
    info!(email = %payload.email, "user logged in");
    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token: jwt,
        expires_in,
        token_type: "Bearer".to_string(),
    }))
}

#[post("/auth/token")]
async fn token(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, DomainError> {
    let expires_in = 3600; // 1 час
    let jwt = service.login(&payload.email, &payload.password).await?;
    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token: jwt,
        expires_in,
        token_type: "Bearer".to_string(),
    }))
}
