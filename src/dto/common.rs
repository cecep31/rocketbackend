use crate::dto::validation::USERNAME_RE;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 0, max = 10_000))]
    pub offset: Option<i64>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
}

#[derive(Deserialize, Validate)]
pub struct UsernamePath {
    #[validate(length(min = 1, max = 50), regex(path = *USERNAME_RE))]
    pub username: String,
}

#[derive(Deserialize, Validate)]
pub struct PostIdPath {
    pub id: Uuid,
}
