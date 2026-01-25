use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<&Row> for Tag {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0),
            name: row.get(1),
            created_at: row.get(2),
        }
    }
}
