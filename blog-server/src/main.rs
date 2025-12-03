mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::web::service;
use actix_web::{App, HttpServer, web};
use application::auth_service::AuthService;
use data::user_repository::PostgresUserRepository;
use infrastructure::config::AppConfig;
use infrastructure::database::{create_pool, run_migrations};
use infrastructure::logging::init_logging;
use infrastructure::security::JwtKeys;
use presentation::handlers;
use presentation::middleware::{JwtAuthMiddleware, RequestIdMiddleware, TimingMiddleware};
use reqwest::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    let auth_service = AuthService::new(
        Arc::clone(&user_repo),
        JwtKeys::new(config.jwt_secret.clone()),
    );
    let post_service = PostService::new(Arc::clone(&post_repo));

    let config_data = config.clone();

    HttpServer::new(move || {
        let cors = build_cors(&config_data);
        App::new()
            .wrap(Logger::default())
            .wrap(RequestIdMiddleware)
            .wrap(TimingMiddleware)
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Referrer-Policy", "no-referrer"))
                    .add(("Permissions-Policy", "geolocation=()"))
                    .add(("Cross-Origin-Opener-Policy", "same-origin")),
            )
            .wrap(cors)
            .app_data(web::Data::new(post_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .service(
                web::scope("/api")
                    .service(handlers::auth::scope())
                    .service(handlers::post::get_posts)
                    .service(
                        web::scope("")
                            .wrap(JwtAuthMiddleware::new(auth_service.keys().clone()))
                            .service(handlers::post::create_post)
                            .service(handlers::post::delete_post)
                            .service(handlers::post::update_post),
                    ),
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}

fn build_cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::AUTHORIZATION,
        ])
        .supports_credentials()
        .max_age(3600);

    for origin in &config.cors_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}
