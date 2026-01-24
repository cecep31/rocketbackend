use crate::models::post::Post;
use crate::models::user::User;
use chrono::{DateTime, Utc};
use tokio_postgres::Client;
use uuid::Uuid;

pub async fn get_all_posts(
    client: &Client,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Post>, i64), tokio_postgres::Error> {
    // Get total count
    let total_row = client
        .query_one("SELECT COUNT(*) FROM posts", &[])
        .await?;
    let total: i64 = total_row.get(0);

    // Get paginated posts
    let rows = client
        .query(
            "SELECT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
             FROM posts p 
             INNER JOIN users u ON p.created_by = u.id 
             ORDER BY p.id
             LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        )
        .await?;

    let posts: Result<Vec<Post>, _> = rows
        .iter()
        .map(|row| {
            let id: Uuid = row.get(0);
            let title: String = row.get(1);
            let body: Option<String> = row.get(2);
            let created_by: Uuid = row.get(3);
            let slug: String = row.get(4);
            let photo_url: Option<String> = row.get(5);
            let created_at: DateTime<Utc> = row.get(6);
            let updated_at: DateTime<Utc> = row.get(7);
            let deleted_at: Option<DateTime<Utc>> = row.get(8);
            let published: bool = row.get(9);
            let view_count: i64 = row.get(10);
            let like_count: i64 = row.get(11);
            let user_id: Uuid = row.get(12);
            let username: String = row.get(13);

            // Substring body to 200 characters max
            let body = body.map(|b| {
                if b.chars().count() > 200 {
                    let truncated_body: String = b.chars().take(200).collect();
                    format!("{}...", truncated_body)
                } else {
                    b
                }
            });

            Ok(Post {
                id,
                title,
                body,
                created_by,
                slug,
                photo_url,
                created_at,
                updated_at,
                deleted_at,
                published,
                view_count,
                like_count,
                creator: User {
                    id: user_id,
                    username,
                },
            })
        })
        .collect();

    posts.map(|posts| (posts, total))
}

pub async fn get_random_posts(client: &Client, limit: i64) -> Result<Vec<Post>, tokio_postgres::Error> {
    let rows = client.query(
        "SELECT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
         FROM posts p 
         INNER JOIN users u ON p.created_by = u.id 
         ORDER BY RANDOM() 
         LIMIT $1",
        &[&limit]
    ).await?;

    let posts: Result<Vec<Post>, _> = rows.iter().map(|row| {
        let id: Uuid = row.get(0);
        let title: String = row.get(1);
        let body: Option<String> = row.get(2);
        let created_by: Uuid = row.get(3);
        let slug: String = row.get(4);
        let photo_url: Option<String> = row.get(5);
        let created_at: DateTime<Utc> = row.get(6);
        let updated_at: DateTime<Utc> = row.get(7);
        let deleted_at: Option<DateTime<Utc>> = row.get(8);
        let published: bool = row.get(9);
        let view_count: i64 = row.get(10);
        let like_count: i64 = row.get(11);
        let user_id: Uuid = row.get(12);
        let username: String = row.get(13);

        // Substring body to 200 characters max
        let body = body.map(|b| {
            if b.chars().count() > 200 {
                let truncated_body: String = b.chars().take(200).collect();
                format!("{}...", truncated_body)
            } else {
                b
            }
        });

        Ok(Post {
            id,
            title,
            body,
            created_by,
            slug,
            photo_url,
            created_at,
            updated_at,
            deleted_at,
            published,
            view_count,
            like_count,
            creator: User {
                id: user_id,
                username,
            },
        })
    }).collect();

    posts
}

pub async fn get_post_by_username_and_slug(
    client: &Client,
    username: &str,
    slug: &str,
) -> Result<Option<Post>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
             FROM posts p 
             INNER JOIN users u ON p.created_by = u.id 
             WHERE u.username = $1 AND p.slug = $2",
            &[&username, &slug],
        )
        .await?;

    match row {
        Some(row) => {
            let id: Uuid = row.get(0);
            let title: String = row.get(1);
            let body: Option<String> = row.get(2);
            let created_by: Uuid = row.get(3);
            let slug: String = row.get(4);
            let photo_url: Option<String> = row.get(5);
            let created_at: DateTime<Utc> = row.get(6);
            let updated_at: DateTime<Utc> = row.get(7);
            let deleted_at: Option<DateTime<Utc>> = row.get(8);
            let published: bool = row.get(9);
            let view_count: i64 = row.get(10);
            let like_count: i64 = row.get(11);
            let user_id: Uuid = row.get(12);
            let username: String = row.get(13);

            Ok(Some(Post {
                id,
                title,
                body,
                created_by,
                slug,
                photo_url,
                created_at,
                updated_at,
                deleted_at,
                published,
                view_count,
                like_count,
                creator: User {
                    id: user_id,
                    username,
                },
            }))
        }
        None => Ok(None),
    }
}
