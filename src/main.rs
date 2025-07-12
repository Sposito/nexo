#[macro_use] extern crate rocket;
use rocket_db_pools::Database;
mod login;
mod crypto;
mod database;

#[get("/")]
async fn index(cookies: &rocket::http::CookieJar<'_>) -> Result<rocket::response::Redirect, rocket::fs::NamedFile> {
    if cookies.get("logged_in").is_some() {
        return Ok(rocket::response::Redirect::to("/home"));
    }
    Err(rocket::fs::NamedFile::open("static/index.html").await.expect("static/index.html not found"))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/home", routes![login::home])
        .mount("/login", routes![login::login])
        .register("/", catchers![not_found])
        .attach(database::NexoDB::init())
}

#[catch(404)]
fn not_found() -> &'static str {
    "File not found"
}

