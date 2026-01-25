use crate::models::post::Post;
use crate::models::tag::Tag;
use tokio_postgres::Client;

pub async fn get_all_posts(
    client: &Client,
    offset: i64,
    limit: i64,
    search: Option<&str>,
    order_by: Option<&str>,
    order_direction: Option<&crate::handlers::OrderDirection>,
) -> Result<(Vec<Post>, i64), tokio_postgres::Error> {
    // Validate and sanitize order_by field
    let valid_order_fields = ["id", "title", "created_at", "updated_at", "view_count", "like_count"];
    let order_field = order_by
        .and_then(|field| {
            if valid_order_fields.contains(&field) {
                Some(field)
            } else {
                None
            }
        })
        .unwrap_or("id");
    
    let order_dir = match order_direction {
        Some(crate::handlers::OrderDirection::Desc) => "DESC",
        _ => "ASC",
    };

    // Build WHERE clause for search
    let search_param = search.map(|s| format!("%{}%", s));

    // Get total count
    let total: i64 = if let Some(ref search_val) = search_param {
        let total_row = client
            .query_one(
                "SELECT COUNT(*) FROM posts p INNER JOIN users u ON p.created_by = u.id WHERE p.published = true AND (p.title ILIKE $1 OR p.body ILIKE $1 OR u.username ILIKE $1)",
                &[search_val],
            )
            .await?;
        total_row.get(0)
    } else {
        let total_row = client
            .query_one("SELECT COUNT(*) FROM posts WHERE published = true", &[])
            .await?;
        total_row.get(0)
    };

    // Build main query - ORDER BY field is validated against whitelist, so safe to format
    let query = if search_param.is_some() {
        format!(
            "SELECT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
             FROM posts p 
             INNER JOIN users u ON p.created_by = u.id 
             WHERE p.published = true AND (p.title ILIKE $1 OR p.body ILIKE $1 OR u.username ILIKE $1)
             ORDER BY p.{} {} 
             LIMIT $2 OFFSET $3",
            order_field, order_dir
        )
    } else {
        format!(
            "SELECT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
             FROM posts p 
             INNER JOIN users u ON p.created_by = u.id 
             WHERE p.published = true
             ORDER BY p.{} {} 
             LIMIT $1 OFFSET $2",
            order_field, order_dir
        )
    };

    // Get paginated posts
    let rows = if let Some(ref search_val) = search_param {
        client.query(&query, &[search_val, &limit, &offset]).await?
    } else {
        client.query(&query, &[&limit, &offset]).await?
    };

    let posts: Vec<Post> = rows
        .iter()
        .map(Post::from)
        .collect();

    Ok((posts, total))
}

pub async fn get_random_posts(client: &Client, limit: i64) -> Result<Vec<Post>, tokio_postgres::Error> {
    let rows = client.query(
        "SELECT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
         FROM posts p 
         INNER JOIN users u ON p.created_by = u.id 
         WHERE p.published = true
         ORDER BY RANDOM() 
         LIMIT $1",
        &[&limit]
    ).await?;

    let posts: Vec<Post> = rows.iter().map(Post::from).collect();

    Ok(posts)
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
             WHERE u.username = $1 AND p.slug = $2 AND p.published = true",
            &[&username, &slug],
        )
        .await?;

    match row {
        Some(row) => {
            let mut post = Post::from(&row);

            // Fetch tags for this post
            let tag_rows = client
                .query(
                    "SELECT t.id, t.name, t.created_at 
                     FROM tags t 
                     INNER JOIN posts_to_tags ptt ON t.id = ptt.tag_id 
                     WHERE ptt.post_id = $1 
                     ORDER BY t.name",
                    &[&post.id],
                )
                .await?;

            let tags: Vec<Tag> = tag_rows.iter().map(Tag::from).collect();
            post.tags = tags;

            Ok(Some(post))
        }
        None => Ok(None),
    }
}

pub async fn get_posts_by_tag(
    client: &Client,
    tag_name: &str,
    offset: i64,
    limit: i64,
    search: Option<&str>,
    order_by: Option<&str>,
    order_direction: Option<&crate::handlers::OrderDirection>,
) -> Result<(Vec<Post>, i64), tokio_postgres::Error> {
    // Validate and sanitize order_by field
    let valid_order_fields = ["id", "title", "created_at", "updated_at", "view_count", "like_count"];
    let order_field = order_by
        .and_then(|field| {
            if valid_order_fields.contains(&field) {
                Some(field)
            } else {
                None
            }
        })
        .unwrap_or("id");
    
    let order_dir = match order_direction {
        Some(crate::handlers::OrderDirection::Desc) => "DESC",
        _ => "ASC",
    };

    // Build WHERE clause for search
    let search_param = search.map(|s| format!("%{}%", s));

    // Get total count
    let total: i64 = if let Some(ref search_val) = search_param {
        let total_row = client
            .query_one(
                "SELECT COUNT(DISTINCT p.id) 
                 FROM posts p 
                 INNER JOIN users u ON p.created_by = u.id 
                 INNER JOIN posts_to_tags ptt ON p.id = ptt.post_id 
                 INNER JOIN tags t ON ptt.tag_id = t.id 
                 WHERE t.name = $1 AND p.published = true AND (p.title ILIKE $2 OR p.body ILIKE $2 OR u.username ILIKE $2)",
                &[&tag_name, search_val],
            )
            .await?;
        total_row.get(0)
    } else {
        let total_row = client
            .query_one(
                "SELECT COUNT(DISTINCT p.id) 
                 FROM posts p 
                 INNER JOIN posts_to_tags ptt ON p.id = ptt.post_id 
                 INNER JOIN tags t ON ptt.tag_id = t.id 
                 WHERE t.name = $1 AND p.published = true",
                &[&tag_name],
            )
            .await?;
        total_row.get(0)
    };

    // Build main query
    let query = if search_param.is_some() {
        format!(
            "SELECT DISTINCT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
             FROM posts p 
             INNER JOIN users u ON p.created_by = u.id 
             INNER JOIN posts_to_tags ptt ON p.id = ptt.post_id 
             INNER JOIN tags t ON ptt.tag_id = t.id 
             WHERE t.name = $1 AND p.published = true AND (p.title ILIKE $2 OR p.body ILIKE $2 OR u.username ILIKE $2)
             ORDER BY p.{} {} 
             LIMIT $3 OFFSET $4",
            order_field, order_dir
        )
    } else {
        format!(
            "SELECT DISTINCT p.id, p.title, p.body, p.created_by, p.slug, p.photo_url, p.created_at, p.updated_at, p.deleted_at, p.published, p.view_count, p.like_count, u.id, u.username 
             FROM posts p 
             INNER JOIN users u ON p.created_by = u.id 
             INNER JOIN posts_to_tags ptt ON p.id = ptt.post_id 
             INNER JOIN tags t ON ptt.tag_id = t.id 
             WHERE t.name = $1 AND p.published = true
             ORDER BY p.{} {} 
             LIMIT $2 OFFSET $3",
            order_field, order_dir
        )
    };

    // Get paginated posts
    let rows = if let Some(ref search_val) = search_param {
        client.query(&query, &[&tag_name, search_val, &limit, &offset]).await?
    } else {
        client.query(&query, &[&tag_name, &limit, &offset]).await?
    };

    // Fetch posts and then fetch their tags
    let mut posts: Vec<Post> = rows.iter().map(Post::from).collect();

    for post in &mut posts {
        // Fetch tags for this post
        let tag_rows = client
            .query(
                "SELECT t.id, t.name, t.created_at 
                 FROM tags t 
                 INNER JOIN posts_to_tags ptt ON t.id = ptt.tag_id 
                 WHERE ptt.post_id = $1 
                 ORDER BY t.name",
                &[&post.id],
            )
            .await?;

        let tags: Vec<Tag> = tag_rows.iter().map(Tag::from).collect();
        post.tags = tags;
    }

    Ok((posts, total))
}
