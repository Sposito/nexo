use rocket::response::content::RawHtml;
use rocket::form::Form;
use crate::crypto::hash_password;
use crate::database::{NexoDB, get_password_hash_from_username as get_psw, ensure_db_initialized, 
                     get_user_id_by_username, create_session, validate_session, get_username_by_id, delete_session};
use rocket::http::{Status, Cookie, CookieJar};
use rocket::response::{Responder, Response};
use rocket::Request;

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[get("/")]
pub async fn home(cookies: &CookieJar<'_>, db: &NexoDB) -> Result<rocket::fs::NamedFile, rocket::response::Redirect> {
    // Check for session token instead of simple logged_in cookie
    if let Some(session_cookie) = cookies.get("session_token") {
        let token = session_cookie.value();
        
        // Validate the session token
        if let Some(_user_id) = validate_session(db, token).await {
            // Session is valid, serve the home page
            Ok(rocket::fs::NamedFile::open("static/home.html")
                .await
                .expect("static/home.html not found"))
        } else {
            // Invalid or expired session, redirect to login
            Err(rocket::response::Redirect::to("/"))
        }
    } else {
        // No session token, redirect to login
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
        // Get user ID for session creation
        if let Some(user_id) = get_user_id_by_username(db, form.username.as_str()).await {
            // Create session (24 hours = 86400 seconds)
            if let Some(session_token) = create_session(db, user_id, 86400).await {
                let mut cookie = Cookie::new("session_token", session_token);
                cookie.set_path("/");
                cookie.set_http_only(true); // Prevent XSS attacks
                cookie.set_secure(false); // Set to true in production with HTTPS
                cookies.add(cookie);
                
                Ok(HxRedirectWithCookie { location: "/home".to_string() })
            } else {
                Err(RawHtml(r#"
                  <div class="text-red-600 text-center">
                    Failed to create session. Please try again.
                  </div>
                "#.to_string()))
            }
        } else {
            Err(RawHtml(r#"
              <div class="text-red-600 text-center">
                User not found. Please try again.
              </div>
            "#.to_string()))
        }
    } else {
        Err(RawHtml(r#"
          <div class="text-red-600 text-center">
            Invalid username or password
          </div>
        "#.to_string()))
    }
}

#[post("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, db: &NexoDB) -> rocket::response::Redirect {
    // Get the session token from the cookie
    if let Some(session_cookie) = cookies.get("session_token") {
        let token = session_cookie.value();
        
        // Delete the session from the database
        if let Err(e) = delete_session(db, token).await {
            eprintln!("Failed to delete session: {:?}", e);
        }
    }
    
    // Remove session cookie
    cookies.remove(Cookie::from("session_token"));
    
    rocket::response::Redirect::to("/")
}

#[get("/user")]
pub async fn get_current_user(cookies: &CookieJar<'_>, db: &NexoDB) -> Result<rocket::serde::json::Json<serde_json::Value>, Status> {
    if let Some(session_cookie) = cookies.get("session_token") {
        let token = session_cookie.value();
        
        if let Some(user_id) = validate_session(db, token).await {
            if let Some(username) = get_username_by_id(db, user_id).await {
                let user_data = serde_json::json!({
                    "username": username,
                    "user_id": user_id
                });
                Ok(rocket::serde::json::Json(user_data))
            } else {
                Err(Status::InternalServerError)
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
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

