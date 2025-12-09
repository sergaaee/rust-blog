use crate::application::auth_service::AuthService;
use crate::application::post_service::PostService;
use crate::blog::blog_service_server::BlogService;
use crate::blog::{
    AuthResponse, CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest,
    ListPostsResponse, LoginRequest, Post as ProtoPost, RegisterRequest,
    UpdatePostRequest as ProtoUpdatePostRequest,
};
use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::domain::error::DomainError;
use crate::domain::post::Post;
use crate::infrastructure::security::Claims;
use crate::presentation::dto::UpdatePostRequest;
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Clone)]
pub struct BlogGrpcService<R, T>
where
    R: UserRepository + Send + Sync + 'static,
    T: PostRepository + Send + Sync + 'static,
{
    auth_service: Arc<AuthService<R>>,
    post_service: Arc<PostService<T>>,
}

impl<R, T> BlogGrpcService<R, T>
where
    R: UserRepository + Send + Sync + 'static,
    T: PostRepository + Send + Sync + 'static,
{
    pub fn new(auth_service: Arc<AuthService<R>>, post_service: Arc<PostService<T>>) -> Self {
        Self {
            auth_service,
            post_service,
        }
    }
}

#[tonic::async_trait]
impl<R, T> BlogService for BlogGrpcService<R, T>
where
    R: UserRepository + Send + Sync + 'static,
    T: PostRepository + Send + Sync + 'static,
{
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        // Базовая валидация
        if req.username.len() < 6 {
            return Err(Status::invalid_argument("Username must be ≥6 chars"));
        }
        if !req.email.to_owned().contains("@") {
            return Err(Status::invalid_argument("Invalid email"));
        }
        if req.password.len() < 8 {
            return Err(Status::invalid_argument("Password must be ≥8 chars"));
        }

        let user = self
            .auth_service
            .register(req.username, req.email, req.password)
            .await
            .map_err(map_domain_error_to_status)?;

        let token = self
            .auth_service
            .keys()
            .generate_token(user.id)
            .map_err(|e| Status::internal(format!("JWT generation failed: {e}")))?;

        tracing::info!("Registered new user: {} ({})", user.username, user.email);

        Ok(Response::new(AuthResponse {
            access_token: token,
            expires_in: 3600,
            token_type: "Bearer".to_string(),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        if req.username.len() < 6 {
            return Err(Status::invalid_argument("Username must be ≥6 chars"));
        }
        if req.password.len() < 8 {
            return Err(Status::invalid_argument("Password must be ≥8 chars"));
        }

        let token = self
            .auth_service
            .login(req.username.as_str(), req.password.as_str())
            .await
            .map_err(map_domain_error_to_status)?;

        tracing::info!("Successful login: username={}", req.username);

        Ok(Response::new(AuthResponse {
            access_token: token,
            expires_in: 3600,
            token_type: "Bearer".to_string(),
        }))
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<ProtoPost>, Status> {
        let token = extract_token_from_request(&request)?;
        let user: Claims = self
            .auth_service
            .keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

        let req = request.into_inner();

        let user_id = Uuid::parse_str(user.sub.as_str()).unwrap();

        let post = self
            .post_service
            .create_post(user_id, req.title, req.content)
            .await
            .map_err(map_domain_error_to_status)?;

        tracing::info!(
            "Created new post: user_id={}, post_id={}",
            user_id,
            post.id.to_string()
        );

        Ok(Response::new(ProtoPost {
            post_id: post.id.to_string(),
            title: post.title,
            content: post.content,
            author_id: user_id.to_string(),
            created_at: Some(post.created_at.to_protobuf()),
            updated_at: Some(post.updated_at.to_protobuf()),
        }))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<ProtoPost>, Status> {
        let req = request.into_inner();
        let post_id = Uuid::parse_str(req.post_id.as_str()).unwrap();

        let post = self
            .post_service
            .get_post(post_id)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(ProtoPost {
            post_id: post.id.to_string(),
            title: post.title,
            content: post.content,
            author_id: post.author_id.to_string(),
            created_at: Some(post.created_at.to_protobuf()),
            updated_at: Some(post.updated_at.to_protobuf()),
        }))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();

        let posts = self
            .post_service
            .get_posts(Some(req.limit as usize), Some(req.offset as usize))
            .await
            .map_err(map_domain_error_to_status)?;

        let proto_posts = ListPostsResponse::from(posts);

        Ok(Response::new(proto_posts))
    }

    async fn update_post(
        &self,
        request: Request<ProtoUpdatePostRequest>,
    ) -> Result<Response<ProtoPost>, Status> {
        let token = extract_token_from_request(&request)?;
        let user: Claims = self
            .auth_service
            .keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

        let req = request.into_inner();

        let user_id = Uuid::parse_str(user.sub.as_str()).unwrap();
        let post_id = Uuid::parse_str(req.post_id.as_str()).unwrap();
        let update_req = UpdatePostRequest::from(req);

        let post = self
            .post_service
            .update_post(post_id, user_id, update_req)
            .await
            .map_err(map_domain_error_to_status)?;

        tracing::info!(
            "Updated post: user_id={}, post_id={}",
            user_id,
            post.id.to_string()
        );

        Ok(Response::new(ProtoPost {
            post_id: post.id.to_string(),
            title: post.title,
            content: post.content,
            author_id: post.author_id.to_string(),
            created_at: Some(post.created_at.to_protobuf()),
            updated_at: Some(post.updated_at.to_protobuf()),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<()>, Status> {
        let token = extract_token_from_request(&request)?;
        let user: Claims = self
            .auth_service
            .keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

        let req = request.into_inner();

        let user_id = Uuid::parse_str(user.sub.as_str()).unwrap();
        let post_id = Uuid::parse_str(req.post_id.as_str()).unwrap();

        self.post_service
            .delete_post(user_id, post_id)
            .await
            .map_err(map_domain_error_to_status)?;

        tracing::info!("Deleted post: user_id={}, post_id={}", user_id, post_id);

        Ok(Response::new(()))
    }
}

fn map_domain_error_to_status(err: DomainError) -> Status {
    match err {
        DomainError::Unauthorized => Status::unauthenticated("Invalid credentials"),
        DomainError::UserNotFound(_) => Status::not_found("User not found"),
        DomainError::Internal(msg) => Status::internal(msg),
        DomainError::PostNotFound(_) => Status::not_found("Post not found"),
        _ => Status::internal("Internal server error"),
    }
}

fn extract_token_from_request<T>(req: &Request<T>) -> Result<String, Status> {
    req.metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("Authorization header missing"))
        .and_then(|auth| {
            auth.strip_prefix("Bearer ")
                .ok_or_else(|| Status::unauthenticated("Invalid authorization header format"))
                .map(String::from)
        })
}

pub trait ChronoToProtobufTimestamp {
    fn into_protobuf(self) -> Timestamp;
    fn to_protobuf(&self) -> Timestamp;
}

impl ChronoToProtobufTimestamp for DateTime<Utc> {
    fn into_protobuf(self) -> Timestamp {
        Timestamp {
            seconds: self.timestamp(),
            nanos: self.timestamp_subsec_nanos() as i32,
        }
    }

    fn to_protobuf(&self) -> Timestamp {
        self.clone().into_protobuf()
    }
}

impl From<Post> for ProtoPost {
    fn from(p: Post) -> Self {
        ProtoPost {
            post_id: p.id.to_string(),
            title: p.title,
            content: p.content,
            author_id: p.author_id.to_string(),
            created_at: Some(p.created_at.to_protobuf()),
            updated_at: Some(p.updated_at.to_protobuf()),
        }
    }
}

impl From<Vec<Post>> for ListPostsResponse {
    fn from(posts: Vec<Post>) -> Self {
        ListPostsResponse {
            posts: posts.clone().into_iter().map(Into::into).collect(),
            total_count: posts.len() as i32,
        }
    }
}

impl From<ProtoUpdatePostRequest> for UpdatePostRequest {
    fn from(update: ProtoUpdatePostRequest) -> Self {
        UpdatePostRequest {
            title: update.title,
            content: update.content,
        }
    }
}
