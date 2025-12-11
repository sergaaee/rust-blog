use crate::application::post_service::PostService;
use crate::blog::DeletePostRequest;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::error::DomainError;
use crate::presentation::dto::{CreatePostRequest, Pagination, UpdatePostRequest};
use crate::presentation::utils::{AuthenticatedUser, ensure_owner};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, post, put, web};
use serde_json::json;
use std::sync::Arc;
use actix_web::web::post;
use tracing::info;
use uuid::Uuid;

#[post("")]
async fn create_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<Arc<PostService<PostgresPostRepository>>>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, DomainError> {
    let post = post.create_post(user.id, payload.0).await?;

    info!(
        request_id = %request_id(&req),
        username = %user.username,
        "post created"
    );

    Ok(HttpResponse::Created().json(post))
}

#[put("/{id}")]
async fn update_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<Arc<PostService<PostgresPostRepository>>>,
    payload: web::Json<UpdatePostRequest>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, DomainError> {
    let post_id = path.into_inner();
    let owner = post.get_post(post_id).await?;
    ensure_owner(&owner.author_id, &user.id)?;

    let post = post.update_post(post_id, user.id, payload.0).await?;

    info!(
        request_id = %request_id(&req),
        username = %user.username,
        post_id = %owner.id,
        "post updated"
    );

    Ok(HttpResponse::Ok().json(post))
}

#[delete("/{id}")]
async fn delete_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<Arc<PostService<PostgresPostRepository>>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, DomainError> {
    let post_id = path.into_inner();
    let owner = post.get_post(post_id).await?;
    ensure_owner(&owner.author_id, &user.id)?;

    let request = DeletePostRequest {
        post_id: post_id.to_string(),
    };

    post.delete_post(user.id, request).await?;

    info!(
        request_id = %request_id(&req),
        username = %user.username,
        post_id = %owner.id,
        "post deleted"
    );

    Ok(HttpResponse::NoContent().json("deleted"))
}

#[get("/posts")]
async fn get_posts(
    req: HttpRequest,
    post: web::Data<Arc<PostService<PostgresPostRepository>>>,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let pagination = query.into_inner();
    let posts = post.get_posts(pagination.limit, pagination.offset).await?;

    info!(
        request_id = %request_id(&req),
        "posts retrieved"
    );

    Ok(HttpResponse::Ok().json(json!({
        "posts": posts,
        "total": posts.len(),
        "limit": pagination.limit,
        "offset": pagination.offset
    })))
}

#[get("/posts/{id}")]
async fn get_post(
    req: HttpRequest,
    post: web::Data<Arc<PostService<PostgresPostRepository>>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, DomainError> {
    let post_id = path.into_inner();
    let post = post.get_post(post_id).await?;

    info!(
        request_id = %request_id(&req),
        "post retrieved"
    );

    Ok(HttpResponse::Ok().json(post))
}
fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<crate::presentation::middleware::RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".into())
}
