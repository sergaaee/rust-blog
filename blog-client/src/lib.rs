use crate::blog::blog_service_client::BlogServiceClient;
use crate::blog::{AuthResponse, Post};
use crate::error::BlogClientError;
use reqwest::Client;
use uuid::Uuid;

mod error;
mod grpc_client;
mod http_client;

pub mod blog {
    tonic::include_proto!("blog");
}

enum Transport {
    Http(String),
    Grpc(String),
}

trait BlogClientTrait {
    async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(), BlogClientError>;
    async fn login(&mut self, username: String, password: String) -> Result<(), BlogClientError>;
    async fn get_post_by_id(&mut self, id: Uuid) -> Result<Post, BlogClientError>;
    async fn list_posts(
        &mut self,
        author_id: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Post>, BlogClientError>;
    async fn create_post(&mut self, title: String, content: String) -> Result<Post, BlogClientError>;
    async fn update_post(
        &mut self,
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError>;
    async fn delete_post(&mut self, id: Uuid) -> Result<(), BlogClientError>;
}
