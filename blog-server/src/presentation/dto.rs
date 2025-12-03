use crate::domain::post::Post;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(rename = "token_type")]
    pub token_type: String, // "Bearer"
}

// ======================= POSTS =======================

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub page_token: Option<String>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ListPostsResponse {
    pub posts: Vec<Post>,
    pub next_page_token: Option<String>,
}

// ======================= Utils =======================
fn default_page_size() -> u32 {
    20
}
