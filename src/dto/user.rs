use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UserIdPath {
    pub id: Uuid,
}

#[derive(Deserialize, Validate)]
pub struct FollowRequest {
    pub user_id: Uuid,
}
