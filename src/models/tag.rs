use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}
