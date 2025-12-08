use crate::error::BlogClientError;
use blog::Post as ProtoPost;
use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<ProtoPost> for Post {
    fn from(p: ProtoPost) -> Self {
        let id = Uuid::parse_str(p.post_id.as_str()).unwrap();
        let author_id = Uuid::parse_str(p.author_id.as_str()).unwrap();
        Post {
            id,
            title: p.title,
            content: p.content,
            author_id,
            created_at: Some(p.created_at.unwrap().to_chrono()),
            updated_at: Some(p.updated_at.unwrap().to_chrono()),
        }
    }
}

pub trait ChronoToProtobufTimestamp {
    fn into_protobuf(self) -> Timestamp;
    fn to_protobuf(&self) -> Timestamp;
}

pub trait ProtobufToChrono {
    fn into_chrono(self) -> DateTime<Utc>;
    fn to_chrono(&self) -> DateTime<Utc>;
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

impl ProtobufToChrono for Timestamp {
    fn into_chrono(self) -> DateTime<Utc> {
        let naive = NaiveDateTime::from_timestamp_opt(self.seconds, self.nanos as u32)
            .expect("Invalid protobuf Timestamp");
        DateTime::<Utc>::from_utc(naive, Utc)
    }

    fn to_chrono(&self) -> DateTime<Utc> {
        self.clone().into_chrono()
    }
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
