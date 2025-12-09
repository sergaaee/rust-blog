use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::error::DomainError;
use crate::presentation::dto::{CreatePostRequest, Pagination, UpdatePostRequest};
use crate::presentation::utils::{AuthenticatedUser, ensure_owner};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, post, put, web};
use serde_json::json;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[post("/")]
async fn create_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<Arc<PostService<PostgresPostRepository>>>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, DomainError> {
    let post = post
        .create_post(user.id, payload.title.clone(), payload.content.clone())
        .await?;

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

    let post = post.update_post(user.id, post_id, payload.0).await?;

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

    post.delete_post(user.id, post_id).await?;

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
fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<crate::presentation::middleware::RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".into())
}
