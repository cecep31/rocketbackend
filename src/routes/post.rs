use crate::models::post::Post;
use crate::services;
use parking_lot::Mutex;
use rocket::serde::json::Json;
use rocket::State;
use rusqlite::Connection;

#[get("/posts")]
pub fn get_posts(conn: &State<Mutex<Connection>>) -> Json<Vec<Post>> {
    let conn = conn.lock();
    Json(services::post::get_all_posts(&conn).unwrap_or_else(|_| vec![]))
}
