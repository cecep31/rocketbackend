use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct NotificationQuery {
    #[serde(default)]
    pub unread: bool,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0, max = 10_000))]
    pub offset: Option<i64>,
}

#[derive(Deserialize, Validate)]
pub struct NotificationPath {
    pub id: Uuid,
}
