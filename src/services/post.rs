use crate::models::post::Post;
use rusqlite::{Connection, Result};
use chrono::Utc;

pub fn get_all_posts(conn: &Connection) -> Result<Vec<Post>> {
    let mut stmt = conn.prepare("SELECT id, title, body, published_at FROM posts")?;
    let post_iter = stmt.query_map([], |row| {
        Ok(Post {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            published_at: row.get_ref(3)?.as_str()?.parse().unwrap_or(Utc::now()),
        })
    })?;

    let mut posts = Vec::new();
    for post in post_iter {
        posts.push(post?);
    }
    Ok(posts)
}
