use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct BookmarkPath {
    pub id: Uuid,
}

#[derive(Deserialize, Validate)]
pub struct FolderIdPath {
    pub folder_id: Uuid,
}

#[derive(Deserialize, Validate)]
pub struct ToggleBookmarkRequest {
    pub folder_id: Option<Uuid>,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateBookmarkRequest {
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct MoveBookmarkRequest {
    pub folder_id: Option<Uuid>,
}

#[derive(Deserialize, Validate)]
pub struct CreateBookmarkFolderRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateBookmarkFolderRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct BookmarkQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0, max = 10_000))]
    pub offset: Option<i64>,
    pub folder_id: Option<String>,
}
