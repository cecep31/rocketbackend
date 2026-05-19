use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct TagIdPath {
    #[validate(range(min = 1))]
    pub id: i32,
}
