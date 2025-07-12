#[macro_use] extern crate rocket;
use rocket_db_pools::Database;
mod login;
mod crypto;
mod database;

#[get("/")]
async fn index(cookies: &rocket::http::CookieJar<'_>, db: &database::NexoDB) -> rocket::fs::NamedFile {
    // If authenticated, redirect to /home (handled by /home route)
    if let Some(session_cookie) = cookies.get("session_token") {
        let token = session_cookie.value();
        if let Some(_user_id) = database::validate_session(db, token).await {
            // Serve home page directly if authenticated
            return rocket::fs::NamedFile::open("static/home.html").await.expect("static/home.html not found");
        }
    }
    // Always serve login page for unauthenticated users
    rocket::fs::NamedFile::open("static/index.html").await.expect("static/index.html not found")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/home", routes![login::home])
        .mount("/login", routes![login::login])
        .mount("/", routes![login::logout])
        .mount("/api", routes![login::get_current_user])
        .register("/", catchers![not_found])
        .attach(database::NexoDB::init())
}

#[catch(404)]
fn not_found() -> &'static str {
    "File not found"
}

