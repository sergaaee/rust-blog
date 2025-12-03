use std::sync::Arc;

use crate::data::post_repository::PostRepository;
use crate::domain::{error::DomainError, post::Post};
use crate::infrastructure::security::JwtKeys;
use crate::presentation::dto::UpdatePostRequest;
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostService<R: PostRepository + 'static> {
    repo: Arc<R>,
}

impl<R> PostService<R>
where
    R: PostRepository + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn get_post(&self, id: Uuid) -> Result<Post, DomainError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(DomainError::from)?
            .ok_or_else(|| DomainError::PostNotFound(id))
    }

    pub async fn get_posts(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError> {
        let posts = self
            .repo
            .get_posts(limit, offset)
            .await
            .map_err(DomainError::from)?;

        Ok(posts)
    }

    #[instrument(skip(self))]
    pub async fn create_post(
        &self,
        author_id: Uuid,
        title: String,
        content: String,
    ) -> Result<Post, DomainError> {
        let post = Post::new(author_id, title, content);
        self.repo.create(post).await.map_err(DomainError::from)
    }

    #[instrument(skip(self))]
    pub async fn update_post(
        &self,
        author_id: Uuid,
        post_id: Uuid,
        update: UpdatePostRequest,
    ) -> Result<Post, DomainError> {
        match self.repo.update_post(author_id, post_id, update).await {
            Ok(Some(post)) => Ok(post),
            Ok(None) => Err(DomainError::PostNotFound(post_id)),
            Err(e) => Err(DomainError::from(e)),
        }
    }

    #[instrument(skip(self))]
    pub async fn delete_post(&self, author_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        self.repo.delete_post(author_id, post_id).await
    }
}
