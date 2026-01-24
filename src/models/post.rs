use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::user::User;
use super::tag::Tag;

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub created_by: Uuid,
    pub slug: String,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub published: bool,
    pub view_count: i64,
    pub like_count: i64,
    pub creator: User,
    pub tags: Vec<Tag>,
}
