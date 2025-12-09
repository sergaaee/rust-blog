use crate::blog::blog_service_client::BlogServiceClient;
use crate::blog::{
    CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest, LoginRequest,
    RegisterRequest, UpdatePostRequest,
};
use crate::error::BlogClientError;
use crate::{BlogClientTrait, Post};
use async_trait::async_trait;
use std::fs;
use tonic::Request;
use tonic::transport::Channel;
use uuid::Uuid;

#[derive(Clone)]
pub struct BlogClientGrpc {
    client: BlogServiceClient<Channel>,
    token: Option<String>,
}

impl BlogClientGrpc {
    /// Создаёт и подключается к gRPC серверу
    pub async fn connect(endpoint: &str) -> Result<Self, BlogClientError> {
        let channel = Channel::from_shared(endpoint.to_owned())
            .unwrap()
            .connect()
            .await?;
        Ok(Self {
            client: BlogServiceClient::new(channel),
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

    fn with_auth<T>(&self, mut req: Request<T>) -> Result<Request<T>, BlogClientError> {
        let token = if let Some(t) = &self.token {
            t.clone()
        } else {
            // пытаемся прочитать из файла
            let t = fs::read_to_string(".blog_token").map_err(|_| BlogClientError::Unauthorized)?;
            t.trim().to_string()
        };

        if token.is_empty() {
            return Err(BlogClientError::Unauthorized);
        }

        let header = format!("Bearer {token}")
            .parse()
            .map_err(|_| BlogClientError::Unauthorized)?;
        req.metadata_mut().insert("authorization", header);

        Ok(req)
    }
}

#[async_trait(?Send)]
impl BlogClientTrait for BlogClientGrpc {
    async fn register(
        &mut self,
        email: String,
        username: String,
        password: String,
    ) -> Result<(), BlogClientError> {
        if username.len() < 6 {
            return Err(BlogClientError::InvalidRequest(
                "Username must be at least 6 chars long".to_string(),
            ));
        }
        if !email.to_owned().contains("@") {
            return Err(BlogClientError::InvalidRequest("Wrong email".to_string()));
        }
        if password.len() < 8 {
            return Err(BlogClientError::InvalidRequest(
                "Passwords must be at least 8 chars long".to_string(),
            ));
        }

        let req = RegisterRequest {
            username,
            email,
            password,
        };
        let response = self.client.register(req).await?;
        self.set_token(response.into_inner().access_token);

        Ok(())
    }

    async fn login(&mut self, username: String, password: String) -> Result<(), BlogClientError> {
        if username.len() < 6 {
            return Err(BlogClientError::InvalidRequest(
                "Username must be at least 6 chars long".to_string(),
            ));
        }

        if password.len() < 8 {
            return Err(BlogClientError::InvalidRequest(
                "Passwords must be at least 8 chars long".to_string(),
            ));
        }

        let req = LoginRequest { username, password };
        let response = self.client.login(req).await?;
        self.set_token(response.into_inner().access_token);

        Ok(())
    }

    async fn get_post_by_id(&mut self, id: Uuid) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get_post(GetPostRequest {
                post_id: id.to_string(),
            })
            .await?;

        let post = response.into_inner();

        Ok(post.into())
    }

    async fn list_posts(
        &mut self,
        author_id: Option<Uuid>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Post>, BlogClientError> {
        let limit = limit.unwrap_or(10).min(100) as i32;
        let offset = offset.unwrap_or(0) as i32;
        let author_id = author_id.map(|id| id.to_string());
        let req = ListPostsRequest {
            limit,
            offset,
            author_id,
        };
        let response = self.client.list_posts(req).await?;

        let proto_posts = response.into_inner().posts;
        let posts: Vec<Post> = proto_posts.into_iter().map(Into::into).collect();

        Ok(posts)
    }

    async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let request = self.with_auth(Request::new(CreatePostRequest { title, content }))?;

        let response = self.client.create_post(request).await?;
        let post = response.into_inner();

        Ok(post.into())
    }

    async fn update_post(
        &mut self,
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError> {
        let request = self.with_auth(Request::new(UpdatePostRequest {
            post_id: id.to_string(),
            title,
            content,
        }))?;

        let response = self.client.update_post(request).await?;
        let post = response.into_inner();

        Ok(post.into())
    }

    async fn delete_post(&mut self, id: Uuid) -> Result<(), BlogClientError> {
        let request = self.with_auth(Request::new(DeletePostRequest {
            post_id: id.to_string(),
        }))?;

        self.client.delete_post(request).await?;

        Ok(())
    }
}
