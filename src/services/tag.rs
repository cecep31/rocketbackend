use crate::models::tag::Tag;
use tokio_postgres::Client;

pub async fn get_all_tags(client: &Client) -> Result<Vec<Tag>, tokio_postgres::Error> {
    let rows = client
        .query("SELECT id, name, created_at FROM tags ORDER BY name", &[])
        .await?;

    let tags: Vec<Tag> = rows.iter().map(Tag::from).collect();

    Ok(tags)
}
