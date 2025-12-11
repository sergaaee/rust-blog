use crate::error::BlogClientError;
use crate::Post;
use async_trait::async_trait;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;
use uuid::Uuid;

#[async_trait(?Send)]
pub trait BlogClientTrait: Send + Sync + 'static {
    async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(), BlogClientError>;
    async fn login(&mut self, email: String, password: String) -> Result<(), BlogClientError>;
    async fn get_post_by_id(&mut self, id: Uuid) -> Result<Post, BlogClientError>;
    async fn list_posts(
        &mut self,
        author_id: Option<Uuid>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Post>, BlogClientError>;
    async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError>;
    async fn update_post(
        &mut self,
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError>;
    async fn delete_post(&mut self, id: Uuid) -> Result<(), BlogClientError>;
}

const TOKEN_KEY: &str = "blog_token";

#[derive(Clone)]
pub struct BlogClientHttp {
    pub base_url: String,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PostsResponse {
    limit: u32,
    offset: u32,
    posts: Vec<Post>,
    total: u64,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(rename = "token_type")]
    pub token_type: String,
}

impl BlogClientHttp {
    pub async fn connect(endpoint: &str) -> Result<Self, BlogClientError> {
        let base_url = endpoint.trim_end_matches('/').to_string();
        let token = LocalStorage::get::<String>(TOKEN_KEY).ok();
        Ok(Self { base_url, token })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token.clone());
        let _ = LocalStorage::set(TOKEN_KEY, &token);
    }

    pub fn token(&self) -> Option<String> {
        self.token
            .clone()
            .or_else(|| LocalStorage::get::<String>(TOKEN_KEY).ok())
            .filter(|s| !s.is_empty())
    }

    fn auth_header(&self) -> Option<String> {
        self.token()
            .map(|t| format!("Bearer {}", t))
            .filter(|s| !s.is_empty())
    }

    // Универсальная отправка запроса
    async fn send<T: DeserializeOwned>(request: Request) -> Result<T, BlogClientError> {
        let response = request.send().await?;

        if response.ok() {
            response.json().await.map_err(BlogClientError::from)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(BlogClientError::Http {
                status,
                message: text,
            })
        }
    }
}

#[async_trait(?Send)]
impl BlogClientTrait for BlogClientHttp {
    async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(), BlogClientError> {
        let url = format!("{}/api/auth/register", self.base_url);
        let body = json!({
            "username": username,
            "email": email,
            "password": password,
        });

        let request = Request::post(&url).json(&body)?;
        let auth: AuthResponse = Self::send(request).await?;
        self.set_token(auth.access_token);
        Ok(())
    }

    async fn login(&mut self, username: String, password: String) -> Result<(), BlogClientError> {
        let url = format!("{}/api/auth/login", self.base_url);
        let body = json!({
            "username": username,
            "password": password,
        });

        let request = Request::post(&url).json(&body)?;
        let auth: AuthResponse = Self::send(request).await?;
        self.set_token(auth.access_token);
        Ok(())
    }

    async fn get_post_by_id(&mut self, id: Uuid) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let mut builder = Request::get(&url);

        if let Some(token) = self.auth_header() {
            builder = builder.header("Authorization", token.as_str());
        }

        let request = builder.header("Accept", "application/json").build()?;
        Self::send(request).await
    }

    async fn list_posts(
        &mut self,
        _author_id: Option<Uuid>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Post>, BlogClientError> {
        let limit = limit.unwrap_or(10).min(100);
        let offset = offset.unwrap_or(0);
        let url = format!(
            "{}/api/posts?limit={}&offset={}",
            self.base_url, limit, offset
        );

        let request = Request::get(&url).build()?;
        let resp: PostsResponse = Self::send(request).await?;
        Ok(resp.posts)
    }

    async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts", self.base_url);
        let body = json!({
            "title": title,
            "content": content,
        });

        let mut builder = Request::post(&url);

        if let Some(token) = self.auth_header() {
            builder = builder.header("Authorization", token.as_str());
        }

        let request = builder
            .header("Content-Type", "application/json")
            .json(&body)?;

        Self::send(request).await
    }

    async fn update_post(
        &mut self,
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let body = json!({
            "title": title,
            "content": content,
        });

        let mut builder = Request::put(&url);

        if let Some(token) = self.auth_header() {
            builder = builder.header("Authorization", token.as_str());
        }

        let request = builder
            .header("Content-Type", "application/json")
            .json(&body)?;

        Self::send(request).await
    }

    async fn delete_post(&mut self, id: Uuid) -> Result<(), BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);

        let mut request = Request::delete(&url);

        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", &header);
        }

        let response = request.send().await?;

        if response.ok() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(BlogClientError::Http {
                status,
                message: text,
            })
        }
    }
}
