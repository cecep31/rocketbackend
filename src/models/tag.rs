use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
}
