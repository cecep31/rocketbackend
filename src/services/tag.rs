use crate::models::tag::Tag;
use chrono::{DateTime, Utc};
use tokio_postgres::Client;

pub async fn get_all_tags(client: &Client) -> Result<Vec<Tag>, tokio_postgres::Error> {
    let rows = client.query(
        "SELECT id, name, created_at FROM tags ORDER BY name",
        &[]
    ).await?;

    let tags: Vec<Tag> = rows.iter().map(|row| {
        let id: i32 = row.get(0);
        let name: String = row.get(1);
        let created_at: Option<DateTime<Utc>> = row.get(2);
        
        Tag {
            id,
            name,
            created_at,
        }
    }).collect();

    Ok(tags)
}
