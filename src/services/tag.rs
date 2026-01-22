use crate::models::tag::Tag;
use tokio_postgres::Client;
use uuid::Uuid;

pub async fn get_all_tags(client: &Client) -> Result<Vec<Tag>, tokio_postgres::Error> {
    let rows = client.query(
        "SELECT id, name FROM tags ORDER BY name",
        &[]
    ).await?;

    let tags: Result<Vec<Tag>, _> = rows.iter().map(|row| {
        let id: Uuid = row.get(0);
        let name: String = row.get(1);

        Ok(Tag {
            id,
            name,
        })
    }).collect();

    tags
}
