use rocket::response::content::RawHtml;
use rocket::form::Form;
use crate::crypto::{hash_password};
use crate::database::{NexoDB, get_password_hash_from_username as get_psw};

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/", data = "<form>")]
pub async fn login(form: Form<LoginForm>, db: &NexoDB) -> RawHtml<String> {
    let stored_hash = get_user_psw_from_db(db, form.username.clone()).await;
    
    let is_valid = validate_user_psw(stored_hash, form.password.clone(), "salt" );
    if is_valid {
        println!("Login successful");
        RawHtml(r#"<div class="alert alert-success">Login successful!</div>"#.to_string())
    } else {
        println!("Invalid username or password");
        RawHtml(r#"<div class="alert alert-danger">Invalid username or password</div>"#.to_string())
    }
}

async fn get_user_psw_from_db(db: &NexoDB, username: String) -> Option<String>{
    let result = get_psw(db, username.as_str()).await;
    result
}
fn validate_user_psw(stored_password_hash: Option<String>, login_password: String, salt:&str) -> bool {
    match stored_password_hash {
        Some(from_db) => {
            from_db == hash_password(salt, login_password)
        }
        None => false
    }
}
