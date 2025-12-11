use crate::application::auth_service::AuthService;
use crate::application::post_service::PostService;
use crate::blog;
use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::infrastructure::config::AppConfig;
use crate::presentation::grpc_service::BlogGrpcService;
use crate::presentation::handlers;
use crate::presentation::middleware::{JwtAuthMiddleware, RequestIdMiddleware, TimingMiddleware};
use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use tokio::signal;
use tonic::transport::Server;

pub async fn start_rest_server<
    R: UserRepository + Send + Sync + 'static,
    T: PostRepository + Send + Sync + 'static,
>(
    auth_service: Arc<AuthService<R>>,
    post_service: Arc<PostService<T>>,
) -> anyhow::Result<()> {
    let config = AppConfig::from_env().expect("invalid configuration");
    let config_bind = AppConfig::from_env().expect("invalid configuration");
    let bind_address = (config_bind.host.as_str(), config_bind.port);

    println!(
        "HTTP server starting on http://{}:{}",
        bind_address.0, bind_address.1
    );

    HttpServer::new(move || {
        let cors = build_cors(&config);

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
                    .route("/health", web::get().to(health))
                    .service(handlers::post::get_posts)
                    .service(handlers::post::get_post)
                    .service(
                        web::scope("/posts")
                            .wrap(JwtAuthMiddleware::new(auth_service.keys().clone()))
                            .service(handlers::post::create_post)
                            .service(handlers::post::delete_post)
                            .service(handlers::post::update_post),
                    )
                    .service(handlers::auth::scope()),
            )
    })
    .bind(bind_address)?
    .run()
    .await
    .map_err(anyhow::Error::new)?;

    Ok(())
}

pub async fn start_grpc_server<
    R: UserRepository + Send + Sync + 'static,
    T: PostRepository + Send + Sync + 'static,
>(
    auth_service: Arc<AuthService<R>>,
    post_service: Arc<PostService<T>>,
) -> anyhow::Result<()> {
    let addr = "0.0.0.0:50051".parse().unwrap();

    let grpc_service = BlogGrpcService::new(auth_service, post_service);

    println!("gRPC server starting on {}", addr);

    let server = Server::builder()
        .add_service(blog::blog_service_server::BlogServiceServer::new(
            grpc_service,
        ))
        .serve_with_shutdown(addr, async {
            signal::ctrl_c().await.expect("failed to listen for ctrl+c");
            println!("gRPC server received shutdown signal");
        });
    server.await?;

    Ok(())
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

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: DateTime<Utc>,
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}
