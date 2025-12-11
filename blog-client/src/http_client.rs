use crate::BlogClientTrait;
use crate::Post;
use crate::error::BlogClientError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Clone)]
pub struct BlogClientHttp {
    client: Arc<Client>,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PostsResponse {
    posts: Vec<Post>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(rename = "token_type")]
    pub token_type: String, // "Bearer"
}

impl BlogClientHttp {
    pub async fn connect(endpoint: &str) -> Result<Self, BlogClientError> {
        let base_url = endpoint.trim_end_matches('/').to_string();
        Ok(Self {
            client: Arc::new(Client::builder().build()?),
            base_url,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token.clone());
        fs::write(".blog_token", token).unwrap();
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn auth_header(&self) -> Result<Option<reqwest::header::HeaderValue>, BlogClientError> {
        // 1. Берём токен из self или из файла
        let token = match &self.token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => {
                // пытаемся прочитать из файла
                let t =
                    fs::read_to_string(".blog_token").map_err(|_| BlogClientError::Unauthorized)?;
                t.trim().to_string()
            }
        };

        // 2. Если после всего токен пустой → нет авторизации
        if token.is_empty() {
            return Ok(None);
        }

        // 3. Формируем заголовок
        let value = format!("Bearer {}", token);
        let header = reqwest::header::HeaderValue::from_str(&value)
            .map_err(|_| BlogClientError::Unauthorized)?;

        Ok(Some(header))
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
        let resp = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "email": email,
                "password": password,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            let auth: AuthResponse = resp.json().await?;
            self.set_token(auth.access_token);
        } else {
            return Err(BlogClientError::from_http_response(resp).await);
        };

        Ok(())
    }

    async fn login(&mut self, username: String, password: String) -> Result<(), BlogClientError> {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            let auth: AuthResponse = resp.json().await?;
            self.set_token(auth.access_token);
        } else {
            return Err(BlogClientError::from_http_response(resp).await);
        };

        Ok(())
    }

    async fn get_post_by_id(&mut self, id: Uuid) -> Result<Post, BlogClientError> {
        let mut req = self
            .client
            .get(format!("{}/api/posts/{}", self.base_url, id));

        if let Some(h) = self.auth_header()? {
            req = req.header(reqwest::header::AUTHORIZATION, h);
        }

        let resp = req.send().await?;

        if resp.status().is_success() {
            let post: Post = resp.json().await?;
            Ok(post)
        } else {
            Err(BlogClientError::from_http_response(resp).await)
        }
    }

    async fn list_posts(
        &mut self,
        author_id: Option<Uuid>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Post>, BlogClientError> {
        let limit = limit.unwrap_or(10).min(100) as i32;
        let offset = offset.unwrap_or(0) as i32;
        let resp = self
            .client
            .get(format!(
                "{}/api/posts?limit={}&offset={}",
                self.base_url, limit, offset
            ))
            .send()
            .await?;

        if resp.status().is_success() {
            let posts: PostsResponse = resp.json().await?;
            Ok(posts.posts)
        } else {
            Err(BlogClientError::from_http_response(resp).await)
        }
    }

    async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let mut req = self.client.post(format!("{}/api/posts", self.base_url));

        if let Some(h) = self.auth_header()? {
            req = req.header(reqwest::header::AUTHORIZATION, h);
        }

        let resp = req
            .json(&serde_json::json!({
                "title": title,
                "content": content,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            let post: Post = resp.json().await?;
            Ok(post)
        } else {
            Err(BlogClientError::from_http_response(resp).await)
        }
    }

    async fn update_post(
        &mut self,
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError> {
        let mut req = self
            .client
            .put(format!("{}/api/posts/{}", self.base_url, id));

        if let Some(h) = self.auth_header()? {
            req = req.header(reqwest::header::AUTHORIZATION, h);
        }

        let resp = req
            .json(&serde_json::json!({
                "title": title,
                "content": content,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            let post: Post = resp.json().await?;
            Ok(post)
        } else {
            Err(BlogClientError::from_http_response(resp).await)
        }
    }

    async fn delete_post(&mut self, id: Uuid) -> Result<(), BlogClientError> {
        let mut req = self
            .client
            .delete(format!("{}/api/posts/{}", self.base_url, id));

        if let Some(h) = self.auth_header()? {
            req = req.header(reqwest::header::AUTHORIZATION, h);
        }

        let resp = req.send().await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(BlogClientError::from_http_response(resp).await)
        }
    }
}
