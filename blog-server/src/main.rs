mod application;
mod data;
mod domain;
mod infrastructure;
pub mod presentation;
mod utils;

use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::utils::{start_grpc_server, start_rest_server};
use application::auth_service::AuthService;
use data::user_repository::PostgresUserRepository;
use infrastructure::config::AppConfig;
use infrastructure::database::{create_pool, run_migrations};
use infrastructure::logging::init_logging;
use infrastructure::security::JwtKeys;
pub use presentation::dto::AuthResponse;

use std::sync::Arc;

pub mod blog {
    tonic::include_proto!("blog");
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let config = AppConfig::from_env().expect("invalid configuration");
    let pool = create_pool(&config.database_url)
        .await
        .expect("failed to connect to database");
    run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()));

    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&user_repo),
        JwtKeys::new(config.jwt_secret.clone()),
    ));

    let post_service = Arc::new(PostService::new(Arc::clone(&post_repo)));

    tokio::try_join!(
        start_rest_server(auth_service.clone(), post_service.clone()),
        start_grpc_server(auth_service, post_service),
    )?;

    Ok(())
}
