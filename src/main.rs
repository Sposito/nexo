#[macro_use] extern crate rocket;
use rocket_db_pools::Database;
mod login;
mod crypto;
mod database;

#[get("/")]
async fn index() -> rocket::fs::NamedFile {
    rocket::fs::NamedFile::open("static/index.html")
        .await
        .expect("static/index.html not found")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/login", routes![login::login])
        .register("/", catchers![not_found])
        .attach(database::NexoDB::init())
}

#[catch(404)]
fn not_found() -> &'static str {
    "File not found"
}

