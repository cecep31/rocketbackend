#[macro_use]
extern crate rocket;

mod database;
mod models;
mod routes;
mod services;

use std::sync::Arc;
use rocket_cors::{CorsOptions, AllowedOrigins};
use routes::health::health;
use routes::post::{get_posts, get_random_posts};

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();

    let db_conn = database::connect().await.expect("failed to connect to database");

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Failed to create CORS");

    rocket::build()
        .manage(Arc::new(db_conn))
        .mount("/v1", routes![health, get_posts, get_random_posts])
        .attach(cors)
}
