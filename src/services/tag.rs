use crate::entities::{posts, tags};
use crate::models::tag::{SitemapTag, Tag};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect};

pub async fn get_tag_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Tag>, DbErr> {
    let tag = tags::Entity::find_by_id(id).one(db).await?;
    Ok(tag.map(Into::into))
}

pub async fn get_tags_for_sitemap(
    db: &DatabaseConnection,
    limit: i64,
) -> Result<Vec<SitemapTag>, DbErr> {
    let tag_models = tags::Entity::find()
        .find_with_related(posts::Entity)
        .all(db)
        .await?
        .into_iter()
        .filter(|(_, posts)| {
            posts
                .iter()
                .any(|post| post.published.unwrap_or(false) && post.deleted_at.is_none())
        })
        .map(|(tag, _)| tag)
        .take(limit.max(0) as usize)
        .collect::<Vec<_>>();

    Ok(tag_models.into_iter().map(Into::into).collect())
}

pub async fn get_all_tags(
    db: &DatabaseConnection,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Tag>, i64), DbErr> {
    let query = tags::Entity::find();
    let total = query.clone().count(db).await? as i64;
    let tag_models = query
        .order_by_asc(tags::Column::Name)
        .limit(limit.max(0) as u64)
        .offset(offset.max(0) as u64)
        .all(db)
        .await?;

    Ok((tag_models.into_iter().map(Into::into).collect(), total))
}
