use rocket::form::Form;
use crate::database::{Nexodb, validate_user};

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/", data = "<form>")]
pub async fn login(form: Form<LoginForm>, db: &Nexodb) -> String {
    let is_valid = validate_user(db, &form.username, &form.password).await;
    
    if is_valid {
        "Login successful".to_string()
    } else {
        "Invalid username or password".to_string()
    }
}