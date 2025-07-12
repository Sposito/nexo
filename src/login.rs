use rocket::response::content::RawHtml;
use rocket::form::Form;
use crate::crypto::hash_password;
use crate::database::{NexoDB, get_password_hash_from_username as get_psw, ensure_db_initialized};
use rocket::http::{Status, Cookie, CookieJar};
use rocket::response::{Responder, Response};
use rocket::Request;

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}
#[get("/")]
pub async fn home(cookies: &CookieJar<'_>) -> Result<rocket::fs::NamedFile, rocket::response::Redirect> {
    if cookies.get("logged_in").is_some() {
        Ok(rocket::fs::NamedFile::open("static/home.html")
            .await
            .expect("static/home.html not found"))
    } else {
        Err(rocket::response::Redirect::to("/"))
    }
}

pub struct HxRedirect(pub String);

impl<'r> Responder<'r, 'static> for HxRedirect {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .status(Status::Ok)
            .raw_header("HX-Redirect", self.0)
            .ok()
    }
}

pub struct HxRedirectWithCookie {
    pub location: String,
}

impl<'r> Responder<'r, 'static> for HxRedirectWithCookie {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .status(Status::Ok)
            .raw_header("HX-Redirect", self.location)
            .ok()
    }
}

#[post("/", data = "<form>")]
pub async fn login(
    form: Form<LoginForm>,
    db: &NexoDB,
    cookies: &CookieJar<'_>,
) -> Result<HxRedirectWithCookie, RawHtml<String>> {
    let stored_hash = get_user_psw_from_db(db, form.username.clone()).await;
    if validate_user_psw(stored_hash, form.password.clone(), "salt") {
        let mut cookie = Cookie::new("logged_in", "1");
        cookie.set_path("/");
        cookies.add(cookie);
        Ok(HxRedirectWithCookie { location: "/home".to_string() })
    } else {
        Err(RawHtml(r#"
          <div class="text-red-600 text-center">
            Invalid username or password
          </div>
        "#.to_string()))
    }
}

async fn get_user_psw_from_db(db: &NexoDB, username: String) -> Option<String>{
    // Ensure database is initialized before querying
    if let Err(e) = ensure_db_initialized(db).await {
        eprintln!("Failed to initialize database: {:?}", e);
        return None;
    }
    
    let result = get_psw(db, username.as_str()).await;
    result
}
fn validate_user_psw(stored_password_hash: Option<String>, login_password: String, salt:&str) -> bool {
    match stored_password_hash {
        Some(from_db) => {
            from_db == hash_password(salt, login_password.as_str())
        }
        None => false
    }
}

#[cfg(test)]
mod tests {
    use super::validate_user_psw;
    use crate::crypto::hash_password;

    const SALT: &str = "test_salt";

    #[test]
    fn test_valid_password() {
        let password = "secure_password";
        let hashed = hash_password(SALT, password);
        let result = validate_user_psw(Some(hashed), password.to_string(), SALT);
        assert!(result, "Password should be valid");
    }

    #[test]
    fn test_invalid_password() {
        let correct_password = "secure_password";
        let wrong_password = "wrong_password";
        let hashed = hash_password(SALT, correct_password);
        let result = validate_user_psw(Some(hashed), wrong_password.to_string(), SALT);
        assert!(!result, "Password should be invalid");
    }

    #[test]
    fn test_none_stored_hash() {
        let result = validate_user_psw(None, "any_password".to_string(), SALT);
        assert!(!result, "Validation should fail if no stored hash is found");
    }

    #[test]
    fn test_empty_password() {
        let hashed = hash_password(SALT, "");
        let result = validate_user_psw(Some(hashed), "".to_string(), SALT);
        assert!(result, "Empty password should match its correct hash");
    }

    #[test]
    fn test_mismatched_hash() {
        let password = "password";
        let hashed = hash_password(SALT, password);
        let tampered_hash = format!("{}_modified", hashed);
        let result = validate_user_psw(Some(tampered_hash), password.to_string(), SALT);
        assert!(!result, "Hash mismatch should result in invalidation");
    }

    #[test]
    fn test_password_with_special_chars() {
        let password = "!@#$%^&*()_+-=[]{}|;':,.<>?";
        let hashed = hash_password(SALT, password);
        let result = validate_user_psw(Some(hashed), password.to_string(), SALT);
        assert!(result, "Password with special characters should be valid");
    }
}

