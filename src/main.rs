#[macro_use]
extern crate rocket;

mod database;
mod models;
mod routes;
mod services;

use parking_lot::Mutex;
use routes::health::health;
use routes::post::get_posts;

#[launch]
fn rocket() -> _ {
    let db_conn = database::connect().expect("failed to connect to database");

    rocket::build()
        .manage(Mutex::new(db_conn))
        .mount("/", routes![health, get_posts])
}
