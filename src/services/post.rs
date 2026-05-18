use crate::entities::{posts, posts_to_tags, tags, users};
use crate::models::post::{Post, SitemapPost};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    ModelTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

fn validate_order_field(order_by: Option<&str>) -> posts::Column {
    match order_by {
        Some("id") => posts::Column::Id,
        Some("title") => posts::Column::Title,
        Some("updated_at") => posts::Column::UpdatedAt,
        Some("view_count") => posts::Column::ViewCount,
        Some("like_count") => posts::Column::LikeCount,
        Some("bookmark_count") => posts::Column::BookmarkCount,
        _ => posts::Column::CreatedAt,
    }
}

fn get_order_dir(dir: Option<&crate::handlers::OrderDirection>) -> Order {
    match dir {
        Some(crate::handlers::OrderDirection::Asc) => Order::Asc,
        _ => Order::Desc,
    }
}

async fn hydrate_post(
    db: &DatabaseConnection,
    post: posts::Model,
    truncate_body: bool,
) -> Result<Post, DbErr> {
    let user = post.find_related(users::Entity).one(db).await?;
    let post_tags = post.clone().find_related(tags::Entity).all(db).await?;
    Ok(Post::from_entity(post, user, post_tags, truncate_body))
}

async fn hydrate_posts(
    db: &DatabaseConnection,
    posts: Vec<posts::Model>,
    truncate_body: bool,
) -> Result<Vec<Post>, DbErr> {
    let mut hydrated = Vec::with_capacity(posts.len());
    for post in posts {
        hydrated.push(hydrate_post(db, post, truncate_body).await?);
    }
    Ok(hydrated)
}

pub async fn get_all_posts(
    db: &DatabaseConnection,
    offset: i64,
    limit: i64,
    search: Option<&str>,
    order_by: Option<&str>,
    order_direction: Option<&crate::handlers::OrderDirection>,
) -> Result<(Vec<Post>, i64), DbErr> {
    let mut query = posts::Entity::find()
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null());

    if let Some(search) = search.filter(|s| !s.trim().is_empty()) {
        query = query.filter(posts::Column::Title.contains(search));
    }

    let total = query.clone().count(db).await? as i64;
    let post_models = query
        .order_by(
            validate_order_field(order_by),
            get_order_dir(order_direction),
        )
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    Ok((hydrate_posts(db, post_models, true).await?, total))
}

pub async fn get_post_by_id(
    db: &DatabaseConnection,
    id: uuid::Uuid,
) -> Result<Option<Post>, DbErr> {
    let post = posts::Entity::find_by_id(id)
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    match post {
        Some(post) => Ok(Some(hydrate_post(db, post, false).await?)),
        None => Ok(None),
    }
}

pub async fn get_posts_by_username(
    db: &DatabaseConnection,
    username: &str,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Post>, i64), DbErr> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    let Some(user) = user else {
        return Ok((Vec::new(), 0));
    };

    let query = user
        .clone()
        .find_related(posts::Entity)
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null());

    let total = query.clone().count(db).await? as i64;
    let post_models = query
        .order_by_desc(posts::Column::CreatedAt)
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    Ok((hydrate_posts(db, post_models, true).await?, total))
}

pub async fn get_random_posts(db: &DatabaseConnection, limit: i64) -> Result<Vec<Post>, DbErr> {
    let post_models = posts::Entity::find()
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null())
        .order_by(sea_orm::sea_query::Expr::cust("RANDOM()"), Order::Asc)
        .limit(limit.max(0) as u64)
        .all(db)
        .await?;

    hydrate_posts(db, post_models, true).await
}

pub async fn get_trending_posts(db: &DatabaseConnection, limit: i64) -> Result<Vec<Post>, DbErr> {
    let post_models = posts::Entity::find()
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null())
        .order_by(
            sea_orm::sea_query::Expr::cust("like_count * 2 + bookmark_count * 2 + view_count"),
            Order::Desc,
        )
        .limit(limit.max(0) as u64)
        .all(db)
        .await?;

    hydrate_posts(db, post_models, true).await
}

pub async fn get_posts_for_sitemap(
    db: &DatabaseConnection,
    limit: i64,
) -> Result<Vec<SitemapPost>, DbErr> {
    let post_models = posts::Entity::find()
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null())
        .order_by_desc(posts::Column::CreatedAt)
        .limit(limit.max(0) as u64)
        .all(db)
        .await?;

    let mut sitemap = Vec::with_capacity(post_models.len());
    for post in post_models {
        if let Some(user) = post.find_related(users::Entity).one(db).await? {
            sitemap.push(SitemapPost::from_entities(post, user));
        }
    }

    Ok(sitemap)
}

pub async fn get_post_by_username_and_slug(
    db: &DatabaseConnection,
    username: &str,
    slug: &str,
) -> Result<Option<Post>, DbErr> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .filter(users::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    let Some(user) = user else {
        return Ok(None);
    };

    let post = user
        .find_related(posts::Entity)
        .filter(posts::Column::Slug.eq(slug))
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    match post {
        Some(post) => Ok(Some(hydrate_post(db, post, false).await?)),
        None => Ok(None),
    }
}

async fn find_or_create_tag(db: &DatabaseConnection, name: &str) -> Result<tags::Model, DbErr> {
    if let Some(tag) = tags::Entity::find()
        .filter(tags::Column::Name.eq(name))
        .one(db)
        .await?
    {
        return Ok(tag);
    }

    tags::ActiveModel {
        name: Set(name.to_string()),
        created_at: Set(Some(Utc::now().into())),
        ..Default::default()
    }
    .insert(db)
    .await
}

async fn replace_post_tags(
    db: &DatabaseConnection,
    post_id: uuid::Uuid,
    tag_names: &[String],
) -> Result<(), DbErr> {
    posts_to_tags::Entity::delete_many()
        .filter(posts_to_tags::Column::PostId.eq(post_id))
        .exec(db)
        .await?;

    for name in tag_names
        .iter()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
    {
        let tag = find_or_create_tag(db, name).await?;
        posts_to_tags::ActiveModel {
            post_id: Set(post_id),
            tag_id: Set(tag.id),
        }
        .insert(db)
        .await?;
    }

    Ok(())
}

pub struct CreatePostInput {
    pub title: String,
    pub photo_url: Option<String>,
    pub slug: String,
    pub body: String,
    pub published: bool,
    pub tags: Vec<String>,
}

pub struct UpdatePostInput {
    pub title: Option<String>,
    pub photo_url: Option<String>,
    pub slug: Option<String>,
    pub body: Option<String>,
    pub published: Option<bool>,
    pub tags: Option<Vec<String>>,
}

pub async fn create_post(
    db: &DatabaseConnection,
    input: CreatePostInput,
    creator_id: uuid::Uuid,
) -> Result<Post, DbErr> {
    let now = Utc::now();
    let post = posts::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set(input.title),
        created_by: Set(creator_id),
        body: Set(Some(input.body)),
        slug: Set(input.slug),
        photo_url: Set(input.photo_url),
        published: Set(Some(input.published)),
        created_at: Set(Some(now.into())),
        updated_at: Set(Some(now.into())),
        view_count: Set(Some(0)),
        like_count: Set(Some(0)),
        bookmark_count: Set(Some(0)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    replace_post_tags(db, post.id, &input.tags).await?;
    hydrate_post(db, post, false).await
}

pub async fn is_author(
    db: &DatabaseConnection,
    post_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<Option<bool>, DbErr> {
    let post = posts::Entity::find_by_id(post_id)
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    Ok(post.map(|post| post.created_by == user_id))
}

pub async fn update_post(
    db: &DatabaseConnection,
    post_id: uuid::Uuid,
    input: UpdatePostInput,
) -> Result<Option<Post>, DbErr> {
    let Some(post) = posts::Entity::find_by_id(post_id)
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?
    else {
        return Ok(None);
    };

    let mut active = post.into_active_model();
    if let Some(title) = input.title.filter(|value| !value.trim().is_empty()) {
        active.title = Set(title);
    }
    if let Some(body) = input.body.filter(|value| !value.trim().is_empty()) {
        active.body = Set(Some(body));
    }
    if let Some(slug) = input.slug.filter(|value| !value.trim().is_empty()) {
        active.slug = Set(slug);
    }
    if input.photo_url.is_some() {
        active.photo_url = Set(input.photo_url);
    }
    if let Some(published) = input.published {
        active.published = Set(Some(published));
    }
    active.updated_at = Set(Some(Utc::now().into()));

    let post = active.update(db).await?;
    if let Some(tags) = input.tags {
        replace_post_tags(db, post.id, &tags).await?;
    }

    Ok(Some(hydrate_post(db, post, false).await?))
}

pub async fn soft_delete_post(db: &DatabaseConnection, post_id: uuid::Uuid) -> Result<bool, DbErr> {
    let Some(post) = posts::Entity::find_by_id(post_id)
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?
    else {
        return Ok(false);
    };

    let mut active = post.into_active_model();
    active.deleted_at = Set(Some(Utc::now().into()));
    active.updated_at = Set(Some(Utc::now().into()));
    active.update(db).await?;
    Ok(true)
}

pub async fn get_posts_by_tag(
    db: &DatabaseConnection,
    tag_name: &str,
    offset: i64,
    limit: i64,
    _search: Option<&str>,
    order_by: Option<&str>,
    order_direction: Option<&crate::handlers::OrderDirection>,
) -> Result<(Vec<Post>, i64), DbErr> {
    let tag = tags::Entity::find()
        .filter(tags::Column::Name.eq(tag_name))
        .one(db)
        .await?;

    let Some(tag) = tag else {
        return Ok((Vec::new(), 0));
    };

    let query = tag
        .find_related(posts::Entity)
        .filter(posts::Column::Published.eq(true))
        .filter(posts::Column::DeletedAt.is_null());

    let total = query.clone().count(db).await? as i64;
    let post_models = query
        .order_by(
            validate_order_field(order_by),
            get_order_dir(order_direction),
        )
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    Ok((hydrate_posts(db, post_models, true).await?, total))
}
